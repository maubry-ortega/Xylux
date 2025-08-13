//! # Buffer Module
//!
//! Text buffer management for the editor.

use std::path::PathBuf;
use std::time::SystemTime;

use crate::core::{Result, XyluxError};

/// A text buffer that holds the content of a file.
#[derive(Debug, Clone)]
pub struct Buffer {
    /// The text content of the buffer.
    lines: Vec<String>,
    /// The file path associated with this buffer (if any).
    file_path: Option<PathBuf>,
    /// Whether the buffer has been modified since last save.
    modified: bool,
    /// Timestamp of last modification.
    last_modified: SystemTime,
    /// Original content for comparison.
    original_content: String,
    /// File encoding.
    encoding: String,
    /// Line ending style.
    line_ending: LineEnding,
}

/// Line ending styles.
#[derive(Debug, Clone, PartialEq)]
pub enum LineEnding {
    /// Unix-style line endings (\n).
    Unix,
    /// Windows-style line endings (\r\n).
    Windows,
    /// Classic Mac-style line endings (\r).
    Mac,
}

impl Buffer {
    /// Create a new buffer with the given content.
    pub fn new(content: String, file_path: Option<PathBuf>) -> Self {
        let line_ending = Self::detect_line_ending(&content);
        let lines = Self::split_lines(&content, &line_ending);

        Self {
            lines,
            file_path,
            modified: false,
            last_modified: SystemTime::now(),
            original_content: content,
            encoding: "UTF-8".to_string(),
            line_ending,
        }
    }

    /// Create an empty buffer.
    pub fn empty() -> Self {
        Self::new(String::new(), None)
    }

    /// Get the complete content of the buffer.
    pub fn get_content(&self) -> String {
        self.lines.join(&self.line_ending.to_string())
    }

    /// Get a specific line from the buffer.
    pub fn get_line(&self, line_index: usize) -> Option<&String> {
        self.lines.get(line_index)
    }

    /// Get the number of lines in the buffer.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get the length of a specific line.
    pub fn line_length(&self, line_index: usize) -> usize {
        self.lines.get(line_index).map_or(0, |line| line.len())
    }

    /// Insert text at the specified position.
    pub fn insert_text(&mut self, line: usize, column: usize, text: &str) -> Result<()> {
        if line >= self.lines.len() {
            return Err(XyluxError::syntax_error(format!(
                "Line index {} out of bounds (max: {})",
                line,
                self.lines.len()
            )));
        }

        let current_line = &mut self.lines[line];
        if column > current_line.len() {
            return Err(XyluxError::syntax_error(format!(
                "Column index {} out of bounds for line {} (max: {})",
                column,
                line,
                current_line.len()
            )));
        }

        // Handle multi-line text insertion
        if text.contains('\n') {
            self.insert_multiline_text(line, column, text)?;
        } else {
            // Simple single-line insertion
            current_line.insert_str(column, text);
        }

        self.mark_modified();
        Ok(())
    }

    /// Insert multi-line text.
    fn insert_multiline_text(&mut self, line: usize, column: usize, text: &str) -> Result<()> {
        let lines_to_insert = text.split('\n').collect::<Vec<_>>();
        let current_line = &mut self.lines[line];

        if lines_to_insert.len() == 1 {
            // Single line, just insert
            current_line.insert_str(column, text);
            return Ok(());
        }

        // Split the current line at the insertion point
        let after_cursor = current_line.split_off(column);

        // Insert the first part of the new text
        current_line.push_str(lines_to_insert[0]);

        // Insert the middle lines
        for (i, &line_text) in lines_to_insert.iter().enumerate().skip(1) {
            if i == lines_to_insert.len() - 1 {
                // Last line: add the text and the remaining part of the original line
                self.lines.insert(line + i, format!("{}{}", line_text, after_cursor));
            } else {
                // Middle lines: just add the text
                self.lines.insert(line + i, line_text.to_string());
            }
        }

        Ok(())
    }

