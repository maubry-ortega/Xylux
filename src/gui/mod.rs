//! # GUI Module
//!
//! Modern GUI interface for Xylux IDE using egui/eframe

pub mod app;
pub mod editor;
pub mod file_tree;
pub mod menu;
pub mod statusbar;
pub mod tools;

pub use app::XyluxIdeApp;
pub use tools::ToolsWindow;

use std::path::PathBuf;

/// Theme colors for the IDE
#[derive(Clone, Debug)]
pub struct IdTheme {
    pub background: egui::Color32,
    pub text: egui::Color32,
    pub selection: egui::Color32,
    pub cursor: egui::Color32,
    pub line_numbers: egui::Color32,
    pub status_bar: egui::Color32,
    pub menu_bar: egui::Color32,
    pub border: egui::Color32,
}

impl Default for IdTheme {
    fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(30, 30, 30),
            text: egui::Color32::from_rgb(220, 220, 220),
            selection: egui::Color32::from_rgb(80, 120, 200),
            cursor: egui::Color32::from_rgb(255, 255, 255),
            line_numbers: egui::Color32::from_rgb(120, 120, 120),
            status_bar: egui::Color32::from_rgb(40, 40, 40),
            menu_bar: egui::Color32::from_rgb(50, 50, 50),
            border: egui::Color32::from_rgb(60, 60, 60),
        }
    }
}

/// File buffer for text editing
#[derive(Clone, Debug)]
pub struct FileBuffer {
    pub path: Option<PathBuf>,
    pub content: String,
    pub modified: bool,
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub scroll_offset: f32,
}

impl FileBuffer {
    pub fn new() -> Self {
        Self {
            path: None,
            content: String::new(),
            modified: false,
            cursor_line: 0,
            cursor_column: 0,
            scroll_offset: 0.0,
        }
    }

    pub fn from_file(path: PathBuf, content: String) -> Self {
        Self {
            path: Some(path),
            content,
            modified: false,
            cursor_line: 0,
            cursor_column: 0,
            scroll_offset: 0.0,
        }
    }

    pub fn get_lines(&self) -> Vec<&str> {
        if self.content.is_empty() { vec![""] } else { self.content.lines().collect() }
    }

    pub fn insert_char(&mut self, c: char) {
        let lines: Vec<&str> = self.content.lines().collect();
        let mut new_lines = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if i == self.cursor_line {
                let mut new_line = line.to_string();
                if self.cursor_column <= new_line.len() {
                    new_line.insert(self.cursor_column, c);
                    self.cursor_column += 1;
                } else {
                    new_line.push(c);
                    self.cursor_column = new_line.len();
                }
                new_lines.push(new_line);
            } else {
                new_lines.push(line.to_string());
            }
        }

        self.content = new_lines.join("\n");
        self.modified = true;
    }

    pub fn insert_newline(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        let mut new_lines = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if i == self.cursor_line {
                let (left, right) = line.split_at(self.cursor_column.min(line.len()));
                new_lines.push(left.to_string());
                new_lines.push(right.to_string());
                self.cursor_line += 1;
                self.cursor_column = 0;
            } else if i > self.cursor_line {
                new_lines.push(line.to_string());
            } else {
                new_lines.push(line.to_string());
            }
        }

        if new_lines.is_empty() {
            new_lines.push(String::new());
            new_lines.push(String::new());
            self.cursor_line = 1;
            self.cursor_column = 0;
        }

        self.content = new_lines.join("\n");
        self.modified = true;
    }

    pub fn delete_char(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        let mut new_lines: Vec<String> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if i == self.cursor_line {
                let mut new_line = line.to_string();
                if self.cursor_column > 0 && self.cursor_column <= new_line.len() {
                    new_line.remove(self.cursor_column - 1);
                    self.cursor_column -= 1;
                } else if self.cursor_column == 0 && i > 0 {
                    // Join with previous line
                    if let Some(prev_line) = new_lines.last_mut() {
                        self.cursor_column = prev_line.len();
                        prev_line.push_str(&new_line);
                        self.cursor_line -= 1;
                        continue;
                    }
                }
                new_lines.push(new_line);
            } else {
                new_lines.push(line.to_string());
            }
        }

        if !new_lines.is_empty() {
            self.content = new_lines.join("\n");
            self.modified = true;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_column > 0 {
            self.cursor_column -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            let lines: Vec<&str> = self.content.lines().collect();
            if self.cursor_line < lines.len() {
                self.cursor_column = lines[self.cursor_line].len();
            }
        }
    }

    pub fn move_cursor_right(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_line < lines.len() {
            let line = lines[self.cursor_line];
            if self.cursor_column < line.len() {
                self.cursor_column += 1;
            } else if self.cursor_line + 1 < lines.len() {
                self.cursor_line += 1;
                self.cursor_column = 0;
            }
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            let lines: Vec<&str> = self.content.lines().collect();
            if self.cursor_line < lines.len() {
                self.cursor_column = self.cursor_column.min(lines[self.cursor_line].len());
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_line + 1 < lines.len() {
            self.cursor_line += 1;
            self.cursor_column = self.cursor_column.min(lines[self.cursor_line].len());
        }
    }

    pub fn get_display_name(&self) -> String {
        match &self.path {
            Some(path) => {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown");
                if self.modified { format!("{} *", name) } else { name.to_string() }
            }
            None => {
                if self.modified {
                    "Untitled *".to_string()
                } else {
                    "Untitled".to_string()
                }
            }
        }
    }
}

impl Default for FileBuffer {
    fn default() -> Self {
        Self::new()
    }
}
