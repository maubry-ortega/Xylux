//! # Editor Module
//!
//! Text editing functionality for Xylux IDE.

pub mod buffer;
pub mod commands;
pub mod cursor;
pub mod selection;
pub mod simple;

pub use buffer::Buffer;
pub use commands::Command;
pub use cursor::Cursor;
pub use selection::Selection;
pub use simple::SimpleEditor;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::core::{Config, Event, EventBus, EventMessage, EventPriority, Result, XyluxError};

use commands::CommandType;

/// Main editor component that manages text buffers and editing operations.
pub struct Editor {
    /// IDE configuration.
    config: Arc<RwLock<Config>>,
    /// Event bus for communication.
    event_bus: Arc<EventBus>,
    /// Open buffers indexed by file path.
    buffers: Arc<RwLock<HashMap<PathBuf, Buffer>>>,
    /// Current active buffer.
    active_buffer: Arc<RwLock<Option<PathBuf>>>,
    /// Cursor state.
    cursor: Arc<RwLock<Cursor>>,
    /// Current selection.
    selection: Arc<RwLock<Option<Selection>>>,
    /// Undo/redo history.
    command_history: Arc<RwLock<Vec<Command>>>,
    /// Current position in command history.
    history_position: Arc<RwLock<usize>>,
}

impl Editor {
    /// Create a new editor instance.
    pub async fn new(config: Arc<RwLock<Config>>, event_bus: Arc<EventBus>) -> Result<Self> {
        debug!("Initializing editor");

        Ok(Self {
            config,
            event_bus,
            buffers: Arc::new(RwLock::new(HashMap::new())),
            active_buffer: Arc::new(RwLock::new(None)),
            cursor: Arc::new(RwLock::new(Cursor::new(0, 0))),
            selection: Arc::new(RwLock::new(None)),
            command_history: Arc::new(RwLock::new(Vec::new())),
            history_position: Arc::new(RwLock::new(0)),
        })
    }

