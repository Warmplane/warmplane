// Rust guideline compliant 2026-08-14

//! Handlers for `warmplane server` and `warmplane config` CLI commands.

use anyhow::{Context, Result};
use colored::Colorize;
use inquire::{Confirm, Select};
use std::collections::HashMap;
use std::path::Path;

use crate::client_sync::{attach_client, detach_client, detect_clients, AttachOptions};
use crate::config::{
    load_config, load_or_default_config, resolve_client_port, save_config, AuthConfig, McpConfig,
    ServerConfig,
};
use crate::config_import::{
    discover_sources, import_servers_into_config, parse_standard_mcp_source,
};
use crate::interactive::{interactive_add_server, parse_args_string};
use crate::models::{
    AliasCommands, ClientCommands, ConfigCommands, PolicyCommands, ServerAddArgs, ServerCommands,
    StateCommands,
};

/// Dispatches `warmplane client` subcommands.
pub async fn handle_client_command(cmd: ClientCommands) -> Result<()> {
    match cmd {
        ClientCommands::List { json } => {
            let statuses = detect_clients();
            if json {
                println!("{}", serde_json::to_string_pretty(&statuses)?);
                return Ok(());
            }

            println!("{}", "\n🔌 Supported AI Client Integrations:".bold());
            println!("{:-<76}", "");
            for client in &statuses {
                let status_badge = if client.is_attached {
                    let prof = client
                        .attached_profile
                        .as_deref()
                        .map(|p| format!(" (profile: {})", p))
                        .unwrap_or_default();
                    format!("{}{}", "⚡ ATTACHED".green().bold(), prof.dimmed())
                } else if client.config_exists {
                    "○ DETECTED (Ready to attach)".yellow().to_string()
                } else if client.app_installed {
                    "○ INSTALLED (Config not created yet)".cyan().to_string()
                } else {
                    "✕ NOT FOUND".dimmed().to_string()
                };

                println!(
                    "  {} {} [{}]",
                    "•".bold(),
                    client.name.bold(),
                    client.category.dimmed()
                );
                println!("    ID:     {}", client.id.cyan());
                println!("    Status: {}", status_badge);
                println!("    Config: {}", client.config_path.dimmed());
                if client.other_servers_count > 0 {
                    println!(
                        "    Other servers: {}",
                        client.other_servers_count.to_string().cyan()
                    );
                }
                println!();
            }
            println!(
                "Run `{}` to connect Warmplane to a client.",
                "warmplane client attach <id>".bold()
            );
        }
        ClientCommands::Attach {
            client,
            profile,
            config,
        } => {
            let options = AttachOptions {
                profile,
                config_path: Some(config),
                binary_path: None,
            };
            let res = attach_client(&client, &options)?;
            if res.ok {
                println!("{}", format!("✔ {}", res.message).green().bold());
                if let Some(bak) = res.backup_path {
                    println!("  Backup saved to: {}", bak.dimmed());
                }
            } else {
                eprintln!("{}", format!("✖ {}", res.message).red().bold());
            }
        }
        ClientCommands::Detach { client } => {
            let res = detach_client(&client)?;
            if res.ok {
                if res.was_attached {
                    println!("{}", format!("✔ {}", res.message).green().bold());
                } else {
                    println!("{}", format!("ℹ {}", res.message).yellow());
                }
            } else {
                eprintln!("{}", format!("✖ {}", res.message).red().bold());
            }
        }
    }
    Ok(())
}

/// Dispatches `warmplane secret` subcommands for OS Keychain management.
pub async fn handle_secret_command(cmd: crate::models::SecretCommands) -> Result<()> {
    match cmd {
        crate::models::SecretCommands::Set {
            key,
            service,
            value,
        } => {
            let secret_val = if let Some(v) = value {
                v
            } else {
                inquire::Password::new(&format!("Enter secret for '{}' (will be masked):", key))
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .without_confirmation()
                    .prompt()?
            };

            crate::vault::set_os_keychain_secret(&service, &key, &secret_val)?;
            println!(
                "{}",
                format!(
                    "✔ Secret '{}' securely stored in OS Keychain (service: '{}')",
                    key, service
                )
                .green()
                .bold()
            );
            println!(
                "  Use in config: {}",
                format!("\"keychain://{}/{}\"", service, key).cyan()
            );
        }
        crate::models::SecretCommands::Get { key, service } => {
            let val = crate::vault::get_os_keychain_secret(&service, &key)?;
            println!("{}", val);
        }
        crate::models::SecretCommands::Delete { key, service } => {
            crate::vault::delete_os_keychain_secret(&service, &key)?;
            println!(
                "{}",
                format!(
                    "✔ Secret '{}' removed from OS Keychain (service: '{}')",
                    key, service
                )
                .green()
                .bold()
            );
        }
    }
    Ok(())
}