    /// Delete text from the buffer.
    pub fn delete_text(&mut self, line: usize, column: usize, count: usize) -> Result<()> {
        if line >= self.lines.len() {
            return Ok(()); // Nothing to delete
        }

        let current_line = &mut self.lines[line];
        if column >= current_line.len() {
            return Ok(()); // Nothing to delete
        }

        let end_pos = (column + count).min(current_line.len());
        current_line.drain(column..end_pos);

        self.mark_modified();
        Ok(())
    }

    /// Delete a range of text.
    pub fn delete_range(
        &mut self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Result<()> {
        if start_line >= self.lines.len() || end_line >= self.lines.len() {
            return Err(XyluxError::syntax_error("Line range out of bounds".to_string()));
        }

        if start_line == end_line {
            // Single line deletion
            let line = &mut self.lines[start_line];
            let start = start_col.min(line.len());
            let end = end_col.min(line.len());
            if start < end {
                line.drain(start..end);
            }
        } else {
            // Multi-line deletion
            let start_line_content =
                &self.lines[start_line][..start_col.min(self.lines[start_line].len())];
            let end_line_content = &self.lines[end_line][end_col.min(self.lines[end_line].len())..];

            // Create the merged line
            let merged_line = format!("{}{}", start_line_content, end_line_content);

            // Remove the lines in between and replace the start line
            self.lines.drain(start_line + 1..=end_line);
            self.lines[start_line] = merged_line;
        }

        self.mark_modified();
        Ok(())
    }

    /// Insert a new line at the specified position.
    pub fn insert_line(&mut self, line: usize, content: String) -> Result<()> {
        if line > self.lines.len() {
            return Err(XyluxError::syntax_error(format!(
                "Line index {} out of bounds (max: {})",
                line,
                self.lines.len()
            )));
        }

        self.lines.insert(line, content);
        self.mark_modified();
        Ok(())
    }

    /// Delete a line from the buffer.
    pub fn delete_line(&mut self, line: usize) -> Result<()> {
        if line >= self.lines.len() {
            return Ok(()); // Nothing to delete
        }

        self.lines.remove(line);
        self.mark_modified();
        Ok(())
    }

    /// Replace the content of a line.
    pub fn replace_line(&mut self, line: usize, content: String) -> Result<()> {
        if line >= self.lines.len() {
            return Err(XyluxError::syntax_error(format!(
                "Line index {} out of bounds (max: {})",
                line,
                self.lines.len()
            )));
        }

        self.lines[line] = content;
        self.mark_modified();
        Ok(())
    }

    /// Check if the buffer has been modified.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Mark the buffer as saved (not modified).
    pub fn mark_saved(&mut self) {
        self.modified = false;
        self.original_content = self.get_content();
    }

    /// Mark the buffer as modified.
    fn mark_modified(&mut self) {
        self.modified = true;
        self.last_modified = SystemTime::now();
    }

    /// Get the file path associated with this buffer.
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    /// Set the file path for this buffer.
    pub fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }

    /// Get the encoding of the buffer.
    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    /// Get the line ending style.
    pub fn line_ending(&self) -> &LineEnding {
        &self.line_ending
    }

    /// Set the line ending style.
    pub fn set_line_ending(&mut self, line_ending: LineEnding) {
        self.line_ending = line_ending;
        self.mark_modified();
    }

    /// Get the last modification time.
    pub fn last_modified(&self) -> SystemTime {
        self.last_modified
    }

    /// Delete a single character at the specified position.
    pub fn delete_char(&mut self, line: usize, column: usize) -> Result<()> {
        if line >= self.lines.len() {
            return Ok(()); // Nothing to delete
        }

        if column >= self.lines[line].len() {
            return Ok(()); // Nothing to delete
        }

        self.lines[line].remove(column);
        self.mark_modified();
        Ok(())
    }

    /// Join the current line with the next line.
    pub fn join_lines(&mut self, line: usize) -> Result<()> {
        if line >= self.lines.len() || line + 1 >= self.lines.len() {
            return Ok(()); // Nothing to join
        }

        let next_line = self.lines.remove(line + 1);
        self.lines[line].push_str(&next_line);
        self.mark_modified();
        Ok(())
    }

