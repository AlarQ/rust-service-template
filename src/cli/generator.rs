use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const EXCLUDED_PATHS: &[(&str, bool)] = &[
    (".git", true),
    ("target", true),
    (".tmp", true),
    ("src/cli", true),
    ("Cargo.lock", false),
    (".env", false),
];

pub struct ProjectGenerator {
    source_dir: PathBuf,
    target_dir: PathBuf,
    without_kafka: bool,
    project_name: String,
}

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

    pub fn generate(&self) -> Result<()> {
        fs::create_dir_all(&self.target_dir)
            .with_context(|| format!("Failed to create directory: {:?}", self.target_dir))?;

        self.copy_files()?;
        self.modify_lib_rs()?;

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
            self.modify_github_workflows()?;
        }

        self.update_project_name()?;
        self.update_main_rs_crate_name()?;
        self.fix_api_mod_type_annotations()?;

        Ok(())
    }

    fn copy_files(&self) -> Result<()> {
        for entry in WalkDir::new(&self.source_dir) {
            let entry = entry.context("Failed to read directory entry")?;
            let source_path = entry.path();

            if self.is_excluded(source_path) {
                continue;
            }

            let relative_path = source_path.strip_prefix(&self.source_dir)?;
            let target_path = self.target_dir.join(relative_path);

            if source_path.is_dir() {
                fs::create_dir_all(&target_path)
                    .with_context(|| format!("Failed to create directory: {:?}", target_path))?;
            } else {
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

    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        let target_str = self.target_dir.to_string_lossy();
        if path_str.starts_with(target_str.as_ref()) {
            return true;
        }

        for (excluded, is_dir) in EXCLUDED_PATHS {
            let excluded_path = self.source_dir.join(excluded);
            let excluded_str = excluded_path.to_string_lossy();

            if *is_dir {
                if path_str.starts_with(excluded_str.as_ref()) {
                    return true;
                }
            } else if path_str == *excluded_str {
                return true;
            }
        }

        false
    }

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

    fn modify_cargo_toml(&self) -> Result<()> {
        let cargo_toml_path = self.target_dir.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read {:?}", cargo_toml_path))?;

        let modified = content
            .lines()
            .filter(|line| !line.contains("rdkafka"))
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&cargo_toml_path, modified)
            .with_context(|| format!("Failed to write {:?}", cargo_toml_path))?;

        Ok(())
    }

    fn modify_config_rs(&self) -> Result<()> {
        let config_path = self.target_dir.join("src/config.rs");
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {:?}", config_path))?;

        let lines: Vec<&str> = content.lines().collect();
        let mut result_lines: Vec<String> = Vec::new();
        let mut skip_mode = false;
        let mut skip_level = 0i32;
        let mut prev_line_was_kafka_config_field = false;

        for line in lines {
            // Handle the domain interfaces import line - remove event_producer but keep task_repository
            if line.contains("use crate::domain::interfaces::{event_producer::EventProducer, task_repository::TaskRepository};") {
                result_lines.push("use crate::domain::interfaces::task_repository::TaskRepository;".to_string());
                continue;
            }

            // Skip event_producer field in AppState
            if line.contains("event_producer:") {
                continue;
            }

            // Skip kafka_config field in AppConfig and track it for removing duplicate #[serde(default)]
            if line.contains("kafka_config: KafkaConfig") {
                prev_line_was_kafka_config_field = true;
                continue;
            }

            // Skip the #[serde(default)] that precedes kafka_config
            if line.contains("#[serde(default)]") && prev_line_was_kafka_config_field {
                prev_line_was_kafka_config_field = false;
                continue;
            }

            // Reset the flag if we see a non-empty line that's not kafka_config field or its attribute
            if !line.trim().is_empty() && !line.contains("#[serde(default)]") {
                prev_line_was_kafka_config_field = false;
            }

            // Start skipping Kafka-related code when we see the doc comment
            if line.contains("/// Kafka configuration for event streaming") {
                skip_mode = true;
                skip_level = 0;
                continue;
            }

            if skip_mode {
                // Count braces to track nesting level
                for c in line.chars() {
                    if c == '{' {
                        skip_level += 1;
                    } else if c == '}' {
                        skip_level -= 1;
                    }
                }

                // When we return to level 0 and see a closing brace, check if we're done
                // The Kafka section has: struct (ends with }), 3 functions, impl block (ends with })
                // We need to skip until we've seen all of these
                if skip_level == 0 && line.trim() == "}" {
                    // We've finished one block, but we need to check if there are more
                    // The next non-empty line after the impl block's closing brace should be the CORS section
                    // So we continue skipping until we see the CORS doc comment
                    continue;
                }

                // If we see the CORS section doc comment, we're done skipping
                if line.contains("/// CORS (Cross-Origin Resource Sharing) configuration") {
                    skip_mode = false;
                    result_lines.push(line.to_string());
                    continue;
                }

                continue;
            }

            result_lines.push(line.to_string());
        }

        fs::write(&config_path, result_lines.join("\n"))
            .with_context(|| format!("Failed to write {:?}", config_path))?;

        Ok(())
    }

    fn modify_main_rs(&self) -> Result<()> {
        let main_path = self.target_dir.join("src/main.rs");
        let content = fs::read_to_string(&main_path)
            .with_context(|| format!("Failed to read {:?}", main_path))?;

        let mut result_lines = Vec::new();
        let mut skip_lines = false;

        for line in content.lines() {
            // Handle the infrastructure import line - remove kafka_producer but keep task::PostgresTaskRepository
            if line.contains(
                "infrastructure::{kafka_producer::KafkaEventService, task::PostgresTaskRepository}",
            ) {
                result_lines.push("    infrastructure::task::PostgresTaskRepository,");
                continue;
            }

            if line.contains("kafka_producer::KafkaEventService") {
                continue;
            }

            if line.contains("Initializing Kafka event producer") {
                skip_lines = true;
                continue;
            }

            if skip_lines && line.contains("let app_state = Arc::new(AppState") {
                skip_lines = false;
            }

            if skip_lines {
                continue;
            }

            if line.contains("event_producer,") {
                continue;
            }

            result_lines.push(line);
        }

        fs::write(&main_path, result_lines.join("\n"))
            .with_context(|| format!("Failed to write {:?}", main_path))?;

        Ok(())
    }

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

    fn modify_docker_compose(&self) -> Result<()> {
        let compose_path = self.target_dir.join("docker-compose.yaml");

        if !compose_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&compose_path)
            .with_context(|| format!("Failed to read {:?}", compose_path))?;

        let mut result_lines = Vec::new();
        let mut in_kafka_service = false;
        let mut indent_level = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == "zookeeper:" || trimmed == "kafka:" || trimmed == "kafka-ui:" {
                in_kafka_service = true;
                indent_level = line.len() - line.trim_start().len();
                continue;
            }

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

    fn modify_github_workflows(&self) -> Result<()> {
        let workflow_path = self.target_dir.join(".github/workflows/ci.yml");

        if !workflow_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&workflow_path)
            .with_context(|| format!("Failed to read {:?}", workflow_path))?;

        let mut result_lines = Vec::new();
        let mut in_kafka_service = false;
        let mut base_indent = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("KAFKA_BOOTSTRAP_SERVERS:") {
                continue;
            }

            if trimmed == "kafka:" {
                in_kafka_service = true;
                base_indent = line.len() - line.trim_start().len();
                continue;
            }

            if in_kafka_service {
                let current_indent = line.len() - line.trim_start().len();
                if !line.trim().is_empty() && current_indent <= base_indent {
                    in_kafka_service = false;
                } else {
                    continue;
                }
            }

            result_lines.push(line);
        }

        fs::write(&workflow_path, result_lines.join("\n"))
            .with_context(|| format!("Failed to write {:?}", workflow_path))?;

        Ok(())
    }

    fn modify_lib_rs(&self) -> Result<()> {
        let lib_path = self.target_dir.join("src/lib.rs");

        if !lib_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&lib_path)
            .with_context(|| format!("Failed to read {:?}", lib_path))?;

        let modified = content
            .lines()
            .filter(|line| !line.contains("pub mod cli"))
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&lib_path, modified)
            .with_context(|| format!("Failed to write {:?}", lib_path))?;

        Ok(())
    }

    fn update_project_name(&self) -> Result<()> {
        let cargo_toml_path = self.target_dir.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read {:?}", cargo_toml_path))?;

        // Replace package name
        let mut modified = content.replacen(
            "name = \"rust-service-template\"",
            &format!("name = \"{}\"", self.project_name),
            1,
        );

        // Remove the rsc binary block
        modified = modified.replace(
            "[[bin]]\nname = \"rsc\"\npath = \"src/cli/main.rs\"\n\n",
            "",
        );

        // Replace binary name
        modified = modified.replace(
            "name = \"rust-service-template\"",
            &format!("name = \"{}\"", self.project_name),
        );

        fs::write(&cargo_toml_path, modified)
            .with_context(|| format!("Failed to write {:?}", cargo_toml_path))?;

        Ok(())
    }

    fn update_main_rs_crate_name(&self) -> Result<()> {
        let main_rs_path = self.target_dir.join("src/main.rs");
        let content = fs::read_to_string(&main_rs_path)
            .with_context(|| format!("Failed to read {:?}", main_rs_path))?;

        // Convert project name to valid Rust crate name (hyphens to underscores)
        let crate_name = self.project_name.replace("-", "_");

        let modified = content.replace("rust_service_template", &crate_name);

        fs::write(&main_rs_path, modified)
            .with_context(|| format!("Failed to write {:?}", main_rs_path))?;

        Ok(())
    }

    fn fix_api_mod_type_annotations(&self) -> Result<()> {
        let api_mod_path = self.target_dir.join("src/api/mod.rs");

        if !api_mod_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&api_mod_path)
            .with_context(|| format!("Failed to read {:?}", api_mod_path))?;

        // Fix type annotations in filter_map closures for CORS configuration
        // Line 97: filter_map(|origin| origin.parse().ok())
        let modified = content
            .replace(
                ".filter_map(|origin| origin.parse().ok())",
                ".filter_map(|origin: &String| origin.parse().ok())",
            )
            // Line 109: filter_map(|method| method.parse().ok())
            .replace(
                ".filter_map(|method| method.parse().ok())",
                ".filter_map(|method: &String| method.parse().ok())",
            )
            // Line 121: filter_map(|header| header.parse().ok())
            .replace(
                ".filter_map(|header| header.parse().ok())",
                ".filter_map(|header: &String| header.parse().ok())",
            );

        fs::write(&api_mod_path, modified)
            .with_context(|| format!("Failed to write {:?}", api_mod_path))?;

        Ok(())
    }
}

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

pub fn git_commit(dir: &Path, message: &str, user_name: &str, user_email: &str) -> Result<()> {
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
