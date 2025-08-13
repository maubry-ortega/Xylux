//! # Cursor Module
//!
//! Cursor position and movement management for the editor.

use std::cmp;

/// Represents a cursor position in the text editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cursor {
    /// Current line (0-indexed).
    pub line: usize,
    /// Current column (0-indexed).
    pub column: usize,
    /// Desired column for vertical movement.
    pub desired_column: usize,
}

impl Cursor {
    /// Create a new cursor at the specified position.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column, desired_column: column }
    }

    /// Create a cursor at the origin (0, 0).
    pub fn origin() -> Self {
        Self::new(0, 0)
    }

    /// Move the cursor to a specific position.
    pub fn move_to(&mut self, line: usize, column: usize) {
        self.line = line;
        self.column = column;
        self.desired_column = column;
    }

    /// Move the cursor left by one character.
    pub fn move_left(&mut self, line_lengths: &[usize]) {
        if self.column > 0 {
            self.column -= 1;
            self.desired_column = self.column;
        } else if self.line > 0 {
            // Move to end of previous line
            self.line -= 1;
            if let Some(&line_length) = line_lengths.get(self.line) {
                self.column = line_length;
                self.desired_column = self.column;
            }
        }
    }

    /// Move the cursor right by one character.
    pub fn move_right(&mut self, line_lengths: &[usize]) {
        if let Some(&line_length) = line_lengths.get(self.line) {
            if self.column < line_length {
                self.column += 1;
                self.desired_column = self.column;
            } else if self.line + 1 < line_lengths.len() {
                // Move to beginning of next line
                self.line += 1;
                self.column = 0;
                self.desired_column = 0;
            }
        }
    }

    /// Move the cursor up by one line.
    pub fn move_up(&mut self, line_lengths: &[usize]) {
        if self.line > 0 {
            self.line -= 1;
            if let Some(&line_length) = line_lengths.get(self.line) {
                self.column = cmp::min(self.desired_column, line_length);
            }
        }
    }

    /// Move the cursor down by one line.
    pub fn move_down(&mut self, line_lengths: &[usize]) {
        if self.line + 1 < line_lengths.len() {
            self.line += 1;
            if let Some(&line_length) = line_lengths.get(self.line) {
                self.column = cmp::min(self.desired_column, line_length);
            }
        }
    }

    /// Move the cursor to the beginning of the current line.
    pub fn move_to_line_start(&mut self) {
        self.column = 0;
        self.desired_column = 0;
    }

    /// Move the cursor to the end of the current line.
    pub fn move_to_line_end(&mut self, line_lengths: &[usize]) {
        if let Some(&line_length) = line_lengths.get(self.line) {
            self.column = line_length;
            self.desired_column = line_length;
        }
    }

    /// Move the cursor to the beginning of the buffer.
    pub fn move_to_buffer_start(&mut self) {
        self.line = 0;
        self.column = 0;
        self.desired_column = 0;
    }

    /// Move the cursor to the end of the buffer.
    pub fn move_to_buffer_end(&mut self, line_lengths: &[usize]) {
        if !line_lengths.is_empty() {
            self.line = line_lengths.len() - 1;
            self.column = line_lengths[self.line];
            self.desired_column = self.column;
        }
    }

    /// Move the cursor by a word to the left.
    pub fn move_word_left(&mut self, lines: &[String]) {
        if let Some(line) = lines.get(self.line) {
            if self.column > 0 {
                let line_chars: Vec<char> = line.chars().collect();
                let mut pos = self.column.saturating_sub(1);

                // Skip whitespace
                while pos > 0 && line_chars[pos].is_whitespace() {
                    pos -= 1;
                }

                // Skip word characters
                while pos > 0 && !line_chars[pos - 1].is_whitespace() {
                    pos -= 1;
                }

                self.column = pos;
                self.desired_column = pos;
            } else {
                // Move to previous line
                self.move_left(&lines.iter().map(|l| l.len()).collect::<Vec<_>>());
            }
        }
    }

    /// Move the cursor by a word to the right.
    pub fn move_word_right(&mut self, lines: &[String]) {
        if let Some(line) = lines.get(self.line) {
            let line_chars: Vec<char> = line.chars().collect();
            if self.column < line_chars.len() {
                let mut pos = self.column;

                // Skip current word
                while pos < line_chars.len() && !line_chars[pos].is_whitespace() {
                    pos += 1;
                }

                // Skip whitespace
                while pos < line_chars.len() && line_chars[pos].is_whitespace() {
                    pos += 1;
                }

                self.column = pos;
                self.desired_column = pos;
            } else {
                // Move to next line
                self.move_right(&lines.iter().map(|l| l.len()).collect::<Vec<_>>());
            }
        }
    }

    /// Move the cursor up by a page.
    pub fn move_page_up(&mut self, page_size: usize, line_lengths: &[usize]) {
        let new_line = self.line.saturating_sub(page_size);
        self.line = new_line;
        if let Some(&line_length) = line_lengths.get(self.line) {
            self.column = cmp::min(self.desired_column, line_length);
        }
    }

    /// Move the cursor down by a page.
    pub fn move_page_down(&mut self, page_size: usize, line_lengths: &[usize]) {
        let new_line = cmp::min(self.line + page_size, line_lengths.len().saturating_sub(1));
        self.line = new_line;
        if let Some(&line_length) = line_lengths.get(self.line) {
            self.column = cmp::min(self.desired_column, line_length);
        }
    }

    /// Check if the cursor is at a valid position in the buffer.
    pub fn is_valid(&self, line_lengths: &[usize]) -> bool {
        if self.line >= line_lengths.len() {
            return false;
        }
        self.column <= line_lengths[self.line]
    }

    /// Clamp the cursor to valid bounds.
    pub fn clamp(&mut self, line_lengths: &[usize]) {
        if line_lengths.is_empty() {
            self.line = 0;
            self.column = 0;
            self.desired_column = 0;
            return;
        }

        self.line = cmp::min(self.line, line_lengths.len() - 1);
        if let Some(&line_length) = line_lengths.get(self.line) {
            self.column = cmp::min(self.column, line_length);
            self.desired_column = cmp::min(self.desired_column, line_length);
        }
    }

    /// Get the position as a tuple.
    pub fn position(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    /// Get the line number (1-indexed for display).
    pub fn display_line(&self) -> usize {
        self.line + 1
    }

    /// Get the column number (1-indexed for display).
    pub fn display_column(&self) -> usize {
        self.column + 1
    }

    /// Check if this cursor is before another cursor.
    pub fn is_before(&self, other: &Cursor) -> bool {
        self.line < other.line || (self.line == other.line && self.column < other.column)
    }

    /// Check if this cursor is after another cursor.
    pub fn is_after(&self, other: &Cursor) -> bool {
        self.line > other.line || (self.line == other.line && self.column > other.column)
    }

    /// Calculate the distance between two cursors (in characters).
    pub fn distance_to(&self, other: &Cursor, line_lengths: &[usize]) -> usize {
        if self.line == other.line {
            return (self.column as isize - other.column as isize).unsigned_abs();
        }

        let (start, end) = if self.is_before(other) { (self, other) } else { (other, self) };

        let mut distance = 0;

        // Distance from start to end of start line
        if let Some(&line_length) = line_lengths.get(start.line) {
            distance += line_length - start.column;
        }

        // Distance for complete lines in between
        for line_idx in (start.line + 1)..end.line {
            if let Some(&line_length) = line_lengths.get(line_idx) {
                distance += line_length + 1; // +1 for newline
            }
        }

        // Distance from start of end line to end position
        distance += end.column + 1; // +1 for newline before end line

        distance
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::origin()
    }
}

