// Rust guideline compliant 2026-11-21

//! Fetches operator-configured specs from validated HTTPS destinations.

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};

use anyhow::{ensure, Context, Result};
use ipnet::IpNet;
use reqwest::{header, Client, StatusCode, Url};
use tokio::sync::Semaphore;

use crate::config::RemoteOpenApiSpec;

static FETCH_SLOTS: Semaphore = Semaphore::const_new(4);
const FETCH_DEADLINE: Duration = Duration::from_secs(10);

struct Endpoint {
    url: Url,
    private_networks: Vec<IpNet>,
}

impl Endpoint {
    fn parse(config: &RemoteOpenApiSpec) -> Result<Self> {
        let url = Url::parse(&config.url)
            .ok()
            .context("Invalid remote OpenAPI spec endpoint")?;
        ensure!(
            url.scheme() == "https"
                && url.domain().is_some_and(valid_hostname)
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none(),
            "Remote OpenAPI requires an exact HTTPS URL with a DNS hostname"
        );
        let private_networks = config
            .private_networks
            .iter()
            .map(|network| {
                network
                    .parse::<IpNet>()
                    .ok()
                    .context("Invalid remote OpenAPI private CIDR")
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            url,
            private_networks,
        })
    }

    async fn addresses(&self, hostname: &str) -> Result<Vec<SocketAddr>> {
        let port = self.url.port_or_known_default().unwrap_or(443);
        let addresses = tokio::net::lookup_host((hostname, port))
            .await
            .ok()
            .context("Remote OpenAPI hostname resolution failed")?;
        self.check_addresses(addresses)
    }

    fn check_addresses(
        &self,
        addresses: impl IntoIterator<Item = SocketAddr>,
    ) -> Result<Vec<SocketAddr>> {
        let mut checked = Vec::new();
        for address in addresses {
            let ip = address.ip().to_canonical();
            ensure!(
                allowed_address(ip, &self.private_networks),
                "Remote OpenAPI hostname resolved outside its egress policy"
            );
            checked.push(SocketAddr::new(ip, address.port()));
        }
        ensure!(
            !checked.is_empty(),
            "Remote OpenAPI hostname returned no addresses"
        );
        Ok(checked)
    }
}

fn valid_hostname(host: &str) -> bool {
    let host = host.strip_suffix('.').unwrap_or(host);
    !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
}

fn allowed_address(address: IpAddr, private_networks: &[IpNet]) -> bool {
    let private = match address {
        IpAddr::V4(ip) => ip.is_private(),
        IpAddr::V6(ip) => ip.is_unique_local() && !is_provider_metadata(ip),
    };
    if private {
        return private_networks
            .iter()
            .any(|network| network.contains(&address));
    }
    match address {
        IpAddr::V4(ip) => public_v4(ip),
        IpAddr::V6(ip) => public_v6(ip),
    }
}

fn is_provider_metadata(ip: Ipv6Addr) -> bool {
    ip == Ipv6Addr::new(0xfd00, 0xec2, 0, 0, 0, 0, 0, 0x254)
}

fn public_v4(address: Ipv4Addr) -> bool {
    // Exclude the non-public and special-purpose blocks used for local infrastructure.
    !matches!(
        address.octets(),
        [0 | 10 | 127 | 224..=255, ..]
            | [100, 64..=127, ..]
            | [169, 254, ..]
            | [172, 16..=31, ..]
            | [192, 0, 0 | 2, _]
            | [192, 88, 99, _]
            | [192, 168, ..]
            | [198, 18 | 19, ..]
            | [198, 51, 100, _]
            | [203, 0, 113, _]
            | [168, 63, 129, 16]
    )
}

fn public_v6(address: Ipv6Addr) -> bool {
    let [first, second, ..] = address.segments();
    (0x2000..=0x3fff).contains(&first)
        && !matches!(first, 0x2002 | 0x3fff)
        && !(first == 0x2001 && (second <= 0x01ff || second == 0x0db8))
}

pub(super) fn validate(config: &RemoteOpenApiSpec) -> Result<()> {
    super::validate_auth(&config.auth)?;
    Endpoint::parse(config).map(|_| ())
}

