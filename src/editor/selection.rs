//! # Selection Module
//!
//! Text selection management for the editor.

use std::cmp;

use super::cursor::Cursor;

/// Represents a text selection in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// Start position of the selection.
    pub start: Cursor,
    /// End position of the selection.
    pub end: Cursor,
    /// Selection mode.
    pub mode: SelectionMode,
}

/// Different modes of text selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Character-based selection.
    Character,
    /// Word-based selection.
    Word,
    /// Line-based selection.
    Line,
    /// Block/column selection.
    Block,
}

impl Selection {
    /// Create a new selection from start to end positions.
    pub fn new(start: Cursor, end: Cursor) -> Self {
        Self { start, end, mode: SelectionMode::Character }
    }

    /// Create a new selection with a specific mode.
    pub fn with_mode(start: Cursor, end: Cursor, mode: SelectionMode) -> Self {
        Self { start, end, mode }
    }

    /// Create a selection from a single cursor position (empty selection).
    pub fn from_cursor(cursor: Cursor) -> Self {
        Self::new(cursor, cursor)
    }

    /// Check if the selection is empty (start equals end).
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Get the normalized selection (start before end).
    pub fn normalized(&self) -> Selection {
        if self.start.is_before(&self.end) {
            self.clone()
        } else {
            Selection { start: self.end, end: self.start, mode: self.mode }
        }
    }

    /// Get the actual start position (always before end).
    pub fn actual_start(&self) -> &Cursor {
        if self.start.is_before(&self.end) { &self.start } else { &self.end }
    }

    /// Get the actual end position (always after start).
    pub fn actual_end(&self) -> &Cursor {
        if self.start.is_before(&self.end) { &self.end } else { &self.start }
    }

    /// Extend the selection to include a new cursor position.
    pub fn extend_to(&mut self, cursor: Cursor) {
        self.end = cursor;
    }

    /// Move the selection by updating the end position.
    pub fn move_end(&mut self, cursor: Cursor) {
        self.end = cursor;
    }

    /// Set the start position of the selection.
    pub fn set_start(&mut self, cursor: Cursor) {
        self.start = cursor;
    }

    /// Set the end position of the selection.
    pub fn set_end(&mut self, cursor: Cursor) {
        self.end = cursor;
    }

    /// Check if a cursor position is within this selection.
    pub fn contains(&self, cursor: &Cursor) -> bool {
        let norm = self.normalized();
        match self.mode {
            SelectionMode::Character | SelectionMode::Word => {
                cursor.is_after(&norm.start) && cursor.is_before(&norm.end)
                    || *cursor == norm.start
                    || *cursor == norm.end
            }
            SelectionMode::Line => cursor.line >= norm.start.line && cursor.line <= norm.end.line,
            SelectionMode::Block => {
                let min_col = cmp::min(norm.start.column, norm.end.column);
                let max_col = cmp::max(norm.start.column, norm.end.column);
                cursor.line >= norm.start.line
                    && cursor.line <= norm.end.line
                    && cursor.column >= min_col
                    && cursor.column <= max_col
            }
        }
    }

    /// Check if a line is within this selection.
    pub fn contains_line(&self, line: usize) -> bool {
        let norm = self.normalized();
        line >= norm.start.line && line <= norm.end.line
    }

    /// Get the range of lines covered by this selection.
    pub fn line_range(&self) -> (usize, usize) {
        let norm = self.normalized();
        (norm.start.line, norm.end.line)
    }

