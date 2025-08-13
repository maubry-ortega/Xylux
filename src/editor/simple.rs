//! Simple editor implementation for direct terminal interaction

use std::fs;
use std::io::{self, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::core::{Config, Result, XyluxError};

/// Simple editor for basic text editing functionality
pub struct SimpleEditor {
    /// Content lines
    lines: Vec<String>,
    /// Current cursor position (row, col)
    cursor: (usize, usize),
    /// Terminal size (width, height)
    terminal_size: (u16, u16),
    /// Current file name
    file_name: Option<String>,
    /// Whether content has been modified
    modified: bool,
    /// Status message
    status_message: String,
    /// Configuration
    #[allow(dead_code)]
    config: Config,
}

impl SimpleEditor {
    /// Create a new simple editor
    pub fn new(config: Config, file_path: Option<String>) -> Result<Self> {
        let mut editor = Self {
            lines: vec![String::new()],
            cursor: (0, 0),
            terminal_size: (80, 24),
            file_name: file_path.clone(),
            modified: false,
            status_message: "Xylux IDE - Press Ctrl+Q to quit".to_string(),
            config,
        };

        // Load file if provided
        if let Some(path) = file_path {
            editor.open_file(&path)?;
        }

        Ok(editor)
    }

    /// Open a file
    pub fn open_file(&mut self, path: &str) -> Result<()> {
        match fs::read_to_string(path) {
            Ok(content) => {
                self.lines = if content.is_empty() {
                    vec![String::new()]
                } else {
                    content.lines().map(|s| s.to_string()).collect()
                };
                self.file_name = Some(path.to_string());
                self.modified = false;
                self.status_message = format!("Opened: {}", path);
                Ok(())
            }
            Err(e) => {
                self.status_message = format!("Error opening {}: {}", path, e);
                Err(XyluxError::io(e, format!("Failed to open file: {}", path)))
            }
        }
    }

    /// Save current file
    fn save_file(&mut self) -> Result<()> {
        if let Some(ref path) = self.file_name.clone() {
            let content = self.lines.join("\n");
            match fs::write(path, content) {
                Ok(()) => {
                    self.modified = false;
                    self.status_message = format!("Saved: {}", path);
                    Ok(())
                }
                Err(e) => {
                    self.status_message = format!("Error saving: {}", e);
                    Err(XyluxError::io(e, format!("Failed to save file: {}", path)))
                }
            }
        } else {
            // TODO: Implement save as dialog
            self.status_message = "No filename specified".to_string();
            Err(XyluxError::project_error("No filename specified".to_string()))
        }
    }

    /// Run the main editor loop
    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;

        // Update terminal size
        if let Ok((width, height)) = terminal::size() {
            self.terminal_size = (width, height);
        }

        let result = self.main_loop();

        // Cleanup
        let _unused = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _unused = terminal::disable_raw_mode();

        result
    }

    /// Main editor loop
    fn main_loop(&mut self) -> Result<()> {
        loop {
            self.refresh_screen()?;

            // Handle events
            match event::read()? {
                Event::Key(key_event) => {
                    if self.handle_key_event(key_event)? {
                        break; // Exit requested
                    }
                }
                Event::Resize(width, height) => {
                    self.terminal_size = (width, height);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle key events, returns true if should exit
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                if self.modified {
                    self.status_message = "File modified! Press Ctrl+Q again to quit without saving, or Ctrl+S to save".to_string();
                    // TODO: Implement confirmation logic
                }
                return Ok(true);
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.save_file()?;
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                return Ok(true); // Also allow Ctrl+C to exit
            }
            (KeyCode::Enter, _) => {
                self.insert_newline();
            }
            (KeyCode::Backspace, _) => {
                self.delete_char();
            }
            (KeyCode::Delete, _) => {
                self.delete_char_forward();
            }
            (KeyCode::Left, _) => {
                self.move_cursor_left();
            }
            (KeyCode::Right, _) => {
                self.move_cursor_right();
            }
            (KeyCode::Up, _) => {
                self.move_cursor_up();
            }
            (KeyCode::Down, _) => {
                self.move_cursor_down();
            }
            (KeyCode::Home, _) => {
                self.cursor.1 = 0;
            }
            (KeyCode::End, _) => {
                if self.cursor.0 < self.lines.len() {
                    self.cursor.1 = self.lines[self.cursor.0].len();
                }
            }
            (KeyCode::Char(c), _) => {
                self.insert_char(c);
            }
            _ => {}
        }
        Ok(false)
    }

    /// Insert a character at cursor position
    fn insert_char(&mut self, c: char) {
        if self.cursor.0 >= self.lines.len() {
            self.lines.push(String::new());
        }

        let line = &mut self.lines[self.cursor.0];
        if self.cursor.1 <= line.len() {
            line.insert(self.cursor.1, c);
            self.cursor.1 += 1;
            self.modified = true;
        }
    }

    /// Insert a newline
    fn insert_newline(&mut self) {
        if self.cursor.0 >= self.lines.len() {
            self.lines.push(String::new());
            self.cursor.0 = self.lines.len() - 1;
        }

        let current_line = self.lines[self.cursor.0].clone();
        let (left, right) = current_line.split_at(self.cursor.1);

        self.lines[self.cursor.0] = left.to_string();
        self.lines.insert(self.cursor.0 + 1, right.to_string());

        self.cursor.0 += 1;
        self.cursor.1 = 0;
        self.modified = true;
    }

    /// Delete character at cursor (backspace)
    fn delete_char(&mut self) {
        if self.cursor.1 > 0 {
            self.lines[self.cursor.0].remove(self.cursor.1 - 1);
            self.cursor.1 -= 1;
            self.modified = true;
        } else if self.cursor.0 > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor.0);
            self.cursor.0 -= 1;
            self.cursor.1 = self.lines[self.cursor.0].len();
            self.lines[self.cursor.0].push_str(&current_line);
            self.modified = true;
        }
    }

    /// Delete character forward (delete key)
    fn delete_char_forward(&mut self) {
        if self.cursor.0 < self.lines.len() {
            let line = &mut self.lines[self.cursor.0];
            if self.cursor.1 < line.len() {
                line.remove(self.cursor.1);
                self.modified = true;
            } else if self.cursor.0 + 1 < self.lines.len() {
                // Join with next line
                let next_line = self.lines.remove(self.cursor.0 + 1);
                self.lines[self.cursor.0].push_str(&next_line);
                self.modified = true;
            }
        }
    }

    /// Move cursor left
    fn move_cursor_left(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = self.lines[self.cursor.0].len();
        }
    }

    /// Move cursor right
    fn move_cursor_right(&mut self) {
        if self.cursor.0 < self.lines.len() {
            if self.cursor.1 < self.lines[self.cursor.0].len() {
                self.cursor.1 += 1;
            } else if self.cursor.0 + 1 < self.lines.len() {
                self.cursor.0 += 1;
                self.cursor.1 = 0;
            }
        }
    }

    /// Move cursor up
    fn move_cursor_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            let line_len = self.lines[self.cursor.0].len();
            if self.cursor.1 > line_len {
                self.cursor.1 = line_len;
            }
        }
    }

    /// Move cursor down
    fn move_cursor_down(&mut self) {
        if self.cursor.0 + 1 < self.lines.len() {
            self.cursor.0 += 1;
            let line_len = self.lines[self.cursor.0].len();
            if self.cursor.1 > line_len {
                self.cursor.1 = line_len;
            }
        }
    }

    /// Refresh the screen
    fn refresh_screen(&mut self) -> Result<()> {
        queue!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0))?;

        // Draw content area
        self.draw_content()?;

        // Draw status bar
        self.draw_status_bar()?;

        // Position cursor
        queue!(io::stdout(), MoveTo(self.cursor.1 as u16, self.cursor.0 as u16))?;

        io::stdout().flush()?;
        Ok(())
    }

    /// Draw the content area
    fn draw_content(&self) -> Result<()> {
        let content_height = (self.terminal_size.1 as usize).saturating_sub(2);

        for row in 0..content_height {
            if row < self.lines.len() {
                let line = &self.lines[row];
                let display_line = if line.len() > self.terminal_size.0 as usize {
                    &line[..self.terminal_size.0 as usize]
                } else {
                    line
                };
                queue!(io::stdout(), Print(display_line))?;
            } else {
                queue!(io::stdout(), Print("~"))?;
            }

            if row < content_height - 1 {
                queue!(io::stdout(), Print("\r\n"))?;
            }
        }

        Ok(())
    }

    /// Draw the status bar
    fn draw_status_bar(&self) -> Result<()> {
        let status_y = self.terminal_size.1.saturating_sub(1);
        queue!(io::stdout(), MoveTo(0, status_y))?;

        // Status bar background
        queue!(
            io::stdout(),
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White)
        )?;

        // File name and modified indicator
        let file_display = match &self.file_name {
            Some(name) => {
                if self.modified {
                    format!("{} [modified]", name)
                } else {
                    name.clone()
                }
            }
            None => {
                if self.modified {
                    "[untitled] [modified]".to_string()
                } else {
                    "[untitled]".to_string()
                }
            }
        };

        // Cursor position
        let cursor_info = format!("{}:{}", self.cursor.0 + 1, self.cursor.1 + 1);

        // Status message
        let status_line = format!("{} | {} | {}", file_display, cursor_info, self.status_message);

        let padded_status =
            format!("{:width$}", status_line, width = self.terminal_size.0 as usize);
        queue!(io::stdout(), Print(padded_status))?;

        queue!(io::stdout(), ResetColor)?;
        Ok(())
    }
}

impl Drop for SimpleEditor {
    fn drop(&mut self) {
        let _unused = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _unused = terminal::disable_raw_mode();
    }
}