    /// Insert a newline at the specified position, splitting the line.
    pub fn insert_newline(&mut self, line: usize, column: usize) -> Result<()> {
        if line >= self.lines.len() {
            return Err(XyluxError::syntax_error(format!(
                "Line index {} out of bounds (max: {})",
                line,
                self.lines.len() - 1
            )));
        }

        let current_line_content = self.lines[line].clone();
        let (left, right) = if column <= current_line_content.len() {
            current_line_content.split_at(column)
        } else {
            (current_line_content.as_str(), "")
        };

        // Replace current line with left part
        self.lines[line] = left.to_string();
        // Insert new line with right part
        self.lines.insert(line + 1, right.to_string());

        self.mark_modified();
        Ok(())
    }

    /// Get text in a specific range.
    pub fn get_text_range(
        &self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Result<String> {
        if start_line >= self.lines.len() || end_line >= self.lines.len() {
            return Err(XyluxError::syntax_error("Line range out of bounds".to_string()));
        }

        if start_line == end_line {
            // Same line
            let line = &self.lines[start_line];
            let start = start_col.min(line.len());
            let end = end_col.min(line.len());
            return Ok(line[start..end].to_string());
        }

        let mut result = String::new();

        // First line (from start_col to end)
        let first_line = &self.lines[start_line];
        let start = start_col.min(first_line.len());
        result.push_str(&first_line[start..]);
        result.push('\n');

        // Middle lines (complete lines)
        for line_idx in start_line + 1..end_line {
            result.push_str(&self.lines[line_idx]);
            result.push('\n');
        }

        // Last line (from start to end_col)
        let last_line = &self.lines[end_line];
        let end = end_col.min(last_line.len());
        result.push_str(&last_line[..end]);

        Ok(result)
    }

    /// Find all occurrences of a pattern in the buffer.
    pub fn find_all(&self, pattern: &str) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();

        for (line_idx, line) in self.lines.iter().enumerate() {
            let mut start = 0;
            while let Some(pos) = line[start..].find(pattern) {
                matches.push((line_idx, start + pos));
                start = start + pos + 1;
            }
        }

        matches
    }

    /// Find text in the buffer.
    pub fn find(&self, query: &str, case_sensitive: bool) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();

        for (line_idx, line) in self.lines.iter().enumerate() {
            let search_line = if case_sensitive { line.clone() } else { line.to_lowercase() };

            let search_query =
                if case_sensitive { query.to_string() } else { query.to_lowercase() };

            let mut start = 0;
            while let Some(pos) = search_line[start..].find(&search_query) {
                matches.push((line_idx, start + pos));
                start += pos + 1;
            }
        }

        matches
    }

    /// Replace text in the buffer.
    pub fn replace(
        &mut self,
        query: &str,
        replacement: &str,
        case_sensitive: bool,
    ) -> Result<usize> {
        let matches = self.find(query, case_sensitive);
        let mut replaced_count = 0;

        // Process matches in reverse order to maintain correct positions
        for (line_idx, col_idx) in matches.into_iter().rev() {
            if let Some(line) = self.lines.get_mut(line_idx) {
                let search_query = if case_sensitive {
                    query
                } else {
                    // For case-insensitive replacement, we need to find the actual match
                    let start = col_idx;
                    let end = start + query.len();
                    if end <= line.len() {
                        let actual_match = &line[start..end];
                        if actual_match.to_lowercase() == query.to_lowercase() {
                            line.replace_range(start..end, replacement);
                            replaced_count += 1;
                        }
                    }
                    continue;
                };

                if line[col_idx..].starts_with(search_query) {
                    let end = col_idx + query.len();
                    line.replace_range(col_idx..end, replacement);
                    replaced_count += 1;
                }
            }
        }

        if replaced_count > 0 {
            self.mark_modified();
        }

        Ok(replaced_count)
    }

    /// Get a range of lines.
    pub fn get_lines(&self, start: usize, end: usize) -> Vec<&String> {
        let end = end.min(self.lines.len());
        if start >= end {
            return Vec::new();
        }
        self.lines[start..end].iter().collect()
    }

    /// Detect line ending style from content.
    fn detect_line_ending(content: &str) -> LineEnding {
        if content.contains("\r\n") {
            LineEnding::Windows
        } else if content.contains('\r') {
            LineEnding::Mac
        } else {
            LineEnding::Unix
        }
    }

    /// Split content into lines based on line ending style.
    fn split_lines(content: &str, line_ending: &LineEnding) -> Vec<String> {
        match line_ending {
            LineEnding::Windows => content.split("\r\n").map(String::from).collect(),
            LineEnding::Mac => content.split('\r').map(String::from).collect(),
            LineEnding::Unix => content.split('\n').map(String::from).collect(),
        }
    }
}

