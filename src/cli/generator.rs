//! File generation and modification logic.

use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Files and directories to exclude from copying
const EXCLUDED_PATHS: &[(&str, bool)] = &[
    (".git", true),        // Directory
    ("target", true),      // Directory
    (".tmp", true),        // Directory
    ("src/cli", true),     // Directory - CLI code itself
    ("Cargo.lock", false), // File - will be regenerated
    (".env", false),       // File - environment specific
];

/// Generate a new project by copying and modifying template files
pub struct ProjectGenerator {
    source_dir: PathBuf,
    target_dir: PathBuf,
    without_kafka: bool,
    project_name: String,
}

/// Validate a service name for use as a project name
///
/// # Arguments
/// * `name` - The service name to validate
///
/// # Errors
/// Returns an error if the name is empty, too long, starts with invalid characters,
/// or contains invalid characters
fn validate_service_name(name: &str) -> Result<()> {
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\\', '/'];

    if name.is_empty() || name.len() > 100 {
        anyhow::bail!("Service name must be between 1 and 100 characters");
    }

    if name.starts_with('.') || name.starts_with('-') {
        anyhow::bail!("Service name cannot start with '.' or '-'");
    }

    if name.chars().any(|c| invalid_chars.contains(&c)) {
        anyhow::bail!("Service name contains invalid characters: < > : \" | ? * \\ /");
    }

    Ok(())
}

impl ProjectGenerator {
    /// Create a new project generator
    ///
    /// # Errors
    /// Returns an error if the project name is invalid
    pub fn new(
        source_dir: PathBuf,
        target_dir: PathBuf,
        without_kafka: bool,
        project_name: String,
    ) -> Result<Self> {
        validate_service_name(&project_name)?;

        Ok(Self {
            source_dir,
            target_dir,
            without_kafka,
            project_name,
        })
    }

    /// Generate the project
    ///
    /// # Errors
    /// Returns an error if any file operation fails
    pub fn generate(&self) -> Result<()> {
        // Create target directory
        fs::create_dir_all(&self.target_dir)
            .with_context(|| format!("Failed to create directory: {:?}", self.target_dir))?;

        // Copy files
        self.copy_files()?;

        // Always remove CLI module export since CLI code is excluded
        self.modify_lib_rs()?;

        // Apply modifications if --without-kafka is set
        if self.without_kafka {
            self.remove_kafka_files()?;
            self.modify_cargo_toml()?;
            self.modify_config_rs()?;
            self.modify_main_rs()?;
            self.modify_infrastructure_mod()?;
            self.modify_domain_interfaces_mod()?;
            self.modify_task_models_mod()?;
            self.modify_docker_compose()?;
            self.modify_env_example()?;
            self.modify_run_sh()?;
        }

        // Update project name in Cargo.toml
        self.update_project_name()?;

        Ok(())
    }

    /// Copy all files from source to target, excluding certain paths
    fn copy_files(&self) -> Result<()> {
        for entry in WalkDir::new(&self.source_dir) {
            let entry = entry.context("Failed to read directory entry")?;
            let source_path = entry.path();

            // Skip excluded paths
            if self.is_excluded(source_path) {
                continue;
            }

            let relative_path = source_path.strip_prefix(&self.source_dir)?;
            let target_path = self.target_dir.join(relative_path);

            if source_path.is_dir() {
                fs::create_dir_all(&target_path)
                    .with_context(|| format!("Failed to create directory: {:?}", target_path))?;
            } else {
                // Ensure parent directory exists
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create directory: {:?}", parent))?;
                }

                fs::copy(source_path, &target_path).with_context(|| {
                    format!(
                        "Failed to copy file: {:?} -> {:?}",
                        source_path, target_path
                    )
                })?;
            }
        }

