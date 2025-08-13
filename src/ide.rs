//! # Main IDE Implementation
//!
//! Core IDE structure that orchestrates all components and manages the application lifecycle.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::build::BuildManager;
use crate::core::{
    Config, Event, EventBus, EventHandler, EventMessage, EventPriority, Result, SystemEvent,
};
use crate::editor::Editor;
use crate::project::ProjectManager;
use crate::syntax::SyntaxManager;
// UI Manager removed (TUI). This file remains for potential future headless coordination.

/// Main IDE application structure.
#[derive(Clone)]
pub struct XyluxIde {
    /// IDE configuration.
    config: Arc<RwLock<Config>>,
    /// Central event bus.
    event_bus: Arc<EventBus>,
    /// UI manager.
    ui_manager: Arc<RwLock<UiManager>>,
    /// Editor component.
    editor: Arc<Editor>,
    /// Project manager.
    project_manager: Arc<ProjectManager>,
    /// Syntax and LSP manager.
    syntax_manager: Arc<SyntaxManager>,
    /// Build system manager.
    build_manager: Arc<BuildManager>,
    /// Shutdown flag.
    shutdown_requested: Arc<RwLock<bool>>,
    /// Current project path.
    current_project: Arc<RwLock<Option<PathBuf>>>,
}

impl XyluxIde {
    /// Create a new Xylux IDE instance.
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Xylux IDE");

        let config = Arc::new(RwLock::new(config));
        let event_bus = Arc::new(EventBus::new());
        let shutdown_requested = Arc::new(RwLock::new(false));
        let current_project = Arc::new(RwLock::new(None));

        // Initialize managers
        let ui_manager = Arc::new(RwLock::new(
            UiManager::new(config.clone(), event_bus.clone(), shutdown_requested.clone()).await?,
        ));
        let editor = Arc::new(Editor::new(config.clone(), event_bus.clone()).await?);
        let project_manager = Arc::new(ProjectManager::new(config.clone(), event_bus.clone()));
        let syntax_manager = Arc::new(SyntaxManager::new(config.clone(), event_bus.clone()).await?);
        let build_manager = Arc::new(BuildManager::new(config.clone(), event_bus.clone()).await?);

        let ide = Self {
            config,
            event_bus,
            ui_manager,
            editor,
            project_manager,
            syntax_manager,
            build_manager,
            shutdown_requested,
            current_project,
        };

        // Register event handlers
        ide.register_event_handlers().await?;