impl LineEnding {
    /// Convert line ending to string representation.
    pub fn to_string(&self) -> String {
        match self {
            LineEnding::Unix => "\n".to_string(),
            LineEnding::Windows => "\r\n".to_string(),
            LineEnding::Mac => "\r".to_string(),
        }
    }

    /// Get the display name of the line ending.
    pub fn display_name(&self) -> &'static str {
        match self {
            LineEnding::Unix => "LF",
            LineEnding::Windows => "CRLF",
            LineEnding::Mac => "CR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let content = "Hello\nWorld\n";
        let buffer = Buffer::new(content.to_string(), None);

        assert_eq!(buffer.line_count(), 3); // Empty line at end
        assert_eq!(buffer.get_line(0), Some(&"Hello".to_string()));
        assert_eq!(buffer.get_line(1), Some(&"World".to_string()));
        assert_eq!(buffer.line_ending(), &LineEnding::Unix);
    }

    #[test]
    fn test_text_insertion() {
        let mut buffer = Buffer::new("Hello World".to_string(), None);

        buffer.insert_text(0, 5, ", Beautiful").unwrap();
        assert_eq!(buffer.get_content(), "Hello, Beautiful World");
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_text_deletion() {
        let mut buffer = Buffer::new("Hello World".to_string(), None);

        buffer.delete_text(0, 5, 6).unwrap(); // Delete " World"
        assert_eq!(buffer.get_content(), "Hello");
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_line_operations() {
        let mut buffer = Buffer::new("Line 1\nLine 2".to_string(), None);

        buffer.insert_line(1, "New Line".to_string()).unwrap();
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.get_line(1), Some(&"New Line".to_string()));

        buffer.delete_line(1).unwrap();
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.get_line(1), Some(&"Line 2".to_string()));
    }

    #[test]
    fn test_find_and_replace() {
        let mut buffer = Buffer::new("Hello world, hello universe".to_string(), None);

        let matches = buffer.find("hello", false);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (0, 0));
        assert_eq!(matches[1], (0, 13));

        let replaced = buffer.replace("hello", "hi", false).unwrap();
        assert_eq!(replaced, 2);
        assert_eq!(buffer.get_content(), "hi world, hi universe");
    }

    #[test]
    fn test_line_ending_detection() {
        let unix_content = "line1\nline2\n";
        let windows_content = "line1\r\nline2\r\n";
        let mac_content = "line1\rline2\r";

        assert_eq!(Buffer::detect_line_ending(unix_content), LineEnding::Unix);
        assert_eq!(Buffer::detect_line_ending(windows_content), LineEnding::Windows);
        assert_eq!(Buffer::detect_line_ending(mac_content), LineEnding::Mac);
    }

    #[test]
    fn test_multiline_insertion() {
        let mut buffer = Buffer::new("Line 1\nLine 3".to_string(), None);

        buffer.insert_text(0, 6, "\nLine 2").unwrap();
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.get_line(0), Some(&"Line 1".to_string()));
        assert_eq!(buffer.get_line(1), Some(&"Line 2".to_string()));
        assert_eq!(buffer.get_line(2), Some(&"Line 3".to_string()));
    }
}
