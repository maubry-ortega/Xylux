//! # Editor Component
//!
//! Text editor component for Xylux IDE.

use std::collections::HashMap;

use super::{Component, Event, KeyCode, KeyEvent, Rect, Size};
use crate::core::Result;
use crate::editor::{Buffer, Cursor};
use crate::syntax::HighlightInfo;

/// Editor component for text editing.
pub struct EditorComponent {
    buffer: Buffer,
    cursor: Cursor,
    viewport: Viewport,
    visible: bool,
    scroll_offset: (usize, usize), // (row, col)
    selection: Option<Selection>,
    search: Option<SearchState>,
    line_numbers: bool,
    tab_width: usize,
    syntax_highlighting: bool,
    #[allow(dead_code)]
    highlights: HashMap<usize, Vec<HighlightInfo>>,
    theme: EditorTheme,
}

/// Viewport information for the editor.
#[derive(Debug, Clone)]
struct Viewport {
    rows: usize,
    cols: usize,
    line_number_width: usize,
}

/// Text selection in the editor.
#[derive(Debug, Clone)]
struct Selection {
    start: Cursor,
    end: Cursor,
    active: bool,
}

/// Search state for find functionality.
#[derive(Debug, Clone)]
struct SearchState {
    #[allow(dead_code)]
    query: String,
    matches: Vec<Cursor>,
    current_match: Option<usize>,
    #[allow(dead_code)]
    case_sensitive: bool,
    #[allow(dead_code)]
    whole_word: bool,
}

/// Editor theme configuration.
#[derive(Debug, Clone)]
pub struct EditorTheme {
    pub background: String,
    pub foreground: String,
    pub cursor: String,
    pub selection: String,
    pub line_number: String,
    pub line_number_background: String,
    pub current_line: String,
    pub search_highlight: String,
    pub keyword: String,
    pub string: String,
    pub comment: String,
    pub number: String,
    pub operator: String,
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self {
            background: "#1e1e1e".to_string(),
            foreground: "#d4d4d4".to_string(),
            cursor: "#ffffff".to_string(),
            selection: "#264f78".to_string(),
            line_number: "#858585".to_string(),
            line_number_background: "#1e1e1e".to_string(),
            current_line: "#2a2d2e".to_string(),
            search_highlight: "#613214".to_string(),
            keyword: "#569cd6".to_string(),
            string: "#ce9178".to_string(),
            comment: "#6a9955".to_string(),
            number: "#b5cea8".to_string(),
            operator: "#d4d4d4".to_string(),
        }
    }
}

