//! # Terminal Module
//!
//! Terminal interface management for Xylux IDE.

use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::core::{Result, XyluxError};

/// Terminal interface wrapper for crossterm.
pub struct TerminalInterface {
    /// Whether raw mode is enabled.
    raw_mode_enabled: bool,
    /// Whether alternate screen is active.
    alternate_screen_active: bool,
}

impl TerminalInterface {
    /// Create a new terminal interface.
    pub fn new() -> Result<Self> {
        Ok(Self { raw_mode_enabled: false, alternate_screen_active: false })
    }

    /// Enable raw mode for terminal input.
    pub fn enable_raw_mode(&mut self) -> Result<()> {
        if !self.raw_mode_enabled {
            terminal::enable_raw_mode()
                .map_err(|e| XyluxError::terminal(format!("Failed to enable raw mode: {}", e)))?;
            self.raw_mode_enabled = true;
        }
        Ok(())
    }

    /// Disable raw mode.
    pub fn disable_raw_mode(&mut self) -> Result<()> {
        if self.raw_mode_enabled {
            terminal::disable_raw_mode()
                .map_err(|e| XyluxError::terminal(format!("Failed to disable raw mode: {}", e)))?;
            self.raw_mode_enabled = false;
        }
        Ok(())
    }

    /// Enter alternate screen mode.
    pub fn enter_alternate_screen(&mut self) -> Result<()> {
        if !self.alternate_screen_active {
            execute!(io::stdout(), terminal::EnterAlternateScreen).map_err(|e| {
                XyluxError::terminal(format!("Failed to enter alternate screen: {}", e))
            })?;
            self.alternate_screen_active = true;
        }
        Ok(())
    }

    /// Leave alternate screen mode.
    pub fn leave_alternate_screen(&mut self) -> Result<()> {
        if self.alternate_screen_active {
            execute!(io::stdout(), terminal::LeaveAlternateScreen).map_err(|e| {
                XyluxError::terminal(format!("Failed to leave alternate screen: {}", e))
            })?;
            self.alternate_screen_active = false;
        }
        Ok(())
    }

    /// Get terminal size.
    pub fn size(&self) -> Result<(u16, u16)> {
        terminal::size()
            .map_err(|e| XyluxError::terminal(format!("Failed to get terminal size: {}", e)))
    }

    /// Clear the terminal screen.
    pub fn clear(&self) -> Result<()> {
        queue!(io::stdout(), terminal::Clear(ClearType::All))
            .map_err(|e| XyluxError::terminal(format!("Failed to clear screen: {}", e)))?;
        Ok(())
    }

    /// Move cursor to position.
    pub fn move_cursor(&self, x: u16, y: u16) -> Result<()> {
        queue!(io::stdout(), cursor::MoveTo(x, y))
            .map_err(|e| XyluxError::terminal(format!("Failed to move cursor: {}", e)))?;
        Ok(())
    }

    /// Print text at specific position with colors.
    pub fn print_at(
        &self,
        x: u16,
        y: u16,
        text: &str,
        foreground: &str,
        background: &str,
    ) -> Result<()> {
        let fg_color = parse_color(foreground)?;
        let bg_color = parse_color(background)?;

        queue!(
            io::stdout(),
            cursor::MoveTo(x, y),
            SetForegroundColor(fg_color),
            SetBackgroundColor(bg_color),
            Print(text),
            ResetColor
        )
        .map_err(|e| XyluxError::terminal(format!("Failed to print text: {}", e)))?;

        Ok(())
    }

    /// Print text with default colors.
    pub fn print(&self, text: &str) -> Result<()> {
        queue!(io::stdout(), Print(text))
            .map_err(|e| XyluxError::terminal(format!("Failed to print text: {}", e)))?;
        Ok(())
    }

    /// Flush the output buffer.
    pub fn flush(&self) -> Result<()> {
        io::stdout()
            .flush()
            .map_err(|e| XyluxError::terminal(format!("Failed to flush output: {}", e)))?;
        Ok(())
    }

