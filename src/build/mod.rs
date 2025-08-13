//! # Build Module
//!
//! Build system integration for Xylux IDE.

pub mod alux_compiler;
pub mod cargo;
pub mod xylux_cli;

pub use alux_compiler::AluxCompiler;
pub use cargo::CargoBuilder;
pub use xylux_cli::XyluxCliBuilder;

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::core::config::BuildConfig;
use crate::core::{Config, EventBus, Result, XyluxError};

/// Main build manager that coordinates different build systems.
pub struct BuildManager {
    /// IDE configuration.
    config: Arc<RwLock<Config>>,
    /// Event bus for communication.
    event_bus: Arc<EventBus>,
    /// Current project root.
    project_root: Arc<RwLock<Option<PathBuf>>>,
    /// Cargo builder for Rust projects.
    cargo_builder: CargoBuilder,
    /// Xylux CLI builder for Xylux projects.
    xylux_builder: XyluxCliBuilder,
    /// Alux compiler for Alux scripts.
    alux_compiler: AluxCompiler,
    /// Whether a build is currently in progress.
    building: Arc<RwLock<bool>>,
}

impl BuildManager {
    /// Create a new build manager.
    pub async fn new(config: Arc<RwLock<Config>>, event_bus: Arc<EventBus>) -> Result<Self> {
        debug!("Initializing build manager");

        let cargo_builder = CargoBuilder::new();
        let xylux_builder = XyluxCliBuilder::new();
        let alux_compiler = AluxCompiler::new();

        Ok(Self {
            config,
            event_bus,
            project_root: Arc::new(RwLock::new(None)),
            cargo_builder,
            xylux_builder,
            alux_compiler,
            building: Arc::new(RwLock::new(false)),
        })
    }

    /// Set the project root directory.
    pub async fn set_project_root(&self, root: &PathBuf) -> Result<()> {
        info!("Setting build project root: {}", root.display());
        let mut project_root = self.project_root.write().await;
        *project_root = Some(root.clone());
        Ok(())
    }

    /// Clear the current project.
    pub async fn clear_project(&self) -> Result<()> {
        debug!("Clearing build project");
        let mut project_root = self.project_root.write().await;
        *project_root = None;
        Ok(())
    }

    /// Build the current project.
    pub async fn build(&self) -> Result<()> {
        let project_root = {
            let root = self.project_root.read().await;
            root.clone()
        };

        let Some(root) = project_root else {
            return Err(XyluxError::build_error("No project root set"));
        };

        // Check if already building
        {
            let building = self.building.read().await;
            if *building {
                return Err(XyluxError::build_error("Build already in progress"));
            }
        }

        // Set building flag
        {
            let mut building = self.building.write().await;
            *building = true;
        }

        let result = self.perform_build(&root).await;

        // Clear building flag
        {
            let mut building = self.building.write().await;
            *building = false;
        }

        result
    }

    /// Perform the actual build based on project type.
    async fn perform_build(&self, project_root: &PathBuf) -> Result<()> {
        info!("Building project at: {}", project_root.display());

        // Get build configuration
        let build_config = {
            let config = self.config.read().await;
            config.build.clone()
        };

        // Use auto_build_on_save setting for target naming
        let target = if build_config.auto_build_on_save {
            "auto-build".to_string()
        } else {
            "manual-build".to_string()
        };

        // Publish build started event
        let event = crate::core::EventMessage::from_event(crate::core::Event::Build(
            crate::core::BuildEvent::Started { target: target.clone() },
        ))
        .with_priority(crate::core::EventPriority::Normal)
        .with_source("build_manager");
        self.event_bus.publish(event).await?;

        let start_time = std::time::Instant::now();

        // Detect project type and build accordingly
        let result = if project_root.join("Cargo.toml").exists() {
            self.cargo_builder.build(project_root).await
        } else if project_root.join("xylux.toml").exists() {
            self.xylux_builder.build(project_root).await
        } else if project_root.join("scripts").exists() {
            self.alux_compiler.compile_project(project_root).await
        } else {
            Err(XyluxError::build_error("Unknown project type"))
        };

        let duration = start_time.elapsed();

        // Publish build result event
        match &result {
            Ok(()) => {
                let event = crate::core::EventMessage::from_event(crate::core::Event::Build(
                    crate::core::BuildEvent::Completed { target, duration },
                ))
                .with_priority(crate::core::EventPriority::Normal)
                .with_source("build_manager");
                self.event_bus.publish(event).await?;
                info!("Build completed successfully in {:?}", duration);
            }
            Err(e) => {
                let event = crate::core::EventMessage::from_event(crate::core::Event::Build(
                    crate::core::BuildEvent::Failed { target, error: e.to_string() },
                ))
                .with_priority(crate::core::EventPriority::High)
                .with_source("build_manager");
                self.event_bus.publish(event).await?;
                error!("Build failed: {}", e);
            }
        }

        result
    }

    /// Run the current project.
    pub async fn run(&self) -> Result<()> {
        let project_root = {
            let root = self.project_root.read().await;
            root.clone()
        };

        let Some(root) = project_root else {
            return Err(XyluxError::build_error("No project root set"));
        };

        info!("Running project at: {}", root.display());

        // Build first if needed
        self.build().await?;

        // Run based on project type
        if root.join("Cargo.toml").exists() {
            self.cargo_builder.run(&root).await
        } else if root.join("xylux.toml").exists() {
            self.xylux_builder.run(&root).await
        } else if root.join("scripts").exists() {
            self.alux_compiler.run_project(&root).await
        } else {
            Err(XyluxError::build_error("Unknown project type"))
        }
    }