impl EditorComponent {
    /// Create a new editor component.
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(String::new(), None),
            cursor: Cursor::new(0, 0),
            viewport: Viewport { rows: 0, cols: 0, line_number_width: 4 },
            visible: true,
            scroll_offset: (0, 0),
            selection: None,
            search: None,
            line_numbers: true,
            tab_width: 4,
            syntax_highlighting: true,
            highlights: HashMap::new(),
            theme: EditorTheme::default(),
        }
    }

    /// Create editor with buffer.
    pub fn with_buffer(buffer: Buffer) -> Self {
        let mut editor = Self::new();
        editor.buffer = buffer;
        editor
    }

    /// Get the current buffer.
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get mutable buffer reference.
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// Set cursor position.
    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
        self.ensure_cursor_visible();
    }

    /// Enable or disable line numbers.
    pub fn set_line_numbers(&mut self, enabled: bool) {
        self.line_numbers = enabled;
        self.update_viewport();
    }

    /// Set tab width.
    pub fn set_tab_width(&mut self, width: usize) {
        self.tab_width = width.max(1);
    }

    /// Enable or disable syntax highlighting.
    pub fn set_syntax_highlighting(&mut self, enabled: bool) {
        self.syntax_highlighting = enabled;
    }

    /// Set editor theme.
    pub fn set_theme(&mut self, theme: EditorTheme) {
        self.theme = theme;
    }

    /// Insert text at cursor position.
    pub fn insert_text(&mut self, text: &str) -> Result<()> {
        self.buffer.insert_text(self.cursor.line, self.cursor.column, text)?;

        // Update cursor position
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() > 1 {
            self.cursor.line += lines.len() - 1;
            self.cursor.column = lines.last().unwrap_or(&"").len();
        } else {
            self.cursor.column += text.len();
        }

        self.ensure_cursor_visible();
        Ok(())
    }

    /// Delete character at cursor position.
    pub fn delete_char(&mut self) -> Result<()> {
        if self.cursor.column > 0 {
            self.buffer.delete_char(self.cursor.line, self.cursor.column - 1)?;
            self.cursor.column -= 1;
        } else if self.cursor.line > 0 {
            // Join with previous line
            let prev_line_len = self.buffer.line_length(self.cursor.line - 1);
            self.buffer.join_lines(self.cursor.line - 1)?;
            self.cursor.line -= 1;
            self.cursor.column = prev_line_len;
        }

        self.ensure_cursor_visible();
        Ok(())
    }

    /// Delete character after cursor position.
    pub fn delete_char_forward(&mut self) -> Result<()> {
        let line_len = self.buffer.line_length(self.cursor.line);

        if self.cursor.column < line_len {
            self.buffer.delete_char(self.cursor.line, self.cursor.column)?;
        } else if self.cursor.line < self.buffer.line_count() - 1 {
            // Join with next line
            self.buffer.join_lines(self.cursor.line)?;
        }

        Ok(())
    }

    /// Insert new line at cursor position.
    pub fn insert_newline(&mut self) -> Result<()> {
        self.buffer.insert_newline(self.cursor.line, self.cursor.column)?;
        self.cursor.line += 1;
        self.cursor.column = 0;
        self.ensure_cursor_visible();
        Ok(())
    }

    /// Move cursor left.
    pub fn move_cursor_left(&mut self) {
        if self.cursor.column > 0 {
            self.cursor.column -= 1;
        } else if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.column = self.buffer.line_length(self.cursor.line);
        }
        self.ensure_cursor_visible();
    }

    /// Move cursor right.
    pub fn move_cursor_right(&mut self) {
        let line_len = self.buffer.line_length(self.cursor.line);
        if self.cursor.column < line_len {
            self.cursor.column += 1;
        } else if self.cursor.line < self.buffer.line_count() - 1 {
            self.cursor.line += 1;
            self.cursor.column = 0;
        }
        self.ensure_cursor_visible();
    }

    /// Move cursor up.
    pub fn move_cursor_up(&mut self) {
        if self.cursor.line > 0 {
            self.cursor.line -= 1;
            let line_len = self.buffer.line_length(self.cursor.line);
            self.cursor.column = self.cursor.column.min(line_len);
        }
        self.ensure_cursor_visible();
    }

    /// Move cursor down.
    pub fn move_cursor_down(&mut self) {
        if self.cursor.line < self.buffer.line_count() - 1 {
            self.cursor.line += 1;
            let line_len = self.buffer.line_length(self.cursor.line);
            self.cursor.column = self.cursor.column.min(line_len);
        }
        self.ensure_cursor_visible();
    }

    /// Move cursor to beginning of line.
    pub fn move_cursor_home(&mut self) {
        self.cursor.column = 0;
        self.ensure_cursor_visible();
    }

    /// Move cursor to end of line.
    pub fn move_cursor_end(&mut self) {
        self.cursor.column = self.buffer.line_length(self.cursor.line);
        self.ensure_cursor_visible();
    }

    /// Move cursor to end of buffer.
    pub fn move_cursor_to_end(&mut self) {
        self.cursor.line = self.buffer.line_count() - 1;
        self.cursor.column = self.buffer.line_length(self.cursor.line);
        self.ensure_cursor_visible();
    }

    /// Move cursor to beginning of buffer.
    pub fn move_cursor_to_start(&mut self) {
        self.cursor.line = 0;
        self.cursor.column = 0;
        self.ensure_cursor_visible();
    }

    /// Move cursor to end of buffer.

    /// Page up.
    pub fn page_up(&mut self) {
        let page_size = self.viewport.rows.saturating_sub(1);
        self.cursor.line = self.cursor.line.saturating_sub(page_size);
        let line_len = self.buffer.line_length(self.cursor.line);
        self.cursor.column = self.cursor.column.min(line_len);
        self.ensure_cursor_visible();
    }

    /// Page down.
    pub fn page_down(&mut self) {
        let page_size = self.viewport.rows.saturating_sub(1);
        let max_row = self.buffer.line_count().saturating_sub(1);
        self.cursor.line = (self.cursor.line + page_size).min(max_row);
        let line_len = self.buffer.line_length(self.cursor.line);
        self.cursor.column = self.cursor.column.min(line_len);
        self.ensure_cursor_visible();
    }

    /// Start text selection.
    pub fn start_selection(&mut self) {
        self.selection =
            Some(Selection { start: self.cursor.clone(), end: self.cursor.clone(), active: true });
    }

    /// Update selection end point.
    pub fn update_selection(&mut self) {
        if let Some(ref mut selection) = self.selection {
            if selection.active {
                selection.end = self.cursor.clone();
            }
        }
    }

    /// Clear selection.
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Get selected text.
    pub fn get_selected_text(&self) -> Option<String> {
        self.selection.as_ref().and_then(|sel| {
            self.buffer
                .get_text_range(sel.start.line, sel.start.column, sel.end.line, sel.end.column)
                .ok()
        })
    }

    /// Delete selected text.
    pub fn delete_selection(&mut self) -> Result<()> {
        if let Some(selection) = &self.selection {
            let (start, end) = if selection.start <= selection.end {
                (selection.start.clone(), selection.end.clone())
            } else {
                (selection.end.clone(), selection.start.clone())
            };

            self.buffer.delete_range(start.line, start.column, end.line, end.column)?;
            self.cursor = start;
            self.clear_selection();
        }
        Ok(())
    }

    /// Start search.
    pub fn start_search(&mut self, query: String) {
        let matches = self
            .buffer
            .find_all(&query)
            .into_iter()
            .map(|(line, column)| Cursor::new(line, column))
            .collect();

        self.search = Some(SearchState {
            query,
            matches,
            current_match: None,
            case_sensitive: false,
            whole_word: false,
        });

        self.find_next();
    }

    /// Find next search match.
    pub fn find_next(&mut self) {
        if let Some(ref mut search) = self.search {
            if search.matches.is_empty() {
                return;
            }

            let current_idx = search.current_match.unwrap_or(0);
            let next_idx = (current_idx + 1) % search.matches.len();

            search.current_match = Some(next_idx);
            self.cursor = search.matches[next_idx].clone();
            self.ensure_cursor_visible();
        }
    }

    /// Find previous search match.
    pub fn find_previous(&mut self) {
        if let Some(ref mut search) = self.search {
            if search.matches.is_empty() {
                return;
            }

            let current_idx = search.current_match.unwrap_or(0);
            let prev_idx =
                if current_idx == 0 { search.matches.len() - 1 } else { current_idx - 1 };

            search.current_match = Some(prev_idx);
            self.cursor = search.matches[prev_idx].clone();
            self.ensure_cursor_visible();
        }
    }

    /// Clear search.
    pub fn clear_search(&mut self) {
        self.search = None;
    }

    /// Ensure cursor is visible in the viewport.
    fn ensure_cursor_visible(&mut self) {
        // Vertical scrolling
        if self.cursor.line < self.scroll_offset.0 {
            self.scroll_offset.0 = self.cursor.line;
        } else if self.cursor.line >= self.scroll_offset.0 + self.viewport.rows {
            self.scroll_offset.0 = self.cursor.line.saturating_sub(self.viewport.rows - 1);
        }

        // Horizontal scrolling
        let visible_cols = self.viewport.cols.saturating_sub(self.viewport.line_number_width);
        if self.cursor.column < self.scroll_offset.1 {
            self.scroll_offset.1 = self.cursor.column;
        } else if self.cursor.column >= self.scroll_offset.1 + visible_cols {
            self.scroll_offset.1 = self.cursor.column.saturating_sub(visible_cols - 1);
        }
    }

    /// Update viewport dimensions.
    fn update_viewport(&mut self) {
        if self.line_numbers {
            let line_count = self.buffer.line_count();
            self.viewport.line_number_width = format!("{}", line_count).len().max(3) + 1;
        } else {
            self.viewport.line_number_width = 0;
        }
    }

    /// Handle tab key.
    fn handle_tab(&mut self, shift: bool) -> Result<()> {
        if shift {
            // Shift+Tab: unindent
            self.unindent_line()
        } else {
            // Tab: indent or insert tab
            if self.selection.is_some() {
                self.indent_selection()
            } else {
                self.insert_text(&" ".repeat(self.tab_width))
            }
        }
    }

    /// Indent current line or selection.
    fn indent_selection(&mut self) -> Result<()> {
        // Implementation for indenting selected lines
        Ok(())
    }

    /// Unindent current line.
    fn unindent_line(&mut self) -> Result<()> {
        // Implementation for unindenting current line
        Ok(())
    }
}