    /// Get the column range for a specific line in block selection mode.
    pub fn column_range_for_line(&self, line: usize) -> Option<(usize, usize)> {
        if !self.contains_line(line) {
            return None;
        }

        match self.mode {
            SelectionMode::Block => {
                let norm = self.normalized();
                let min_col = cmp::min(norm.start.column, norm.end.column);
                let max_col = cmp::max(norm.start.column, norm.end.column);
                Some((min_col, max_col))
            }
            SelectionMode::Character | SelectionMode::Word => {
                let norm = self.normalized();
                if line == norm.start.line && line == norm.end.line {
                    // Selection within a single line
                    Some((norm.start.column, norm.end.column))
                } else if line == norm.start.line {
                    // First line of multi-line selection
                    Some((norm.start.column, usize::MAX))
                } else if line == norm.end.line {
                    // Last line of multi-line selection
                    Some((0, norm.end.column))
                } else {
                    // Middle line of multi-line selection
                    Some((0, usize::MAX))
                }
            }
            SelectionMode::Line => {
                // Entire line is selected
                Some((0, usize::MAX))
            }
        }
    }

    /// Expand selection to word boundaries.
    pub fn expand_to_word_boundaries(&mut self, lines: &[String]) {
        self.mode = SelectionMode::Word;
        let norm = self.normalized();

        // Expand start to word boundary
        if let Some(line) = lines.get(norm.start.line) {
            let mut start_col = norm.start.column;
            let chars: Vec<char> = line.chars().collect();

            // Move to start of word
            while start_col > 0 && is_word_char(chars.get(start_col.saturating_sub(1))) {
                start_col -= 1;
            }

            self.start.column = start_col;
        }

        // Expand end to word boundary
        if let Some(line) = lines.get(norm.end.line) {
            let mut end_col = norm.end.column;
            let chars: Vec<char> = line.chars().collect();

            // Move to end of word
            while end_col < chars.len() && is_word_char(chars.get(end_col)) {
                end_col += 1;
            }

            self.end.column = end_col;
        }
    }

    /// Expand selection to line boundaries.
    pub fn expand_to_line_boundaries(&mut self) {
        self.mode = SelectionMode::Line;
        let norm = self.normalized();

        self.start.column = 0;
        self.end.column = 0;
        if norm.end.line + 1 < usize::MAX {
            self.end.line = norm.end.line + 1;
        }
    }

    /// Get the selected text from the given lines.
    pub fn get_text(&self, lines: &[String]) -> String {
        if self.is_empty() {
            return String::new();
        }

        let norm = self.normalized();
        let mut result = Vec::new();

        match self.mode {
            SelectionMode::Character | SelectionMode::Word => {
                if norm.start.line == norm.end.line {
                    // Single line selection
                    if let Some(line) = lines.get(norm.start.line) {
                        let start_idx = cmp::min(norm.start.column, line.len());
                        let end_idx = cmp::min(norm.end.column, line.len());
                        result.push(line[start_idx..end_idx].to_string());
                    }
                } else {
                    // Multi-line selection
                    for line_idx in norm.start.line..=norm.end.line {
                        if let Some(line) = lines.get(line_idx) {
                            if line_idx == norm.start.line {
                                // First line
                                let start_idx = cmp::min(norm.start.column, line.len());
                                result.push(line[start_idx..].to_string());
                            } else if line_idx == norm.end.line {
                                // Last line
                                let end_idx = cmp::min(norm.end.column, line.len());
                                result.push(line[..end_idx].to_string());
                            } else {
                                // Middle lines
                                result.push(line.clone());
                            }
                        }
                    }
                }
            }
            SelectionMode::Line => {
                for line_idx in norm.start.line..=norm.end.line {
                    if let Some(line) = lines.get(line_idx) {
                        result.push(line.clone());
                    }
                }
            }
            SelectionMode::Block => {
                let min_col = cmp::min(norm.start.column, norm.end.column);
                let max_col = cmp::max(norm.start.column, norm.end.column);

                for line_idx in norm.start.line..=norm.end.line {
                    if let Some(line) = lines.get(line_idx) {
                        let start_idx = cmp::min(min_col, line.len());
                        let end_idx = cmp::min(max_col, line.len());
                        if start_idx <= end_idx {
                            result.push(line[start_idx..end_idx].to_string());
                        } else {
                            result.push(String::new());
                        }
                    }
                }
            }
        }

        result.join("\n")
    }

