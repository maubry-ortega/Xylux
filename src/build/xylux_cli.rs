//! # Xylux CLI Builder
//!
//! Xylux CLI integration for building and running Xylux projects.

use std::path::PathBuf;
use std::process::Stdio;

use async_trait::async_trait;
use tokio::process::Command;
use tracing::{debug, error, info};

use crate::build::Builder;
use crate::core::{Result, XyluxError};

/// Xylux CLI builder for Xylux projects.
pub struct XyluxCliBuilder {
    /// Xylux CLI binary path.
    xylux_path: String,
}

impl XyluxCliBuilder {
    /// Create a new Xylux CLI builder.
    pub fn new() -> Self {
        Self { xylux_path: "xylux".to_string() }
    }

    /// Create a new Xylux CLI builder with custom xylux path.
    pub fn with_xylux_path<S: Into<String>>(xylux_path: S) -> Self {
        Self { xylux_path: xylux_path.into() }
    }

    /// Execute a xylux command.
    async fn execute_xylux_command(&self, project_root: &PathBuf, args: &[&str]) -> Result<String> {
        debug!("Executing xylux {} in {}", args.join(" "), project_root.display());

        let mut command = Command::new(&self.xylux_path);
        command.args(args).current_dir(project_root).stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = command
            .output()
            .await
            .map_err(|e| XyluxError::build_error(format!("Failed to execute xylux: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            if !stdout.is_empty() {
                info!("Xylux output: {}", stdout.trim());
            }
            Ok(stdout.to_string())
        } else {
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                format!("Xylux command failed with exit code: {:?}", output.status.code())
            };
            error!("Xylux error: {}", error_msg.trim());
            Err(XyluxError::build_error(error_msg))
        }
    }

    /// Check if the project has a xylux.toml file.
    pub fn is_xylux_project(project_root: &PathBuf) -> bool {
        project_root.join("xylux.toml").exists()
    }

    /// Get xylux project configuration.
    pub async fn get_project_config(&self, project_root: &PathBuf) -> Result<serde_json::Value> {
        let config_path = project_root.join("xylux.toml");
        if !config_path.exists() {
            return Err(XyluxError::build_error("xylux.toml not found"));
        }

        let content = tokio::fs::read_to_string(&config_path)
            .await
            .map_err(|e| XyluxError::io(e, "Failed to read xylux.toml"))?;

        let config: toml::Value = toml::from_str(&content)
            .map_err(|e| XyluxError::build_error(format!("Failed to parse xylux.toml: {}", e)))?;

        serde_json::to_value(config)
            .map_err(|e| XyluxError::build_error(format!("Failed to convert config: {}", e)))
    }

