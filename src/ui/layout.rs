//! # Layout Module
//!
//! Layout management for UI components in Xylux IDE.

use crate::core::Result;
use crate::ui::components::{Component, Rect, Size};

/// Layout manager for arranging UI components.
pub struct Layout {
    /// Root container.
    root: Option<Box<dyn Component + Send + Sync>>,
    /// Current terminal size.
    terminal_size: Size,
    /// Whether the layout needs to be recalculated.
    dirty: bool,
}

impl Layout {
    /// Create a new layout manager.
    pub fn new() -> Self {
        Self {
            root: None,
            terminal_size: Size::new(80, 24), // Default terminal size
            dirty: true,
        }
    }

    /// Set the root component.
    pub fn set_root(&mut self, root: Box<dyn Component + Send + Sync>) {
        self.root = Some(root);
        self.dirty = true;
    }

    /// Update terminal size.
    pub fn update_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = Size::new(width, height);
        self.dirty = true;
    }

    /// Get the current terminal size.
    pub fn terminal_size(&self) -> Size {
        self.terminal_size
    }

    /// Calculate layout if needed.
    pub fn calculate(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        if let Some(ref root) = self.root {
            let area = Rect::new(0, 0, self.terminal_size.width, self.terminal_size.height);
            root.render(area)?;
        }

        self.dirty = false;
        Ok(())
    }

    /// Mark layout as dirty (needs recalculation).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if layout is dirty.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get the root component.
    pub fn root(&self) -> Option<&Box<dyn Component + Send + Sync>> {
        self.root.as_ref()
    }

    /// Get the root component mutably.
    pub fn root_mut(&mut self) -> Option<&mut Box<dyn Component + Send + Sync>> {
        self.root.as_mut()
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout constraints for components.
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Fixed size in characters.
    Length(u16),
    /// Percentage of available space.
    Percentage(u16),
    /// Minimum size.
    Min(u16),
    /// Maximum size.
    Max(u16),
    /// Take remaining space.
    Fill,
}

/// Direction for layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

/// Alignment for layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Start,
    Center,
    End,
}

/// Layout helper functions.
pub mod helpers {
    use super::*;

    /// Split a rectangle into multiple areas based on constraints.
    pub fn split_rect(area: Rect, direction: Direction, constraints: &[Constraint]) -> Vec<Rect> {
        if constraints.is_empty() {
            return vec![area];
        }

        let mut rects = Vec::new();
        let total_space = match direction {
            Direction::Horizontal => area.width,
            Direction::Vertical => area.height,
        };

        // Calculate fixed sizes first
        let mut fixed_total = 0;
        let mut percentage_total = 0;
        let mut fill_count = 0;

        for constraint in constraints {
            match constraint {
                Constraint::Length(len) => fixed_total += len,
                Constraint::Percentage(pct) => percentage_total += pct,
                Constraint::Min(_) | Constraint::Max(_) => {}
                Constraint::Fill => fill_count += 1,
            }
        }

        // Calculate percentage space
        let percentage_space = (total_space as f32 * percentage_total as f32 / 100.0) as u16;
        let remaining_space = total_space.saturating_sub(fixed_total + percentage_space);
        let fill_space = if fill_count > 0 { remaining_space / fill_count } else { 0 };

        let mut current_pos = match direction {
            Direction::Horizontal => area.x,
            Direction::Vertical => area.y,
        };

        for constraint in constraints {
            let size = match constraint {
                Constraint::Length(len) => *len,
                Constraint::Percentage(pct) => (total_space as f32 * *pct as f32 / 100.0) as u16,
                Constraint::Min(min) => (*min).max(fill_space),
                Constraint::Max(max) => (*max).min(fill_space),
                Constraint::Fill => fill_space,
            };

            let rect = match direction {
                Direction::Horizontal => Rect::new(current_pos, area.y, size, area.height),
                Direction::Vertical => Rect::new(area.x, current_pos, area.width, size),
            };

            rects.push(rect);
            current_pos += size;
        }

        rects
    }

    /// Center a rectangle within another rectangle.
    pub fn center_rect(area: Rect, size: Size) -> Rect {
        let x = area.x + (area.width.saturating_sub(size.width)) / 2;
        let y = area.y + (area.height.saturating_sub(size.height)) / 2;
        Rect::new(x, y, size.width, size.height)
    }

