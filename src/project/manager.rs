//! # Project Manager
//!
//! Central manager for handling multiple projects in Xylux IDE.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::core::{Config, EventBus, EventMessage, Result, XyluxError};
use crate::project::{Project, ProjectType};

/// Project manager for handling multiple projects.
pub struct ProjectManager {
    /// Currently open projects.
    projects: Arc<RwLock<HashMap<PathBuf, Project>>>,
    /// Currently active project.
    active_project: Arc<RwLock<Option<PathBuf>>>,
    /// Recent projects list.
    recent_projects: Arc<RwLock<Vec<PathBuf>>>,
    /// Configuration.
    config: Arc<RwLock<Config>>,
    /// Event bus for project-related events.
    event_bus: Arc<EventBus>,
    /// Maximum number of recent projects to remember.
    max_recent_projects: usize,
}

/// Project manager configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManagerConfig {
    /// Maximum number of recent projects.
    pub max_recent_projects: usize,
    /// Auto-save project state.
    pub auto_save_state: bool,
    /// Watch project files for changes.
    pub watch_files: bool,
    /// Default project template.
    pub default_template: String,
}

impl Default for ProjectManagerConfig {
    fn default() -> Self {
        Self {
            max_recent_projects: 10,
            auto_save_state: true,
            watch_files: true,
            default_template: "rust-basic".to_string(),
        }
    }
}

/// Project state for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectState {
    path: PathBuf,
    name: String,
    project_type: ProjectType,
    recent_files: Vec<PathBuf>,
}

impl ProjectManager {
    /// Create a new project manager.
    pub fn new(config: Arc<RwLock<Config>>, event_bus: Arc<EventBus>) -> Self {
        Self {
            projects: Arc::new(RwLock::new(HashMap::new())),
            active_project: Arc::new(RwLock::new(None)),
            recent_projects: Arc::new(RwLock::new(Vec::new())),
            config,
            event_bus,
            max_recent_projects: 10, // Will be overridden by config
        }
    }

    /// Initialize the project manager.
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing project manager");

        // Update max_recent_projects from config
        {
            let _config = self.config.read().await;
            // Note: Using default value since config doesn't have max_recent_projects field yet
            self.max_recent_projects = 10;
        }

        // Load recent projects from config
        self.load_recent_projects().await?;

        // Register for file system events
        self.setup_file_watching().await?;

