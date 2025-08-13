//! # Project Module
//!
//! Project management functionality for Xylux IDE.

pub mod manager;
pub mod xylux_project;

pub use manager::ProjectManager;
pub use xylux_project::XyluxProject;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use tracing::info;

use crate::core::Result;

/// Represents different types of projects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Standard Rust project with Cargo.toml.
    Rust,
    /// Xylux game project with xylux.toml.
    Xylux,
    /// Mixed project with both Rust and Xylux components.
    XyluxRust,
    /// Standalone Alux script project.
    Alux,
    /// Unknown or generic project type.
    Unknown,
}

/// Represents a project in the IDE.
#[derive(Debug, Clone)]
pub struct Project {
    /// Project name.
    pub name: String,
    /// Project root directory.
    pub root_path: PathBuf,
    /// Project type.
    pub project_type: ProjectType,
    /// Project configuration file path.
    pub config_path: Option<PathBuf>,
    /// Whether the project is currently open.
    pub is_open: bool,
    /// Recent files in the project.
    pub recent_files: Vec<PathBuf>,
}

impl Project {
    /// Create a new project.
    pub fn new(name: String, root_path: PathBuf, project_type: ProjectType) -> Self {
        Self {
            name,
            root_path,
            project_type,
            config_path: None,
            is_open: false,
            recent_files: Vec::new(),
        }
    }

    /// Detect the project type from the root directory.
    pub fn detect_type(root_path: &PathBuf) -> ProjectType {
        let cargo_toml = root_path.join("Cargo.toml");
        let xylux_toml = root_path.join("xylux.toml");
        let scripts_dir = root_path.join("scripts");

        match (cargo_toml.exists(), xylux_toml.exists(), scripts_dir.exists()) {
            (true, true, _) => ProjectType::XyluxRust,
            (false, true, _) => ProjectType::Xylux,
            (true, false, _) => ProjectType::Rust,
            (false, false, true) => ProjectType::Alux,
            _ => ProjectType::Unknown,
        }
    }

    /// Open the project.
    pub fn open(&mut self) -> Result<()> {
        info!("Opening project: {}", self.name);
        self.is_open = true;
        self.scan_recent_files()?;
        Ok(())
    }

    /// Close the project.
    pub fn close(&mut self) {
        info!("Closing project: {}", self.name);
        self.is_open = false;
        self.recent_files.clear();
    }

    /// Scan for recent files in the project.
    fn scan_recent_files(&mut self) -> Result<()> {
        self.recent_files.clear();

        // Add common files based on project type
        match self.project_type {
            ProjectType::Rust | ProjectType::XyluxRust => {
                self.add_if_exists("Cargo.toml");
                self.add_if_exists("src/main.rs");
                self.add_if_exists("src/lib.rs");
            }
            ProjectType::Xylux => {
                self.add_if_exists("xylux.toml");
                self.add_if_exists("scripts/main.aux");
            }
            ProjectType::Alux => {
                self.add_if_exists("main.aux");
            }
            ProjectType::Unknown => {}
        }

        Ok(())
    }

    /// Add a file to recent files if it exists.
    fn add_if_exists(&mut self, relative_path: &str) {
        let file_path = self.root_path.join(relative_path);
        if file_path.exists() {
            self.recent_files.push(file_path);
        }
    }

    /// Get the project's build command.
    pub fn build_command(&self) -> Option<String> {
        match self.project_type {
            ProjectType::Rust | ProjectType::XyluxRust => Some("cargo build".to_string()),
            ProjectType::Xylux => Some("xylux build".to_string()),
            ProjectType::Alux => Some("alux-compile".to_string()),
            ProjectType::Unknown => None,
        }
    }

    /// Get the project's run command.
    pub fn run_command(&self) -> Option<String> {
        match self.project_type {
            ProjectType::Rust | ProjectType::XyluxRust => Some("cargo run".to_string()),
            ProjectType::Xylux => Some("xylux run".to_string()),
            ProjectType::Alux => Some("alux-vm main.aux".to_string()),
            ProjectType::Unknown => None,
        }
    }

    /// Get the project's test command.
    pub fn test_command(&self) -> Option<String> {
        match self.project_type {
            ProjectType::Rust | ProjectType::XyluxRust => Some("cargo test".to_string()),
            ProjectType::Xylux => Some("xylux test".to_string()),
            ProjectType::Alux => None, // No standard test command for Alux
            ProjectType::Unknown => None,
        }
    }

