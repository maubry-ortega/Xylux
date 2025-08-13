//! # File Explorer Component
//!
//! File and directory browser component for Xylux IDE.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{Component, Event, KeyCode, KeyEvent, MouseEvent, MouseEventType, Rect, Size};
use crate::core::Result;

/// File explorer component for browsing project files.
pub struct FileExplorerComponent {
    root_path: PathBuf,
    current_path: PathBuf,
    entries: Vec<FileEntry>,
    selected_index: Option<usize>,
    scroll_offset: usize,
    visible: bool,
    expanded_dirs: HashMap<PathBuf, bool>,
    show_hidden: bool,
    sort_by: SortBy,
    sort_ascending: bool,
    filter: Option<String>,
    theme: FileExplorerTheme,
}

/// File or directory entry.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_hidden: bool,
    pub size: Option<u64>,
    pub modified: Option<std::time::SystemTime>,
    pub expanded: bool,
    pub depth: usize,
}

/// Sorting options for file explorer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortBy {
    Name,
    Size,
    Modified,
    Type,
}

/// File explorer theme configuration.
#[derive(Debug, Clone)]
pub struct FileExplorerTheme {
    pub background: String,
    pub foreground: String,
    pub selected_background: String,
    pub selected_foreground: String,
    pub directory_color: String,
    pub file_color: String,
    pub hidden_color: String,
    pub symlink_color: String,
    pub executable_color: String,
    pub border_color: String,
}

impl Default for FileExplorerTheme {
    fn default() -> Self {
        Self {
            background: "#1e1e1e".to_string(),
            foreground: "#d4d4d4".to_string(),
            selected_background: "#094771".to_string(),
            selected_foreground: "#ffffff".to_string(),
            directory_color: "#569cd6".to_string(),
            file_color: "#d4d4d4".to_string(),
            hidden_color: "#6a6a6a".to_string(),
            symlink_color: "#4ec9b0".to_string(),
            executable_color: "#b5cea8".to_string(),
            border_color: "#3c3c3c".to_string(),
        }
    }
}

impl FileExplorerComponent {
    /// Create a new file explorer component.
    pub fn new<P: AsRef<Path>>(root_path: P) -> Result<Self> {
        let root_path = root_path.as_ref().to_path_buf();
        let current_path = root_path.clone();

        let mut explorer = Self {
            root_path: root_path.clone(),
            current_path,
            entries: Vec::new(),
            selected_index: None,
            scroll_offset: 0,
            visible: true,
            expanded_dirs: HashMap::new(),
            show_hidden: false,
            sort_by: SortBy::Name,
            sort_ascending: true,
            filter: None,
            theme: FileExplorerTheme::default(),
        };

        explorer.refresh()?;
        Ok(explorer)
    }