        Ok(())
    }

    /// Check if a path should be excluded
    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Always exclude the target directory itself to prevent recursive copying
        let target_str = self.target_dir.to_string_lossy();
        if path_str.starts_with(target_str.as_ref()) {
            return true;
        }

        for (excluded, is_dir) in EXCLUDED_PATHS {
            let excluded_path = self.source_dir.join(excluded);
            let excluded_str = excluded_path.to_string_lossy();

            if *is_dir {
                // For directories, check if the path is within the excluded directory
                if path_str.starts_with(excluded_str.as_ref()) {
                    return true;
                }
            } else if path_str == *excluded_str {
                // For files, exact match
                return true;
            }
        }

        false
    }

    /// Remove Kafka-related files
    fn remove_kafka_files(&self) -> Result<()> {
        let files_to_remove = [
            "src/infrastructure/kafka_producer.rs",
            "src/domain/interfaces/event_producer.rs",
            "src/domain/task/models/events.rs",
        ];

        for file in &files_to_remove {
            let file_path = self.target_dir.join(file);
            if file_path.exists() {
                fs::remove_file(&file_path)
                    .with_context(|| format!("Failed to remove file: {:?}", file_path))?;
            }
        }

        Ok(())
    }

    /// Modify Cargo.toml to remove Kafka dependencies
    fn modify_cargo_toml(&self) -> Result<()> {
        let cargo_toml_path = self.target_dir.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read {:?}", cargo_toml_path))?;

        // Remove rdkafka dependency line
        let modified = content
            .lines()
            .filter(|line| !line.contains("rdkafka"))
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&cargo_toml_path, modified)
            .with_context(|| format!("Failed to write {:?}", cargo_toml_path))?;

        Ok(())
    }

    /// Modify src/config.rs to remove Kafka configuration
    fn modify_config_rs(&self) -> Result<()> {
        let config_path = self.target_dir.join("src/config.rs");
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {:?}", config_path))?;

        let lines: Vec<&str> = content.lines().collect();
        let mut result_lines: Vec<String> = Vec::new();
        let mut skip_until: Option<&str> = None;

        for line in lines {
            // Skip event_producer import
            if line.contains("event_producer::EventProducer") {
                continue;
            }

            // Skip event_producer field in AppState
            if line.contains("event_producer:") {
                continue;
            }

            // Skip kafka_config field and its serde attribute
            // Note: We handle the serde attribute separately below
            if line.contains("kafka_config: KafkaConfig") {
                continue;
            }

            // Skip #[serde(default)] that precedes kafka_config
            if line.contains("#[serde(default)]")
                && result_lines
                    .last()
                    .is_some_and(|prev: &String| prev.contains("kafka_config"))
            {
                continue;
            }

            // Start skipping KafkaConfig struct and its impl
            if line.contains("pub struct KafkaConfig") {
                skip_until = Some("impl Default for KafkaConfig");
                continue;
            }

            // Handle skipping blocks
            if let Some(end_marker) = skip_until {
                if line.contains(end_marker) {
                    // Skip until the end of the impl block
                    skip_until = Some("}");
                    continue;
                }
                if end_marker == "}" && line.trim() == "}" {
                    skip_until = None;
                    continue;
                }
                continue;
            }

            // Skip Kafka-related default functions
            if line.contains("default_bootstrap_servers")
                || line.contains("default_client_id")
                || line.contains("default_task_topic")
            {
                continue;
            }

            result_lines.push(line.to_string());
        }

        fs::write(&config_path, result_lines.join("\n"))
            .with_context(|| format!("Failed to write {:?}", config_path))?;

        Ok(())
    }

    /// Modify src/main.rs to remove Kafka initialization
    fn modify_main_rs(&self) -> Result<()> {
        let main_path = self.target_dir.join("src/main.rs");
        let content = fs::read_to_string(&main_path)
            .with_context(|| format!("Failed to read {:?}", main_path))?;

        let mut result_lines = Vec::new();
        let mut skip_lines = false;

        for line in content.lines() {
            // Skip Kafka-related imports
            if line.contains("kafka_producer::KafkaEventService") {
                continue;
            }

            // Start skipping Kafka initialization block
            if line.contains("Initializing Kafka event producer") {
                skip_lines = true;
                continue;
            }

            // End skipping after the Kafka initialization block
            if skip_lines && line.contains("let app_state = Arc::new(AppState") {
                skip_lines = false;
            }

            if skip_lines {
                continue;
            }

            // Skip event_producer field in AppState creation
            if line.contains("event_producer,") {
                continue;
            }

            result_lines.push(line);
        }

        fs::write(&main_path, result_lines.join("\n"))
            .with_context(|| format!("Failed to write {:?}", main_path))?;

        Ok(())
    }

    /// Modify src/infrastructure/mod.rs to remove kafka_producer module
    fn modify_infrastructure_mod(&self) -> Result<()> {
        let mod_path = self.target_dir.join("src/infrastructure/mod.rs");
        let content = fs::read_to_string(&mod_path)
            .with_context(|| format!("Failed to read {:?}", mod_path))?;

        let modified = content
            .lines()
            .filter(|line| !line.contains("kafka_producer"))
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&mod_path, modified)
            .with_context(|| format!("Failed to write {:?}", mod_path))?;

        Ok(())
    }

    /// Modify src/domain/interfaces/mod.rs to remove event_producer module
    fn modify_domain_interfaces_mod(&self) -> Result<()> {
        let mod_path = self.target_dir.join("src/domain/interfaces/mod.rs");
        let content = fs::read_to_string(&mod_path)
            .with_context(|| format!("Failed to read {:?}", mod_path))?;

        let modified = content
            .lines()
            .filter(|line| !line.contains("event_producer"))
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&mod_path, modified)
            .with_context(|| format!("Failed to write {:?}", mod_path))?;

        Ok(())
    }

    /// Modify src/domain/task/models/mod.rs to remove events module
    fn modify_task_models_mod(&self) -> Result<()> {
        let mod_path = self.target_dir.join("src/domain/task/models/mod.rs");
        let content = fs::read_to_string(&mod_path)
            .with_context(|| format!("Failed to read {:?}", mod_path))?;

        let modified = content
            .lines()
            .filter(|line| !line.contains("events") && !line.contains("TaskEvent"))
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&mod_path, modified)
            .with_context(|| format!("Failed to write {:?}", mod_path))?;

        Ok(())
    }

    /// Modify docker-compose.yaml to remove Kafka services
    fn modify_docker_compose(&self) -> Result<()> {
        let compose_path = self.target_dir.join("docker-compose.yaml");

        // If the file doesn't exist, skip
        if !compose_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&compose_path)
            .with_context(|| format!("Failed to read {:?}", compose_path))?;

        // Parse YAML and remove Kafka-related services
        // For simplicity, we'll use string manipulation
        let mut result_lines = Vec::new();
        let mut in_kafka_service = false;
        let mut indent_level = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Detect start of Kafka-related services
            if trimmed == "zookeeper:" || trimmed == "kafka:" || trimmed == "kafka-ui:" {
                in_kafka_service = true;
                indent_level = line.len() - line.trim_start().len();
                continue;
            }

            // Check if we've exited the service block
            if in_kafka_service {
                let current_indent = line.len() - line.trim_start().len();
                if !line.trim().is_empty() && current_indent <= indent_level {
                    in_kafka_service = false;
                } else {
                    continue;
                }
            }

            result_lines.push(line);
        }

        fs::write(&compose_path, result_lines.join("\n"))
            .with_context(|| format!("Failed to write {:?}", compose_path))?;

        Ok(())
    }

    /// Modify .env.example to remove Kafka-related environment variables
    fn modify_env_example(&self) -> Result<()> {
        let env_path = self.target_dir.join(".env.example");

        if !env_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&env_path)
            .with_context(|| format!("Failed to read {:?}", env_path))?;

        let modified = content
            .lines()
            .filter(|line| !line.contains("KAFKA"))
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&env_path, modified)
            .with_context(|| format!("Failed to write {:?}", env_path))?;

        Ok(())
    }

    /// Modify run.sh to remove Kafka-related exports
    fn modify_run_sh(&self) -> Result<()> {
        let run_sh_path = self.target_dir.join("run.sh");

        if !run_sh_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&run_sh_path)
            .with_context(|| format!("Failed to read {:?}", run_sh_path))?;

        let modified = content
            .lines()
            .filter(|line| !line.contains("KAFKA"))
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&run_sh_path, modified)
            .with_context(|| format!("Failed to write {:?}", run_sh_path))?;

        Ok(())
    }

    /// Modify src/lib.rs to remove exports for deleted modules
    fn modify_lib_rs(&self) -> Result<()> {
        let lib_path = self.target_dir.join("src/lib.rs");

        if !lib_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&lib_path)
            .with_context(|| format!("Failed to read {:?}", lib_path))?;

        // Remove the cli module export since CLI code is excluded
        let modified = content
            .lines()
            .filter(|line| !line.contains("pub mod cli"))
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&lib_path, modified)
            .with_context(|| format!("Failed to write {:?}", lib_path))?;

        Ok(())
    }

    /// Update project name in Cargo.toml
    fn update_project_name(&self) -> Result<()> {
        let cargo_toml_path = self.target_dir.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read {:?}", cargo_toml_path))?;

        let modified = content.replacen(
            "name = \"rust-service-template\"",
            &format!("name = \"{}\"", self.project_name),
            1,
        );

        fs::write(&cargo_toml_path, modified)
            .with_context(|| format!("Failed to write {:?}", cargo_toml_path))?;

        Ok(())
    }
}

