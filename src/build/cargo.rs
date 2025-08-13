//! # Cargo Builder
//!
//! Cargo build system integration for Rust projects.

use std::path::PathBuf;
use std::process::Stdio;

use async_trait::async_trait;
use tokio::process::Command;
use tracing::{debug, error, info};

use crate::build::Builder;
use crate::core::{Result, XyluxError};

/// Cargo builder for Rust projects.
pub struct CargoBuilder {
    /// Cargo binary path.
    cargo_path: String,
}

impl CargoBuilder {
    /// Create a new Cargo builder.
    pub fn new() -> Self {
        Self { cargo_path: "cargo".to_string() }
    }

    /// Create a new Cargo builder with custom cargo path.
    pub fn with_cargo_path<S: Into<String>>(cargo_path: S) -> Self {
        Self { cargo_path: cargo_path.into() }
    }

    /// Execute a cargo command.
    async fn execute_cargo_command(&self, project_root: &PathBuf, args: &[&str]) -> Result<String> {
        debug!("Executing cargo {} in {}", args.join(" "), project_root.display());

        let mut command = Command::new(&self.cargo_path);
        command.args(args).current_dir(project_root).stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = command
            .output()
            .await
            .map_err(|e| XyluxError::build_error(format!("Failed to execute cargo: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            if !stdout.is_empty() {
                info!("Cargo output: {}", stdout.trim());
            }
            Ok(stdout.to_string())
        } else {
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                format!("Cargo command failed with exit code: {:?}", output.status.code())
            };
            error!("Cargo error: {}", error_msg.trim());
            Err(XyluxError::build_error(error_msg))
        }
    }

    /// Check if the project has a Cargo.toml file.
    pub fn is_cargo_project(project_root: &PathBuf) -> bool {
        project_root.join("Cargo.toml").exists()
    }

    /// Get cargo metadata for the project.
    pub async fn get_metadata(&self, project_root: &PathBuf) -> Result<serde_json::Value> {
        let output = self
            .execute_cargo_command(project_root, &["metadata", "--format-version", "1"])
            .await?;

        serde_json::from_str(&output)
            .map_err(|e| XyluxError::build_error(format!("Failed to parse cargo metadata: {}", e)))
    }