        info!("Xylux IDE initialized successfully");
        Ok(ide)
    }

    /// Run the IDE main loop.
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting Xylux IDE");

        // Start background tasks
        let shutdown_flag = self.shutdown_requested.clone();
        let event_bus = self.event_bus.clone();

        // Performance monitoring task
        let perf_shutdown = shutdown_flag.clone();
        let perf_event_bus = event_bus.clone();
        tokio::spawn(async move {
            Self::performance_monitor_task(perf_shutdown, perf_event_bus).await;
        });

        // Auto-save task
        let autosave_shutdown = shutdown_flag.clone();
        let autosave_config = self.config.clone();
        let autosave_editor = self.editor.clone();
        tokio::spawn(async move {
            Self::auto_save_task(autosave_shutdown, autosave_config, autosave_editor).await;
        });

        // Main UI loop
        self.ui_manager.write().await.run().await?;

        info!("Xylux IDE shutting down");
        self.shutdown().await?;

        Ok(())
    }

    /// Open a file or project.
    pub async fn open<P: Into<PathBuf>>(&self, path: P) -> Result<()> {
        let path = path.into();
        info!("Opening: {}", path.display());

        if path.is_dir() {
            // Open as project
            self.project_manager.open_project(&path).await?;
            let mut current_project = self.current_project.write().await;
            *current_project = Some(path.clone());

            // Publish project opened event
            let event =
                EventMessage::from_event(Event::Project(crate::core::ProjectEvent::Opened {
                    path,
                    project_type: "auto-detected".to_string(),
                }))
                .with_priority(EventPriority::High)
                .with_source("ide");

            self.event_bus.publish(event).await?;
        } else {
            // Open as file
            self.editor.open_file(&path).await?;

            // Publish file opened event
            let event =
                EventMessage::from_event(Event::Editor(crate::core::EditorEvent::FileOpened {
                    path,
                }))
                .with_priority(EventPriority::Normal)
                .with_source("ide");

            self.event_bus.publish(event).await?;
        }

        Ok(())
    }

    /// Create a new project.
    pub async fn create_project<P: Into<PathBuf>>(&self, path: P, template: &str) -> Result<()> {
        let path = path.into();
        info!("Creating new project at: {}", path.display());

        let project_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("new_project");
        let parent_dir = path.parent().unwrap_or(std::path::Path::new("."));

        let project_type = if template == "rust" {
            crate::project::ProjectType::Rust
        } else if template == "xylux" {
            crate::project::ProjectType::Xylux
        } else {
            crate::project::ProjectType::Unknown
        };

        let created_path = self
            .project_manager
            .create_project(project_name, parent_dir, project_type, Some(template))
            .await?;

        self.open(created_path).await?;

        Ok(())
    }

    /// Save the current file or project.
    pub async fn save(&self) -> Result<()> {
        debug!("Saving current file/project");
        self.editor.save_current().await?;
        Ok(())
    }

    /// Save all open files.
    pub async fn save_all(&self) -> Result<()> {
        debug!("Saving all open files");
        self.editor.save_all().await?;
        Ok(())
    }

    /// Build the current project.
    pub async fn build(&self) -> Result<()> {
        info!("Building current project");
        self.build_manager.build().await?;
        Ok(())
    }

    /// Run the current project.
    pub async fn run_project(&self) -> Result<()> {
        info!("Running current project");
        self.build_manager.run().await?;
        Ok(())
    }

    /// Test the current project.
    pub async fn test(&self) -> Result<()> {
        info!("Testing current project");
        self.build_manager.test().await?;
        Ok(())
    }

    /// Request shutdown.
    pub async fn request_shutdown(&self) -> Result<()> {
        info!("Shutdown requested");

        let event = EventMessage::from_event(Event::System(SystemEvent::ShutdownRequested))
            .with_priority(EventPriority::Critical)
            .with_source("ide");

        self.event_bus.publish(event).await?;

        let mut shutdown = self.shutdown_requested.write().await;
        *shutdown = true;

        Ok(())
    }

    /// Get the current configuration.
    pub async fn get_config(&self) -> Config {
        let config = self.config.read().await;
        config.clone()
    }

    /// Update configuration.
    pub async fn update_config(&self, new_config: Config) -> Result<()> {
        info!("Updating configuration");

        let mut config = self.config.write().await;
        *config = new_config;

        let event = EventMessage::from_event(Event::System(SystemEvent::ConfigReloaded))
            .with_priority(EventPriority::High)
            .with_source("ide");

        self.event_bus.publish(event).await?;

        Ok(())
    }

    /// Get event statistics.
    pub async fn get_event_stats(&self) -> crate::core::events::EventStats {
        self.event_bus.get_stats().await
    }

    /// Register event handlers for IDE coordination.
    async fn register_event_handlers(&self) -> Result<()> {
        debug!("Registering IDE event handlers");

        // Register shutdown handler
        let shutdown_handler = ShutdownHandler { shutdown_flag: self.shutdown_requested.clone() };
        self.event_bus.register_handler("ide_shutdown", Arc::new(shutdown_handler)).await?;

        // Register project change handler
        let project_handler = ProjectChangeHandler {
            syntax_manager: self.syntax_manager.clone(),
            build_manager: self.build_manager.clone(),
        };
        self.event_bus.register_handler("ide_project_change", Arc::new(project_handler)).await?;

        Ok(())
    }

    /// Shutdown the IDE gracefully.
    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down IDE components");

        // Save all open files
        if let Err(e) = self.save_all().await {
            warn!("Failed to save all files during shutdown: {}", e);
        }

        // Shutdown components in reverse order
        if let Err(e) = self.build_manager.shutdown().await {
            warn!("Failed to shutdown build manager: {}", e);
        }

        if let Err(e) = self.syntax_manager.shutdown().await {
            warn!("Failed to shutdown syntax manager: {}", e);
        }

        if let Err(e) = self.project_manager.shutdown().await {
            warn!("Failed to shutdown project manager: {}", e);
        }

        if let Err(e) = self.editor.shutdown().await {
            warn!("Failed to shutdown editor: {}", e);
        }

        if let Err(e) = self.ui_manager.write().await.shutdown().await {
            warn!("Failed to shutdown UI manager: {}", e);
        }

        info!("IDE shutdown complete");
        Ok(())
    }

    /// Performance monitoring background task.
    async fn performance_monitor_task(shutdown_flag: Arc<RwLock<bool>>, event_bus: Arc<EventBus>) {
        let mut interval = interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            // Check if shutdown requested
            {
                let shutdown = shutdown_flag.read().await;
                if *shutdown {
                    break;
                }
            }

            // Check memory usage
            if let Ok(memory_info) = Self::get_memory_info() {
                if memory_info.used_mb > 512 {
                    // Warn if using more than 512MB
                    let event =
                        EventMessage::from_event(Event::System(SystemEvent::MemoryWarning {
                            used_mb: memory_info.used_mb,
                            available_mb: memory_info.available_mb,
                        }))
                        .with_priority(EventPriority::Normal)
                        .with_source("performance_monitor");

                    if let Err(e) = event_bus.publish(event).await {
                        error!("Failed to publish memory warning event: {}", e);
                    }
                }
            }
        }
    }

    /// Auto-save background task.
    async fn auto_save_task(
        shutdown_flag: Arc<RwLock<bool>>,
        config: Arc<RwLock<Config>>,
        editor: Arc<Editor>,
    ) {
        loop {
            // Get auto-save interval from config
            let auto_save_interval = {
                let config = config.read().await;
                config.editor.auto_save_interval
            };

            if auto_save_interval == 0 {
                // Auto-save disabled, wait 5 seconds and check again
                tokio::time::sleep(Duration::from_secs(5)).await;
            } else {
                let mut interval = interval(Duration::from_secs(auto_save_interval));
                interval.tick().await; // Skip first tick

                interval.tick().await;
            }

            // Check if shutdown requested
            {
                let shutdown = shutdown_flag.read().await;
                if *shutdown {
                    break;
                }
            }

            // Perform auto-save if enabled
            let auto_save_interval = {
                let config = config.read().await;
                config.editor.auto_save_interval
            };

            if auto_save_interval > 0 {
                if let Err(e) = editor.auto_save().await {
                    error!("Auto-save failed: {}", e);
                }
            }
        }
    }

    /// Get memory information.
    fn get_memory_info() -> Result<MemoryInfo> {
        // This is a placeholder implementation
        // In a real implementation, you would use platform-specific APIs
        Ok(MemoryInfo { used_mb: 64, available_mb: 1024 })
    }
}

