//! Command implementations for the CLI
//!
//! This module contains the implementation logic for the `create` and `scaffold`
//! commands, orchestrating the GitHub API, file generation, and git operations.

use anyhow::{Context, Result};
use std::{env, path::Path};
use tempfile::TempDir;

use crate::cli::{
    args::{CreateArgs, ScaffoldArgs},
    generator::{self, ProjectGenerator},
    github::{get_github_token, GitHubClient},
};

/// Validate the output path to prevent path traversal attacks
///
/// # Arguments
/// * `path` - The output path to validate
///
/// # Errors
/// Returns an error if the path attempts to escape the current directory
fn validate_output_path(path: &Path) -> Result<()> {
    // Check for path traversal attempts
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let current = std::env::current_dir()?.canonicalize()?;

    if !canonical.starts_with(&current) {
        anyhow::bail!("Output path must be within the current directory");
    }

    Ok(())
}

/// Execute the `create` command
///
/// This command:
/// 1. Validates the GitHub token
/// 2. Creates a new repository on GitHub
/// 3. Generates the service files (with optional Kafka removal)
/// 4. Initializes git and pushes to the new repository
///
/// # Arguments
/// * `args` - Parsed command arguments
///
/// # Errors
/// Returns an error if any step of the process fails
pub async fn execute_create(args: CreateArgs) -> Result<()> {
    // Get GitHub token from environment
    let github_token = get_github_token()
        .context("GITHUB_TOKEN environment variable is required. Please set it and try again.")?;

    println!("Creating GitHub repository '{}'...", args.name);

    // Create GitHub client and repository
    let github = GitHubClient::new(&github_token)?;

    let repo = github
        .create_repository(
            &args.name,
            args.description.as_deref(),
            args.private,
            &args.github_user,
        )
        .await
        .context("Failed to create GitHub repository")?;

    println!("✓ Created repository: {}", repo.html_url);

    // Create temporary directory for file generation
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();

    println!("Generating service files...");

    // Get the current directory (template source)
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Generate the service
    let generator = ProjectGenerator::new(
        current_dir,
        temp_path.to_path_buf(),
        args.without_kafka,
        args.name.clone(),
    )
    .context("Failed to create project generator")?;
    generator
        .generate()
        .context("Failed to generate service files")?;

    if args.without_kafka {
        println!("✓ Generated service without Kafka support");
    } else {
        println!("✓ Generated service with Kafka support");
    }

    // Initialize git repository
    println!("Initializing git repository...");
    generator::init_git_repo(temp_path).context("Failed to initialize git repository")?;

    // Add remote
    let remote_url = format!("https://github.com/{}/{}.git", args.github_user, args.name);
    generator::git_add_remote(temp_path, "origin", &remote_url)
        .context("Failed to add git remote")?;

    // Stage all files
    generator::git_add_all(temp_path).context("Failed to stage files")?;

    // Commit
    generator::git_commit(
        temp_path,
        if args.without_kafka {
            "feat: initial commit without Kafka"
        } else {
            "feat: initial commit with Kafka support"
        },
        "Rust Service CLI",
        "cli@localhost",
    )
    .context("Failed to commit changes")?;

    println!("Pushing to GitHub...");

    // Push to remote
    generator::git_push(temp_path, "origin", "main")
        .or_else(|_| generator::git_push(temp_path, "origin", "master"))
        .context("Failed to push to remote. Make sure you have SSH access to GitHub.")?;

    println!("\n✅ Success! Repository created and pushed to GitHub.");
    println!("   Repository URL: {}", repo.html_url);
    println!("   Clone URL: {}", repo.ssh_url);

    if args.without_kafka {
        println!("\nNote: Kafka support has been excluded from this service.");
    }

    Ok(())
}

/// Execute the `scaffold` command
///
/// This command:
/// 1. Determines the output directory
/// 2. Generates the service files locally (with optional Kafka removal)
/// 3. Initializes git in the output directory
///
/// # Arguments
/// * `args` - Parsed command arguments
///
/// # Errors
/// Returns an error if any step of the process fails
pub fn execute_scaffold(args: ScaffoldArgs) -> Result<()> {
    // Determine output directory
    let output_dir = match args.output {
        Some(path) => std::path::PathBuf::from(path),
        None => {
            let current_dir = env::current_dir().context("Failed to get current directory")?;
            current_dir.join(&args.name)
        }
    };

    // Validate output path to prevent path traversal
    validate_output_path(&output_dir)?;

    // Check if output directory already exists
    if output_dir.exists() {
        anyhow::bail!(
            "Output directory '{}' already exists. Please remove it or choose a different location.",
            output_dir.display()
        );
    }

    println!("Scaffolding service '{}'...", args.name);

    // Get the current directory (template source)
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Generate the service
    let generator = ProjectGenerator::new(
        current_dir,
        output_dir.clone(),
        args.without_kafka,
        args.name.clone(),
    )
    .context("Failed to create project generator")?;
    generator
        .generate()
        .context("Failed to generate service files")?;

    if args.without_kafka {
        println!("✓ Generated service without Kafka support");
    } else {
        println!("✓ Generated service with Kafka support");
    }

    // Initialize git repository
    println!("Initializing git repository...");
    generator::init_git_repo(&output_dir).context("Failed to initialize git repository")?;

    // Stage all files
    generator::git_add_all(&output_dir).context("Failed to stage files")?;

    // Initial commit
    generator::git_commit(
        &output_dir,
        if args.without_kafka {
            "feat: initial scaffold without Kafka"
        } else {
            "feat: initial scaffold with Kafka support"
        },
        "Rust Service CLI",
        "cli@localhost",
    )
    .context("Failed to commit changes")?;

    println!("\n✅ Success! Service scaffolded locally.");
    println!("   Location: {}", output_dir.canonicalize()?.display());
    println!("\nNext steps:");
    println!(
        "   cd {}",
        output_dir.file_name().unwrap().to_string_lossy()
    );
    println!("   docker-compose up -d");
    println!("   cargo run");

    if args.without_kafka {
        println!("\nNote: Kafka support has been excluded from this service.");
    }

    Ok(())
}