    /// Get the current root path.
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Set the root path and refresh.
    pub fn set_root_path<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.root_path = path.as_ref().to_path_buf();
        self.current_path = self.root_path.clone();
        self.selected_index = None;
        self.scroll_offset = 0;
        self.expanded_dirs.clear();
        self.refresh()
    }

    /// Get the currently selected entry.
    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.selected_index.and_then(|idx| self.entries.get(idx))
    }

    /// Get the currently selected path.
    pub fn selected_path(&self) -> Option<&Path> {
        self.selected_entry().map(|entry| entry.path.as_path())
    }

    /// Set whether to show hidden files.
    pub fn set_show_hidden(&mut self, show: bool) -> Result<()> {
        self.show_hidden = show;
        self.refresh()
    }

    /// Set the sort criteria.
    pub fn set_sort_by(&mut self, sort_by: SortBy, ascending: bool) -> Result<()> {
        self.sort_by = sort_by;
        self.sort_ascending = ascending;
        self.refresh()
    }

    /// Set a filter pattern.
    pub fn set_filter(&mut self, filter: Option<String>) -> Result<()> {
        self.filter = filter;
        self.refresh()
    }

    /// Set the theme.
    pub fn set_theme(&mut self, theme: FileExplorerTheme) {
        self.theme = theme;
    }

    /// Refresh the file listing.
    pub fn refresh(&mut self) -> Result<()> {
        self.entries.clear();
        let root_path = self.root_path.clone();
        self.load_entries(&root_path, 0)?;
        self.sort_entries();
        self.apply_filter();

        // Ensure selected index is valid
        if let Some(idx) = self.selected_index {
            if idx >= self.entries.len() {
                self.selected_index = if self.entries.is_empty() { None } else { Some(0) };
            }
        }

        Ok(())
    }

    /// Load entries from a directory.
    fn load_entries(&mut self, dir_path: &Path, depth: usize) -> Result<()> {
        let entries = fs::read_dir(dir_path).map_err(|e| {
            crate::core::XyluxError::io(e, &format!("Failed to read directory: {:?}", dir_path))
        })?;

        let mut dir_entries = Vec::new();

        for entry in entries {
            let entry = entry
                .map_err(|e| crate::core::XyluxError::io(e, "Failed to read directory entry"))?;

            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("<invalid>").to_string();

            let is_hidden = name.starts_with('.');
            if !self.show_hidden && is_hidden {
                continue;
            }

            let metadata = entry.metadata().ok();
            let is_dir = metadata.as_ref().map_or(false, |m| m.is_dir());
            let size =
                metadata.as_ref().and_then(|m| if m.is_file() { Some(m.len()) } else { None });
            let modified = metadata.and_then(|m| m.modified().ok());

            let file_entry = FileEntry {
                path: path.clone(),
                name,
                is_dir,
                is_hidden,
                size,
                modified,
                expanded: false,
                depth,
            };

            dir_entries.push(file_entry);
        }

        // Sort directory entries
        dir_entries.sort_by(|a, b| {
            // Always put directories first
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        // Add to main entries list
        for entry in dir_entries {
            let is_expanded = self.expanded_dirs.get(&entry.path).copied().unwrap_or(false);
            let mut file_entry = entry;
            file_entry.expanded = is_expanded;

            self.entries.push(file_entry.clone());

            // If directory is expanded, recursively load its contents
            if file_entry.is_dir && is_expanded {
                self.load_entries(&file_entry.path, depth + 1)?;
            }
        }

        Ok(())
    }

    /// Sort entries according to current criteria.
    fn sort_entries(&mut self) {
        let ascending = self.sort_ascending;
        let sort_by = &self.sort_by;

        self.entries.sort_by(|a, b| {
            // Keep hierarchy intact by depth
            if a.depth != b.depth {
                return a.depth.cmp(&b.depth);
            }

            let ordering = match sort_by {
                SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortBy::Size => match (a.size, b.size) {
                    (Some(a_size), Some(b_size)) => a_size.cmp(&b_size),
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (None, None) => std::cmp::Ordering::Equal,
                },
                SortBy::Modified => match (a.modified, b.modified) {
                    (Some(a_time), Some(b_time)) => a_time.cmp(&b_time),
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (None, None) => std::cmp::Ordering::Equal,
                },
                SortBy::Type => {
                    // Sort by file extension
                    let a_ext = a.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let b_ext = b.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    a_ext.cmp(b_ext)
                }
            };

            if ascending { ordering } else { ordering.reverse() }
        });
    }

    /// Apply current filter to entries.
    fn apply_filter(&mut self) {
        if let Some(ref filter) = self.filter {
            let filter_lower = filter.to_lowercase();
            self.entries.retain(|entry| {
                entry.name.to_lowercase().contains(&filter_lower)
                    || (entry.is_dir && entry.depth == 0) // Always show root directories
            });
        }
    }

    /// Move selection up.
    pub fn move_selection_up(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        match self.selected_index {
            Some(idx) if idx > 0 => {
                self.selected_index = Some(idx - 1);
            }
            Some(_) => {
                self.selected_index = Some(self.entries.len() - 1);
            }
            None => {
                self.selected_index = Some(0);
            }
        }

        self.ensure_selection_visible();
    }

    /// Move selection down.
    pub fn move_selection_down(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        match self.selected_index {
            Some(idx) if idx < self.entries.len() - 1 => {
                self.selected_index = Some(idx + 1);
            }
            Some(_) => {
                self.selected_index = Some(0);
            }
            None => {
                self.selected_index = Some(0);
            }
        }

        self.ensure_selection_visible();
    }

    /// Expand or collapse the selected directory.
    pub fn toggle_selected(&mut self) -> Result<()> {
        if let Some(idx) = self.selected_index {
            if let Some(entry) = self.entries.get(idx) {
                if entry.is_dir {
                    let path = entry.path.clone();
                    let was_expanded = self.expanded_dirs.get(&path).copied().unwrap_or(false);
                    self.expanded_dirs.insert(path, !was_expanded);
                    self.refresh()?;
                }
            }
        }
        Ok(())
    }

    /// Enter the selected directory.
    pub fn enter_selected(&mut self) -> Result<Option<PathBuf>> {
        if let Some(entry) = self.selected_entry() {
            if entry.is_dir {
                // Toggle expansion instead of changing directory
                self.toggle_selected()?;
                Ok(None)
            } else {
                // Return the selected file path
                Ok(Some(entry.path.clone()))
            }
        } else {
            Ok(None)
        }
    }

    /// Go up one directory level.
    pub fn go_up(&mut self) -> Result<()> {
        if let Some(parent) = self.current_path.parent() {
            if parent >= self.root_path {
                self.current_path = parent.to_path_buf();
                self.selected_index = None;
                self.scroll_offset = 0;
                self.refresh()?;
            }
        }
        Ok(())
    }

    /// Ensure the selected item is visible in the viewport.
    fn ensure_selection_visible(&mut self) {
        // This would be implemented based on the actual viewport size
        // For now, it's a placeholder
    }

    /// Get the icon for a file entry.
    pub fn get_entry_icon(&self, entry: &FileEntry) -> &'static str {
        if entry.is_dir {
            if entry.expanded { "📂" } else { "📁" }
        } else {
            match entry.path.extension().and_then(|e| e.to_str()) {
                Some("rs") => "🦀",
                Some("toml") => "⚙️",
                Some("md") => "📝",
                Some("txt") => "📄",
                Some("json") => "📋",
                Some("yaml" | "yml") => "📋",
                Some("png" | "jpg" | "jpeg" | "gif" | "bmp") => "🖼️",
                Some("mp3" | "wav" | "ogg") => "🎵",
                Some("mp4" | "avi" | "mkv") => "🎬",
                Some("zip" | "tar" | "gz" | "rar") => "📦",
                Some("exe" | "bin") => "⚡",
                _ => "📄",
            }
        }
    }

    /// Format file size for display.
    pub fn format_size(&self, size: Option<u64>) -> String {
        match size {
            Some(bytes) => {
                const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
                const THRESHOLD: f64 = 1024.0;

                if bytes == 0 {
                    return "0 B".to_string();
                }

                let size = bytes as f64;
                let unit_index = (size.log10() / THRESHOLD.log10()).floor() as usize;
                let unit_index = unit_index.min(UNITS.len() - 1);

                let value = size / THRESHOLD.powi(unit_index as i32);

                if unit_index == 0 {
                    format!("{} {}", bytes, UNITS[unit_index])
                } else {
                    format!("{:.1} {}", value, UNITS[unit_index])
                }
            }
            None => "-".to_string(),
        }
    }
}