    /// Align a rectangle within another rectangle.
    pub fn align_rect(area: Rect, size: Size, horizontal: Alignment, vertical: Alignment) -> Rect {
        let x = match horizontal {
            Alignment::Start => area.x,
            Alignment::Center => area.x + (area.width.saturating_sub(size.width)) / 2,
            Alignment::End => area.x + area.width.saturating_sub(size.width),
        };

        let y = match vertical {
            Alignment::Start => area.y,
            Alignment::Center => area.y + (area.height.saturating_sub(size.height)) / 2,
            Alignment::End => area.y + area.height.saturating_sub(size.height),
        };

        Rect::new(x, y, size.width, size.height)
    }

    /// Calculate minimum size for a set of constraints.
    pub fn min_size_for_constraints(constraints: &[Constraint], _direction: Direction) -> u16 {
        let mut total = 0;

        for constraint in constraints {
            match constraint {
                Constraint::Length(len) => total += len,
                Constraint::Min(min) => total += min,
                Constraint::Percentage(_) => total += 1, // Assume minimum 1 for percentages
                Constraint::Max(_) => total += 1,        // Assume minimum 1 for max
                Constraint::Fill => total += 1,          // Assume minimum 1 for fill
            }
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::*;
    use super::*;

    #[test]
    fn test_layout_creation() {
        let layout = Layout::new();
        assert_eq!(layout.terminal_size().width, 80);
        assert_eq!(layout.terminal_size().height, 24);
        assert!(layout.is_dirty());
    }

    #[test]
    fn test_terminal_size_update() {
        let mut layout = Layout::new();
        layout.update_terminal_size(120, 40);

        assert_eq!(layout.terminal_size().width, 120);
        assert_eq!(layout.terminal_size().height, 40);
        assert!(layout.is_dirty());
    }

    #[test]
    fn test_split_rect_horizontal() {
        let area = Rect::new(0, 0, 100, 50);
        let constraints =
            vec![Constraint::Length(20), Constraint::Percentage(50), Constraint::Fill];

        let rects = split_rect(area, Direction::Horizontal, &constraints);
        assert_eq!(rects.len(), 3);
        assert_eq!(rects[0].width, 20);
        assert_eq!(rects[1].width, 50); // 50% of 100
        // Fill takes remaining space: 100 - 20 - 50 = 30
        assert_eq!(rects[2].width, 30);
    }

    #[test]
    fn test_split_rect_vertical() {
        let area = Rect::new(0, 0, 100, 50);
        let constraints = vec![Constraint::Length(10), Constraint::Fill];

        let rects = split_rect(area, Direction::Vertical, &constraints);
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].height, 10);
        assert_eq!(rects[1].height, 40); // 50 - 10
    }

    #[test]
    fn test_center_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let size = Size::new(20, 10);
        let centered = center_rect(area, size);

        assert_eq!(centered.x, 40); // (100 - 20) / 2
        assert_eq!(centered.y, 20); // (50 - 10) / 2
        assert_eq!(centered.width, 20);
        assert_eq!(centered.height, 10);
    }

    #[test]
    fn test_align_rect() {
        let area = Rect::new(10, 10, 100, 50);
        let size = Size::new(20, 10);

        // Test different alignments
        let top_left = align_rect(area, size, Alignment::Start, Alignment::Start);
        assert_eq!(top_left.x, 10);
        assert_eq!(top_left.y, 10);

        let bottom_right = align_rect(area, size, Alignment::End, Alignment::End);
        assert_eq!(bottom_right.x, 90); // 10 + 100 - 20
        assert_eq!(bottom_right.y, 50); // 10 + 50 - 10

        let center = align_rect(area, size, Alignment::Center, Alignment::Center);
        assert_eq!(center.x, 50); // 10 + (100 - 20) / 2
        assert_eq!(center.y, 30); // 10 + (50 - 10) / 2
    }

    #[test]
    fn test_min_size_calculation() {
        let constraints =
            vec![Constraint::Length(10), Constraint::Min(5), Constraint::Percentage(50)];

        let min_size = min_size_for_constraints(&constraints, Direction::Horizontal);
        assert_eq!(min_size, 16); // 10 + 5 + 1
    }

    #[test]
    fn test_empty_constraints() {
        let area = Rect::new(0, 0, 100, 50);
        let rects = split_rect(area, Direction::Horizontal, &[]);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0], area);
    }
}
