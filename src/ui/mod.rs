//! # UI Module
//!
//! User interface management for Xylux IDE.

pub mod components;
pub mod layout;
pub mod terminal;
pub mod theme;

pub use components::Component;
pub use layout::Layout;
pub use terminal::TerminalInterface;
pub use theme::Theme;

use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::core::{Config, EventBus, Result};

/// Main UI manager that coordinates the user interface.
pub struct UiManager {
    /// IDE configuration.
    config: Arc<RwLock<Config>>,
    /// Event bus for communication.
    event_bus: Arc<EventBus>,
    /// Terminal interface.
    terminal: TerminalInterface,
    /// Current theme.
    theme: Theme,
    /// Layout manager.
    layout: Layout,
    /// Whether the UI is running.
    running: Arc<RwLock<bool>>,
    /// Shutdown flag from IDE.
    shutdown_flag: Arc<RwLock<bool>>,
}

impl UiManager {
    /// Create a new UI manager.
    pub async fn new(
        config: Arc<RwLock<Config>>,
        event_bus: Arc<EventBus>,
        shutdown_flag: Arc<RwLock<bool>>,
    ) -> Result<Self> {
        debug!("Initializing UI manager");

        let theme = {
            let config = config.read().await;
            Theme::load(&config.ui.theme)?
        };

        let terminal = TerminalInterface::new()?;
        let layout = Layout::new();

        let mut ui_manager = Self {
            config,
            event_bus,
            terminal,
            theme,
            layout,
            running: Arc::new(RwLock::new(false)),
            shutdown_flag,
        };

        // Initialize basic IDE layout
        ui_manager.setup_basic_ide_layout().await?;

        Ok(ui_manager)
    }

    /// Setup a basic IDE layout with main components.
    async fn setup_basic_ide_layout(&mut self) -> Result<()> {
        use crate::ui::components::{Container, LayoutType, TextComponent};

        // Create main container with vertical layout
        let mut main_container = Container::new(LayoutType::Vertical);

        // Add header
        let header = TextComponent::new("Xylux IDE v0.1.0".to_string())
            .with_background("#2d3748".to_string())
            .with_color("#e2e8f0".to_string());
        main_container.add_child(Box::new(header));

        // Add main content area (horizontal split)
        let mut content_container = Container::new(LayoutType::Horizontal);

        // File explorer (left panel - 1/4 width)
        let file_explorer = TextComponent::new("File Explorer\n\n(Coming soon...)".to_string())
            .with_background("#1a202c".to_string())
            .with_color("#cbd5e0".to_string());
        content_container.add_child(Box::new(file_explorer));

        // Editor area (right panel - 3/4 width)
        let editor_text = TextComponent::new(
            "Welcome to Xylux IDE\n\nPress Ctrl+O to open a file\nPress Ctrl+N for new file\nPress Ctrl+Q to quit".to_string()
        ).with_background("#2d3748".to_string())
         .with_color("#e2e8f0".to_string());
        content_container.add_child(Box::new(editor_text));

        main_container.add_child(Box::new(content_container));

        // Add status bar
        let status_bar = TextComponent::new("Ready | Ctrl+Q: Quit".to_string())
            .with_background("#4a5568".to_string())
            .with_color("#e2e8f0".to_string());
        main_container.add_child(Box::new(status_bar));

        // Set as root component
        self.layout.set_root(Box::new(main_container));

        Ok(())
    }

    /// Run the main UI loop.
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting UI main loop");

        {
            let mut running = self.running.write().await;
            *running = true;
        }

        self.terminal.enter_alternate_screen()?;
        self.terminal.enable_raw_mode()?;

        let result = self.main_loop().await;

        // Cleanup
        if let Err(e) = self.terminal.leave_alternate_screen() {
            error!("Failed to leave alternate screen: {}", e);
        }
        if let Err(e) = self.terminal.disable_raw_mode() {
            error!("Failed to disable raw mode: {}", e);
        }

        {
            let mut running = self.running.write().await;
            *running = false;
        }

