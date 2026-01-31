use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rsc")]
#[command(about = "CLI tool for creating Rust microservices from the service template")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new repository on GitHub
    Create(CreateArgs),
    /// Scaffold a new service locally without creating a GitHub repository
    Scaffold(ScaffoldArgs),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    #[arg(value_name = "NAME")]
    pub name: String,

    #[arg(short, long, value_name = "USER")]
    pub github_user: String,

    #[arg(short, long)]
    pub private: bool,

    #[arg(short, long, value_name = "DESC")]
    pub description: Option<String>,

    #[arg(long)]
    pub without_kafka: bool,
}

#[derive(Args, Debug)]
pub struct ScaffoldArgs {
    #[arg(value_name = "NAME")]
    pub name: String,

    #[arg(short, long, value_name = "PATH")]
    pub output: Option<String>,

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
