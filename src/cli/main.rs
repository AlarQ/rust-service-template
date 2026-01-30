//! Entry point for the rsc CLI tool
//!
//! This binary provides commands for creating and scaffolding Rust microservices
//! based on the rust-service-template repository.

use anyhow::Result;
use clap::Parser;

use rust_service_template::cli::{
    args::{Cli, Commands},
    commands::{execute_create, execute_scaffold},
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create(args) => execute_create(args).await,
        Commands::Scaffold(args) => execute_scaffold(args),
    }
}