pub(super) async fn download(config: &RemoteOpenApiSpec) -> Result<Vec<u8>> {
    let _slot = FETCH_SLOTS
        .acquire()
        .await
        .context("Remote OpenAPI fetch queue closed")?;
    tokio::time::timeout(FETCH_DEADLINE, fetch(config))
        .await
        .ok()
        .context("Remote OpenAPI fetch timed out")?
}

async fn fetch(config: &RemoteOpenApiSpec) -> Result<Vec<u8>> {
    let endpoint = Endpoint::parse(config)?;
    let hostname = endpoint
        .url
        .domain()
        .context("Remote OpenAPI endpoint requires a hostname")?;
    let addresses = endpoint.addresses(hostname).await?;
    let builder = spec_client(hostname, &addresses);
    let auth = super::worker::authorization(&config.auth)?;
    fetch_spec(builder, endpoint.url, auth).await
}

fn spec_client(hostname: &str, addresses: &[SocketAddr]) -> reqwest::ClientBuilder {
    Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .connect_timeout(Duration::from_secs(3))
        .resolve_to_addrs(hostname, addresses)
}

async fn fetch_spec(
    builder: reqwest::ClientBuilder,
    url: Url,
    auth: Option<header::HeaderValue>,
) -> Result<Vec<u8>> {
    let client = builder
        .build()
        .ok()
        .context("Could not prepare the remote OpenAPI client")?;
    let mut request = client.get(url).header(header::ACCEPT_ENCODING, "identity");
    if let Some(auth) = auth {
        request = request.header(header::AUTHORIZATION, auth);
    }
    let mut response = request
        .send()
        .await
        .ok()
        .context("Remote OpenAPI fetch failed")?;
    let json = validate_response(&response)?;
    let body = super::worker::bounded_body(&mut response, super::MAX_SPEC_BYTES)
        .await
        .ok()
        .context("Remote OpenAPI body could not be read within the byte limit")?;
    if json {
        super::compiler::require_json(&body)?;
    }
    Ok(body)
}

fn validate_response(response: &reqwest::Response) -> Result<bool> {
    ensure!(
        response.status() == StatusCode::OK,
        "Remote OpenAPI endpoint did not return HTTP 200"
    );
    let encoding = response.headers().get_all(header::CONTENT_ENCODING);
    ensure!(
        encoding
            .iter()
            .all(|value| value.as_bytes().eq_ignore_ascii_case(b"identity")),
        "Remote OpenAPI endpoint returned an unsupported content encoding"
    );
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_ascii_lowercase());
    ensure!(
        matches!(
            content_type.as_deref(),
            Some(
                "application/json"
                    | "application/yaml"
                    | "application/x-yaml"
                    | "text/yaml"
                    | "application/vnd.oai.openapi"
                    | "application/vnd.oai.openapi+json"
            )
        ),
        "Remote OpenAPI endpoint returned an unsupported media type"
    );
    Ok(matches!(
        content_type.as_deref(),
        Some("application/json" | "application/vnd.oai.openapi+json")
    ))
}

#[cfg(test)]
mod https_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OpenApiAuth;

    #[test]
    fn egress_checks_the_complete_answer_and_normalizes_mapped_addresses() {
        let endpoint = Endpoint::parse(&RemoteOpenApiSpec {
            url: "https://registry.example.test/spec.yaml".into(),
            auth: OpenApiAuth::None,
            private_networks: vec!["10.8.0.0/16".into(), "fd00::/8".into()],
        })
        .unwrap();
        let address = |value: &str| value.parse::<SocketAddr>().unwrap();
        assert!(endpoint
            .check_addresses([address("10.8.0.3:443"), address("[fd01::1]:443")])
            .is_ok());
        assert_eq!(
            endpoint
                .check_addresses([address("[::ffff:10.8.0.3]:443")])
                .unwrap(),
            [address("10.8.0.3:443")]
        );
        for denied in [
            "10.9.0.3:443",
            "127.0.0.1:443",
            "169.254.169.254:443",
            "[::ffff:127.0.0.1]:443",
            "[fd00:ec2::254]:443",
            "[::1]:443",
        ] {
            assert!(endpoint
                .check_addresses([address("10.8.0.3:443"), address(denied)])
                .is_err());
        }
    }
}
