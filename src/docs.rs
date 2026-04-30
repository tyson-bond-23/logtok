use anyhow::{Context, Result};
use askama::Template;
use clap::CommandFactory;
use std::path::Path;

use crate::cli::Cli;

/// Metadata for a single CLI argument/flag
pub struct ArgInfo {
    pub name: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub help: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Metadata for a single CLI subcommand
pub struct CommandInfo {
    pub name: String,
    pub about: String,
    pub long_about: Option<String>,
    pub after_long_help: Option<String>,
    pub args: Vec<ArgInfo>,
}

/// A token category entry for the reference table
pub struct TokenCategory {
    pub prefix: String,
    pub description: String,
}

#[derive(Template)]
#[template(path = "docs.html")]
struct DocsTemplate<'a> {
    version: &'a str,
    commands: &'a [CommandInfo],
    global_args: &'a [ArgInfo],
    token_categories: &'a [TokenCategory],
}

/// Returns all 19 token categories with descriptions.
pub fn get_token_categories() -> Vec<TokenCategory> {
    vec![
        TokenCategory { prefix: "IP".into(), description: "IP addresses (v4 and v6)".into() },
        TokenCategory { prefix: "HOST".into(), description: "Hostnames and domain names".into() },
        TokenCategory { prefix: "URL".into(), description: "Internal URLs and endpoints".into() },
        TokenCategory { prefix: "PATH".into(), description: "File system paths".into() },
        TokenCategory { prefix: "PORT".into(), description: "Network port numbers".into() },
        TokenCategory { prefix: "EMAIL".into(), description: "Email addresses".into() },
        TokenCategory { prefix: "USER".into(), description: "Usernames and user identifiers".into() },
        TokenCategory { prefix: "PHONE".into(), description: "Phone numbers".into() },
        TokenCategory { prefix: "KEY".into(), description: "API keys and access tokens".into() },
        TokenCategory { prefix: "PASS".into(), description: "Passwords and secrets".into() },
        TokenCategory { prefix: "CONN".into(), description: "Database connection strings".into() },
        TokenCategory { prefix: "JWT".into(), description: "JSON Web Tokens".into() },
        TokenCategory { prefix: "PEM".into(), description: "PEM-encoded certificates/keys".into() },
        TokenCategory { prefix: "UUID".into(), description: "UUIDs and GUIDs".into() },
        TokenCategory { prefix: "MAC".into(), description: "MAC addresses".into() },
        TokenCategory { prefix: "CC".into(), description: "Credit card numbers".into() },
        TokenCategory { prefix: "SSN".into(), description: "Social security numbers".into() },
        TokenCategory { prefix: "DOB".into(), description: "Dates of birth".into() },
        TokenCategory { prefix: "CUSTOM".into(), description: "User-defined custom patterns".into() },
    ]
}

/// Extract command metadata and global args from the clap Command tree.
pub fn extract_commands() -> (Vec<CommandInfo>, Vec<ArgInfo>) {
    let mut cmd = Cli::command();
    cmd.build();

    // Extract global args from root command (--quiet, --config)
    // Filter out "help" and "version" built-in args
    let global_args: Vec<ArgInfo> = cmd
        .get_arguments()
        .filter(|a| {
            let id = a.get_id().as_str();
            id != "help" && id != "version"
        })
        .map(|a| ArgInfo {
            name: a.get_id().to_string(),
            short: a.get_short(),
            long: a.get_long().map(|s| s.to_string()),
            help: a.get_help().map(|s| s.to_string()).unwrap_or_default(),
            required: a.is_required_set(),
            default_value: a.get_default_values()
                .first()
                .map(|v| v.to_string_lossy().to_string()),
        })
        .collect();

    // Extract subcommands, filtering out "docs" itself and "help"
    let commands: Vec<CommandInfo> = cmd
        .get_subcommands()
        .filter(|s| s.get_name() != "docs" && s.get_name() != "help")
        .map(|sub| {
            let args: Vec<ArgInfo> = sub
                .get_arguments()
                .filter(|a| {
                    let id = a.get_id().as_str();
                    // Filter out help, version, and global args
                    id != "help" && id != "version" && id != "quiet" && id != "config"
                })
                .map(|a| ArgInfo {
                    name: a.get_id().to_string(),
                    short: a.get_short(),
                    long: a.get_long().map(|s| s.to_string()),
                    help: a.get_help().map(|s| s.to_string()).unwrap_or_default(),
                    required: a.is_required_set(),
                    default_value: a.get_default_values()
                        .first()
                        .map(|v| v.to_string_lossy().to_string()),
                })
                .collect();

            CommandInfo {
                name: sub.get_name().to_string(),
                about: sub.get_about().map(|s| s.to_string()).unwrap_or_default(),
                long_about: sub.get_long_about().map(|s| s.to_string()),
                after_long_help: sub.get_after_long_help().map(|s| s.to_string()),
                args,
            }
        })
        .collect();

    (commands, global_args)
}

/// Generate HTML documentation and write to the specified path.
pub fn generate_docs(output: &Path) -> Result<()> {
    let (commands, global_args) = extract_commands();
    let token_categories = get_token_categories();
    let version = env!("CARGO_PKG_VERSION");

    let template = DocsTemplate {
        version,
        commands: &commands,
        global_args: &global_args,
        token_categories: &token_categories,
    };

    let html = template.render().context("Failed to render documentation template")?;
    std::fs::write(output, html)
        .with_context(|| format!("Failed to write documentation to {}", output.display()))?;

    Ok(())
}
