//! # UI Components Module
//!
//! Basic UI components for Xylux IDE.

#![allow(clippy::large_enum_variant)]

pub mod editor;
pub mod file_explorer;

pub use editor::{EditorComponent, EditorTheme};
pub use file_explorer::{FileEntry, FileExplorerComponent, FileExplorerTheme, SortBy};

use crate::core::Result;

/// Trait for UI components.
pub trait Component: Send + Sync {
    /// Render the component.
    fn render(&self, area: Rect) -> Result<()>;

    /// Handle input events.
    fn handle_event(&mut self, event: &Event) -> Result<bool>;

    /// Handle key events specifically.
    fn handle_key_event(&mut self, event: &KeyEvent) -> Result<bool> {
        // Default implementation delegates to handle_event
        self.handle_event(&Event::Key(event.clone()))
    }

    /// Update the component state.
    fn update(&mut self) -> Result<()>;

    /// Get the minimum size required by this component.
    fn min_size(&self) -> Size {
        Size { width: 0, height: 0 }
    }

    /// Check if the component is visible.
    fn is_visible(&self) -> bool {
        true
    }

    /// Set visibility of the component.
    fn set_visible(&mut self, visible: bool);
}

/// Represents a rectangular area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }

    pub fn area(&self) -> u32 {
        u32::from(self.width) * u32::from(self.height)
    }

    pub fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }
}

/// Represents a size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

/// Basic event types for UI components.
#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Focus(bool),
    Custom(String),
}

/// Key event.
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

/// Key codes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(variant_size_differences)]
pub enum KeyCode {
    Char(char),
    Enter,
    Tab,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Escape,
    F(u8),
}

/// Key modifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

/// Mouse event.
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub event_type: MouseEventType,
    pub x: u16,
    pub y: u16,
    pub modifiers: KeyModifiers,
}

/// Mouse event types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseEventType {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    Move,
    ScrollUp,
    ScrollDown,
}

/// Mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Basic text display component.
pub struct TextComponent {
    text: String,
    visible: bool,
    color: Option<String>,
    background: Option<String>,
}

