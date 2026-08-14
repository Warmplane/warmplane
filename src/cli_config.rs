// Rust guideline compliant 2026-08-14

//! Handlers for `warmplane server` and `warmplane config` CLI commands.

use anyhow::{Context, Result};
use colored::Colorize;
use inquire::{Confirm, Select};
use std::collections::HashMap;
use std::path::Path;

use crate::config::{
    load_config, load_or_default_config, save_config, AuthConfig, McpConfig, ServerConfig,
};
use crate::config_import::{
    discover_sources, import_servers_into_config, parse_standard_mcp_source,
};
use crate::interactive::{interactive_add_server, parse_args_string};
use crate::models::{AliasCommands, ConfigCommands, PolicyCommands, ServerAddArgs, ServerCommands};

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
            let mcp_config = load_or_default_config(&config)?;
            if json {
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
            let mcp_config = load_config(&config)?;
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
            let mcp_config = load_or_default_config(&config)?;
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
    }
    Ok(())
}

fn handle_alias_command(cmd: AliasCommands) -> Result<()> {
    match cmd {
        AliasCommands::Set {
            kind,
            alias,
            target,
            config,
        } => {
            let mut mcp_config = load_or_default_config(&config)?;
            match kind.to_lowercase().as_str() {
                "tool" | "capability" | "cap" => {
                    mcp_config
                        .capability_aliases
                        .insert(alias.clone(), target.clone());
                }
                "resource" | "res" => {
                    mcp_config
                        .resource_aliases
                        .insert(alias.clone(), target.clone());
                }
                "prompt" => {
                    mcp_config
                        .prompt_aliases
                        .insert(alias.clone(), target.clone());
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
                    println!("  {} -> {}", a.cyan(), t);
                }
            }

            println!("\n{}", "Resource Aliases:".bold());
            if mcp_config.resource_aliases.is_empty() {
                println!("  (none)");
            } else {
                for (a, t) in &mcp_config.resource_aliases {
                    println!("  {} -> {}", a.cyan(), t);
                }
            }

            println!("\n{}", "Prompt Aliases:".bold());
            if mcp_config.prompt_aliases.is_empty() {
                println!("  (none)");
            } else {
                for (a, t) in &mcp_config.prompt_aliases {
                    println!("  {} -> {}", a.cyan(), t);
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
                println!("  {} {:?}", "Redact:".bold(), p.redact_keys);
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
            config: cfg_str.clone(),
        })
        .unwrap();

        let cfg = load_config(&cfg_str).unwrap();
        assert_eq!(
            cfg.capability_aliases.get("git-commit").unwrap(),
            "github.create_commit"
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
}