        result
    }

    /// Main UI event loop.
    async fn main_loop(&mut self) -> Result<()> {
        loop {
            // Check if we should stop running
            {
                let running = self.running.read().await;
                if !*running {
                    break;
                }
            }

            // Check if shutdown was requested
            {
                let shutdown = self.shutdown_flag.read().await;
                if *shutdown {
                    let mut running = self.running.write().await;
                    *running = false;
                    break;
                }
            }

            // Handle terminal events
            if let Some(event) = self.terminal.poll_event()? {
                self.handle_terminal_event(event).await?;
            }

            // Update components
            if let Some(root) = self.layout.root_mut() {
                root.update()?;
            }

            // Render the UI
            self.render().await?;

            // Small delay to prevent busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(16)).await; // ~60 FPS
        }

        Ok(())
    }

    /// Handle terminal input events.
    async fn handle_terminal_event(&mut self, event: crossterm::event::Event) -> Result<()> {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};

        match event {
            Event::Key(KeyEvent { code, modifiers, .. }) => {
                match (code, modifiers) {
                    (KeyCode::Char('c'), KeyModifiers::CONTROL)
                    | (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                        // Ctrl+C or Ctrl+Q - request shutdown
                        info!("Shutdown requested via keyboard shortcut");
                        let shutdown_event = crate::core::EventMessage::from_event(
                            crate::core::Event::System(crate::core::SystemEvent::ShutdownRequested),
                        )
                        .with_priority(crate::core::EventPriority::Critical)
                        .with_source("ui");

                        self.event_bus.publish(shutdown_event).await?;
                    }
                    (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                        // Ctrl+O - Open file (placeholder)
                        info!("Open file requested");
                        // TODO: Implement file open dialog
                    }
                    (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                        // Ctrl+N - New file (placeholder)
                        info!("New file requested");
                        // TODO: Implement new file creation
                    }
                    _ => {
                        // Handle other key events
                        let ui_event = crate::core::EventMessage::from_event(
                            crate::core::Event::Ui(crate::core::UiEvent::KeyPressed {
                                key: format!("{:?}", code),
                                modifiers: vec![format!("{:?}", modifiers)],
                            }),
                        )
                        .with_priority(crate::core::EventPriority::High)
                        .with_source("ui");

                        self.event_bus.publish(ui_event).await?;
                    }
                }
            }
            Event::Mouse(MouseEvent { .. }) => {
                // Handle mouse events
                // TODO: Implement mouse handling
            }
            Event::Resize(width, height) => {
                // Handle terminal resize
                self.layout.update_terminal_size(width, height);
                self.layout.mark_dirty();

                let resize_event = crate::core::EventMessage::from_event(crate::core::Event::Ui(
                    crate::core::UiEvent::WindowResized { width, height },
                ))
                .with_priority(crate::core::EventPriority::Normal)
                .with_source("ui");

                self.event_bus.publish(resize_event).await?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Render the UI.
    async fn render(&mut self) -> Result<()> {
        self.terminal.clear()?;

        // Update layout with current terminal size
        let (width, height) = self.terminal.size()?;
        self.layout.update_terminal_size(width, height);

        // If layout needs recalculation, do it now
        if self.layout.is_dirty() {
            self.layout.calculate()?;
        }

        // Render layout components
        if let Some(root) = self.layout.root() {
            let area = components::Rect { x: 0, y: 0, width, height };
            root.render(area)?;
        } else {
            // Render a basic IDE layout
            self.render_basic_ide_layout(width, height)?;
        }

        self.terminal.flush()?;
        Ok(())
    }

    /// Render a basic IDE layout when no root component is set.
    fn render_basic_ide_layout(&mut self, width: u16, height: u16) -> Result<()> {
        // Header bar
        let header_text = "Xylux IDE v0.1.0";
        self.terminal.print_at(
            0,
            0,
            &format!("{:width$}", header_text, width = width as usize),
            &self.theme.foreground,
            &self.theme.background,
        )?;

        // Status line
        let status_text = "Ready | Ctrl+Q: Quit | Ctrl+O: Open File | Ctrl+N: New File";
        let status_y = height.saturating_sub(1);
        self.terminal.print_at(
            0,
            status_y,
            &format!("{:width$}", status_text, width = width as usize),
            &self.theme.foreground,
            &self.theme.background,
        )?;

        // Main editor area
        let editor_start_y = 1;
        let editor_height = height.saturating_sub(2);

        // Show welcome message in editor area
        if editor_height > 0 {
            let welcome_lines = vec![
                "Welcome to Xylux IDE",
                "",
                "Getting Started:",
                "• Press Ctrl+O to open a file",
                "• Press Ctrl+N to create a new file",
                "• Press Ctrl+Q to quit",
                "",
                "Features:",
                "• Rust development with rust-analyzer",
                "• Alux scripting support",
                "• Xylux engine integration",
                "• Project management",
            ];

            let start_line =
                editor_start_y + (editor_height / 2).saturating_sub(welcome_lines.len() as u16 / 2);

            for (i, line) in welcome_lines.iter().enumerate() {
                let y = start_line + i as u16;
                if y < height.saturating_sub(1) {
                    let x = (width.saturating_sub(line.len() as u16)) / 2;
                    self.terminal.print_at(
                        x,
                        y,
                        line,
                        &self.theme.foreground,
                        &self.theme.background,
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Add a component to the layout.
    pub fn add_component(&mut self, component: Box<dyn Component + Send + Sync>) {
        self.layout.set_root(component);
    }

    /// Get a mutable reference to the layout.
    pub fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }

    /// Get a reference to the layout.
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Force layout recalculation.
    pub fn invalidate_layout(&mut self) {
        self.layout.mark_dirty();
    }

    /// Request shutdown of the UI.
    pub async fn request_shutdown(&self) -> Result<()> {
        debug!("UI shutdown requested");
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    /// Shutdown the UI manager.
    pub async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down UI manager");
        self.request_shutdown().await?;
        Ok(())
    }

    /// Get the current theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Update the theme.
    pub async fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        self.theme = Theme::load(theme_name)?;

        // Update config
        {
            let mut config = self.config.write().await;
            config.ui.theme = theme_name.to_string();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ui_manager_creation() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());

        let shutdown_flag = Arc::new(RwLock::new(false));
        let ui_manager = UiManager::new(config, event_bus, shutdown_flag).await;
        assert!(ui_manager.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_request() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let shutdown_flag = Arc::new(RwLock::new(false));
        let ui_manager = UiManager::new(config, event_bus, shutdown_flag).await.unwrap();

        // Test that shutdown flag is initially false
        let shutdown = ui_manager.shutdown_flag.read().await;
        assert!(!*shutdown);

        let running = ui_manager.running.read().await;
        assert!(!*running);
    }
}