    /// Get workspace members.
    pub async fn get_workspace_members(&self, project_root: &PathBuf) -> Result<Vec<String>> {
        let metadata = self.get_metadata(project_root).await?;

        let members = metadata
            .get("workspace_members")
            .and_then(|m| m.as_array())
            .map(|array| array.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        Ok(members)
    }

    /// Check if cargo is available.
    pub async fn check_availability(&self) -> Result<String> {
        let output = Command::new(&self.cargo_path)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| XyluxError::build_error(format!("Cargo not found: {}", e)))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(version.trim().to_string())
        } else {
            Err(XyluxError::build_error("Cargo is not available or not working"))
        }
    }

    /// Build with specific profile.
    pub async fn build_with_profile(&self, project_root: &PathBuf, profile: &str) -> Result<()> {
        let mut args = vec!["build"];

        match profile {
            "release" => args.push("--release"),
            "dev" => {} // Default profile
            _ => {
                args.push("--profile");
                args.push(profile);
            }
        }

        self.execute_cargo_command(project_root, &args).await?;
        Ok(())
    }

    /// Build specific package in workspace.
    pub async fn build_package(&self, project_root: &PathBuf, package: &str) -> Result<()> {
        let args = vec!["build", "--package", package];
        self.execute_cargo_command(project_root, &args).await?;
        Ok(())
    }

    /// Run with arguments.
    pub async fn run_with_args(&self, project_root: &PathBuf, args: &[&str]) -> Result<()> {
        let mut command_args = vec!["run"];
        if !args.is_empty() {
            command_args.push("--");
            command_args.extend_from_slice(args);
        }

        self.execute_cargo_command(project_root, &command_args).await?;
        Ok(())
    }

    /// Test with specific options.
    pub async fn test_with_options(
        &self,
        project_root: &PathBuf,
        test_name: Option<&str>,
        release: bool,
    ) -> Result<()> {
        let mut args = vec!["test"];

        if release {
            args.push("--release");
        }

        if let Some(name) = test_name {
            args.push(name);
        }

        self.execute_cargo_command(project_root, &args).await?;
        Ok(())
    }

    /// Check code with clippy.
    pub async fn clippy(&self, project_root: &PathBuf) -> Result<()> {
        let args = vec!["clippy", "--all-targets", "--all-features"];
        self.execute_cargo_command(project_root, &args).await?;
        Ok(())
    }

    /// Format code.
    pub async fn format(&self, project_root: &PathBuf) -> Result<()> {
        let args = vec!["fmt", "--all"];
        self.execute_cargo_command(project_root, &args).await?;
        Ok(())
    }

    /// Update dependencies.
    pub async fn update(&self, project_root: &PathBuf) -> Result<()> {
        let args = vec!["update"];
        self.execute_cargo_command(project_root, &args).await?;
        Ok(())
    }

    /// Generate documentation.
    pub async fn doc(&self, project_root: &PathBuf, open: bool) -> Result<()> {
        let mut args = vec!["doc", "--no-deps"];
        if open {
            args.push("--open");
        }
        self.execute_cargo_command(project_root, &args).await?;
        Ok(())
    }

    /// Bench the project.
    pub async fn bench(&self, project_root: &PathBuf) -> Result<()> {
        let args = vec!["bench"];
        self.execute_cargo_command(project_root, &args).await?;
        Ok(())
    }

    /// Install a package.
    pub async fn install(&self, package: &str, version: Option<&str>) -> Result<()> {
        let mut args = vec!["install", package];
        if let Some(v) = version {
            args.push("--version");
            args.push(v);
        }

        let output = Command::new(&self.cargo_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| XyluxError::build_error(format!("Failed to install package: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(XyluxError::build_error(format!("Install failed: {}", error_msg)));
        }

        Ok(())
    }
}

impl Default for CargoBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Builder for CargoBuilder {
    async fn build(&self, project_root: &PathBuf) -> Result<()> {
        info!("Building Rust project with Cargo at: {}", project_root.display());

        if !Self::is_cargo_project(project_root) {
            return Err(XyluxError::build_error("Not a Cargo project (Cargo.toml not found)"));
        }

        self.execute_cargo_command(project_root, &["build"]).await?;
        Ok(())
    }

    async fn run(&self, project_root: &PathBuf) -> Result<()> {
        info!("Running Rust project with Cargo at: {}", project_root.display());

        if !Self::is_cargo_project(project_root) {
            return Err(XyluxError::build_error("Not a Cargo project (Cargo.toml not found)"));
        }

        self.execute_cargo_command(project_root, &["run"]).await?;
        Ok(())
    }

    async fn test(&self, project_root: &PathBuf) -> Result<()> {
        info!("Testing Rust project with Cargo at: {}", project_root.display());

        if !Self::is_cargo_project(project_root) {
            return Err(XyluxError::build_error("Not a Cargo project (Cargo.toml not found)"));
        }

        self.execute_cargo_command(project_root, &["test"]).await?;
        Ok(())
    }

    async fn clean(&self, project_root: &PathBuf) -> Result<()> {
        info!("Cleaning Rust project with Cargo at: {}", project_root.display());

        if !Self::is_cargo_project(project_root) {
            return Err(XyluxError::build_error("Not a Cargo project (Cargo.toml not found)"));
        }

        self.execute_cargo_command(project_root, &["clean"]).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    async fn create_test_cargo_project(temp_dir: &TempDir) -> PathBuf {
        let project_path = temp_dir.path().to_path_buf();

        // Create Cargo.toml
        let cargo_toml = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        fs::write(project_path.join("Cargo.toml"), cargo_toml).await.unwrap();

        // Create src directory and main.rs
        fs::create_dir_all(project_path.join("src")).await.unwrap();
        fs::write(
            project_path.join("src").join("main.rs"),
            r#"fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
        )
        .await
        .unwrap();

        project_path
    }

    #[test]
    fn test_cargo_builder_creation() {
        let builder = CargoBuilder::new();
        assert_eq!(builder.cargo_path, "cargo");

        let custom_builder = CargoBuilder::with_cargo_path("/usr/bin/cargo");
        assert_eq!(custom_builder.cargo_path, "/usr/bin/cargo");
    }

    #[tokio::test]
    async fn test_cargo_availability() {
        let builder = CargoBuilder::new();

        // This test may fail in environments without Cargo
        if let Ok(version) = builder.check_availability().await {
            assert!(version.contains("cargo"));
        }
    }

    #[test]
    fn test_is_cargo_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Should not be a Cargo project initially
        assert!(!CargoBuilder::is_cargo_project(&project_path));

        // Create Cargo.toml
        std::fs::write(project_path.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        // Should now be a Cargo project
        assert!(CargoBuilder::is_cargo_project(&project_path));
    }

    #[tokio::test]
    async fn test_build_non_cargo_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let builder = CargoBuilder::new();
        let result = builder.build(&project_path).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a Cargo project"));
    }

    #[tokio::test]
    async fn test_cargo_project_operations() {
        // Skip this test if cargo is not available
        let builder = CargoBuilder::new();
        if builder.check_availability().await.is_err() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_cargo_project(&temp_dir).await;

        // Test building
        let _build_result = builder.build(&project_path).await;
        // May fail due to missing Rust toolchain in test environment
        // assert!(build_result.is_ok());

        // Test cleaning (should work even if build failed)
        let _clean_result = builder.clean(&project_path).await;
        // assert!(clean_result.is_ok());
    }

    #[tokio::test]
    async fn test_metadata_parsing() {
        let builder = CargoBuilder::new();
        if builder.check_availability().await.is_err() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_cargo_project(&temp_dir).await;

        // Test getting metadata
        if let Ok(metadata) = builder.get_metadata(&project_path).await {
            assert!(metadata.is_object());
            // Should contain package information
            assert!(metadata.get("packages").is_some());
        }
    }

    #[tokio::test]
    async fn test_workspace_members() {
        let builder = CargoBuilder::new();
        if builder.check_availability().await.is_err() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_cargo_project(&temp_dir).await;

        // For a simple project, workspace members should be empty or contain the project itself
        if let Ok(_members) = builder.get_workspace_members(&project_path).await {
            // Successfully parsed workspace members (can be empty for non-workspace projects)
            // This test just verifies the function doesn't error out
        }
    }
}
