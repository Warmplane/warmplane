// Rust guideline compliant 2026-08-13

//! Main entrypoint for Warmplane local MCP control plane CLI and daemon executable.

use anyhow::{Context, Result};
use clap::Parser;
use serde_json::{json, Value};

use warmplane::{
    cli_config,
    config::{load_config, resolve_client_port, DEFAULT_PORT},
    context::RequestContext,
    daemon, mcp_server,
    models::{Cli, Commands},
    telemetry,
};

/// Warmplane CLI binary entrypoint parsing arguments and dispatching commands.
///
/// # Errors
/// Returns an error if command execution or daemon startup fails.
#[tokio::main]
async fn main() -> Result<()> {
    let _telemetry = telemetry::init()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::ValidateConfig { config } => {
            let cfg = load_config(&config)?;
            let server_count = cfg.mcp_servers.len();
            println!(
                "{{\"ok\":true,\"config\":\"{}\",\"servers\":{}}}",
                config, server_count
            );
        }
        Commands::Server { command } => {
            cli_config::handle_server_command(command).await?;
        }
        Commands::Config { command } => {
            cli_config::handle_config_command(command).await?;
        }
        Commands::Reload { port, config } => {
            cli_config::trigger_daemon_reload(port, &config).await?;
        }
        Commands::Daemon {
            port,
            config,
            auth_token,
        } => {
            let mut config_data = load_config(&config)?;
            if auth_token.is_some() {
                config_data.auth_token = auth_token;
            }
            let resolved_port = port.or(config_data.port).unwrap_or(DEFAULT_PORT);
            daemon::run_daemon(resolved_port, config_data, config).await?;
        }
        Commands::McpServer { config, profile } => {
            let config_data = load_config(&config)?;
            mcp_server::run_mcp_server(config_data, config, profile).await?;
        }
        Commands::McpHttpServer {
            port,
            config,
            profile,
            bind,
        } => {
            let config_data = load_config(&config)?;
            mcp_server::run_mcp_http_server(config_data, config, port, bind, profile).await?;
        }
        Commands::ListCapabilities {
            port,
            config,
            profile,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let mut req = client.get(format!(
                "http://127.0.0.1:{}/v1/capabilities",
                resolved_port
            ));
            if let Some(prof) = profile {
                req = req.header("x-warmplane-profile", prof);
            }
            let res = req.send().await?;
            println!("{}", res.text().await?);
        }
        Commands::SearchCapabilities {
            port,
            config,
            query,
            limit,
            server,
            tag,
            profile,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let payload = json!({
                "query": query,
                "limit": limit,
                "server_ids": server,
                "tags": tag,
            });

            let client = reqwest::Client::new();
            let mut req = client
                .post(format!(
                    "http://127.0.0.1:{}/v1/capabilities/search",
                    resolved_port
                ))
                .json(&payload);
            if let Some(prof) = profile {
                req = req.header("x-warmplane-profile", prof);
            }
            let res = req.send().await?;
            println!("{}", res.text().await?);
        }
        Commands::DescribeCapability {
            port,
            config,
            id,
            profile,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let mut req = client.get(format!(
                "http://127.0.0.1:{}/v1/capabilities/{}",
                resolved_port, id
            ));
            if let Some(prof) = profile {
                req = req.header("x-warmplane-profile", prof);
            }
            let res = req.send().await?;
            println!("{}", res.text().await?);
        }
        Commands::CallCapability {
            port,
            config,
            id,
            params,
            request_id,
            operation_id,
            work_item_id,
            actor_id,
            grant_id,
            idempotency_key,
            profile,
            jsonpath,
            limit_lines,
            truncate_bytes,
            async_task,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let mut parsed_params: Value =
                serde_json::from_str(&params).context("Invalid JSON parameters provided")?;

            if let Some(obj) = parsed_params.as_object_mut() {
                if let Some(jp) = jsonpath {
                    obj.insert("_jsonpath".to_string(), Value::String(jp));
                }
                if let Some(ll) = limit_lines {
                    obj.insert("_limit_lines".to_string(), json!(ll));
                }
                if let Some(tb) = truncate_bytes {
                    obj.insert("_truncate_bytes".to_string(), json!(tb));
                }
            }

            let context = RequestContext {
                operation_id,
                work_item_id,
                actor_id,
                grant_id,
            };

            let payload = json!({
                "capability_id": id,
                "args": parsed_params,
                "request_id": request_id,
                "context": context,
                "idempotency_key": idempotency_key,
                "async_task": async_task,
            });

            let client = reqwest::Client::new();
            let mut req = client
                .post(format!("http://127.0.0.1:{}/v1/tools/call", resolved_port))
                .json(&payload);
            if let Some(prof) = profile {
                req = req.header("x-warmplane-profile", prof);
            }
            let res = req.send().await?;
            println!("{}", res.text().await?);
        }
        Commands::BatchCallCapabilities {
            port,
            config,
            steps,
            file,
            profile,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let steps_json: Value = if let Some(path) = file {
                let content = std::fs::read_to_string(&path)
                    .with_context(|| format!("Could not read steps file: {}", path))?;
                serde_json::from_str(&content).context("Invalid JSON in steps file")?
            } else if let Some(s) = steps {
                serde_json::from_str(&s).context("Invalid JSON steps string")?
            } else {
                anyhow::bail!("Either --steps '<json>' or --file '<path>' must be provided");
            };

            let payload = if steps_json.is_array() {
                json!({ "steps": steps_json })
            } else {
                steps_json
            };

            let client = reqwest::Client::new();
            let mut req = client
                .post(format!(
                    "http://127.0.0.1:{}/v1/tools/batch_call",
                    resolved_port
                ))
                .json(&payload);
            if let Some(prof) = profile {
                req = req.header("x-warmplane-profile", prof);
            }
            let res = req.send().await?;
            println!("{}", res.text().await?);
        }
        Commands::ListResources {
            port,
            config,
            profile,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let mut req = client.get(format!("http://127.0.0.1:{}/v1/resources", resolved_port));
            if let Some(prof) = profile {
                req = req.header("x-warmplane-profile", prof);
            }
            let res = req.send().await?;
            println!("{}", res.text().await?);
        }
        Commands::ReadResource {
            port,
            config,
            id,
            request_id,
            operation_id,
            work_item_id,
            actor_id,
            grant_id,
            profile,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let context = RequestContext {
                operation_id,
                work_item_id,
                actor_id,
                grant_id,
            };

            let payload = json!({
                "resource_id": id,
                "request_id": request_id,
                "context": context,
            });

            let client = reqwest::Client::new();
            let mut req = client
                .post(format!(
                    "http://127.0.0.1:{}/v1/resources/read",
                    resolved_port
                ))
                .json(&payload);
            if let Some(prof) = profile {
                req = req.header("x-warmplane-profile", prof);
            }
            let res = req.send().await?;
            println!("{}", res.text().await?);
        }
        Commands::ListPrompts {
            port,
            config,
            profile,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let mut req = client.get(format!("http://127.0.0.1:{}/v1/prompts", resolved_port));
            if let Some(prof) = profile {
                req = req.header("x-warmplane-profile", prof);
            }
            let res = req.send().await?;
            println!("{}", res.text().await?);
        }
        Commands::GetPrompt {
            port,
            config,
            id,
            arguments,
            request_id,
            operation_id,
            work_item_id,
            actor_id,
            grant_id,
            profile,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let parsed_arguments: Value =
                serde_json::from_str(&arguments).context("Invalid JSON arguments provided")?;

            let context = RequestContext {
                operation_id,
                work_item_id,
                actor_id,
                grant_id,
            };

            let payload = json!({
                "prompt_id": id,
                "arguments": parsed_arguments,
                "request_id": request_id,
                "context": context,
            });

            let client = reqwest::Client::new();
            let mut req = client
                .post(format!("http://127.0.0.1:{}/v1/prompts/get", resolved_port))
                .json(&payload);
            if let Some(prof) = profile {
                req = req.header("x-warmplane-profile", prof);
            }
            let res = req.send().await?;
            println!("{}", res.text().await?);
        }
        Commands::ListCatalogEvents {
            port,
            config,
            after,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let mut url = format!("http://127.0.0.1:{}/v1/catalog/events", resolved_port);
            if let Some(cursor) = after {
                url = format!("{}?after={}", url, cursor);
            }
            let res = reqwest::get(url).await?;
            println!("{}", res.text().await?);
        }
        Commands::CancelOperation { port, config, id } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let res = client
                .post(format!(
                    "http://127.0.0.1:{}/v1/operations/{}/cancel",
                    resolved_port, id
                ))
                .send()
                .await?;
            println!("{}", res.text().await?);
        }
        Commands::Approvals { command } => {
            cli_config::handle_approvals_command(command).await?;
        }
        Commands::Tasks { command } => {
            cli_config::handle_task_command(command).await?;
        }
        Commands::Idempotency { command } => {
            cli_config::handle_idempotency_command(command).await?;
        }
    }
    Ok(())
}