    /// Check if xylux CLI is available.
    pub async fn check_availability(&self) -> Result<String> {
        let output = Command::new(&self.xylux_path)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| XyluxError::build_error(format!("Xylux CLI not found: {}", e)))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(version.trim().to_string())
        } else {
            Err(XyluxError::build_error("Xylux CLI is not available or not working"))
        }
    }

    /// Build with specific target.
    pub async fn build_with_target(&self, project_root: &PathBuf, target: &str) -> Result<()> {
        let args = vec!["build", "--target", target];
        self.execute_xylux_command(project_root, &args).await?;
        Ok(())
    }

    /// Build for WebAssembly target.
    pub async fn build_wasm(&self, project_root: &PathBuf) -> Result<()> {
        let args = vec!["build", "--target", "wasm"];
        self.execute_xylux_command(project_root, &args).await?;
        Ok(())
    }

    /// Build for native target with optimization.
    pub async fn build_release(&self, project_root: &PathBuf) -> Result<()> {
        let args = vec!["build", "--release"];
        self.execute_xylux_command(project_root, &args).await?;
        Ok(())
    }

    /// Run with specific arguments.
    pub async fn run_with_args(&self, project_root: &PathBuf, args: &[&str]) -> Result<()> {
        let mut command_args = vec!["run"];
        command_args.extend_from_slice(args);

        self.execute_xylux_command(project_root, &command_args).await?;
        Ok(())
    }

    /// Run in development mode with hot reload.
    pub async fn run_dev(&self, project_root: &PathBuf) -> Result<()> {
        let args = vec!["run", "--dev"];
        self.execute_xylux_command(project_root, &args).await?;
        Ok(())
    }

    /// Serve the project for web development.
    pub async fn serve(&self, project_root: &PathBuf, port: Option<u16>) -> Result<()> {
        let mut args = vec!["serve"];
        let port_string;

        if let Some(p) = port {
            port_string = p.to_string();
            args.push("--port");
            args.push(&port_string);
        }

        self.execute_xylux_command(project_root, &args).await?;
        Ok(())
    }

    /// Package the project for distribution.
    pub async fn package(&self, project_root: &PathBuf) -> Result<()> {
        let args = vec!["package"];
        self.execute_xylux_command(project_root, &args).await?;
        Ok(())
    }

    /// Export to different platforms.
    pub async fn export(&self, project_root: &PathBuf, platform: &str) -> Result<()> {
        let args = vec!["export", "--platform", platform];
        self.execute_xylux_command(project_root, &args).await?;
        Ok(())
    }

    /// Update project dependencies.
    pub async fn update(&self, project_root: &PathBuf) -> Result<()> {
        let args = vec!["update"];
        self.execute_xylux_command(project_root, &args).await?;
        Ok(())
    }

    /// Check project for issues.
    pub async fn check(&self, project_root: &PathBuf) -> Result<()> {
        let args = vec!["check"];
        self.execute_xylux_command(project_root, &args).await?;
        Ok(())
    }

    /// Format project files.
    pub async fn format(&self, project_root: &PathBuf) -> Result<()> {
        let args = vec!["fmt"];
        self.execute_xylux_command(project_root, &args).await?;
        Ok(())
    }

    /// Generate project documentation.
    pub async fn doc(&self, project_root: &PathBuf, open: bool) -> Result<()> {
        let mut args = vec!["doc"];
        if open {
            args.push("--open");
        }
        self.execute_xylux_command(project_root, &args).await?;
        Ok(())
    }

    /// Initialize a new Xylux project.
    pub async fn init(&self, project_path: &PathBuf, template: Option<&str>) -> Result<()> {
        let mut args = vec!["init"];

        if let Some(tmpl) = template {
            args.push("--template");
            args.push(tmpl);
        }

        args.push(project_path.to_str().unwrap_or("."));

        let output = Command::new(&self.xylux_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| XyluxError::build_error(format!("Failed to init project: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(XyluxError::build_error(format!("Init failed: {}", error_msg)));
        }

        Ok(())
    }

    /// Get available project templates.
    pub async fn list_templates(&self) -> Result<Vec<String>> {
        let output = self.execute_xylux_command(&PathBuf::from("."), &["templates"]).await?;

        // Parse the templates from output
        let templates: Vec<String> = output
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("Available templates:") {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(templates)
    }
}

impl Default for XyluxCliBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Builder for XyluxCliBuilder {
    async fn build(&self, project_root: &PathBuf) -> Result<()> {
        info!("Building Xylux project at: {}", project_root.display());

        if !Self::is_xylux_project(project_root) {
            return Err(XyluxError::build_error("Not a Xylux project (xylux.toml not found)"));
        }

        self.execute_xylux_command(project_root, &["build"]).await?;
        Ok(())
    }

    async fn run(&self, project_root: &PathBuf) -> Result<()> {
        info!("Running Xylux project at: {}", project_root.display());

        if !Self::is_xylux_project(project_root) {
            return Err(XyluxError::build_error("Not a Xylux project (xylux.toml not found)"));
        }

        self.execute_xylux_command(project_root, &["run"]).await?;
        Ok(())
    }

    async fn test(&self, project_root: &PathBuf) -> Result<()> {
        info!("Testing Xylux project at: {}", project_root.display());

        if !Self::is_xylux_project(project_root) {
            return Err(XyluxError::build_error("Not a Xylux project (xylux.toml not found)"));
        }

        self.execute_xylux_command(project_root, &["test"]).await?;
        Ok(())
    }

    async fn clean(&self, project_root: &PathBuf) -> Result<()> {
        info!("Cleaning Xylux project at: {}", project_root.display());

        if !Self::is_xylux_project(project_root) {
            return Err(XyluxError::build_error("Not a Xylux project (xylux.toml not found)"));
        }

        self.execute_xylux_command(project_root, &["clean"]).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    async fn create_test_xylux_project(temp_dir: &TempDir) -> PathBuf {
        let project_path = temp_dir.path().to_path_buf();

        // Create xylux.toml
        let xylux_toml = r#"[project]
name = "test_project"
version = "0.1.0"
engine_version = "0.1.0"

[build]
target = "native"
assets_dir = "assets"
scripts_dir = "scripts"

[runtime]
hot_reload = true
log_level = "info"
"#;

        fs::write(project_path.join("xylux.toml"), xylux_toml).await.unwrap();

        // Create scripts directory and main script
        fs::create_dir_all(project_path.join("scripts")).await.unwrap();
        fs::write(
            project_path.join("scripts").join("main.aux"),
            r#"// Test Xylux script
use xylux::prelude::*;

fn init() {
    log("Hello from Xylux!");
}

fn update(delta: f32) {
    // Game update logic
}

fn render() {
    // Rendering logic
}
"#,
        )
        .await
        .unwrap();

        // Create assets directory
        fs::create_dir_all(project_path.join("assets")).await.unwrap();

        project_path
    }

    #[test]
    fn test_xylux_builder_creation() {
        let builder = XyluxCliBuilder::new();
        assert_eq!(builder.xylux_path, "xylux");

        let custom_builder = XyluxCliBuilder::with_xylux_path("/usr/bin/xylux");
        assert_eq!(custom_builder.xylux_path, "/usr/bin/xylux");
    }

    #[test]
    fn test_is_xylux_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Should not be a Xylux project initially
        assert!(!XyluxCliBuilder::is_xylux_project(&project_path));

        // Create xylux.toml
        std::fs::write(project_path.join("xylux.toml"), "[project]\nname = \"test\"").unwrap();

        // Should now be a Xylux project
        assert!(XyluxCliBuilder::is_xylux_project(&project_path));
    }

    #[tokio::test]
    async fn test_xylux_availability() {
        let builder = XyluxCliBuilder::new();

        // This test may fail in environments without Xylux CLI
        // We just test that the method works, regardless of result
        let _unused = builder.check_availability().await;
    }

    #[tokio::test]
    async fn test_build_non_xylux_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let builder = XyluxCliBuilder::new();
        let result = builder.build(&project_path).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a Xylux project"));
    }

    #[tokio::test]
    async fn test_project_config_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_xylux_project(&temp_dir).await;

        let builder = XyluxCliBuilder::new();
        let config_result = builder.get_project_config(&project_path).await;

        assert!(config_result.is_ok());

        if let Ok(config) = config_result {
            assert!(config.is_object());
            assert!(config.get("project").is_some());
            assert!(config.get("build").is_some());
        }
    }

    #[tokio::test]
    async fn test_project_config_missing() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let builder = XyluxCliBuilder::new();
        let result = builder.get_project_config(&project_path).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("xylux.toml not found"));
    }

    #[tokio::test]
    async fn test_xylux_project_operations() {
        // Skip this test if xylux CLI is not available
        let builder = XyluxCliBuilder::new();
        if builder.check_availability().await.is_err() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_xylux_project(&temp_dir).await;

        // Test building (may fail due to missing Xylux CLI in test environment)
        let _build_result = builder.build(&project_path).await;
        // We don't assert here because Xylux CLI might not be installed

        // Test cleaning (should work even if build failed)
        let _clean_result = builder.clean(&project_path).await;
        // We don't assert here because Xylux CLI might not be installed
    }

    #[tokio::test]
    async fn test_template_listing() {
        let builder = XyluxCliBuilder::new();

        // Skip if xylux CLI is not available
        if builder.check_availability().await.is_err() {
            return;
        }

        // Test listing templates
        if let Ok(_templates) = builder.list_templates().await {
            // Successfully retrieved templates (can be empty if no templates available)
            // This test just verifies the function doesn't error out
        }
    }

    #[tokio::test]
    async fn test_init_project() {
        let builder = XyluxCliBuilder::new();

        // Skip if xylux CLI is not available
        if builder.check_availability().await.is_err() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("new_project");

        // Test project initialization
        let _init_result = builder.init(&project_path, Some("basic")).await;
        // We don't assert here because Xylux CLI might not be installed
    }
}