impl Component for FileExplorerComponent {
    fn render(&self, _area: Rect) -> Result<()> {
        // Rendering would be implemented using the terminal backend
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
            Event::Mouse(mouse_event) => {
                self.handle_mouse_event(mouse_event)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn update(&mut self) -> Result<()> {
        // Check if any watched directories have changed
        // This would be implemented with file system watching
        Ok(())
    }

    fn min_size(&self) -> Size {
        Size::new(20, 10)
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl FileExplorerComponent {
    /// Handle keyboard events.
    fn handle_key_event(&mut self, key_event: &KeyEvent) -> Result<()> {
        match &key_event.code {
            KeyCode::Up => self.move_selection_up(),
            KeyCode::Down => self.move_selection_down(),
            KeyCode::Enter => {
                self.enter_selected()?;
            }
            KeyCode::Left => {
                if let Some(entry) = self.selected_entry() {
                    if entry.is_dir && entry.expanded {
                        self.toggle_selected()?;
                    } else {
                        self.go_up()?;
                    }
                }
            }
            KeyCode::Right => {
                if let Some(entry) = self.selected_entry() {
                    if entry.is_dir && !entry.expanded {
                        self.toggle_selected()?;
                    }
                }
            }
            KeyCode::Char(' ') => {
                self.toggle_selected()?;
            }
            KeyCode::Char('h') if key_event.modifiers.ctrl => {
                self.set_show_hidden(!self.show_hidden)?;
            }
            KeyCode::Char('r') if key_event.modifiers.ctrl => {
                self.refresh()?;
            }
            KeyCode::F(5) => {
                self.refresh()?;
            }
            _ => return Ok(()),
        }
        Ok(())
    }

    /// Handle mouse events.
    fn handle_mouse_event(&mut self, mouse_event: &MouseEvent) -> Result<()> {
        match mouse_event.event_type {
            MouseEventType::Down(super::MouseButton::Left) => {
                // Calculate which entry was clicked based on mouse position
                // This would need the actual rendering implementation
                // For now, it's a placeholder
            }
            MouseEventType::ScrollUp => {
                self.move_selection_up();
            }
            MouseEventType::ScrollDown => {
                self.move_selection_down();
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_explorer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let explorer = FileExplorerComponent::new(temp_dir.path()).unwrap();

        assert_eq!(explorer.root_path(), temp_dir.path());
        assert!(explorer.is_visible());
    }

    #[test]
    fn test_file_listing() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create some test files and directories
        fs::create_dir(temp_path.join("test_dir")).unwrap();
        fs::write(temp_path.join("test_file.txt"), "content").unwrap();
        fs::write(temp_path.join(".hidden_file"), "hidden").unwrap();

        let mut explorer = FileExplorerComponent::new(temp_path).unwrap();

        // Should have at least the directory and visible file
        assert!(!explorer.entries.is_empty());

        // Test showing hidden files
        explorer.set_show_hidden(true).unwrap();
        let hidden_count = explorer.entries.iter().filter(|e| e.is_hidden).count();
        assert!(hidden_count > 0);
    }

    #[test]
    fn test_selection_movement() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        fs::write(temp_path.join("file1.txt"), "content1").unwrap();
        fs::write(temp_path.join("file2.txt"), "content2").unwrap();

        let mut explorer = FileExplorerComponent::new(temp_path).unwrap();

        assert!(explorer.selected_index.is_none());

        explorer.move_selection_down();
        assert_eq!(explorer.selected_index, Some(0));

        explorer.move_selection_down();
        assert_eq!(explorer.selected_index, Some(1));

        explorer.move_selection_up();
        assert_eq!(explorer.selected_index, Some(0));
    }

    #[test]
    fn test_sorting() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        fs::write(temp_path.join("z_file.txt"), "content").unwrap();
        fs::write(temp_path.join("a_file.txt"), "content").unwrap();
        fs::create_dir(temp_path.join("b_dir")).unwrap();

        let mut explorer = FileExplorerComponent::new(temp_path).unwrap();

        // Test name sorting
        explorer.set_sort_by(SortBy::Name, true).unwrap();

        // Directories should come first, then files in alphabetical order
        assert!(explorer.entries[0].is_dir); // b_dir
        assert_eq!(explorer.entries[1].name, "a_file.txt");
        assert_eq!(explorer.entries[2].name, "z_file.txt");
    }

    #[test]
    fn test_file_size_formatting() {
        let temp_dir = TempDir::new().unwrap();
        let explorer = FileExplorerComponent::new(temp_dir.path()).unwrap();

        assert_eq!(explorer.format_size(None), "-");
        assert_eq!(explorer.format_size(Some(0)), "0 B");
        assert_eq!(explorer.format_size(Some(512)), "512 B");
        assert_eq!(explorer.format_size(Some(1024)), "1.0 KB");
        assert_eq!(explorer.format_size(Some(1536)), "1.5 KB");
        assert_eq!(explorer.format_size(Some(1024 * 1024)), "1.0 MB");
    }

    #[test]
    fn test_icons() {
        let temp_dir = TempDir::new().unwrap();
        let explorer = FileExplorerComponent::new(temp_dir.path()).unwrap();

        let dir_entry = FileEntry {
            path: PathBuf::from("/test"),
            name: "test".to_string(),
            is_dir: true,
            is_hidden: false,
            size: None,
            modified: None,
            expanded: false,
            depth: 0,
        };

        let rust_file = FileEntry {
            path: PathBuf::from("/test.rs"),
            name: "test.rs".to_string(),
            is_dir: false,
            is_hidden: false,
            size: Some(100),
            modified: None,
            expanded: false,
            depth: 0,
        };

        assert_eq!(explorer.get_entry_icon(&dir_entry), "📁");
        assert_eq!(explorer.get_entry_icon(&rust_file), "🦀");
    }
}
