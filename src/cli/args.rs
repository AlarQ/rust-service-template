//! CLI argument definitions for the rsc tool
//!
//! This module defines the command-line interface structure using clap derive macros.

use clap::{Args, Parser, Subcommand};

/// Rust Service CLI - A tool for creating and scaffolding Rust microservices
#[derive(Parser, Debug)]
#[command(name = "rsc")]
#[command(about = "CLI tool for creating Rust microservices from the service template")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new repository on GitHub
    Create(CreateArgs),
    /// Scaffold a new service locally without creating a GitHub repository
    Scaffold(ScaffoldArgs),
}

/// Arguments for the `create` command
#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Name of the repository/service to create
    #[arg(value_name = "NAME")]
    pub name: String,

    /// GitHub username or organization
    #[arg(short, long, value_name = "USER")]
    pub github_user: String,

    /// Create a private repository (default is public)
    #[arg(short, long)]
    pub private: bool,

    /// Description for the repository
    #[arg(short, long, value_name = "DESC")]
    pub description: Option<String>,

    /// Exclude Kafka support from the generated service
    #[arg(long)]
    pub without_kafka: bool,
}

/// Arguments for the `scaffold` command
#[derive(Args, Debug)]
pub struct ScaffoldArgs {
    /// Name of the service to scaffold
    #[arg(value_name = "NAME")]
    pub name: String,

    /// Output directory for the scaffolded service
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<String>,

    /// Exclude Kafka support from the generated service
    #[arg(long)]
    pub without_kafka: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_create_args_parsing() {
        let args = CreateArgs {
            name: "my-service".to_string(),
            github_user: "myuser".to_string(),
            private: true,
            description: Some("A test service".to_string()),
            without_kafka: true,
        };

        assert_eq!(args.name, "my-service");
        assert_eq!(args.github_user, "myuser");
        assert!(args.private);
        assert_eq!(args.description, Some("A test service".to_string()));
        assert!(args.without_kafka);
    }

    #[test]
    fn test_scaffold_args_parsing() {
        let args = ScaffoldArgs {
            name: "my-service".to_string(),
            output: Some("/tmp/output".to_string()),
            without_kafka: false,
        };

        assert_eq!(args.name, "my-service");
        assert_eq!(args.output, Some("/tmp/output".to_string()));
        assert!(!args.without_kafka);
    }
}