    /// Open a file in the editor.
    pub async fn open_file(&self, path: &PathBuf) -> Result<()> {
        info!("Opening file: {}", path.display());

        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read file {}: {}", path.display(), e);
                return Err(XyluxError::Io(e));
            }
        };

        let buffer = Buffer::new(content, Some(path.clone()));

        {
            let mut buffers = self.buffers.write().await;
            buffers.insert(path.clone(), buffer);
        }

        {
            let mut active = self.active_buffer.write().await;
            *active = Some(path.clone());
        }

        // Reset cursor to beginning of file
        {
            let mut cursor = self.cursor.write().await;
            *cursor = Cursor::new(0, 0);
        }

        // Clear selection
        {
            let mut selection = self.selection.write().await;
            *selection = None;
        }

        // Publish file opened event
        let event = EventMessage::from_event(Event::Editor(crate::core::EditorEvent::FileOpened {
            path: path.clone(),
        }))
        .with_priority(EventPriority::Normal)
        .with_source("editor");

        self.event_bus.publish(event).await?;

        Ok(())
    }

    /// Save the current buffer.
    pub async fn save_current(&self) -> Result<()> {
        let active_path = {
            let active = self.active_buffer.read().await;
            active.clone()
        };

        if let Some(path) = active_path {
            self.save_buffer(&path).await
        } else {
            warn!("No active buffer to save");
            Ok(())
        }
    }

    /// Save all open buffers.
    pub async fn save_all(&self) -> Result<()> {
        let paths: Vec<PathBuf> = {
            let buffers = self.buffers.read().await;
            buffers.keys().cloned().collect()
        };

        for path in paths {
            if let Err(e) = self.save_buffer(&path).await {
                error!("Failed to save {}: {}", path.display(), e);
            }
        }

        Ok(())
    }

    /// Save a specific buffer.
    async fn save_buffer(&self, path: &PathBuf) -> Result<()> {
        let content = {
            let buffers = self.buffers.read().await;
            if let Some(buffer) = buffers.get(path) {
                buffer.get_content()
            } else {
                warn!("Buffer not found for path: {}", path.display());
                return Ok(());
            }
        };

        std::fs::write(path, content)?;

        // Mark buffer as saved
        {
            let mut buffers = self.buffers.write().await;
            if let Some(buffer) = buffers.get_mut(path) {
                buffer.mark_saved();
            }
        }

        // Publish file saved event
        let event = EventMessage::from_event(Event::Editor(crate::core::EditorEvent::FileSaved {
            path: path.clone(),
        }))
        .with_priority(EventPriority::Normal)
        .with_source("editor");

        self.event_bus.publish(event).await?;

        info!("Saved: {}", path.display());
        Ok(())
    }

    /// Auto-save modified buffers.
    pub async fn auto_save(&self) -> Result<()> {
        let config = self.config.read().await;
        if config.editor.auto_save_interval == 0 {
            return Ok(());
        }

        let paths: Vec<PathBuf> = {
            let buffers = self.buffers.read().await;
            buffers
                .iter()
                .filter(|(_, buffer)| buffer.is_modified())
                .map(|(path, _)| path.clone())
                .collect()
        };

        for path in paths {
            if let Err(e) = self.save_buffer(&path).await {
                error!("Auto-save failed for {}: {}", path.display(), e);
            }
        }

        Ok(())
    }

    /// Insert text at the current cursor position.
    pub async fn insert_text(&self, text: &str) -> Result<()> {
        let active_path = {
            let active = self.active_buffer.read().await;
            active.clone()
        };

        if let Some(path) = active_path {
            let cursor_pos = {
                let cursor = self.cursor.read().await;
                (cursor.line, cursor.column)
            };

            // Create command for undo/redo
            let command = Command::new(
                CommandType::InsertText {
                    line: cursor_pos.0,
                    column: cursor_pos.1,
                    text: text.to_string(),
                },
                Some(path.clone()),
            );

            // Execute the command
            self.execute_command(command).await?;
        }

        Ok(())
    }

    /// Delete text at the current cursor position.
    pub async fn delete_text(&self, count: usize) -> Result<()> {
        let active_path = {
            let active = self.active_buffer.read().await;
            active.clone()
        };

        if let Some(path) = active_path {
            let cursor_pos = {
                let cursor = self.cursor.read().await;
                (cursor.line, cursor.column)
            };

            // Get the text that will be deleted for undo
            let deleted_text = {
                let buffers = self.buffers.read().await;
                if let Some(buffer) = buffers.get(&path) {
                    buffer
                        .get_text_range(
                            cursor_pos.0,
                            cursor_pos.1,
                            cursor_pos.0,
                            cursor_pos.1 + count,
                        )
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            };

            // Create command for undo/redo
            let command = Command::new(
                CommandType::DeleteText {
                    line: cursor_pos.0,
                    column: cursor_pos.1,
                    text: deleted_text,
                },
                Some(path.clone()),
            );

            // Execute the command
            self.execute_command(command).await?;
        }

        Ok(())
    }

    /// Move cursor to a specific position.
    pub async fn move_cursor(&self, line: usize, column: usize) -> Result<()> {
        {
            let mut cursor = self.cursor.write().await;
            cursor.line = line;
            cursor.column = column;
        }

        // Publish cursor moved event
        let event =
            EventMessage::from_event(Event::Editor(crate::core::EditorEvent::CursorMoved {
                line,
                column,
            }))
            .with_priority(EventPriority::Low)
            .with_source("editor");

        self.event_bus.publish(event).await?;

        Ok(())
    }

    /// Get the current cursor position.
    pub async fn get_cursor_position(&self) -> (usize, usize) {
        let cursor = self.cursor.read().await;
        (cursor.line, cursor.column)
    }

    /// Get the content of the active buffer.
    pub async fn get_active_content(&self) -> Option<String> {
        let active_path = {
            let active = self.active_buffer.read().await;
            active.clone()
        };

        if let Some(path) = active_path {
            let buffers = self.buffers.read().await;
            buffers.get(&path).map(|buffer| buffer.get_content())
        } else {
            None
        }
    }

    /// Get a list of all open files.
    pub async fn get_open_files(&self) -> Vec<PathBuf> {
        let buffers = self.buffers.read().await;
        buffers.keys().cloned().collect()
    }

    /// Check if there are any modified buffers.
    pub async fn has_unsaved_changes(&self) -> bool {
        let buffers = self.buffers.read().await;
        buffers.values().any(|buffer| buffer.is_modified())
    }

    /// Close a file.
    pub async fn close_file(&self, path: &PathBuf) -> Result<()> {
        {
            let mut buffers = self.buffers.write().await;
            buffers.remove(path);
        }

        // If this was the active buffer, clear it
        {
            let mut active = self.active_buffer.write().await;
            if active.as_ref() == Some(path) {
                *active = None;
            }
        }

        // Publish file closed event
        let event = EventMessage::from_event(Event::Editor(crate::core::EditorEvent::FileClosed {
            path: path.clone(),
        }))
        .with_priority(EventPriority::Normal)
        .with_source("editor");

        self.event_bus.publish(event).await?;

        info!("Closed: {}", path.display());
        Ok(())
    }

    /// Execute a command and add it to history.
    async fn execute_command(&self, command: Command) -> Result<()> {
        // Execute the command
        match &command.command_type {
            CommandType::InsertText { line, column, text } => {
                if let Some(path) = &command.buffer_path {
                    {
                        let mut buffers = self.buffers.write().await;
                        if let Some(buffer) = buffers.get_mut(path) {
                            buffer.insert_text(*line, *column, text)?;
                        }
                    }

                    // Update cursor position
                    {
                        let mut cursor = self.cursor.write().await;
                        cursor.column += text.len();
                    }

                    // Publish event
                    let event = EventMessage::from_event(Event::Editor(
                        crate::core::EditorEvent::TextInserted {
                            line: *line,
                            column: *column,
                            text: text.clone(),
                        },
                    ))
                    .with_priority(EventPriority::Normal)
                    .with_source("editor");

                    self.event_bus.publish(event).await?;
                }
            }
            CommandType::DeleteText { line, column, text } => {
                if let Some(path) = &command.buffer_path {
                    {
                        let mut buffers = self.buffers.write().await;
                        if let Some(buffer) = buffers.get_mut(path) {
                            buffer.delete_text(*line, *column, text.len())?;
                        }
                    }

                    // Publish event
                    let event = EventMessage::from_event(Event::Editor(
                        crate::core::EditorEvent::TextDeleted {
                            start_line: *line,
                            start_column: *column,
                            end_line: *line,
                            end_column: *column + text.len(),
                        },
                    ))
                    .with_priority(EventPriority::Normal)
                    .with_source("editor");

                    self.event_bus.publish(event).await?;
                }
            }
            _ => {} // Handle other command types as needed
        }

        // Add to command history
        {
            let mut history = self.command_history.write().await;
            let mut position = self.history_position.write().await;

            // Truncate history after current position (when executing new command after undo)
            history.truncate(*position);

            // Add new command
            history.push(command);
            *position = history.len();
        }

        Ok(())
    }

    /// Undo the last command.
    pub async fn undo(&self) -> Result<bool> {
        let command_to_undo = {
            let mut position = self.history_position.write().await;
            if *position == 0 {
                return Ok(false); // Nothing to undo
            }

            *position -= 1;
            let history = self.command_history.read().await;
            history[*position].clone()
        };

        // Execute the reverse command
        match &command_to_undo.command_type {
            CommandType::InsertText { line, column, text } => {
                if let Some(path) = &command_to_undo.buffer_path {
                    // Undo insert by deleting the inserted text
                    let mut buffers = self.buffers.write().await;
                    if let Some(buffer) = buffers.get_mut(path) {
                        buffer.delete_text(*line, *column, text.len())?;
                    }
                }
            }
            CommandType::DeleteText { line, column, text } => {
                if let Some(path) = &command_to_undo.buffer_path {
                    // Undo delete by inserting the deleted text back
                    let mut buffers = self.buffers.write().await;
                    if let Some(buffer) = buffers.get_mut(path) {
                        buffer.insert_text(*line, *column, text)?;
                    }
                }
            }
            _ => {} // Handle other command types as needed
        }

        // Publish undo event
        let event = EventMessage::from_event(Event::Editor(crate::core::EditorEvent::Undo))
            .with_priority(EventPriority::Normal)
            .with_source("editor");

        self.event_bus.publish(event).await?;

        Ok(true)
    }

    /// Redo the next command.
    pub async fn redo(&self) -> Result<bool> {
        let command_to_redo = {
            let mut position = self.history_position.write().await;
            let history = self.command_history.read().await;

            if *position >= history.len() {
                return Ok(false); // Nothing to redo
            }

            let command = history[*position].clone();
            *position += 1;
            command
        };

        // Re-execute the command (without adding to history again)
        match &command_to_redo.command_type {
            CommandType::InsertText { line, column, text } => {
                if let Some(path) = &command_to_redo.buffer_path {
                    let mut buffers = self.buffers.write().await;
                    if let Some(buffer) = buffers.get_mut(path) {
                        buffer.insert_text(*line, *column, text)?;
                    }
                }
            }
            CommandType::DeleteText { line, column, text } => {
                if let Some(path) = &command_to_redo.buffer_path {
                    let mut buffers = self.buffers.write().await;
                    if let Some(buffer) = buffers.get_mut(path) {
                        buffer.delete_text(*line, *column, text.len())?;
                    }
                }
            }
            _ => {} // Handle other command types as needed
        }

        // Publish redo event
        let event = EventMessage::from_event(Event::Editor(crate::core::EditorEvent::Redo))
            .with_priority(EventPriority::Normal)
            .with_source("editor");

        self.event_bus.publish(event).await?;

        Ok(true)
    }

    /// Clear command history.
    pub async fn clear_history(&self) {
        let mut history = self.command_history.write().await;
        let mut position = self.history_position.write().await;

        history.clear();
        *position = 0;
    }

    /// Get the number of commands that can be undone.
    pub async fn can_undo_count(&self) -> usize {
        let position = self.history_position.read().await;
        *position
    }

    /// Get the number of commands that can be redone.
    pub async fn can_redo_count(&self) -> usize {
        let position = self.history_position.read().await;
        let history = self.command_history.read().await;
        history.len() - *position
    }

    /// Shutdown the editor.
    pub async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down editor");

        // Save all modified files
        self.save_all().await?;

        // Clear all buffers
        {
            let mut buffers = self.buffers.write().await;
            buffers.clear();
        }

        {
            let mut active = self.active_buffer.write().await;
            *active = None;
        }

        // Clear command history
        self.clear_history().await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_editor_creation() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());

        let editor = Editor::new(config, event_bus).await;
        assert!(editor.is_ok());
    }

    #[tokio::test]
    async fn test_file_operations() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());
        let editor = Editor::new(config, event_bus).await.unwrap();

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "Hello, world!").unwrap();

        // Test opening file
        editor.open_file(&test_file).await.unwrap();

        // Test getting content
        let content = editor.get_active_content().await;
        assert_eq!(content, Some("Hello, world!".to_string()));

        // Test cursor position
        let (line, column) = editor.get_cursor_position().await;
        assert_eq!((line, column), (0, 0));

        // Test saving
        editor.save_current().await.unwrap();
    }
}