impl Default for EditorComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for EditorComponent {
    fn render(&self, area: Rect) -> Result<()> {
        // Update viewport size
        let mut viewport = self.viewport.clone();
        viewport.rows = area.height as usize;
        viewport.cols = area.width as usize;

        // Render would be implemented using the terminal backend
        // This is a placeholder for the actual rendering logic
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> Result<bool> {
        if !self.visible {
            return Ok(false);
        }

        match event {
            Event::Key(key_event) => {
                self.handle_key_event(key_event)?;
                Ok(true)
            }
            Event::Resize(width, height) => {
                self.viewport.cols = *width as usize;
                self.viewport.rows = *height as usize;
                self.ensure_cursor_visible();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> Result<bool> {
        match &key_event.code {
            KeyCode::Char(c) => {
                if key_event.modifiers.ctrl {
                    match *c {
                        'a' => self.move_cursor_home(),
                        'e' => self.move_cursor_end(),
                        'f' => self.move_cursor_right(),
                        'b' => self.move_cursor_left(),
                        'n' => self.move_cursor_down(),
                        'p' => self.move_cursor_up(),
                        'd' => self.delete_char_forward()?,
                        'k' => {
                            // Kill line (delete from cursor to end of line)
                            let line_len = self.buffer.line_length(self.cursor.line);
                            if self.cursor.column < line_len {
                                self.buffer.delete_range(
                                    self.cursor.line,
                                    self.cursor.column,
                                    self.cursor.line,
                                    line_len,
                                )?;
                            }
                        }
                        _ => return Ok(false),
                    }
                } else {
                    self.insert_text(&c.to_string())?;
                }
            }
            KeyCode::Enter => self.insert_newline()?,
            KeyCode::Backspace => self.delete_char()?,
            KeyCode::Delete => self.delete_char_forward()?,
            KeyCode::Tab => self.handle_tab(key_event.modifiers.shift)?,
            KeyCode::Left => {
                if key_event.modifiers.shift {
                    if self.selection.is_none() {
                        self.start_selection();
                    }
                }
                self.move_cursor_left();
                if key_event.modifiers.shift {
                    self.update_selection();
                } else {
                    self.clear_selection();
                }
            }
            KeyCode::Right => {
                if key_event.modifiers.shift {
                    if self.selection.is_none() {
                        self.start_selection();
                    }
                }
                self.move_cursor_right();
                if key_event.modifiers.shift {
                    self.update_selection();
                } else {
                    self.clear_selection();
                }
            }
            KeyCode::Up => {
                if key_event.modifiers.shift {
                    if self.selection.is_none() {
                        self.start_selection();
                    }
                }
                self.move_cursor_up();
                if key_event.modifiers.shift {
                    self.update_selection();
                } else {
                    self.clear_selection();
                }
            }
            KeyCode::Down => {
                if key_event.modifiers.shift {
                    if self.selection.is_none() {
                        self.start_selection();
                    }
                }
                self.move_cursor_down();
                if key_event.modifiers.shift {
                    self.update_selection();
                } else {
                    self.clear_selection();
                }
            }
            KeyCode::Home => {
                if key_event.modifiers.shift {
                    if self.selection.is_none() {
                        self.start_selection();
                    }
                }
                if key_event.modifiers.ctrl {
                    self.move_cursor_to_start();
                } else {
                    self.move_cursor_home();
                }
                if key_event.modifiers.shift {
                    self.update_selection();
                } else {
                    self.clear_selection();
                }
            }
            KeyCode::End => {
                if key_event.modifiers.shift {
                    if self.selection.is_none() {
                        self.start_selection();
                    }
                }
                if key_event.modifiers.ctrl {
                    self.move_cursor_to_end();
                } else {
                    self.move_cursor_end();
                }
                if key_event.modifiers.shift {
                    self.update_selection();
                } else {
                    self.clear_selection();
                }
            }
            KeyCode::PageUp => {
                if key_event.modifiers.shift {
                    if self.selection.is_none() {
                        self.start_selection();
                    }
                }
                self.page_up();
                if key_event.modifiers.shift {
                    self.update_selection();
                } else {
                    self.clear_selection();
                }
            }
            KeyCode::PageDown => {
                if key_event.modifiers.shift {
                    if self.selection.is_none() {
                        self.start_selection();
                    }
                }
                self.page_down();
                if key_event.modifiers.shift {
                    self.update_selection();
                } else {
                    self.clear_selection();
                }
            }
            KeyCode::Escape => {
                self.clear_selection();
                self.clear_search();
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn update(&mut self) -> Result<()> {
        self.update_viewport();
        Ok(())
    }

    fn min_size(&self) -> Size {
        Size::new(40, 10) // Minimum reasonable size for an editor
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

// Extension trait for KeyEvent to add handle_key_event method

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = EditorComponent::new();
        assert!(editor.is_visible());
        assert_eq!(editor.cursor().line, 1);
        assert_eq!(editor.cursor().column, 0);
    }

    #[test]
    fn test_text_insertion() {
        let mut editor = EditorComponent::new();
        editor.insert_text("Hello, World!").unwrap();

        assert_eq!(editor.cursor().column, 13);
        // Buffer content testing would require more buffer implementation
    }

    #[test]
    fn test_cursor_movement() {
        let mut editor = EditorComponent::new();
        editor.insert_text("Line 1\nLine 2\nLine 3").unwrap();

        // Test basic movement
        editor.move_cursor_left();
        assert_eq!(editor.cursor().column, 5);

        editor.move_cursor_up();
        assert_eq!(editor.cursor().line, 1);

        editor.move_cursor_home();
        assert_eq!(editor.cursor().column, 0);

        editor.move_cursor_end();
        assert_eq!(editor.cursor().column, 6); // "Line 2".len()
    }

    #[test]
    fn test_selection() {
        let mut editor = EditorComponent::new();
        editor.insert_text("Hello, World!").unwrap();

        editor.move_cursor_home();
        editor.start_selection();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.update_selection();

        assert!(editor.selection.is_some());
        // More detailed selection testing would require buffer implementation
    }

    #[test]
    fn test_theme() {
        let mut editor = EditorComponent::new();
        let custom_theme = EditorTheme {
            background: "#000000".to_string(),
            foreground: "#ffffff".to_string(),
            ..EditorTheme::default()
        };

        editor.set_theme(custom_theme.clone());
        assert_eq!(editor.theme.background, "#000000");
        assert_eq!(editor.theme.foreground, "#ffffff");
    }

    #[test]
    fn test_tab_width() {
        let mut editor = EditorComponent::new();
        editor.set_tab_width(8);
        assert_eq!(editor.tab_width, 8);

        editor.set_tab_width(0); // Should be clamped to 1
        assert_eq!(editor.tab_width, 1);
    }
}
