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
}

impl UiManager {
    /// Create a new UI manager.
    pub async fn new(config: Arc<RwLock<Config>>, event_bus: Arc<EventBus>) -> Result<Self> {
        debug!("Initializing UI manager");

        let theme = {
            let config = config.read().await;
            Theme::load(&config.ui.theme)?
        };

        let terminal = TerminalInterface::new()?;
        let layout = Layout::new();

        Ok(Self {
            config,
            event_bus,
            terminal,
            theme,
            layout,
            running: Arc::new(RwLock::new(false)),
        })
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

            // Handle terminal events
            if let Some(event) = self.terminal.poll_event()? {
                self.handle_terminal_event(event).await?;
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
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        // Ctrl+C - request shutdown
                        let shutdown_event = crate::core::EventMessage::from_event(
                            crate::core::Event::System(crate::core::SystemEvent::ShutdownRequested),
                        )
                        .with_priority(crate::core::EventPriority::Critical)
                        .with_source("ui");

                        self.event_bus.publish(shutdown_event).await?;
                    }
                    (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                        // Ctrl+Q - request shutdown
                        let shutdown_event = crate::core::EventMessage::from_event(
                            crate::core::Event::System(crate::core::SystemEvent::ShutdownRequested),
                        )
                        .with_priority(crate::core::EventPriority::Critical)
                        .with_source("ui");

                        self.event_bus.publish(shutdown_event).await?;
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
            // Fallback: render welcome screen
            let welcome_text = "Xylux IDE - Press Ctrl+C or Ctrl+Q to exit";
            let x = (width.saturating_sub(welcome_text.len() as u16)) / 2;
            let y = height / 2;

            self.terminal.print_at(
                x,
                y,
                welcome_text,
                &self.theme.foreground,
                &self.theme.background,
            )?;
        }

        self.terminal.flush()?;
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

        let ui_manager = UiManager::new(config, event_bus).await;
        assert!(ui_manager.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_request() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let ui_manager = UiManager::new(config, event_bus).await.unwrap();

        ui_manager.request_shutdown().await.unwrap();

        let running = ui_manager.running.read().await;
        assert!(!*running);
    }
}