impl TextComponent {
    pub fn new(text: String) -> Self {
        Self { text, visible: true, color: None, background: None }
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_background(mut self, background: String) -> Self {
        self.background = Some(background);
        self
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Component for TextComponent {
    fn render(&self, _area: Rect) -> Result<()> {
        // Basic text rendering would go here
        // For now, this is a placeholder
        Ok(())
    }

    fn handle_event(&mut self, _event: &Event) -> Result<bool> {
        // Text components don't handle events by default
        Ok(false)
    }

    fn update(&mut self) -> Result<()> {
        // Nothing to update for basic text
        Ok(())
    }

    fn min_size(&self) -> Size {
        Size::new(self.text.len() as u16, 1)
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

/// Container component that can hold other components.
pub struct Container {
    children: Vec<Box<dyn Component + Send + Sync>>,
    visible: bool,
    layout: LayoutType,
}

/// Layout types for containers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutType {
    Vertical,
    Horizontal,
    Grid { cols: u16, rows: u16 },
    Absolute,
}

impl Container {
    pub fn new(layout: LayoutType) -> Self {
        Self { children: Vec::new(), visible: true, layout }
    }

    pub fn add_child(&mut self, child: Box<dyn Component + Send + Sync>) {
        self.children.push(child);
    }

    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}

impl Component for Container {
    fn render(&self, area: Rect) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        match self.layout {
            LayoutType::Vertical => {
                let child_height = area.height / self.children.len() as u16;
                for (i, child) in self.children.iter().enumerate() {
                    let child_area = Rect::new(
                        area.x,
                        area.y + (i as u16 * child_height),
                        area.width,
                        child_height,
                    );
                    child.render(child_area)?;
                }
            }
            LayoutType::Horizontal => {
                let child_width = area.width / self.children.len() as u16;
                for (i, child) in self.children.iter().enumerate() {
                    let child_area = Rect::new(
                        area.x + (i as u16 * child_width),
                        area.y,
                        child_width,
                        area.height,
                    );
                    child.render(child_area)?;
                }
            }
            LayoutType::Grid { cols, rows: _ } => {
                let child_width = area.width / cols;
                let child_height = area.height / (self.children.len() as u16 / cols + 1);
                for (i, child) in self.children.iter().enumerate() {
                    let col = i as u16 % cols;
                    let row = i as u16 / cols;
                    let child_area = Rect::new(
                        area.x + (col * child_width),
                        area.y + (row * child_height),
                        child_width,
                        child_height,
                    );
                    child.render(child_area)?;
                }
            }
            LayoutType::Absolute => {
                // Each child renders at full area (they manage their own positioning)
                for child in &self.children {
                    child.render(area)?;
                }
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> Result<bool> {
        if !self.visible {
            return Ok(false);
        }

        // Forward event to children (in reverse order for proper z-ordering)
        for child in self.children.iter_mut().rev() {
            if child.handle_event(event)? {
                return Ok(true); // Event was handled
            }
        }

        Ok(false)
    }

    fn update(&mut self) -> Result<()> {
        for child in &mut self.children {
            child.update()?;
        }
        Ok(())
    }

    fn min_size(&self) -> Size {
        match self.layout {
            LayoutType::Vertical => {
                let mut total_height = 0;
                let mut max_width = 0;
                for child in &self.children {
                    let child_size = child.min_size();
                    total_height += child_size.height;
                    max_width = max_width.max(child_size.width);
                }
                Size::new(max_width, total_height)
            }
            LayoutType::Horizontal => {
                let mut total_width = 0;
                let mut max_height = 0;
                for child in &self.children {
                    let child_size = child.min_size();
                    total_width += child_size.width;
                    max_height = max_height.max(child_size.height);
                }
                Size::new(total_width, max_height)
            }
            LayoutType::Grid { cols, rows: _ } => {
                let mut max_child_width = 0;
                let mut max_child_height = 0;
                for child in &self.children {
                    let child_size = child.min_size();
                    max_child_width = max_child_width.max(child_size.width);
                    max_child_height = max_child_height.max(child_size.height);
                }
                let rows = (self.children.len() as u16 + cols - 1) / cols;
                Size::new(max_child_width * cols, max_child_height * rows)
            }
            LayoutType::Absolute => {
                let mut max_width = 0;
                let mut max_height = 0;
                for child in &self.children {
                    let child_size = child.min_size();
                    max_width = max_width.max(child_size.width);
                    max_height = max_height.max(child_size.height);
                }
                Size::new(max_width, max_height)
            }
        }
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_creation() {
        let rect = Rect::new(10, 20, 30, 40);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 30);
        assert_eq!(rect.height, 40);
        assert_eq!(rect.right(), 40);
        assert_eq!(rect.bottom(), 60);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 10, 20, 20);
        assert!(rect.contains(15, 15));
        assert!(!rect.contains(5, 5));
        assert!(!rect.contains(35, 35));
    }

    #[test]
    fn test_rect_intersects() {
        let rect1 = Rect::new(10, 10, 20, 20);
        let rect2 = Rect::new(15, 15, 20, 20);
        let rect3 = Rect::new(50, 50, 10, 10);

        assert!(rect1.intersects(&rect2));
        assert!(!rect1.intersects(&rect3));
    }

    #[test]
    fn test_text_component() {
        let mut text = TextComponent::new("Hello, World!".to_string());
        assert_eq!(text.text(), "Hello, World!");
        assert!(text.is_visible());

        text.set_text("Updated text".to_string());
        assert_eq!(text.text(), "Updated text");

        text.set_visible(false);
        assert!(!text.is_visible());
    }

    #[test]
    fn test_container() {
        let mut container = Container::new(LayoutType::Vertical);
        assert_eq!(container.child_count(), 0);

        let text1 = Box::new(TextComponent::new("Child 1".to_string()));
        let text2 = Box::new(TextComponent::new("Child 2".to_string()));

        container.add_child(text1);
        container.add_child(text2);
        assert_eq!(container.child_count(), 2);

        container.clear_children();
        assert_eq!(container.child_count(), 0);
    }

    #[test]
    fn test_size() {
        let size = Size::new(100, 50);
        assert_eq!(size.width, 100);
        assert_eq!(size.height, 50);
    }

    #[test]
    fn test_key_event() {
        let event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers { ctrl: true, alt: false, shift: false },
        };

        assert_eq!(event.code, KeyCode::Char('a'));
        assert!(event.modifiers.ctrl);
        assert!(!event.modifiers.alt);
    }
}