/// Dispatches `warmplane server` subcommands.
pub async fn handle_server_command(cmd: ServerCommands) -> Result<()> {
    match cmd {
        ServerCommands::Add(boxed_args) => {
            let ServerAddArgs {
                name,
                command,
                arg,
                env,
                url,
                bearer_token,
                bearer_env,
                username,
                password,
                password_env,
                client_id,
                auth_server,
                scopes,
                failure_threshold,
                cooldown_ms,
                consecutive_successes,
                auto_restart,
                max_restarts,
                interactive,
                config,
            } = *boxed_args;

            // If interactive requested or no command/url flags supplied, enter interactive flow
            if interactive || (command.is_none() && url.is_none()) {
                interactive_add_server(&config, name)?;
                return Ok(());
            }

            let server_name =
                name.context("Server name is required when not in interactive mode")?;
            let mut server_cfg = ServerConfig::default();

            if let Some(cmd) = command {
                server_cfg.command = Some(cmd);
                let mut all_args = Vec::new();
                for a in arg {
                    all_args.extend(parse_args_string(&a));
                }
                server_cfg.args = all_args;

                let mut env_map = HashMap::new();
                for e in env {
                    if let Some((k, v)) = e.split_once('=') {
                        env_map.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
                server_cfg.env = env_map;
            } else if let Some(endpoint) = url {
                server_cfg.url = Some(endpoint);

                if let Some(token_env) = bearer_env {
                    server_cfg.auth = Some(AuthConfig::Bearer {
                        token: None,
                        token_env: Some(token_env),
                    });
                } else if let Some(token) = bearer_token {
                    server_cfg.auth = Some(AuthConfig::Bearer {
                        token: Some(token),
                        token_env: None,
                    });
                } else if let Some(user) = username {
                    server_cfg.auth = Some(AuthConfig::Basic {
                        username: user,
                        password,
                        password_env,
                    });
                } else if let (Some(cid), Some(auth_url)) = (client_id, auth_server) {
                    let scope_list = scopes
                        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
                        .unwrap_or_default();
                    server_cfg.auth = Some(AuthConfig::Oauth2 {
                        client_id: cid,
                        authorization_server_url: auth_url,
                        scopes: scope_list,
                        client_metadata_url: None,
                    });
                }
            }

            if failure_threshold.is_some()
                || cooldown_ms.is_some()
                || consecutive_successes.is_some()
                || auto_restart.is_some()
                || max_restarts.is_some()
            {
                let mut res_cfg = crate::circuit_breaker::ResilienceConfig::default();
                if let Some(ft) = failure_threshold {
                    res_cfg.failure_threshold = ft;
                }
                if let Some(cd) = cooldown_ms {
                    res_cfg.cooldown_ms = cd;
                }
                if let Some(cs) = consecutive_successes {
                    res_cfg.consecutive_successes = cs;
                }
                if let Some(ar) = auto_restart {
                    res_cfg.auto_restart = ar;
                }
                if let Some(mr) = max_restarts {
                    res_cfg.max_restarts = mr;
                }
                server_cfg.resilience = Some(res_cfg);
            }

            let mut mcp_config = load_or_default_config(&config)?;
            mcp_config
                .mcp_servers
                .insert(server_name.clone(), server_cfg);
            save_config(&config, &mcp_config)?;

            println!(
                "{} Added upstream server '{}' to {}",
                "✔".green().bold(),
                server_name.cyan().bold(),
                config.bold()
            );
        }
        ServerCommands::Remove { name, yes, config } => {
            let mut mcp_config = load_config(&config)?;
            if !mcp_config.mcp_servers.contains_key(&name) {
                anyhow::bail!("Server '{}' not found in {}", name, config);
            }

            if !yes {
                let confirmed = Confirm::new(&format!(
                    "Are you sure you want to remove server '{}'?",
                    name.yellow()
                ))
                .with_default(false)
                .prompt()?;
                if !confirmed {
                    println!("{}", "Removal cancelled.".yellow());
                    return Ok(());
                }
            }

            mcp_config.mcp_servers.remove(&name);
            save_config(&config, &mcp_config)?;
            println!(
                "{} Removed server '{}' from {}",
                "✔".green().bold(),
                name.cyan().bold(),
                config.bold()
            );
        }
        ServerCommands::List { json, config } => {
            let mut mcp_config = load_or_default_config(&config)?;
            if json {
                mcp_config.sanitize_secrets();
                println!("{}", serde_json::to_string_pretty(&mcp_config.mcp_servers)?);
            } else {
                if mcp_config.mcp_servers.is_empty() {
                    println!("{}", "No upstream MCP servers configured.".yellow());
                    println!("Add one with: {}", "warmplane server add".bold());
                    return Ok(());
                }

                println!(
                    "{:<20} {:<8} {:<45} {:<15}",
                    "NAME".bold(),
                    "TYPE".bold(),
                    "TARGET / COMMAND".bold(),
                    "AUTH".bold()
                );
                println!("{}", "-".repeat(90).dimmed());

                for (name, s) in &mcp_config.mcp_servers {
                    let transport = if s.command.is_some() { "stdio" } else { "http" };
                    let target = if let Some(cmd) = &s.command {
                        let full = format!("{} {}", cmd, s.args.join(" "));
                        if full.len() > 42 {
                            format!("{}...", &full[..39])
                        } else {
                            full
                        }
                    } else if let Some(url) = &s.url {
                        if url.len() > 42 {
                            format!("{}...", &url[..39])
                        } else {
                            url.clone()
                        }
                    } else {
                        "-".to_string()
                    };

                    let auth = match &s.auth {
                        Some(AuthConfig::Bearer {
                            token_env: Some(_), ..
                        }) => "Bearer (env)",
                        Some(AuthConfig::Bearer { .. }) => "Bearer (static)",
                        Some(AuthConfig::Basic {
                            password_env: Some(_),
                            ..
                        }) => "Basic (env)",
                        Some(AuthConfig::Basic { .. }) => "Basic (static)",
                        Some(AuthConfig::Oauth2 { .. }) => "OAuth2 PKCE",
                        None => "-",
                    };

                    println!(
                        "{:<20} {:<8} {:<45} {:<15}",
                        name.cyan(),
                        transport,
                        target,
                        auth
                    );
                }
            }
        }
        ServerCommands::Get { name, json, config } => {
            let mut mcp_config = load_config(&config)?;
            mcp_config.sanitize_secrets();
            let server = mcp_config
                .mcp_servers
                .get(&name)
                .with_context(|| format!("Server '{}' not found", name))?;

            if json {
                println!("{}", serde_json::to_string_pretty(server)?);
            } else {
                println!("{} {}", "Server:".bold(), name.cyan().bold());
                if let Some(cmd) = &server.command {
                    println!("  {} stdio", "Transport:".bold());
                    println!("  {} {}", "Command:".bold(), cmd);
                    println!("  {} {:?}", "Arguments:".bold(), server.args);
                    if !server.env.is_empty() {
                        println!("  {} {:?}", "Environment:".bold(), server.env);
                    }
                } else if let Some(url) = &server.url {
                    println!("  {} http", "Transport:".bold());
                    println!("  {} {}", "URL:".bold(), url);
                    if let Some(auth) = &server.auth {
                        println!("  {} {:?}", "Auth:".bold(), auth);
                    }
                }
                if let Some(res) = &server.resilience {
                    println!(
                        "  {} failure_threshold={}, cooldown_ms={}, auto_restart={}, max_restarts={}",
                        "Resilience:".bold(),
                        res.failure_threshold,
                        res.cooldown_ms,
                        res.auto_restart,
                        res.max_restarts
                    );
                }
            }
        }
        ServerCommands::Test { name, config } => {
            let mcp_config = load_config(&config)?;
            let server = mcp_config
                .mcp_servers
                .get(&name)
                .with_context(|| format!("Server '{}' not found", name))?;

            println!(
                "{} Testing connection to '{}'...",
                "•".cyan().bold(),
                name.bold()
            );

            if let Some(cmd) = &server.command {
                let test_proc = std::process::Command::new("which").arg(cmd).output();
                match test_proc {
                    Ok(out) if out.status.success() => {
                        println!(
                            "  {} Executable '{}' located in system PATH",
                            "✔".green(),
                            cmd
                        );
                    }
                    _ => {
                        println!(
                            "  {} Warning: Command '{}' could not be resolved in PATH",
                            "⚠".yellow(),
                            cmd
                        );
                    }
                }
                println!(
                    "  {} Stdio server definition is structurally valid",
                    "✔".green()
                );
            } else if let Some(url) = &server.url {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(5))
                    .build()?;
                match client.get(url).send().await {
                    Ok(resp) => {
                        println!(
                            "  {} HTTP endpoint reachable (Status {})",
                            "✔".green(),
                            resp.status()
                        );
                    }
                    Err(e) => {
                        println!("  {} Could not reach HTTP endpoint: {}", "⚠".yellow(), e);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Dispatches `warmplane config` subcommands.
pub async fn handle_config_command(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Init { config, force } => {
            let path = Path::new(&config);
            if path.exists() && !force {
                let overwrite = Confirm::new(&format!(
                    "File '{}' already exists. Overwrite with default config?",
                    config.yellow()
                ))
                .with_default(false)
                .prompt()?;
                if !overwrite {
                    println!("{}", "Initialization cancelled.".yellow());
                    return Ok(());
                }
            }

            let default_cfg = McpConfig::default();
            save_config(&config, &default_cfg)?;
            println!(
                "{} Initialized clean Warmplane configuration at {}",
                "✔".green().bold(),
                config.bold()
            );
        }
        ConfigCommands::Show { config } => {
            let mut mcp_config = load_or_default_config(&config)?;
            mcp_config.sanitize_secrets();
            println!("{}", serde_json::to_string_pretty(&mcp_config)?);
        }
        ConfigCommands::Import {
            yes,
            from_file,
            config,
        } => {
            if let Some(custom_file) = from_file {
                let path = std::path::PathBuf::from(&custom_file);
                let src = parse_standard_mcp_source("Custom File", path)?;
                let (count, skipped) = import_servers_into_config(&config, src.servers, yes)?;
                println!(
                    "{} Imported {} servers from {} (skipped: {})",
                    "✔".green().bold(),
                    count,
                    custom_file.bold(),
                    skipped.len()
                );
                return Ok(());
            }

            let sources = discover_sources();
            if sources.is_empty() {
                println!(
                    "{}",
                    "No existing Claude Desktop or Cursor configurations found.".yellow()
                );
                return Ok(());
            }

            println!("{}", "=== Discovered MCP Configurations ===".cyan().bold());
            let mut options = Vec::new();
            for src in &sources {
                options.push(format!(
                    "{} ({} servers) - {}",
                    src.name,
                    src.server_count,
                    src.path.display()
                ));
            }

            let selection =
                Select::new("Select configuration source to import:", options).prompt()?;
            let idx = sources
                .iter()
                .position(|s| {
                    format!(
                        "{} ({} servers) - {}",
                        s.name,
                        s.server_count,
                        s.path.display()
                    ) == selection
                })
                .unwrap();

            let chosen = &sources[idx];
            let (count, skipped) =
                import_servers_into_config(&config, chosen.servers.clone(), yes)?;

            println!(
                "{} Successfully imported {} servers from {} into {}",
                "✔".green().bold(),
                count,
                chosen.name.bold(),
                config.bold()
            );
            if !skipped.is_empty() {
                println!(
                    "  {} Skipped existing servers: {}",
                    "ℹ".blue(),
                    skipped.join(", ")
                );
            }
        }
        ConfigCommands::Alias { command } => {
            handle_alias_command(command)?;
        }
        ConfigCommands::Policy { command } => {
            handle_policy_command(command)?;
        }
        ConfigCommands::Resilience { command } => {
            handle_resilience_command(command)?;
        }
        ConfigCommands::Audit { command } => {
            handle_audit_command(command)?;
        }
        ConfigCommands::State { command } => {
            handle_state_command(command)?;
        }
        ConfigCommands::Reload { port, config } => {
            trigger_daemon_reload(port, &config).await?;
        }
    }
    Ok(())
}

/// Triggers dynamic reload on a running daemon instance via REST API.
pub async fn trigger_daemon_reload(port: Option<u16>, config_path: &str) -> Result<()> {
    let resolved_port = crate::config::resolve_client_port(port, config_path)?;
    println!(
        "{} Triggering hot-reload on daemon at port {}...",
        "•".cyan().bold(),
        resolved_port
    );

    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "http://127.0.0.1:{}/v1/config/reload",
            resolved_port
        ))
        .send()
        .await
        .with_context(|| format!("Could not reach daemon at 127.0.0.1:{}", resolved_port))?;

    let status = res.status();
    let body: serde_json::Value = res.json().await.unwrap_or(serde_json::json!({}));

    if status.is_success() {
        println!("{} Hot-reload completed successfully!", "✔".green().bold());
        if let Some(mounted) = body.get("mounted").and_then(|v| v.as_array()) {
            if !mounted.is_empty() {
                let names: Vec<_> = mounted.iter().filter_map(|v| v.as_str()).collect();
                println!("  {} Mounted servers: {}", "✔".green(), names.join(", "));
            }
        }
        if let Some(unmounted) = body.get("unmounted").and_then(|v| v.as_array()) {
            if !unmounted.is_empty() {
                let names: Vec<_> = unmounted.iter().filter_map(|v| v.as_str()).collect();
                println!("  {} Unmounted servers: {}", "✔".yellow(), names.join(", "));
            }
        }
        if let Some(warnings) = body.get("warnings").and_then(|v| v.as_array()) {
            for w in warnings {
                if let Some(msg) = w.as_str() {
                    println!("  {} Warning: {}", "⚠".yellow(), msg);
                }
            }
        }
    } else {
        let err_msg = body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown daemon error");
        println!("{} Hot-reload failed: {}", "✖".red().bold(), err_msg);
    }
    Ok(())
}

fn handle_alias_command(cmd: AliasCommands) -> Result<()> {
    match cmd {
        AliasCommands::Set {
            kind,
            alias,
            target,
            summary,
            description,
            config,
        } => {
            let mut mcp_config = load_or_default_config(&config)?;
            let alias_target = if summary.is_some() || description.is_some() {
                crate::config::AliasTarget::Detailed {
                    target: target.clone(),
                    summary,
                    description,
                }
            } else {
                crate::config::AliasTarget::Simple(target.clone())
            };

            match kind.to_lowercase().as_str() {
                "tool" | "capability" | "cap" => {
                    mcp_config
                        .capability_aliases
                        .insert(alias.clone(), alias_target);
                }
                "resource" | "res" => {
                    mcp_config
                        .resource_aliases
                        .insert(alias.clone(), alias_target);
                }
                "prompt" => {
                    mcp_config
                        .prompt_aliases
                        .insert(alias.clone(), alias_target);
                }
                _ => {
                    anyhow::bail!(
                        "Invalid alias kind '{}'. Expected 'tool', 'resource', or 'prompt'",
                        kind
                    );
                }
            }
            save_config(&config, &mcp_config)?;
            println!(
                "{} Set {} alias '{}' -> '{}'",
                "✔".green().bold(),
                kind,
                alias.bold(),
                target.bold()
            );
        }
        AliasCommands::Remove {
            kind,
            alias,
            config,
        } => {
            let mut mcp_config = load_config(&config)?;
            match kind.to_lowercase().as_str() {
                "tool" | "capability" | "cap" => {
                    mcp_config.capability_aliases.remove(&alias);
                }
                "resource" | "res" => {
                    mcp_config.resource_aliases.remove(&alias);
                }
                "prompt" => {
                    mcp_config.prompt_aliases.remove(&alias);
                }
                _ => {
                    anyhow::bail!(
                        "Invalid alias kind '{}'. Expected 'tool', 'resource', or 'prompt'",
                        kind
                    );
                }
            }
            save_config(&config, &mcp_config)?;
            println!(
                "{} Removed {} alias '{}'",
                "✔".green().bold(),
                kind,
                alias.bold()
            );
        }
        AliasCommands::List { config } => {
            let mcp_config = load_or_default_config(&config)?;
            println!("{}", "=== Warmplane Aliases ===".cyan().bold());
            println!("{}", "Capability / Tool Aliases:".bold());
            if mcp_config.capability_aliases.is_empty() {
                println!("  (none)");
            } else {
                for (a, t) in &mcp_config.capability_aliases {
                    if let Some(s) = t.summary() {
                        println!("  {} -> {} ({})", a.cyan(), t.target(), s.dimmed());
                    } else {
                        println!("  {} -> {}", a.cyan(), t.target());
                    }
                }
            }

            println!("\n{}", "Resource Aliases:".bold());
            if mcp_config.resource_aliases.is_empty() {
                println!("  (none)");
            } else {
                for (a, t) in &mcp_config.resource_aliases {
                    if let Some(s) = t.summary() {
                        println!("  {} -> {} ({})", a.cyan(), t.target(), s.dimmed());
                    } else {
                        println!("  {} -> {}", a.cyan(), t.target());
                    }
                }
            }

            println!("\n{}", "Prompt Aliases:".bold());
            if mcp_config.prompt_aliases.is_empty() {
                println!("  (none)");
            } else {
                for (a, t) in &mcp_config.prompt_aliases {
                    if let Some(s) = t.summary() {
                        println!("  {} -> {} ({})", a.cyan(), t.target(), s.dimmed());
                    } else {
                        println!("  {} -> {}", a.cyan(), t.target());
                    }
                }
            }
        }
    }
    Ok(())
}

/// Dispatches `warmplane config resilience` subcommands.
pub fn handle_resilience_command(cmd: crate::models::ResilienceCommands) -> Result<()> {
    match cmd {
        crate::models::ResilienceCommands::Set {
            failure_threshold,
            cooldown_ms,
            consecutive_successes,
            auto_restart,
            max_restarts,
            config,
        } => {
            let mut mcp_config = load_or_default_config(&config)?;
            let mut res_cfg = mcp_config.resilience.unwrap_or_default();

            if let Some(ft) = failure_threshold {
                res_cfg.failure_threshold = ft;
            }
            if let Some(cd) = cooldown_ms {
                res_cfg.cooldown_ms = cd;
            }
            if let Some(cs) = consecutive_successes {
                res_cfg.consecutive_successes = cs;
            }
            if let Some(ar) = auto_restart {
                res_cfg.auto_restart = ar;
            }
            if let Some(mr) = max_restarts {
                res_cfg.max_restarts = mr;
            }

            mcp_config.resilience = Some(res_cfg.clone());
            save_config(&config, &mcp_config)?;

            println!(
                "{} Updated global resilience configuration in {}",
                "✔".green().bold(),
                config.bold()
            );
            println!(
                "  • Failure Threshold: {}",
                res_cfg.failure_threshold.to_string().cyan()
            );
            println!("  • Cooldown: {}ms", res_cfg.cooldown_ms.to_string().cyan());
            println!(
                "  • Consecutive Successes: {}",
                res_cfg.consecutive_successes.to_string().cyan()
            );
            println!(
                "  • Auto-Restart: {}",
                res_cfg.auto_restart.to_string().cyan()
            );
            println!(
                "  • Max Restarts: {}",
                res_cfg.max_restarts.to_string().cyan()
            );
        }
        crate::models::ResilienceCommands::Show { config } => {
            let mcp_config = load_or_default_config(&config)?;
            println!(
                "{}",
                "=== Warmplane Global Resilience Settings ===".cyan().bold()
            );
            match &mcp_config.resilience {
                Some(r) => {
                    println!(
                        "  • Failure Threshold: {}",
                        r.failure_threshold.to_string().cyan()
                    );
                    println!("  • Cooldown: {}ms", r.cooldown_ms.to_string().cyan());
                    println!(
                        "  • Consecutive Successes: {}",
                        r.consecutive_successes.to_string().cyan()
                    );
                    println!("  • Auto-Restart: {}", r.auto_restart.to_string().cyan());
                    println!("  • Max Restarts: {}", r.max_restarts.to_string().cyan());
                }
                None => {
                    println!("  (using system defaults)");
                    let d = crate::circuit_breaker::ResilienceConfig::default();
                    println!("  • Failure Threshold (default): {}", d.failure_threshold);
                    println!("  • Cooldown (default): {}ms", d.cooldown_ms);
                    println!(
                        "  • Consecutive Successes (default): {}",
                        d.consecutive_successes
                    );
                    println!("  • Auto-Restart (default): {}", d.auto_restart);
                    println!("  • Max Restarts (default): {}", d.max_restarts);
                }
            }
        }
    }
    Ok(())
}

/// Dispatches `warmplane config audit` subcommands.
pub fn handle_audit_command(cmd: crate::models::AuditCommands) -> Result<()> {
    match cmd {
        crate::models::AuditCommands::Set {
            enabled,
            file_path,
            buffer_capacity,
            flush_interval_ms,
            max_batch_size,
            siem_webhook_url,
            siem_webhook_auth,
            siem_splunk_url,
            siem_splunk_token,
            config,
        } => {
            let mut mcp_config = load_or_default_config(&config)?;
            let mut audit_cfg = mcp_config
                .audit
                .unwrap_or_else(|| crate::config::AuditConfig {
                    enabled: true,
                    file_path: Some("warmplane_audit.jsonl".to_string()),
                    hmac_key: None,
                    hmac_key_env: None,
                    buffer_capacity: Some(10000),
                    flush_interval_ms: Some(250),
                    max_batch_size: Some(100),
                    siem: None,
                });

            if let Some(en) = enabled {
                audit_cfg.enabled = en;
            }
            if let Some(fp) = file_path {
                audit_cfg.file_path = Some(fp);
            }
            if let Some(bc) = buffer_capacity {
                audit_cfg.buffer_capacity = Some(bc);
            }
            if let Some(fi) = flush_interval_ms {
                audit_cfg.flush_interval_ms = Some(fi);
            }
            if let Some(mbs) = max_batch_size {
                audit_cfg.max_batch_size = Some(mbs);
            }

            if let Some(wh_url) = siem_webhook_url {
                let mut siem = audit_cfg.siem.unwrap_or_default();
                siem.targets.push(crate::config::SiemTargetConfig::Webhook {
                    url: wh_url,
                    auth_header: siem_webhook_auth,
                    headers: HashMap::new(),
                });
                audit_cfg.siem = Some(siem);
            }

            if let Some(splunk_url) = siem_splunk_url {
                let token = siem_splunk_token.unwrap_or_default();
                let mut siem = audit_cfg.siem.unwrap_or_default();
                siem.targets
                    .push(crate::config::SiemTargetConfig::SplunkHec {
                        url: splunk_url,
                        token,
                        index: None,
                        source: Some("warmplane".to_string()),
                    });
                audit_cfg.siem = Some(siem);
            }

            mcp_config.audit = Some(audit_cfg.clone());
            save_config(&config, &mcp_config)?;

            println!(
                "{} Updated WORM audit configuration in {}",
                "✔".green().bold(),
                config.bold()
            );
            println!("  • Enabled: {}", audit_cfg.enabled.to_string().cyan());
            println!(
                "  • File Path: {}",
                audit_cfg.file_path.as_deref().unwrap_or("none").cyan()
            );
        }
        crate::models::AuditCommands::Show { config } => {
            let mcp_config = load_or_default_config(&config)?;
            println!(
                "{}",
                "=== Warmplane WORM Audit & SIEM Settings ===".cyan().bold()
            );
            match &mcp_config.audit {
                Some(a) => {
                    println!("  • Enabled: {}", a.enabled.to_string().cyan());
                    println!(
                        "  • File Path: {}",
                        a.file_path.as_deref().unwrap_or("none").cyan()
                    );
                    println!(
                        "  • Buffer Capacity: {}",
                        a.buffer_capacity.unwrap_or(10000).to_string().cyan()
                    );
                    println!(
                        "  • Flush Interval: {}ms",
                        a.flush_interval_ms.unwrap_or(250).to_string().cyan()
                    );
                    println!(
                        "  • Max Batch Size: {}",
                        a.max_batch_size.unwrap_or(100).to_string().cyan()
                    );
                    if let Some(ref s) = a.siem {
                        println!("  • SIEM Targets ({} configured):", s.targets.len());
                        for t in &s.targets {
                            match t {
                                crate::config::SiemTargetConfig::Webhook { url, .. } => {
                                    println!("    - Webhook: {}", url.cyan());
                                }
                                crate::config::SiemTargetConfig::SplunkHec { url, .. } => {
                                    println!("    - Splunk HEC: {}", url.cyan());
                                }
                            }
                        }
                    }
                }
                None => {
                    println!("  (audit subsystem disabled / unconfigured)");
                }
            }
        }
    }
    Ok(())
}

/// Dispatches `warmplane config state` subcommands.
pub fn handle_state_command(cmd: StateCommands) -> Result<()> {
    match cmd {
        StateCommands::Set {
            enabled,
            dir,
            config,
        } => {
            let mut mcp_config = load_or_default_config(&config)?;
            let mut state_cfg = mcp_config.state.unwrap_or_default();

            if let Some(en) = enabled {
                state_cfg.enabled = en;
            }
            if let Some(d) = dir {
                state_cfg.dir = Some(d);
            }

            mcp_config.state = Some(state_cfg.clone());
            save_config(&config, &mcp_config)?;

            println!(
                "{} Updated persistent state configuration in {}",
                "✔".green().bold(),
                config.bold()
            );
            println!("  • Enabled: {}", state_cfg.enabled.to_string().cyan());
            println!(
                "  • Directory: {}",
                state_cfg
                    .dir
                    .as_deref()
                    .unwrap_or(".warmplane/state")
                    .cyan()
            );
        }
        StateCommands::Show { config } => {
            let mcp_config = load_or_default_config(&config)?;
            println!(
                "{}",
                "=== Warmplane Persistent State Settings ===".cyan().bold()
            );
            match &mcp_config.state {
                Some(s) => {
                    println!("  • Enabled: {}", s.enabled.to_string().cyan());
                    println!(
                        "  • Directory: {}",
                        s.dir.as_deref().unwrap_or(".warmplane/state").cyan()
                    );
                }
                None => {
                    println!("  (using default persistent state configuration)");
                    println!("  • Enabled: {}", "true".cyan());
                    println!("  • Directory: {}", ".warmplane/state".cyan());
                }
            }
        }
    }
    Ok(())
}

fn handle_policy_command(cmd: PolicyCommands) -> Result<()> {
    match cmd {
        PolicyCommands::Allow { patterns, config } => {
            let mut mcp_config = load_or_default_config(&config)?;
            let mut policy = mcp_config.policy.unwrap_or_default();
            policy.allow.extend(patterns.clone());
            mcp_config.policy = Some(policy);
            save_config(&config, &mcp_config)?;
            println!(
                "{} Added allow patterns: {:?}",
                "✔".green().bold(),
                patterns
            );
        }
        PolicyCommands::Deny { patterns, config } => {
            let mut mcp_config = load_or_default_config(&config)?;
            let mut policy = mcp_config.policy.unwrap_or_default();
            policy.deny.extend(patterns.clone());
            mcp_config.policy = Some(policy);
            save_config(&config, &mcp_config)?;
            println!("{} Added deny patterns: {:?}", "✔".green().bold(), patterns);
        }
        PolicyCommands::RequireApproval { patterns, config } => {
            let mut mcp_config = load_or_default_config(&config)?;
            let mut policy = mcp_config.policy.unwrap_or_default();
            policy.require_approval.extend(patterns.clone());
            mcp_config.policy = Some(policy);
            save_config(&config, &mcp_config)?;
            println!(
                "{} Added human approval patterns: {:?}",
                "✔".green().bold(),
                patterns
            );
        }
        PolicyCommands::Redact { keys, config } => {
            let mut mcp_config = load_or_default_config(&config)?;
            let mut policy = mcp_config.policy.unwrap_or_default();
            policy.redact_keys.extend(keys.clone());
            mcp_config.policy = Some(policy);
            save_config(&config, &mcp_config)?;
            println!("{} Added redact keys: {:?}", "✔".green().bold(), keys);
        }
        PolicyCommands::Show { config } => {
            let mcp_config = load_or_default_config(&config)?;
            if let Some(p) = mcp_config.policy {
                println!("{}", "=== Security Policies ===".cyan().bold());
                println!("  {} {:?}", "Allow:".bold(), p.allow);
                println!("  {} {:?}", "Deny:".bold(), p.deny);
                println!("  {} {:?}", "Require Approval:".bold(), p.require_approval);
                println!("  {} {:?}", "Redact:".bold(), p.redact_keys);
                if let Some(wh) = p.webhook {
                    println!("  {} {}", "Webhook:".bold(), wh.url);
                }
            } else {
                println!(
                    "{}",
                    "No security policies configured (open mode).".yellow()
                );
            }
        }
    }
    Ok(())
}

/// Handles Human-in-the-Loop approval CLI commands (`warmplane approvals ...`).
pub async fn handle_approvals_command(cmd: crate::models::ApprovalCommands) -> Result<()> {
    match cmd {
        crate::models::ApprovalCommands::List { port, config } => {
            let resolved_port = crate::config::resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let res = client
                .get(format!("http://127.0.0.1:{}/v1/approvals", resolved_port))
                .send()
                .await
                .with_context(|| {
                    format!("Could not reach daemon at 127.0.0.1:{}", resolved_port)
                })?;

            let body: serde_json::Value = res.json().await?;
            if let Some(approvals) = body.get("approvals").and_then(|v| v.as_array()) {
                println!("{}", "=== Human-in-the-Loop Approvals ===".cyan().bold());
                if approvals.is_empty() {
                    println!("  (no approval tickets)");
                } else {
                    for appr in approvals {
                        let id = appr.get("id").and_then(|v| v.as_str()).unwrap_or("-");
                        let cap = appr
                            .get("capability_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("-");
                        let status = appr
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let status_display = match status {
                            "pending" => status.yellow().bold(),
                            "approved" => status.green().bold(),
                            "rejected" => status.red().bold(),
                            _ => status.normal(),
                        };
                        println!("  • {} | {} | {}", id.bold(), cap.cyan(), status_display);
                    }
                }
            }
        }
        crate::models::ApprovalCommands::Get { id, port, config } => {
            let resolved_port = crate::config::resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let res = client
                .get(format!(
                    "http://127.0.0.1:{}/v1/approvals/{}",
                    resolved_port, id
                ))
                .send()
                .await
                .with_context(|| {
                    format!("Could not reach daemon at 127.0.0.1:{}", resolved_port)
                })?;

            let body: serde_json::Value = res.json().await?;
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
        crate::models::ApprovalCommands::Approve {
            id,
            operator,
            modified_args,
            port,
            config,
        } => {
            let resolved_port = crate::config::resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let mod_val: Option<serde_json::Value> = match modified_args {
                Some(ref s) => Some(serde_json::from_str(s)?),
                None => None,
            };

            let res = client
                .post(format!(
                    "http://127.0.0.1:{}/v1/approvals/{}/approve",
                    resolved_port, id
                ))
                .json(&serde_json::json!({
                    "operator": operator,
                    "modified_args": mod_val
                }))
                .send()
                .await
                .with_context(|| {
                    format!("Could not reach daemon at 127.0.0.1:{}", resolved_port)
                })?;

            let status = res.status();
            let body: serde_json::Value = res.json().await?;
            if status.is_success() {
                println!(
                    "{} Ticket '{}' approved by {}",
                    "✔".green().bold(),
                    id.bold(),
                    operator.cyan()
                );
            } else {
                let err = body
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Failed to approve ticket");
                println!("{} {}", "✖".red().bold(), err);
            }
        }
        crate::models::ApprovalCommands::Reject {
            id,
            operator,
            reason,
            port,
            config,
        } => {
            let resolved_port = crate::config::resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let res = client
                .post(format!(
                    "http://127.0.0.1:{}/v1/approvals/{}/reject",
                    resolved_port, id
                ))
                .json(&serde_json::json!({
                    "operator": operator,
                    "reason": reason
                }))
                .send()
                .await
                .with_context(|| {
                    format!("Could not reach daemon at 127.0.0.1:{}", resolved_port)
                })?;

            let status = res.status();
            let body: serde_json::Value = res.json().await?;
            if status.is_success() {
                println!(
                    "{} Ticket '{}' rejected by {}",
                    "✔".green().bold(),
                    id.bold(),
                    operator.cyan()
                );
            } else {
                let err = body
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Failed to reject ticket");
                println!("{} {}", "✖".red().bold(), err);
            }
        }
    }
    Ok(())
}

/// Dispatches `warmplane idempotency` subcommands.
pub async fn handle_idempotency_command(cmd: crate::models::IdempotencyCommands) -> Result<()> {
    match cmd {
        crate::models::IdempotencyCommands::List {
            port,
            config,
            limit,
            offset,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let url = format!(
                "http://127.0.0.1:{}/v1/idempotency/records?limit={}&offset={}",
                resolved_port, limit, offset
            );
            let client = reqwest::Client::new();
            let res = client.get(&url).send().await?;
            let body: serde_json::Value = res.json().await?;
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
        crate::models::IdempotencyCommands::Get { key, port, config } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let url = format!(
                "http://127.0.0.1:{}/v1/idempotency/records/{}",
                resolved_port, key
            );
            let client = reqwest::Client::new();
            let res = client.get(&url).send().await?;
            let body: serde_json::Value = res.json().await?;
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
    }
    Ok(())
}

/// Dispatches `warmplane tasks` subcommands.
pub async fn handle_task_command(cmd: crate::models::TaskCommands) -> Result<()> {
    match cmd {
        crate::models::TaskCommands::List { port, config } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let res = client
                .get(format!("http://127.0.0.1:{}/v1/tasks", resolved_port))
                .send()
                .await
                .with_context(|| {
                    format!("Could not reach daemon at 127.0.0.1:{}", resolved_port)
                })?;

            let body: serde_json::Value = res.json().await?;
            if let Some(tasks) = body.get("tasks").and_then(|t| t.as_array()) {
                if tasks.is_empty() {
                    println!("{}", "No active or recorded tasks.".dimmed());
                    return Ok(());
                }
                println!(
                    "{:<26} {:<16} {:<24} {:<8}",
                    "TASK ID".bold(),
                    "STATUS".bold(),
                    "CREATED AT".bold(),
                    "TTL (s)".bold()
                );
                println!("{}", "─".repeat(78).dimmed());
                for task in tasks {
                    let id = task.get("taskId").and_then(|v| v.as_str()).unwrap_or("-");
                    let status = task.get("status").and_then(|v| v.as_str()).unwrap_or("-");
                    let created = task
                        .get("createdAt")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
                    let ttl_s = task
                        .get("ttlMs")
                        .and_then(|v| v.as_u64())
                        .map(|ms| (ms / 1000).to_string())
                        .unwrap_or_else(|| "∞".to_string());

                    let status_styled = match status {
                        "working" => status.yellow(),
                        "input_required" => status.cyan().bold(),
                        "completed" => status.green(),
                        "failed" => status.red(),
                        "cancelled" => status.dimmed(),
                        _ => status.normal(),
                    };

                    println!(
                        "{:<26} {:<16} {:<24} {:<8}",
                        id, status_styled, created, ttl_s
                    );
                }
            } else {
                println!("{}", serde_json::to_string_pretty(&body)?);
            }
        }
        crate::models::TaskCommands::Get { id, port, config } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let res = client
                .get(format!(
                    "http://127.0.0.1:{}/v1/tasks/{}",
                    resolved_port, id
                ))
                .send()
                .await
                .with_context(|| {
                    format!("Could not reach daemon at 127.0.0.1:{}", resolved_port)
                })?;

            let body: serde_json::Value = res.json().await?;
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
        crate::models::TaskCommands::Update {
            id,
            responses,
            port,
            config,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let parsed_responses: serde_json::Value = serde_json::from_str(&responses)
                .context("Responses must be a valid JSON object")?;

            let client = reqwest::Client::new();
            let res = client
                .post(format!(
                    "http://127.0.0.1:{}/v1/tasks/{}/update",
                    resolved_port, id
                ))
                .json(&serde_json::json!({ "inputResponses": parsed_responses }))
                .send()
                .await
                .with_context(|| {
                    format!("Could not reach daemon at 127.0.0.1:{}", resolved_port)
                })?;

            let status = res.status();
            let body: serde_json::Value = res.json().await?;
            if status.is_success() {
                println!(
                    "{} Task '{}' updated with input responses",
                    "✔".green().bold(),
                    id.bold()
                );
            } else {
                let err = body
                    .get("error")
                    .and_then(|v| v.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Failed to update task");
                println!("{} {}", "✖".red().bold(), err);
            }
        }
        crate::models::TaskCommands::Cancel {
            id,
            reason,
            port,
            config,
        } => {
            let resolved_port = resolve_client_port(port, &config)?;
            let client = reqwest::Client::new();
            let res = client
                .post(format!(
                    "http://127.0.0.1:{}/v1/tasks/{}/cancel",
                    resolved_port, id
                ))
                .json(&serde_json::json!({ "reason": reason }))
                .send()
                .await
                .with_context(|| {
                    format!("Could not reach daemon at 127.0.0.1:{}", resolved_port)
                })?;

            let status = res.status();
            let body: serde_json::Value = res.json().await?;
            if status.is_success() {
                println!("{} Task '{}' cancelled", "✔".green().bold(), id.bold());
            } else {
                let err = body
                    .get("error")
                    .and_then(|v| v.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Failed to cancel task");
                println!("{} {}", "✖".red().bold(), err);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_lifecycle() {
        let temp_dir =
            std::env::temp_dir().join(format!("warmplane_alias_test_{}", std::process::id()));
        let config_file = temp_dir.join("mcp_servers.json");
        let cfg_str = config_file.to_str().unwrap().to_string();

        handle_alias_command(AliasCommands::Set {
            kind: "tool".to_string(),
            alias: "git-commit".to_string(),
            target: "github.create_commit".to_string(),
            summary: Some("Create a git commit".to_string()),
            description: None,
            config: cfg_str.clone(),
        })
        .unwrap();

        let cfg = load_config(&cfg_str).unwrap();
        assert_eq!(
            cfg.capability_aliases.get("git-commit").unwrap().target(),
            "github.create_commit"
        );
        assert_eq!(
            cfg.capability_aliases.get("git-commit").unwrap().summary(),
            Some("Create a git commit")
        );

        handle_alias_command(AliasCommands::Remove {
            kind: "tool".to_string(),
            alias: "git-commit".to_string(),
            config: cfg_str.clone(),
        })
        .unwrap();

        let cfg = load_config(&cfg_str).unwrap();
        assert!(!cfg.capability_aliases.contains_key("git-commit"));

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_policy_mutation() {
        let temp_dir =
            std::env::temp_dir().join(format!("warmplane_policy_test_{}", std::process::id()));
        let config_file = temp_dir.join("mcp_servers.json");
        let cfg_str = config_file.to_str().unwrap().to_string();

        handle_policy_command(PolicyCommands::Allow {
            patterns: vec!["github.*".to_string(), "fetch.*".to_string()],
            config: cfg_str.clone(),
        })
        .unwrap();

        handle_policy_command(PolicyCommands::Deny {
            patterns: vec!["filesystem.write*".to_string()],
            config: cfg_str.clone(),
        })
        .unwrap();

        let cfg = load_config(&cfg_str).unwrap();
        let policy = cfg.policy.unwrap();
        assert_eq!(policy.allow, vec!["github.*", "fetch.*"]);
        assert_eq!(policy.deny, vec!["filesystem.write*"]);

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_resilience_mutation() {
        let temp_dir =
            std::env::temp_dir().join(format!("warmplane_resilience_test_{}", std::process::id()));
        let config_file = temp_dir.join("mcp_servers.json");
        let cfg_str = config_file.to_str().unwrap().to_string();

        handle_resilience_command(crate::models::ResilienceCommands::Set {
            failure_threshold: Some(5),
            cooldown_ms: Some(45000),
            consecutive_successes: Some(3),
            auto_restart: Some(false),
            max_restarts: Some(10),
            config: cfg_str.clone(),
        })
        .unwrap();

        let cfg = load_config(&cfg_str).unwrap();
        let res = cfg.resilience.unwrap();
        assert_eq!(res.failure_threshold, 5);
        assert_eq!(res.cooldown_ms, 45000);
        assert_eq!(res.consecutive_successes, 3);
        assert!(!res.auto_restart);
        assert_eq!(res.max_restarts, 10);

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_audit_mutation() {
        let temp_dir =
            std::env::temp_dir().join(format!("warmplane_audit_test_{}", std::process::id()));
        let config_file = temp_dir.join("mcp_servers.json");
        let cfg_str = config_file.to_str().unwrap().to_string();

        handle_audit_command(crate::models::AuditCommands::Set {
            enabled: Some(true),
            file_path: Some("custom_audit.jsonl".to_string()),
            buffer_capacity: Some(5000),
            flush_interval_ms: Some(500),
            max_batch_size: Some(50),
            siem_webhook_url: Some("https://siem.test/events".to_string()),
            siem_webhook_auth: Some("Bearer test-token".to_string()),
            siem_splunk_url: None,
            siem_splunk_token: None,
            config: cfg_str.clone(),
        })
        .unwrap();

        let cfg = load_config(&cfg_str).unwrap();
        let audit = cfg.audit.unwrap();
        assert!(audit.enabled);
        assert_eq!(audit.file_path, Some("custom_audit.jsonl".to_string()));
        assert_eq!(audit.buffer_capacity, Some(5000));
        assert_eq!(audit.flush_interval_ms, Some(500));
        assert_eq!(audit.max_batch_size, Some(50));
        assert_eq!(audit.siem.unwrap().targets.len(), 1);

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_state_mutation() {
        let temp_dir =
            std::env::temp_dir().join(format!("warmplane_state_test_{}", std::process::id()));
        let config_file = temp_dir.join("mcp_servers.json");
        let cfg_str = config_file.to_str().unwrap().to_string();

        handle_state_command(crate::models::StateCommands::Set {
            enabled: Some(true),
            dir: Some(".custom_state".to_string()),
            config: cfg_str.clone(),
        })
        .unwrap();

        let cfg = load_config(&cfg_str).unwrap();
        let state = cfg.state.unwrap();
        assert!(state.enabled);
        assert_eq!(state.dir, Some(".custom_state".to_string()));

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