impl std::fmt::Display for Cursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.display_line(), self.display_column())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lines() -> Vec<String> {
        vec![
            "Hello world".to_string(),
            "This is a test".to_string(),
            "Short".to_string(),
            "".to_string(),
            "Last line".to_string(),
        ]
    }

    fn line_lengths() -> Vec<usize> {
        sample_lines().iter().map(|l| l.len()).collect()
    }

    #[test]
    fn test_cursor_creation() {
        let cursor = Cursor::new(5, 10);
        assert_eq!(cursor.line, 5);
        assert_eq!(cursor.column, 10);
        assert_eq!(cursor.desired_column, 10);

        let origin = Cursor::origin();
        assert_eq!(origin.line, 0);
        assert_eq!(origin.column, 0);
    }

    #[test]
    fn test_horizontal_movement() {
        let mut cursor = Cursor::new(0, 5);
        let lengths = line_lengths();

        cursor.move_left(&lengths);
        assert_eq!(cursor.column, 4);

        cursor.move_right(&lengths);
        assert_eq!(cursor.column, 5);

        // Test line wrapping
        cursor.move_to(0, 0);
        cursor.move_left(&lengths);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 0);

        cursor.move_to(0, 11); // End of first line
        cursor.move_right(&lengths);
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.column, 0);
    }

    #[test]
    fn test_vertical_movement() {
        let mut cursor = Cursor::new(1, 10);
        let lengths = line_lengths();

        cursor.move_up(&lengths);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 10);

        cursor.move_down(&lengths);
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.column, 10);

        // Test column clamping
        cursor.move_to(1, 10);
        cursor.move_down(&lengths); // Move to "Short" line
        assert_eq!(cursor.line, 2);
        assert_eq!(cursor.column, 5); // Clamped to line length
        assert_eq!(cursor.desired_column, 10); // But desired column preserved
    }

    #[test]
    fn test_line_start_end() {
        let mut cursor = Cursor::new(1, 5);
        let lengths = line_lengths();

        cursor.move_to_line_start();
        assert_eq!(cursor.column, 0);

        cursor.move_to_line_end(&lengths);
        assert_eq!(cursor.column, 14); // "This is a test".len()
    }

    #[test]
    fn test_buffer_start_end() {
        let mut cursor = Cursor::new(2, 3);
        let lengths = line_lengths();

        cursor.move_to_buffer_start();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 0);

        cursor.move_to_buffer_end(&lengths);
        assert_eq!(cursor.line, 4);
        assert_eq!(cursor.column, 9); // "Last line".len()
    }

    #[test]
    fn test_word_movement() {
        let lines = vec!["hello world test".to_string()];
        let mut cursor = Cursor::new(0, 12); // At 't' in "test"

        cursor.move_word_left(&lines);
        assert_eq!(cursor.column, 6); // Start of "world"

        cursor.move_word_left(&lines);
        assert_eq!(cursor.column, 0); // Start of "hello"

        cursor.move_word_right(&lines);
        assert_eq!(cursor.column, 6); // Start of "world"

        cursor.move_word_right(&lines);
        assert_eq!(cursor.column, 12); // Start of "test"
    }

    #[test]
    fn test_cursor_validation() {
        let lengths = line_lengths();

        let valid_cursor = Cursor::new(1, 5);
        assert!(valid_cursor.is_valid(&lengths));

        let invalid_line = Cursor::new(10, 0);
        assert!(!invalid_line.is_valid(&lengths));

        let invalid_column = Cursor::new(0, 50);
        assert!(!invalid_column.is_valid(&lengths));
    }

    #[test]
    fn test_cursor_clamping() {
        let lengths = line_lengths();
        let mut cursor = Cursor::new(10, 50);

        Cursor::clamp(&mut cursor, &lengths);
        assert_eq!(cursor.line, 4); // Last line
        assert_eq!(cursor.column, 9); // End of last line
    }

    #[test]
    fn test_cursor_comparison() {
        let cursor1 = Cursor::new(1, 5);
        let cursor2 = Cursor::new(1, 10);
        let cursor3 = Cursor::new(2, 0);

        assert!(cursor1.is_before(&cursor2));
        assert!(cursor2.is_after(&cursor1));
        assert!(cursor2.is_before(&cursor3));
    }

    #[test]
    fn test_distance_calculation() {
        let lengths = vec![10, 20, 15];
        let cursor1 = Cursor::new(0, 5);
        let cursor2 = Cursor::new(0, 8);

        assert_eq!(cursor1.distance_to(&cursor2, &lengths), 3);

        let cursor3 = Cursor::new(2, 5);
        // Distance includes: 5 chars to end of line 0, newline, all of line 1, newline, 5 chars in line 2
        let expected = (10 - 5) + 1 + 20 + 1 + 5;
        assert_eq!(cursor1.distance_to(&cursor3, &lengths), expected);
    }

    #[test]
    fn test_display_format() {
        let cursor = Cursor::new(4, 9);
        assert_eq!(format!("{}", cursor), "5:10"); // 1-indexed for display
    }
}