        Ok(())
    }

    /// Open a project from a directory.
    pub async fn open_project<P: AsRef<Path>>(&self, project_path: P) -> Result<()> {
        let project_path = project_path.as_ref().to_path_buf();
        let canonical_path = project_path
            .canonicalize()
            .map_err(|e| XyluxError::io(e, "Failed to canonicalize project path"))?;

        info!("Opening project: {:?}", canonical_path);

        // Check if project is already open
        {
            let projects = self.projects.read().await;
            if projects.contains_key(&canonical_path) {
                warn!("Project already open: {:?}", canonical_path);
                self.set_active_project(Some(canonical_path.clone())).await?;
                return Ok(());
            }
        }

        // Detect project type
        let project_type = Project::detect_type(&canonical_path);

        // Get project name from directory name
        let project_name = canonical_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Create project instance
        let mut project = Project::new(project_name, canonical_path.clone(), project_type.clone());
        project.open()?;

        // Add to projects map
        {
            let mut projects = self.projects.write().await;
            projects.insert(canonical_path.clone(), project);
        }

        // Set as active project
        self.set_active_project(Some(canonical_path.clone())).await?;

        // Add to recent projects
        self.add_to_recent_projects(canonical_path.clone()).await;

        // Emit project opened event
        let event = EventMessage::from_event(crate::core::Event::Project(
            crate::core::ProjectEvent::Opened {
                path: canonical_path.clone(),
                project_type: format!("{:?}", project_type),
            },
        ))
        .with_priority(crate::core::EventPriority::Normal)
        .with_source("project_manager");

        self.event_bus.publish(event).await?;

        Ok(())
    }

    /// Close a project.
    pub async fn close_project<P: AsRef<Path>>(&self, project_path: P) -> Result<()> {
        let project_path = project_path.as_ref().to_path_buf();

        info!("Closing project: {:?}", project_path);

        // Remove from projects map
        let mut project = {
            let mut projects = self.projects.write().await;
            projects.remove(&project_path)
        };

        if let Some(ref mut project) = project {
            project.close();

            // If this was the active project, clear active project
            {
                let mut active = self.active_project.write().await;
                if active.as_ref() == Some(&project_path) {
                    *active = None;
                }
            }

            // Emit project closed event
            let event = EventMessage::from_event(crate::core::Event::Project(
                crate::core::ProjectEvent::Closed { path: project_path.clone() },
            ))
            .with_priority(crate::core::EventPriority::Normal)
            .with_source("project_manager");

            self.event_bus.publish(event).await?;
        }

        Ok(())
    }

    /// Close all projects.
    pub async fn close_all_projects(&self) -> Result<()> {
        info!("Closing all projects");

        let project_paths: Vec<PathBuf> = {
            let projects = self.projects.read().await;
            projects.keys().cloned().collect()
        };

        for path in project_paths {
            self.close_project(path).await?;
        }

        Ok(())
    }

    /// Get the currently active project.
    pub async fn active_project(&self) -> Option<Project> {
        let active_path = self.active_project.read().await.clone()?;
        let projects = self.projects.read().await;
        projects.get(&active_path).cloned()
    }

    /// Set the active project.
    pub async fn set_active_project(&self, project_path: Option<PathBuf>) -> Result<()> {
        {
            let mut active = self.active_project.write().await;
            *active = project_path.clone();
        }

        // Emit active project changed event
        if let Some(path) = project_path.clone() {
            let event = EventMessage::from_event(crate::core::Event::Project(
                crate::core::ProjectEvent::ConfigChanged { path },
            ))
            .with_priority(crate::core::EventPriority::Normal)
            .with_source("project_manager");

            self.event_bus.publish(event).await?;
        }

        Ok(())
    }

    /// Get all open projects.
    pub async fn open_projects(&self) -> Vec<Project> {
        let projects = self.projects.read().await;
        projects.values().cloned().collect()
    }

    /// Get recent projects.
    pub async fn recent_projects(&self) -> Vec<PathBuf> {
        let recent = self.recent_projects.read().await;
        recent.clone()
    }

    /// Create a new project.
    pub async fn create_project(
        &self,
        name: &str,
        location: &Path,
        project_type: ProjectType,
        template: Option<&str>,
    ) -> Result<PathBuf> {
        let project_path = location.join(name);

        info!("Creating new project: {:?} (type: {})", project_path, project_type);

        // Get default template from config if none provided
        let template = if template.is_none() {
            let config = self.config.read().await;
            // Use first default template if available
            if let Some(default_template) = config.project.default_templates.get("default") {
                Some(default_template.to_string_lossy().to_string())
            } else {
                Some("basic".to_string())
            }
        } else {
            template.map(|s| s.to_string())
        };

        // Create project directory
        tokio::fs::create_dir_all(&project_path)
            .await
            .map_err(|e| XyluxError::io(e, "Failed to create project directory"))?;

        // Initialize project based on type
        match project_type {
            ProjectType::Rust => {
                self.create_rust_project(&project_path, name, template.as_deref()).await?
            }
            ProjectType::Xylux => {
                self.create_xylux_project(&project_path, name, template.as_deref()).await?
            }
            ProjectType::XyluxRust => {
                self.create_rust_project(&project_path, name, template.as_deref()).await?;
                self.create_xylux_project(&project_path, name, Some("xylux-addon")).await?;
            }
            ProjectType::Alux => {
                self.create_alux_project(&project_path, name, template.as_deref()).await?
            }
            ProjectType::Unknown => {
                return Err(XyluxError::invalid_input("Cannot create project of unknown type"));
            }
        }

        // Open the newly created project
        self.open_project(&project_path).await?;

        Ok(project_path)
    }

    /// Create a Rust project.
    async fn create_rust_project(
        &self,
        project_path: &Path,
        name: &str,
        template: Option<&str>,
    ) -> Result<()> {
        // Create Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
            name
        );

        tokio::fs::write(project_path.join("Cargo.toml"), cargo_toml)
            .await
            .map_err(|e| XyluxError::io(e, "Failed to create Cargo.toml"))?;

        // Create src directory and main.rs
        let src_dir = project_path.join("src");
        tokio::fs::create_dir_all(&src_dir)
            .await
            .map_err(|e| XyluxError::io(e, "Failed to create src directory"))?;

        let main_rs = match template {
            Some("rust-lib") => {
                r#"//! # {name}
//!
//! A new Rust library.

pub fn hello() -> String {
    "Hello, World!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello, World!");
    }
}
"#
            }
            Some("rust-bin") | _ => {
                r#"fn main() {
    println!("Hello, World!");
}
"#
            }
        };

        let main_file = if template == Some("rust-lib") { "lib.rs" } else { "main.rs" };

        tokio::fs::write(src_dir.join(main_file), main_rs.replace("{name}", name))
            .await
            .map_err(|e| XyluxError::io(e, "Failed to create main source file"))?;

        // Create .gitignore
        let gitignore = r#"/target/