    /// Check if the project supports hot reload.
    pub fn supports_hot_reload(&self) -> bool {
        matches!(self.project_type, ProjectType::Xylux | ProjectType::XyluxRust | ProjectType::Alux)
    }

    /// Get important directories for this project type.
    pub fn important_directories(&self) -> Vec<PathBuf> {
        let mut dirs = vec![self.root_path.clone()];

        match self.project_type {
            ProjectType::Rust => {
                dirs.push(self.root_path.join("src"));
                dirs.push(self.root_path.join("tests"));
                dirs.push(self.root_path.join("examples"));
            }
            ProjectType::Xylux => {
                dirs.push(self.root_path.join("scripts"));
                dirs.push(self.root_path.join("assets"));
                dirs.push(self.root_path.join("shaders"));
            }
            ProjectType::XyluxRust => {
                // Hybrid project - include both Rust and Xylux directories
                dirs.push(self.root_path.join("src"));
                dirs.push(self.root_path.join("tests"));
                dirs.push(self.root_path.join("examples"));
                dirs.push(self.root_path.join("scripts"));
                dirs.push(self.root_path.join("assets"));
                dirs.push(self.root_path.join("shaders"));
            }
            ProjectType::Alux => {
                dirs.push(self.root_path.join("scripts"));
            }
            ProjectType::Unknown => {}
        }

        dirs.into_iter().filter(|d| d.exists()).collect()
    }

    /// Get file patterns to watch for changes.
    pub fn watch_patterns(&self) -> Vec<String> {
        match self.project_type {
            ProjectType::Rust => {
                vec!["**/*.rs".to_string(), "Cargo.toml".to_string(), "Cargo.lock".to_string()]
            }
            ProjectType::Xylux => vec![
                "**/*.aux".to_string(),
                "**/*.wgsl".to_string(),
                "xylux.toml".to_string(),
                "assets/**/*".to_string(),
            ],
            ProjectType::XyluxRust => vec![
                "**/*.rs".to_string(),
                "**/*.aux".to_string(),
                "**/*.wgsl".to_string(),
                "Cargo.toml".to_string(),
                "xylux.toml".to_string(),
                "assets/**/*".to_string(),
            ],
            ProjectType::Alux => vec!["**/*.aux".to_string()],
            ProjectType::Unknown => vec!["**/*".to_string()],
        }
    }
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Rust => write!(f, "Rust"),
            ProjectType::Xylux => write!(f, "Xylux"),
            ProjectType::XyluxRust => write!(f, "Xylux + Rust"),
            ProjectType::Alux => write!(f, "Alux"),
            ProjectType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_project_type_detection() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().to_path_buf();

        // Initially unknown
        assert_eq!(Project::detect_type(&project_dir), ProjectType::Unknown);

        // Create Cargo.toml
        std::fs::write(project_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        assert_eq!(Project::detect_type(&project_dir), ProjectType::Rust);

        // Add xylux.toml
        std::fs::write(project_dir.join("xylux.toml"), "[project]\nname = \"test\"").unwrap();
        assert_eq!(Project::detect_type(&project_dir), ProjectType::XyluxRust);

        // Remove Cargo.toml
        std::fs::remove_file(project_dir.join("Cargo.toml")).unwrap();
        assert_eq!(Project::detect_type(&project_dir), ProjectType::Xylux);
    }

    #[test]
    fn test_project_commands() {
        let project =
            Project::new("test".to_string(), PathBuf::from("/tmp/test"), ProjectType::Rust);

        assert_eq!(project.build_command(), Some("cargo build".to_string()));
        assert_eq!(project.run_command(), Some("cargo run".to_string()));
        assert_eq!(project.test_command(), Some("cargo test".to_string()));
        assert!(!project.supports_hot_reload());
    }

    #[test]
    fn test_xylux_project_features() {
        let project =
            Project::new("game".to_string(), PathBuf::from("/tmp/game"), ProjectType::Xylux);

        assert!(project.supports_hot_reload());
        assert_eq!(project.build_command(), Some("xylux build".to_string()));

        let patterns = project.watch_patterns();
        assert!(patterns.contains(&"**/*.aux".to_string()));
        assert!(patterns.contains(&"**/*.wgsl".to_string()));
    }

    #[test]
    fn test_project_open_close() {
        let mut project =
            Project::new("test".to_string(), PathBuf::from("/tmp/test"), ProjectType::Unknown);

        assert!(!project.is_open);

        project.open().unwrap();
        assert!(project.is_open);

        project.close();
        assert!(!project.is_open);
        assert!(project.recent_files.is_empty());
    }
}
