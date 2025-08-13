//! # Status Bar Module
//!
//! Status bar component for the GUI interface

use std::path::PathBuf;

/// Status bar widget for the main application
pub struct StatusBarWidget {
    /// Current status message
    status_message: String,
    /// Current file path
    current_file: Option<PathBuf>,
    /// Current cursor position (line, column)
    cursor_position: (usize, usize),
    /// Whether current file is modified
    is_modified: bool,
    /// Current language/file type
    file_type: String,
    /// Encoding
    encoding: String,
    /// Line ending type
    line_ending: String,
}

impl StatusBarWidget {
    /// Create a new status bar widget
    pub fn new() -> Self {
        Self {
            status_message: "Ready".to_string(),
            current_file: None,
            cursor_position: (1, 1),
            is_modified: false,
            file_type: "Plain Text".to_string(),
            encoding: "UTF-8".to_string(),
            line_ending: "LF".to_string(),
        }
    }

    /// Set the status message
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = message;
    }

    /// Set the current file
    pub fn set_current_file(&mut self, file: Option<PathBuf>) {
        self.current_file = file.clone();

        // Update file type based on extension
        if let Some(path) = &file {
            self.file_type = self.detect_file_type(path);
        } else {
            self.file_type = "Plain Text".to_string();
        }
    }

    /// Set cursor position
    pub fn set_cursor_position(&mut self, line: usize, column: usize) {
        self.cursor_position = (line, column);
    }

    /// Set modified state
    pub fn set_modified(&mut self, modified: bool) {
        self.is_modified = modified;
    }

    /// Set encoding
    pub fn set_encoding(&mut self, encoding: String) {
        self.encoding = encoding;
    }

    /// Set line ending type
    pub fn set_line_ending(&mut self, line_ending: String) {
        self.line_ending = line_ending;
    }

    /// Detect file type from extension
    fn detect_file_type(&self, path: &PathBuf) -> String {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
                "rs" => "Rust".to_string(),
                "toml" => "TOML".to_string(),
                "json" => "JSON".to_string(),
                "md" => "Markdown".to_string(),
                "txt" => "Plain Text".to_string(),
                "py" => "Python".to_string(),
                "js" => "JavaScript".to_string(),
                "ts" => "TypeScript".to_string(),
                "html" => "HTML".to_string(),
                "css" => "CSS".to_string(),
                "xml" => "XML".to_string(),
                "yaml" | "yml" => "YAML".to_string(),
                "sh" => "Shell Script".to_string(),
                "bat" => "Batch File".to_string(),
                "ps1" => "PowerShell".to_string(),
                "c" => "C".to_string(),
                "cpp" | "cxx" | "cc" => "C++".to_string(),
                "h" | "hpp" => "C/C++ Header".to_string(),
                "java" => "Java".to_string(),
                "go" => "Go".to_string(),
                "php" => "PHP".to_string(),
                "rb" => "Ruby".to_string(),
                "swift" => "Swift".to_string(),
                "kt" => "Kotlin".to_string(),
                "cs" => "C#".to_string(),
                "fs" => "F#".to_string(),
                "scala" => "Scala".to_string(),
                "clj" => "Clojure".to_string(),
                "hs" => "Haskell".to_string(),
                "elm" => "Elm".to_string(),
                "ex" | "exs" => "Elixir".to_string(),
                "erl" => "Erlang".to_string(),
                "lua" => "Lua".to_string(),
                "r" => "R".to_string(),
                "m" => "MATLAB".to_string(),
                "sql" => "SQL".to_string(),
                "dockerfile" => "Dockerfile".to_string(),
                _ => format!("{}", extension.to_uppercase()),
            }
        } else {
            "Plain Text".to_string()
        }
    }

    /// Draw the status bar
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left side - status message and file info
            ui.label(&self.status_message);

            if let Some(file) = &self.current_file {
                ui.separator();
                let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown");

                let display_name = if self.is_modified {
                    format!("{} ●", file_name)
                } else {
                    file_name.to_string()
                };

                ui.label(display_name);
            }

            // Spacing to push right side content to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Right side - cursor position, file type, encoding, etc.
                ui.label(&self.line_ending);
                ui.separator();
                ui.label(&self.encoding);
                ui.separator();
                ui.label(&self.file_type);
                ui.separator();
                ui.label(format!("Ln {}, Col {}", self.cursor_position.0, self.cursor_position.1));

                if self.is_modified {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "Modified");
                }
            });
        });
    }
}

impl Default for StatusBarWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Status bar information that can be updated
#[derive(Debug, Clone)]
pub struct StatusInfo {
    pub message: String,
    pub file_path: Option<PathBuf>,
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub is_modified: bool,
    pub total_lines: usize,
    pub total_characters: usize,
    pub selection_length: usize,
}

impl StatusInfo {
    /// Create new status info
    pub fn new() -> Self {
        Self {
            message: "Ready".to_string(),
            file_path: None,
            cursor_line: 1,
            cursor_column: 1,
            is_modified: false,
            total_lines: 1,
            total_characters: 0,
            selection_length: 0,
        }
    }

    /// Update from file buffer
    pub fn update_from_buffer(
        &mut self,
        content: &str,
        cursor_line: usize,
        cursor_column: usize,
        is_modified: bool,
    ) {
        self.cursor_line = cursor_line + 1; // Convert to 1-based
        self.cursor_column = cursor_column + 1; // Convert to 1-based
        self.is_modified = is_modified;
        self.total_lines = content.lines().count().max(1);
        self.total_characters = content.chars().count();
    }

    /// Set status message
    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    /// Set file path
    pub fn set_file_path(&mut self, path: Option<PathBuf>) {
        self.file_path = path;
    }
}

impl Default for StatusInfo {
    fn default() -> Self {
        Self::new()
    }
}