    /// Calculate the size of the selection in characters.
    pub fn size(&self, line_lengths: &[usize]) -> usize {
        if self.is_empty() {
            return 0;
        }

        let norm = self.normalized();
        match self.mode {
            SelectionMode::Character | SelectionMode::Word => {
                norm.start.distance_to(&norm.end, line_lengths)
            }
            SelectionMode::Line => {
                let mut size = 0;
                for line_idx in norm.start.line..=norm.end.line {
                    if let Some(&line_length) = line_lengths.get(line_idx) {
                        size += line_length + 1; // +1 for newline
                    }
                }
                size.saturating_sub(1) // Remove last newline
            }
            SelectionMode::Block => {
                let col_width =
                    (norm.end.column as isize - norm.start.column as isize).abs() as usize;
                let line_count = norm.end.line - norm.start.line + 1;
                col_width * line_count
            }
        }
    }

    /// Check if this selection overlaps with another selection.
    pub fn overlaps_with(&self, other: &Selection) -> bool {
        let norm_self = self.normalized();
        let norm_other = other.normalized();

        !(norm_self.end.is_before(&norm_other.start) || norm_other.end.is_before(&norm_self.start))
    }

    /// Merge this selection with another selection if they overlap.
    pub fn merge_with(&self, other: &Selection) -> Option<Selection> {
        if !self.overlaps_with(other) {
            return None;
        }

        let norm_self = self.normalized();
        let norm_other = other.normalized();

        let start = if norm_self.start.is_before(&norm_other.start) {
            norm_self.start
        } else {
            norm_other.start
        };

        let end =
            if norm_self.end.is_after(&norm_other.end) { norm_self.end } else { norm_other.end };

        Some(Selection::new(start, end))
    }

    /// Convert selection to a different mode.
    pub fn convert_to_mode(&mut self, mode: SelectionMode, lines: &[String]) {
        match mode {
            SelectionMode::Word => self.expand_to_word_boundaries(lines),
            SelectionMode::Line => self.expand_to_line_boundaries(),
            SelectionMode::Block => self.mode = SelectionMode::Block,
            SelectionMode::Character => self.mode = SelectionMode::Character,
        }
    }
}

/// Check if a character is part of a word.
fn is_word_char(ch: Option<&char>) -> bool {
    match ch {
        Some(c) => c.is_alphanumeric() || *c == '_',
        None => false,
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::from_cursor(Cursor::default())
    }
}