/// Memory information structure.
#[derive(Debug)]
struct MemoryInfo {
    used_mb: u64,
    available_mb: u64,
}

/// Event handler for shutdown events.
struct ShutdownHandler {
    shutdown_flag: Arc<RwLock<bool>>,
}

#[async_trait::async_trait]
impl EventHandler for ShutdownHandler {
    async fn handle(&self, event: &EventMessage) -> Result<()> {
        if event.event_type == "system.shutdown_requested" {
            debug!("Processing shutdown request");
            let mut shutdown = self.shutdown_flag.write().await;
            *shutdown = true;
        }
        Ok(())
    }

    fn can_handle(&self, event_type: &str) -> bool {
        event_type == "system.shutdown_requested"
    }

    fn priority(&self) -> EventPriority {
        EventPriority::Critical
    }
}

/// Event handler for project changes.
struct ProjectChangeHandler {
    syntax_manager: Arc<SyntaxManager>,
    build_manager: Arc<BuildManager>,
}

#[async_trait::async_trait]
impl EventHandler for ProjectChangeHandler {
    async fn handle(&self, event: &EventMessage) -> Result<()> {
        if event.event_type.starts_with("project.") {
            match event.event_type.as_str() {
                "project.opened" => {
                    debug!("Project opened, updating syntax and build configuration");

                    // Try to extract path from data
                    if let Ok(path_str) = serde_json::from_value::<String>(event.data.clone()) {
                        let path = PathBuf::from(path_str);

                        // Update syntax manager for new project
                        if let Err(e) = self.syntax_manager.set_project_root(&path).await {
                            error!("Failed to update syntax manager for project: {}", e);
                        }

                        // Update build manager for new project
                        if let Err(e) = self.build_manager.set_project_root(&path).await {
                            error!("Failed to update build manager for project: {}", e);
                        }
                    }
                }
                "project.closed" => {
                    debug!("Project closed, clearing configurations");

                    // Clear project from syntax manager
                    if let Err(e) = self.syntax_manager.clear_project().await {
                        error!("Failed to clear project from syntax manager: {}", e);
                    }

                    // Clear project from build manager
                    if let Err(e) = self.build_manager.clear_project().await {
                        error!("Failed to clear project from build manager: {}", e);
                    }
                }
                _ => {
                    // Handle other project events
                }
            }
        }
        Ok(())
    }

    fn can_handle(&self, event_type: &str) -> bool {
        event_type.starts_with("project.")
    }

    fn priority(&self) -> EventPriority {
        EventPriority::High
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_ide_creation() {
        let config = Config::default();
        let ide = XyluxIde::new(config).await;
        assert!(ide.is_ok());
    }

    #[tokio::test]
    async fn test_config_operations() {
        let config = Config::default();
        let ide = XyluxIde::new(config.clone()).await.unwrap();

        // Test getting config
        let retrieved_config = ide.get_config().await;
        assert_eq!(retrieved_config.editor.tab_size, config.editor.tab_size);

        // Test updating config
        let mut new_config = config;
        new_config.editor.tab_size = 8;
        ide.update_config(new_config.clone()).await.unwrap();

        let updated_config = ide.get_config().await;
        assert_eq!(updated_config.editor.tab_size, 8);
    }

    #[tokio::test]
    async fn test_shutdown_request() {
        let config = Config::default();
        let ide = XyluxIde::new(config).await.unwrap();

        ide.request_shutdown().await.unwrap();

        let shutdown = ide.shutdown_requested.read().await;
        assert!(*shutdown);
    }

    #[tokio::test]
    async fn test_file_operations() {
        let config = Config::default();
        let ide = XyluxIde::new(config).await.unwrap();

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        // Test opening file
        ide.open(&test_file).await.unwrap();

        // Test save operations
        ide.save().await.unwrap();
        ide.save_all().await.unwrap();
    }
}
