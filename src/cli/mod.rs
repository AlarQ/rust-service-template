//! CLI module for the Rust Service Creator (rsc) tool.
//!
//! This module provides functionality to create and scaffold new Rust microservices
//! based on the rust-service-template repository.

pub mod args;
pub mod commands;
pub mod generator;
pub mod github;

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn test_module_exports() {
        // Ensure all modules are properly exported
        let _ = args::Cli::parse_from(["rsc", "--version"]);
    }
}