/// Initialize a git repository in the given directory
///
/// # Arguments
/// * `dir` - Path to the directory where git should be initialized
///
/// # Errors
/// Returns an error if git initialization fails
pub fn init_git_repo(dir: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .arg("init")
        .current_dir(dir)
        .output()
        .context("Failed to execute git init")?;

    if !output.status.success() {
        anyhow::bail!(
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Add all files to git staging area
///
/// # Arguments
/// * `dir` - Path to the git repository
///
/// # Errors
/// Returns an error if git add fails
pub fn git_add_all(dir: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .context("Failed to execute git add")?;

    if !output.status.success() {
        anyhow::bail!(
            "git add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Commit changes with the given message
///
/// # Arguments
/// * `dir` - Path to the git repository
/// * `message` - Commit message
/// * `user_name` - Git user name for the commit
/// * `user_email` - Git user email for the commit
///
/// # Errors
/// Returns an error if git commit fails
pub fn git_commit(dir: &Path, message: &str, user_name: &str, user_email: &str) -> Result<()> {
    // Set git user config
    let output = std::process::Command::new("git")
        .args(["config", "user.name", user_name])
        .current_dir(dir)
        .output()
        .context("Failed to set git user.name")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to set git user.name: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = std::process::Command::new("git")
        .args(["config", "user.email", user_email])
        .current_dir(dir)
        .output()
        .context("Failed to set git user.email")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to set git user.email: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Then commit
    let output = std::process::Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(dir)
        .output()
        .context("Failed to execute git commit")?;

    if !output.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Add a remote to the git repository
///
/// # Arguments
/// * `dir` - Path to the git repository
/// * `name` - Name of the remote (e.g., "origin")
/// * `url` - URL of the remote repository
///
/// # Errors
/// Returns an error if git remote add fails
pub fn git_add_remote(dir: &Path, name: &str, url: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["remote", "add", name, url])
        .current_dir(dir)
        .output()
        .context("Failed to execute git remote add")?;

    if !output.status.success() {
        anyhow::bail!(
            "git remote add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Push to remote repository
///
/// # Arguments
/// * `dir` - Path to the git repository
/// * `remote` - Name of the remote (e.g., "origin")
/// * `branch` - Branch name to push
///
/// # Errors
/// Returns an error if git push fails
pub fn git_push(dir: &Path, remote: &str, branch: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["push", "-u", remote, branch])
        .current_dir(dir)
        .output()
        .context("Failed to execute git push")?;

    if !output.status.success() {
        anyhow::bail!(
            "git push failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}