Cargo.lock
"#;

        tokio::fs::write(project_path.join(".gitignore"), gitignore)
            .await
            .map_err(|e| XyluxError::io(e, "Failed to create .gitignore"))?;

        Ok(())
    }

    /// Create a Xylux project.
    async fn create_xylux_project(
        &self,
        project_path: &Path,
        name: &str,
        template: Option<&str>,
    ) -> Result<()> {
        // Create xylux.toml
        let xylux_toml = format!(
            r#"[project]
name = "{}"
version = "0.1.0"
engine_version = "0.1.0"

[build]
target = "native"
assets_dir = "assets"
scripts_dir = "scripts"

[runtime]
hot_reload = true
log_level = "debug"
"#,
            name
        );

        tokio::fs::write(project_path.join("xylux.toml"), xylux_toml)
            .await
            .map_err(|e| XyluxError::io(e, "Failed to create xylux.toml"))?;

        // Create directories
        for dir in &["scripts", "assets", "shaders"] {
            tokio::fs::create_dir_all(project_path.join(dir))
                .await
                .map_err(|e| XyluxError::io(e, &format!("Failed to create {} directory", dir)))?;
        }

        // Create main script
        let main_script = match template {
            Some("xylux-3d") => {
                r#"// Xylux 3D Game Script
use xylux::prelude::*;

fn init() {
    log("Initializing 3D game...");

    // Set up 3D scene
    scene::set_background_color(0.1, 0.1, 0.1);
    scene::set_camera_position(0.0, 5.0, 10.0);
}

fn update(delta: f32) {
    // Game update logic here
    if input::key_pressed(Key::Escape) {
        engine::quit();
    }
}

fn render() {
    // Custom rendering logic here
}
"#
            }
            Some("xylux-2d") => {
                r#"// Xylux 2D Game Script
use xylux::prelude::*;

fn init() {
    log("Initializing 2D game...");

    // Set up 2D scene
    scene::set_background_color(0.2, 0.3, 0.8);
    renderer::set_mode(RenderMode::Mode2D);
}

fn update(delta: f32) {
    // Game update logic here
    if input::key_pressed(Key::Escape) {
        engine::quit();
    }
}

fn render() {
    // 2D rendering logic here
}
"#
            }
            _ => {
                r#"// Xylux Game Script
use xylux::prelude::*;

fn init() {
    log("Hello from Xylux!");
}

fn update(delta: f32) {
    if input::key_pressed(Key::Escape) {
        engine::quit();
    }
}

fn render() {
    // Rendering logic here
}
"#
            }
        };

        tokio::fs::write(project_path.join("scripts").join("main.aux"), main_script)
            .await
            .map_err(|e| XyluxError::io(e, "Failed to create main.aux"))?;

        Ok(())
    }

    /// Create an Alux script project.
    async fn create_alux_project(
        &self,
        project_path: &Path,
        name: &str,
        _template: Option<&str>,
    ) -> Result<()> {
        // Create main script
        let main_script = format!(
            r#"// {} - Alux Script
//
// A standalone Alux script project.

fn main() {{
    print("Hello from {}!");
}}
"#,
            name, name
        );

        tokio::fs::write(project_path.join("main.aux"), main_script)
            .await
            .map_err(|e| XyluxError::io(e, "Failed to create main.aux"))?;

        // Create .gitignore
        let gitignore = r#"*.aux.cache
/target/
"#;

        tokio::fs::write(project_path.join(".gitignore"), gitignore)
            .await
            .map_err(|e| XyluxError::io(e, "Failed to create .gitignore"))?;

        Ok(())
    }

    /// Add a project to recent projects list.
    async fn add_to_recent_projects(&self, project_path: PathBuf) {
        let mut recent = self.recent_projects.write().await;

        // Remove if already exists
        recent.retain(|p| p != &project_path);

        // Add to front
        recent.insert(0, project_path);

        // Get max recent projects from config
        let max_recent = {
            let _config = self.config.read().await;
            // Use default value since config doesn't have this field yet
            self.max_recent_projects
        };

        // Limit size
        if recent.len() > max_recent {
            recent.truncate(max_recent);
        }

        // Save to config if auto-save is enabled
        let should_save = {
            let config = self.config.read().await;
            config.project.auto_reload_files
        };

        if should_save {
            if let Err(e) = self.save_recent_projects().await {
                warn!("Failed to save recent projects: {}", e);
            }
        }
    }

    /// Load recent projects from configuration.
    async fn load_recent_projects(&self) -> Result<()> {
        // TODO: Implement actual loading from config file
        // For now, this is a placeholder since the config structure doesn't have recent_projects field
        debug!("Loaded {} recent projects", self.recent_projects.read().await.len());
        Ok(())
    }

    /// Save recent projects to configuration.
    async fn save_recent_projects(&self) -> Result<()> {
        // TODO: Implement actual saving to config file
        // For now, this is a placeholder since the config structure doesn't have recent_projects field
        debug!("Saved recent projects to configuration");
        Ok(())
    }

    /// Setup file system watching for open projects.
    async fn setup_file_watching(&self) -> Result<()> {
        let watch_enabled = {
            let config = self.config.read().await;
            config.project.file_watching
        };

        if watch_enabled {
            debug!("File watching is enabled in configuration");
            // TODO: Implement actual file watching using notify crate
            // This would watch for changes in project files and emit events
        } else {
            debug!("File watching is disabled in configuration");
        }

        Ok(())
    }

    /// Get project templates.
    pub fn available_templates(&self, project_type: ProjectType) -> Vec<&'static str> {
        match project_type {
            ProjectType::Rust => vec!["rust-bin", "rust-lib"],
            ProjectType::Xylux => vec!["xylux-2d", "xylux-3d", "xylux-minimal"],
            ProjectType::XyluxRust => vec!["xylux-rust-game", "xylux-rust-engine"],
            ProjectType::Alux => vec!["alux-script"],
            ProjectType::Unknown => vec![],
        }
    }

    /// Update project manager configuration.
    pub async fn update_config(&self, project_config: ProjectManagerConfig) -> Result<()> {
        {
            let mut config = self.config.write().await;
            config.project.file_watching = project_config.watch_files;
            config.project.auto_reload_files = project_config.auto_save_state;
            // Store default template in the templates map
            config
                .project
                .default_templates
                .insert("default".to_string(), PathBuf::from(project_config.default_template));
        }

        debug!("Updated project manager configuration");
        Ok(())
    }

    /// Get current project configuration.
    pub async fn get_config(&self) -> ProjectManagerConfig {
        let config = self.config.read().await;
        let default_template = config
            .project
            .default_templates
            .get("default")
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "basic".to_string());

        ProjectManagerConfig {
            max_recent_projects: 10, // Use hardcoded default for now
            auto_save_state: config.project.auto_reload_files,
            watch_files: config.project.file_watching,
            default_template,
        }
    }

    /// Check if auto-save is enabled for project state.
    pub async fn is_auto_save_enabled(&self) -> bool {
        let config = self.config.read().await;
        config.project.auto_reload_files
    }

    /// Check if file watching is enabled.
    pub async fn is_file_watching_enabled(&self) -> bool {
        let config = self.config.read().await;
        config.project.file_watching
    }

    /// Get the default template for a project type.
    pub async fn get_default_template(&self, project_type: ProjectType) -> String {
        let config = self.config.read().await;
        let default = config
            .project
            .default_templates
            .get("default")
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "basic".to_string());

        // Validate that the default template is available for this project type
        let available = self.available_templates(project_type);
        if available.contains(&default.as_str()) {
            default
        } else {
            // Fallback to first available template
            available.first().unwrap_or(&"basic").to_string()
        }
    }

    /// Shutdown the project manager.
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down project manager");

        // Save recent projects
        self.save_recent_projects().await?;

        // Close all projects
        self.close_all_projects().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_project_manager_creation() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let manager = ProjectManager::new(config, event_bus);

        assert!(manager.open_projects().await.is_empty());
        assert!(manager.active_project().await.is_none());
    }

    #[tokio::test]
    async fn test_create_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let manager = ProjectManager::new(config, event_bus);

        let project_path = manager
            .create_project("test_project", temp_dir.path(), ProjectType::Rust, None)
            .await
            .unwrap();

        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src").join("main.rs").exists());
        assert!(project_path.join(".gitignore").exists());
    }

    #[tokio::test]
    async fn test_create_xylux_project() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let manager = ProjectManager::new(config, event_bus);

        let project_path = manager
            .create_project("game_project", temp_dir.path(), ProjectType::Xylux, None)
            .await
            .unwrap();

        assert!(project_path.join("xylux.toml").exists());
        assert!(project_path.join("scripts").join("main.aux").exists());
        assert!(project_path.join("assets").exists());
    }

    #[tokio::test]
    async fn test_open_close_project() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let manager = ProjectManager::new(config, event_bus);

        // Create a test project
        let project_path = manager
            .create_project("test_project", temp_dir.path(), ProjectType::Rust, None)
            .await
            .unwrap();

        // Verify it's open and active
        assert_eq!(manager.open_projects().await.len(), 1);
        assert!(manager.active_project().await.is_some());

        // Close it
        manager.close_project(&project_path).await.unwrap();

        // Verify it's closed
        assert!(manager.open_projects().await.is_empty());
        assert!(manager.active_project().await.is_none());
    }

    #[tokio::test]
    async fn test_recent_projects() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let manager = ProjectManager::new(config, event_bus);

        // Create and open multiple projects
        let project1 = manager
            .create_project("project1", temp_dir.path(), ProjectType::Rust, None)
            .await
            .unwrap();

        let project2 = manager
            .create_project("project2", temp_dir.path(), ProjectType::Xylux, None)
            .await
            .unwrap();

        let recent = manager.recent_projects().await;
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], project2); // Most recent first
        assert_eq!(recent[1], project1);
    }

    #[tokio::test]
    async fn test_project_templates() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let manager = ProjectManager::new(config, event_bus);

        let rust_templates = manager.available_templates(ProjectType::Rust);
        assert!(rust_templates.contains(&"rust-bin"));
        assert!(rust_templates.contains(&"rust-lib"));

        let xylux_templates = manager.available_templates(ProjectType::Xylux);
        assert!(xylux_templates.contains(&"xylux-2d"));
        assert!(xylux_templates.contains(&"xylux-3d"));
    }
}