    /// Poll for terminal events (non-blocking).
    pub fn poll_event(&self) -> Result<Option<Event>> {
        if event::poll(std::time::Duration::from_millis(0))
            .map_err(|e| XyluxError::terminal(format!("Failed to poll events: {}", e)))?
        {
            let event = event::read()
                .map_err(|e| XyluxError::terminal(format!("Failed to read event: {}", e)))?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }

    /// Wait for and read a terminal event (blocking).
    pub fn read_event(&self) -> Result<Event> {
        event::read().map_err(|e| XyluxError::terminal(format!("Failed to read event: {}", e)))
    }

    /// Hide the cursor.
    pub fn hide_cursor(&self) -> Result<()> {
        queue!(io::stdout(), cursor::Hide)
            .map_err(|e| XyluxError::terminal(format!("Failed to hide cursor: {}", e)))?;
        Ok(())
    }

    /// Show the cursor.
    pub fn show_cursor(&self) -> Result<()> {
        queue!(io::stdout(), cursor::Show)
            .map_err(|e| XyluxError::terminal(format!("Failed to show cursor: {}", e)))?;
        Ok(())
    }

    /// Set cursor style.
    pub fn set_cursor_style(&self, style: CursorStyle) -> Result<()> {
        let crossterm_style = match style {
            CursorStyle::Block => cursor::SetCursorStyle::SteadyBlock,
            CursorStyle::Underline => cursor::SetCursorStyle::SteadyUnderScore,
            CursorStyle::Bar => cursor::SetCursorStyle::SteadyBar,
            CursorStyle::BlinkingBlock => cursor::SetCursorStyle::BlinkingBlock,
            CursorStyle::BlinkingUnderline => cursor::SetCursorStyle::BlinkingUnderScore,
            CursorStyle::BlinkingBar => cursor::SetCursorStyle::BlinkingBar,
        };

        queue!(io::stdout(), crossterm_style)
            .map_err(|e| XyluxError::terminal(format!("Failed to set cursor style: {}", e)))?;
        Ok(())
    }
}

impl Drop for TerminalInterface {
    fn drop(&mut self) {
        drop(self.leave_alternate_screen());
        drop(self.disable_raw_mode());
        drop(execute!(io::stdout(), ResetColor, cursor::Show));
    }
}

/// Cursor styles available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
    BlinkingBlock,
    BlinkingUnderline,
    BlinkingBar,
}

/// Parse a color string into crossterm Color.
fn parse_color(color_str: &str) -> Result<Color> {
    match color_str.to_lowercase().as_str() {
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "white" => Ok(Color::White),
        "dark_grey" | "dark_gray" => Ok(Color::DarkGrey),
        "light_red" => Ok(Color::DarkRed),
        "light_green" => Ok(Color::DarkGreen),
        "light_yellow" => Ok(Color::DarkYellow),
        "light_blue" => Ok(Color::DarkBlue),
        "light_magenta" => Ok(Color::DarkMagenta),
        "light_cyan" => Ok(Color::DarkCyan),
        "grey" | "gray" => Ok(Color::Grey),
        _ => {
            // Try to parse as hex color
            if color_str.starts_with('#') && color_str.len() == 7 {
                let r = u8::from_str_radix(&color_str[1..3], 16)
                    .map_err(|_| XyluxError::config(format!("Invalid hex color: {}", color_str)))?;
                let g = u8::from_str_radix(&color_str[3..5], 16)
                    .map_err(|_| XyluxError::config(format!("Invalid hex color: {}", color_str)))?;
                let b = u8::from_str_radix(&color_str[5..7], 16)
                    .map_err(|_| XyluxError::config(format!("Invalid hex color: {}", color_str)))?;
                Ok(Color::Rgb { r, g, b })
            } else {
                Err(XyluxError::config(format!("Unknown color: {}", color_str)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_parsing() {
        assert_eq!(parse_color("red").unwrap(), Color::Red);
        assert_eq!(parse_color("blue").unwrap(), Color::Blue);
        assert_eq!(parse_color("#FF0000").unwrap(), Color::Rgb { r: 255, g: 0, b: 0 });
        assert_eq!(parse_color("#00FF00").unwrap(), Color::Rgb { r: 0, g: 255, b: 0 });
        assert_eq!(parse_color("#0000FF").unwrap(), Color::Rgb { r: 0, g: 0, b: 255 });

        assert!(parse_color("invalid").is_err());
        assert!(parse_color("#ZZZ").is_err());
    }

    #[test]
    fn test_terminal_interface_creation() {
        let terminal = TerminalInterface::new();
        assert!(terminal.is_ok());

        let terminal = terminal.unwrap();
        assert!(!terminal.raw_mode_enabled);
        assert!(!terminal.alternate_screen_active);
    }

    #[test]
    fn test_cursor_styles() {
        let styles = [
            CursorStyle::Block,
            CursorStyle::Underline,
            CursorStyle::Bar,
            CursorStyle::BlinkingBlock,
            CursorStyle::BlinkingUnderline,
            CursorStyle::BlinkingBar,
        ];

        // Just ensure they can be created and compared
        for style in &styles {
            assert_eq!(*style, *style);
        }
    }
}
