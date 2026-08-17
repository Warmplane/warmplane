// Rust guideline compliant 2026-08-14

//! Interactive terminal wizards for configuring MCP servers and policies.

use anyhow::Result;
use colored::Colorize;
use inquire::{Confirm, Select, Text};

use crate::config::{load_or_default_config, save_config, AuthConfig, ServerConfig};

/// Runs an interactive wizard to configure and add or update an upstream server.
///
/// # Arguments
/// * `config_path` - Path to the Warmplane JSON config file.
/// * `preset_name` - Optional pre-filled server name.
///
/// # Errors
/// Returns an error if user aborts prompt or saving config fails.
pub fn interactive_add_server(config_path: &str, preset_name: Option<String>) -> Result<()> {
    println!(
        "{}",
        "=== Warmplane Interactive Server Setup ===".cyan().bold()
    );

    let mut config = load_or_default_config(config_path)?;

    let name = match preset_name {
        Some(n) => n,
        None => Text::new("Server identifier (e.g. github, filesystem, weather):")
            .with_validator(|input: &str| {
                if input.trim().is_empty() {
                    Ok(inquire::validator::Validation::Invalid(
                        "Server identifier cannot be empty".into(),
                    ))
                } else {
                    Ok(inquire::validator::Validation::Valid)
                }
            })
            .prompt()?,
    };

    let name = name.trim().to_string();

    if config.mcp_servers.contains_key(&name) {
        let overwrite = Confirm::new(&format!(
            "Server '{}' already exists. Overwrite?",
            name.yellow()
        ))
        .with_default(false)
        .prompt()?;
        if !overwrite {
            println!("{}", "Operation cancelled.".yellow());
            return Ok(());
        }
    }

    let transport_options = vec![
        "Stdio (Subprocess executable / CLI tool)",
        "HTTP / SSE (Remote endpoint or streamable HTTP)",
    ];

    let transport_choice = Select::new("Transport type:", transport_options).prompt()?;

    let mut server = ServerConfig::default();

    if transport_choice.starts_with("Stdio") {
        let command = Text::new("Executable command (e.g. npx, uvx, node, python):")
            .with_validator(|input: &str| {
                if input.trim().is_empty() {
                    Ok(inquire::validator::Validation::Invalid(
                        "Command cannot be empty".into(),
                    ))
                } else {
                    Ok(inquire::validator::Validation::Valid)
                }
            })
            .prompt()?;
        server.command = Some(command.trim().to_string());

        let args_raw = Text::new("Arguments (separated by space or comma, leave empty if none):")
            .with_placeholder("-y @modelcontextprotocol/server-github")
            .prompt()?;

        if !args_raw.trim().is_empty() {
            server.args = parse_args_string(&args_raw);
        }

        let add_env = Confirm::new("Configure environment variables for this process?")
            .with_default(false)
            .prompt()?;

        if add_env {
            loop {
                let env_key =
                    Text::new("Environment variable key (or press Enter to finish):").prompt()?;
                if env_key.trim().is_empty() {
                    break;
                }
                let env_val = Text::new(&format!("Value for {}:", env_key.trim())).prompt()?;
                server
                    .env
                    .insert(env_key.trim().to_string(), env_val.trim().to_string());

                let add_more = Confirm::new("Add another environment variable?")
                    .with_default(false)
                    .prompt()?;
                if !add_more {
                    break;
                }
            }
        }
    } else {
        let url = Text::new("Remote URL endpoint (e.g. https://mcp.example.com/sse):")
            .with_validator(|input: &str| {
                if input.starts_with("http://") || input.starts_with("https://") {
                    Ok(inquire::validator::Validation::Valid)
                } else {
                    Ok(inquire::validator::Validation::Invalid(
                        "URL must start with http:// or https://".into(),
                    ))
                }
            })
            .prompt()?;
        server.url = Some(url.trim().to_string());

        let auth_options = vec![
            "None / Public",
            "Bearer Token (Environment Variable)",
            "Bearer Token (Static String)",
            "HTTP Basic Auth",
            "OAuth2 PKCE",
        ];

        let auth_choice = Select::new("Authentication method:", auth_options).prompt()?;

        match auth_choice {
            "Bearer Token (Environment Variable)" => {
                let env_var =
                    Text::new("Environment variable containing token (e.g. MCP_API_KEY):")
                        .with_validator(|s: &str| {
                            if s.trim().is_empty() {
                                Ok(inquire::validator::Validation::Invalid(
                                    "Variable name required".into(),
                                ))
                            } else {
                                Ok(inquire::validator::Validation::Valid)
                            }
                        })
                        .prompt()?;
                server.auth = Some(AuthConfig::Bearer {
                    token: None,
                    token_env: Some(env_var.trim().to_string()),
                });
            }
            "Bearer Token (Static String)" => {
                let token = Text::new("Bearer token string:").prompt()?;
                server.auth = Some(AuthConfig::Bearer {
                    token: Some(token.trim().to_string()),
                    token_env: None,
                });
            }
            "HTTP Basic Auth" => {
                let username = Text::new("Username:").prompt()?;
                let use_env = Confirm::new("Read password from environment variable?")
                    .with_default(true)
                    .prompt()?;
                if use_env {
                    let env_var = Text::new("Environment variable name:").prompt()?;
                    server.auth = Some(AuthConfig::Basic {
                        username: username.trim().to_string(),
                        password: None,
                        password_env: Some(env_var.trim().to_string()),
                    });
                } else {
                    let pw = Text::new("Password:").prompt()?;
                    server.auth = Some(AuthConfig::Basic {
                        username: username.trim().to_string(),
                        password: Some(pw.trim().to_string()),
                        password_env: None,
                    });
                }
            }
            "OAuth2 PKCE" => {
                let client_id = Text::new("OAuth2 Client ID:").prompt()?;
                let auth_server = Text::new("Authorization Server URL:").prompt()?;
                let scopes_raw = Text::new("Scopes (comma-separated, optional):").prompt()?;
                let scopes = if scopes_raw.trim().is_empty() {
                    vec![]
                } else {
                    scopes_raw
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                };
                server.auth = Some(AuthConfig::Oauth2 {
                    client_id: client_id.trim().to_string(),
                    authorization_server_url: auth_server.trim().to_string(),
                    scopes,
                    client_metadata_url: None,
                });
            }
            _ => {
                server.auth = None;
            }
        }
    }

    let configure_resilience = Confirm::new(
        "Configure custom fault tolerance & process supervision (circuit breaker, auto-restart)?",
    )
    .with_default(false)
    .prompt()?;

    if configure_resilience {
        let mut res = crate::circuit_breaker::ResilienceConfig::default();

        let threshold_str = Text::new("Consecutive failure threshold to trip circuit breaker:")
            .with_default("3")
            .prompt()?;
        if let Ok(t) = threshold_str.trim().parse::<u32>() {
            res.failure_threshold = t;
        }

        let cooldown_str = Text::new("Cooldown time in milliseconds before probe test:")
            .with_default("30000")
            .prompt()?;
        if let Ok(c) = cooldown_str.trim().parse::<u64>() {
            res.cooldown_ms = c;
        }

        let auto_restart = Confirm::new("Automatically restart crashed child processes?")
            .with_default(true)
            .prompt()?;
        res.auto_restart = auto_restart;

        server.resilience = Some(res);
    }

    config.mcp_servers.insert(name.clone(), server);
    save_config(config_path, &config)?;

    println!(
        "{} Successfully configured server '{}' in {}",
        "✔".green().bold(),
        name.cyan().bold(),
        config_path.bold()
    );

    Ok(())
}

/// Splits an argument string respecting quotes or commas.
pub fn parse_args_string(input: &str) -> Vec<String> {
    if input.contains(',') {
        input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        // Space separated, handling simple quotes
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';

        for c in input.chars() {
            match c {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = c;
                }
                c if in_quotes && c == quote_char => {
                    in_quotes = false;
                }
                ' ' | '\t' if !in_quotes => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }
        if !current.is_empty() {
            args.push(current);
        }
        args
    }
}