    /// Test the current project.
    pub async fn test(&self) -> Result<()> {
        let project_root = {
            let root = self.project_root.read().await;
            root.clone()
        };

        let Some(root) = project_root else {
            return Err(XyluxError::build_error("No project root set"));
        };

        info!("Testing project at: {}", root.display());

        // Publish test started event
        let event = crate::core::EventMessage::from_event(crate::core::Event::Build(
            crate::core::BuildEvent::TestsStarted,
        ))
        .with_priority(crate::core::EventPriority::Normal)
        .with_source("build_manager");
        self.event_bus.publish(event).await?;

        // Test based on project type
        let result = if root.join("Cargo.toml").exists() {
            self.cargo_builder.test(&root).await
        } else if root.join("xylux.toml").exists() {
            self.xylux_builder.test(&root).await
        } else {
            Err(XyluxError::build_error("Testing not supported for this project type"))
        };

        // Publish test result (placeholder counts)
        let event = crate::core::EventMessage::from_event(crate::core::Event::Build(
            crate::core::BuildEvent::TestsCompleted {
                passed: if result.is_ok() { 1 } else { 0 },
                failed: if result.is_err() { 1 } else { 0 },
            },
        ))
        .with_priority(crate::core::EventPriority::Normal)
        .with_source("build_manager");
        self.event_bus.publish(event).await?;

        result
    }

    /// Clean build artifacts.
    pub async fn clean(&self) -> Result<()> {
        let project_root = {
            let root = self.project_root.read().await;
            root.clone()
        };

        let Some(root) = project_root else {
            return Err(XyluxError::build_error("No project root set"));
        };

        info!("Cleaning project at: {}", root.display());

        // Clean based on project type
        if root.join("Cargo.toml").exists() {
            self.cargo_builder.clean(&root).await
        } else if root.join("xylux.toml").exists() {
            self.xylux_builder.clean(&root).await
        } else if root.join("scripts").exists() {
            self.alux_compiler.clean_project(&root).await
        } else {
            Err(XyluxError::build_error("Unknown project type"))
        }
    }

    /// Check if a build is currently in progress.
    pub async fn is_building(&self) -> bool {
        let building = self.building.read().await;
        *building
    }

    /// Get the current project root.
    pub async fn get_project_root(&self) -> Option<PathBuf> {
        let root = self.project_root.read().await;
        root.clone()
    }

    /// Execute a custom command in the project directory.
    pub async fn execute_command(&self, command: &str, args: &[&str]) -> Result<String> {
        let project_root = {
            let root = self.project_root.read().await;
            root.clone()
        };

        let Some(root) = project_root else {
            return Err(XyluxError::build_error("No project root set"));
        };

        debug!("Executing command: {} {} in {}", command, args.join(" "), root.display());

        let output = Command::new(command)
            .args(args)
            .current_dir(&root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| XyluxError::build_error(format!("Failed to execute command: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(XyluxError::build_error(format!("Command failed: {}", error_msg)))
        }
    }

    /// Update build configuration.
    pub async fn update_config(&self, build_config: BuildConfig) -> Result<()> {
        {
            let mut config = self.config.write().await;
            config.build = build_config;
        }

        debug!("Updated build manager configuration");
        Ok(())
    }

    /// Get current build configuration.
    pub async fn get_build_config(&self) -> BuildConfig {
        let config = self.config.read().await;
        config.build.clone()
    }

    /// Check if auto-build on save is enabled.
    pub async fn is_auto_build_enabled(&self) -> bool {
        let config = self.config.read().await;
        config.build.auto_build_on_save
    }

    /// Check if build output should be shown.
    pub async fn should_show_build_output(&self) -> bool {
        let config = self.config.read().await;
        config.build.show_build_output
    }

    /// Get build environment variables.
    pub async fn get_build_env_vars(&self) -> std::collections::HashMap<String, String> {
        let config = self.config.read().await;
        config.build.env_vars.clone()
    }

    /// Shutdown the build manager.
    pub async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down build manager");

        // Wait for any ongoing builds to complete
        while self.is_building().await {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }
}

/// Trait for build system implementations.
#[async_trait::async_trait]
pub trait Builder {
    /// Build the project.
    async fn build(&self, project_root: &PathBuf) -> Result<()>;

    /// Run the project.
    async fn run(&self, project_root: &PathBuf) -> Result<()>;

    /// Test the project.
    async fn test(&self, project_root: &PathBuf) -> Result<()>;

    /// Clean build artifacts.
    async fn clean(&self, project_root: &PathBuf) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_build_manager_creation() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());

        let build_manager = BuildManager::new(config, event_bus).await;
        assert!(build_manager.is_ok());
    }

    #[tokio::test]
    async fn test_project_root_management() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let build_manager = BuildManager::new(config, event_bus).await.unwrap();

        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        build_manager.set_project_root(&project_path).await.unwrap();
        assert_eq!(build_manager.get_project_root().await, Some(project_path));

        build_manager.clear_project().await.unwrap();
        assert_eq!(build_manager.get_project_root().await, None);
    }

    #[tokio::test]
    async fn test_build_without_project() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let build_manager = BuildManager::new(config, event_bus).await.unwrap();

        let result = build_manager.build().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No project root set"));
    }

    #[tokio::test]
    async fn test_building_flag() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let build_manager = BuildManager::new(config, event_bus).await.unwrap();

        assert!(!build_manager.is_building().await);

        // The building flag is managed internally during build operations
        // This test just verifies the flag can be read
    }
}
