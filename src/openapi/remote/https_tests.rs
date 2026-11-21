// Rust guideline compliant 2026-11-21

use super::*;

use std::sync::Arc;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::mpsc,
    task::JoinHandle,
};
use tokio_rustls::{
    rustls::{
        pki_types::{CertificateDer, PrivatePkcs8KeyDer},
        ServerConfig,
    },
    TlsAcceptor,
};

const CA: &[u8] = include_bytes!("../../../tests/fixtures/openapi/registry-ca.der");
const CERT: &[u8] = include_bytes!("../../../tests/fixtures/openapi/registry-cert.der");
const TEST_KEY: &[u8] = include_bytes!("../../../tests/fixtures/openapi/registry-test-key.der");
const SPEC_JSON: &[u8] = include_bytes!("../../../tests/fixtures/openapi/inventory.json");
const SPEC_YAML: &[u8] = include_bytes!("../../../tests/fixtures/openapi/inventory.yaml");
const HOST: &str = "registry.example.test";

struct Registry {
    address: SocketAddr,
    requests: mpsc::UnboundedReceiver<String>,
    server: JoinHandle<()>,
}

impl Registry {
    async fn start() -> Self {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        let config = ServerConfig::builder_with_provider(Arc::new(
            tokio_rustls::rustls::crypto::aws_lc_rs::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(CERT.to_vec())],
            PrivatePkcs8KeyDer::from(TEST_KEY.to_vec()).into(),
        )
        .unwrap();
        let acceptor = TlsAcceptor::from(Arc::new(config));
        let (tx, requests) = mpsc::unbounded_channel();
        let server = tokio::spawn(async move {
            while let Ok((tcp, _)) = listener.accept().await {
                let acceptor = acceptor.clone();
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Ok(mut stream) = acceptor.accept(tcp).await {
                        let _ = serve(&mut stream, tx).await;
                    }
                });
            }
        });
        Self {
            address,
            requests,
            server,
        }
    }

    async fn request(&self, hostname: &str, path: &str, trust: bool) -> Result<Vec<u8>> {
        let builder = spec_client(hostname, &[self.address]);
        let builder = if trust {
            builder.tls_certs_only([reqwest::Certificate::from_der(CA)?])
        } else {
            builder
        };
        let url = Url::parse(&format!("https://{hostname}:{}{path}", self.address.port()))?;
        fetch_spec(
            builder,
            url,
            Some(header::HeaderValue::from_static("Bearer registry-fixture")),
        )
        .await
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn serve(
    stream: &mut tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
    requests: mpsc::UnboundedSender<String>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(&mut *stream);
    let mut request = String::new();
    while let Ok(count) = reader.read_line(&mut request).await {
        if count == 0 || request.ends_with("\r\n\r\n") {
            break;
        }
    }
    let path = request.split_whitespace().nth(1).unwrap_or_default();
    let oversized = vec![b' '; super::super::MAX_SPEC_BYTES + 1];
    let changed = String::from_utf8_lossy(SPEC_JSON).replace(
        "Read an asset by identifier",
        "Read a reviewed inventory asset",
    );
    let (headers, body) = fixture_response(path, &oversized, changed.as_bytes());
    let _ = requests.send(request);
    stream.write_all(headers.as_bytes()).await?;
    stream.write_all(body).await?;
    stream.shutdown().await
}

fn fixture_response<'a>(path: &str, oversized: &'a [u8], changed: &'a [u8]) -> (String, &'a [u8]) {
    let (status, media, extra, body): (_, _, _, &[u8]) = match path {
        "/inventory.json" => ("200 OK", "application/json", "", SPEC_JSON),
        "/inventory.yaml" => ("200 OK", "application/yaml", "", SPEC_YAML),
        "/changed" => ("200 OK", "application/json", "", changed),
        "/redirect" => ("302 Found", "application/json", "Location: /never\r\n", b""),
        "/oversized" => ("200 OK", "application/yaml", "", oversized),
        "/wrong-media" => ("200 OK", "application/json", "", SPEC_YAML),
        _ => ("404 Not Found", "text/plain", "", b""),
    };
    (format!("HTTP/1.1 {status}\r\nContent-Type: {media}\r\nContent-Length: {}\r\nConnection: close\r\n{extra}\r\n", body.len()), body)
}