impl std::fmt::Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let norm = self.normalized();
        write!(f, "{} -> {} ({:?})", norm.start, norm.end, self.mode)
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

    #[test]
    fn test_selection_creation() {
        let start = Cursor::new(0, 0);
        let end = Cursor::new(0, 5);
        let selection = Selection::new(start, end);

        assert_eq!(selection.start, start);
        assert_eq!(selection.end, end);
        assert_eq!(selection.mode, SelectionMode::Character);
        assert!(!selection.is_empty());
    }

    #[test]
    fn test_empty_selection() {
        let cursor = Cursor::new(1, 5);
        let selection = Selection::from_cursor(cursor);

        assert!(selection.is_empty());
        assert_eq!(selection.start, cursor);
        assert_eq!(selection.end, cursor);
    }

    #[test]
    fn test_selection_normalization() {
        let start = Cursor::new(1, 10);
        let end = Cursor::new(0, 5);
        let selection = Selection::new(start, end);

        let normalized = selection.normalized();
        assert_eq!(normalized.start, end);
        assert_eq!(normalized.end, start);
    }

    #[test]
    fn test_selection_contains() {
        let start = Cursor::new(0, 5);
        let end = Cursor::new(1, 10);
        let selection = Selection::new(start, end);

        let inside = Cursor::new(0, 8);
        let outside = Cursor::new(2, 0);

        assert!(selection.contains(&inside));
        assert!(!selection.contains(&outside));
    }

    #[test]
    fn test_line_range() {
        let start = Cursor::new(1, 5);
        let end = Cursor::new(3, 2);
        let selection = Selection::new(start, end);

        let (start_line, end_line) = selection.line_range();
        assert_eq!(start_line, 1);
        assert_eq!(end_line, 3);

        assert!(selection.contains_line(1));
        assert!(selection.contains_line(2));
        assert!(selection.contains_line(3));
        assert!(!selection.contains_line(0));
        assert!(!selection.contains_line(4));
    }

    #[test]
    fn test_column_range() {
        let start = Cursor::new(0, 3);
        let end = Cursor::new(0, 8);
        let selection = Selection::new(start, end);

        let range = selection.column_range_for_line(0);
        assert_eq!(range, Some((3, 8)));

        let no_range = selection.column_range_for_line(1);
        assert_eq!(no_range, None);
    }

    #[test]
    fn test_block_selection() {
        let start = Cursor::new(0, 2);
        let end = Cursor::new(2, 6);
        let mut selection = Selection::new(start, end);
        selection.mode = SelectionMode::Block;

        let range = selection.column_range_for_line(1);
        assert_eq!(range, Some((2, 6)));

        assert!(selection.contains(&Cursor::new(1, 4)));
        assert!(!selection.contains(&Cursor::new(1, 1)));
        assert!(!selection.contains(&Cursor::new(1, 8)));
    }

    #[test]
    fn test_word_expansion() {
        let lines = vec!["hello world test".to_string()];
        let start = Cursor::new(0, 7); // In "world"
        let end = Cursor::new(0, 10);
        let mut selection = Selection::new(start, end);

        selection.expand_to_word_boundaries(&lines);

        assert_eq!(selection.mode, SelectionMode::Word);
        assert_eq!(selection.start.column, 6); // Start of "world"
        assert_eq!(selection.end.column, 11); // End of "world"
    }

    #[test]
    fn test_line_expansion() {
        let start = Cursor::new(1, 5);
        let end = Cursor::new(1, 10);
        let mut selection = Selection::new(start, end);

        selection.expand_to_line_boundaries();

        assert_eq!(selection.mode, SelectionMode::Line);
        assert_eq!(selection.start.column, 0);
        assert_eq!(selection.end.column, 0);
        assert_eq!(selection.end.line, 2);
    }

    #[test]
    fn test_get_text() {
        let lines = sample_lines();

        // Single line selection
        let start = Cursor::new(0, 0);
        let end = Cursor::new(0, 5);
        let selection = Selection::new(start, end);
        assert_eq!(selection.get_text(&lines), "Hello");

        // Multi-line selection
        let start = Cursor::new(0, 6);
        let end = Cursor::new(1, 4);
        let selection = Selection::new(start, end);
        assert_eq!(selection.get_text(&lines), "world\nThis");
    }

    #[test]
    fn test_selection_overlap() {
        let sel1 = Selection::new(Cursor::new(0, 0), Cursor::new(0, 10));
        let sel2 = Selection::new(Cursor::new(0, 5), Cursor::new(0, 15));
        let sel3 = Selection::new(Cursor::new(1, 0), Cursor::new(1, 5));

        assert!(sel1.overlaps_with(&sel2));
        assert!(!sel1.overlaps_with(&sel3));
    }

    #[test]
    fn test_selection_merge() {
        let sel1 = Selection::new(Cursor::new(0, 0), Cursor::new(0, 10));
        let sel2 = Selection::new(Cursor::new(0, 5), Cursor::new(0, 15));

        let merged = sel1.merge_with(&sel2).unwrap();
        assert_eq!(merged.start, Cursor::new(0, 0));
        assert_eq!(merged.end, Cursor::new(0, 15));
    }

    #[test]
    fn test_selection_size() {
        let line_lengths = vec![11, 14, 5, 0, 9]; // Lengths of sample lines

        let sel = Selection::new(Cursor::new(0, 0), Cursor::new(0, 5));
        assert_eq!(sel.size(&line_lengths), 5);

        let multi_line_sel = Selection::new(Cursor::new(0, 0), Cursor::new(1, 4));
        // Should be: 11 chars + newline + 4 chars = 16
        assert_eq!(multi_line_sel.size(&line_lengths), 16);
    }
}