#[tokio::test]
async fn registry_tls_verifies_identity_and_enforces_fetch_contract() {
    let mut registry = Registry::start().await;
    assert!(registry
        .request(HOST, "/inventory.json", false)
        .await
        .is_err());
    assert!(registry
        .request("other.example.test", "/inventory.json", true)
        .await
        .is_err());
    assert!(registry.requests.is_empty());

    for (path, expected) in [
        ("/inventory.json", SPEC_JSON),
        ("/inventory.yaml", SPEC_YAML),
    ] {
        assert_eq!(registry.request(HOST, path, true).await.unwrap(), expected);
        let received = registry.requests.recv().await.unwrap();
        assert!(received.starts_with(&format!("GET {path} HTTP/")));
        assert!(received.contains("authorization: Bearer registry-fixture"));
        assert!(received.contains("accept-encoding: identity"));
    }

    for path in ["/redirect", "/oversized", "/wrong-media"] {
        assert!(registry.request(HOST, path, true).await.is_err());
        let received = registry.requests.recv().await.unwrap();
        assert!(received.starts_with(&format!("GET {path} HTTP/")));
        assert!(registry.requests.is_empty());
    }
}

async fn load_remote_revision(
    registry: &Registry,
    state: &crate::daemon::AppState,
    config: &crate::config::ServerConfig,
    path: &str,
) -> Result<()> {
    use super::super::{compiler, mount_prepared, prepare_definition, MountOrigin};

    let source_config = config.open_api.as_ref().unwrap();
    mount_prepared(
        state,
        "inventory",
        config,
        &std::collections::HashMap::new(),
        MountOrigin::Management(Some("fixture-operator")),
        async {
            let bytes = registry.request(HOST, path, true).await?;
            let source = compiler::compile(&bytes, source_config)?;
            prepare_definition(state, source_config, source)
        },
    )
    .await
}

fn observed_digest(state: &crate::daemon::AppState) -> String {
    serde_json::to_value(state.tool_pins.reviews())
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["tool"] == "getAsset")
        .and_then(|item| item.pointer("/current/digest"))
        .and_then(serde_json::Value::as_str)
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn revisions_from_https_stage_atomically_and_revalidate_pins() {
    use serde_json::json;

    let registry = Registry::start().await;
    let directory = tempfile::tempdir().unwrap();
    let config: crate::config::ServerConfig = serde_json::from_value(json!({
        "requireToolPins":true,
        "openApi":{
            "baseUrl":"http://127.0.0.1:1/api",
            "remoteSpec":{"url":format!("https://{HOST}:{}/inventory.json",registry.address.port()),"auth":{"type":"none"}},
            "operations":["getAsset","listAssets"],
            "auth":{"type":"none"}
        }
    })).unwrap();
    let state = crate::daemon::server::initialize_state(
        serde_json::from_value(json!({
            "state":{"enabled":true, "dir":directory.path()},
            "mcpServers":{}
        }))
        .unwrap(),
        directory.path().join("config.json").to_string_lossy(),
    )
    .await
    .unwrap();

    load_remote_revision(&registry, &state, &config, "/inventory.json")
        .await
        .unwrap();
    assert!(state.tool_catalog().await.items.is_empty());
    let digest = observed_digest(&state);
    let request = crate::tool_pins::ApprovePin {
        server: "inventory".into(),
        tool: "getAsset".into(),
        digest: digest.clone(),
    };
    state
        .tool_pins
        .approve(&state, &request, Some("fixture-operator".into()))
        .await
        .unwrap();
    let visible = |catalog: &crate::tool_pins::ToolCatalog<'_>| {
        catalog.items.contains_key("inventory.getAsset")
    };
    assert!(visible(&state.tool_catalog().await));

    load_remote_revision(&registry, &state, &config, "/inventory.yaml")
        .await
        .unwrap();
    assert_eq!(observed_digest(&state), digest);
    assert!(visible(&state.tool_catalog().await));
    assert!(
        load_remote_revision(&registry, &state, &config, "/redirect")
            .await
            .is_err()
    );
    assert!(visible(&state.tool_catalog().await));
    assert_eq!(
        state.server_statuses.read().await["inventory"]["serving_previous_revision"],
        true
    );

    load_remote_revision(&registry, &state, &config, "/changed")
        .await
        .unwrap();
    assert_ne!(observed_digest(&state), digest);
    assert!(!visible(&state.tool_catalog().await));
    assert!(state
        .tool_pins
        .approve(&state, &request, None)
        .await
        .is_err());

    load_remote_revision(&registry, &state, &config, "/inventory.json")
        .await
        .unwrap();
    assert_eq!(observed_digest(&state), digest);
    assert!(!visible(&state.tool_catalog().await));
    state.shutdown().await;
}
