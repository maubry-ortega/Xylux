//! # Commands Module
//!
//! Command system for undo/redo functionality and editor operations.

use std::path::PathBuf;

use super::{Buffer, Cursor, Selection};
use crate::core::{Result, XyluxError};

/// Represents an editor command that can be undone/redone.
#[derive(Debug, Clone)]
pub struct Command {
    /// The type of command.
    pub command_type: CommandType,
    /// The buffer this command applies to.
    pub buffer_path: Option<PathBuf>,
    /// Whether this command has been executed.
    pub executed: bool,
    /// Timestamp when the command was created.
    pub timestamp: std::time::SystemTime,
}

/// Different types of editor commands.
#[derive(Debug, Clone)]
pub enum CommandType {
    /// Insert text at a position.
    InsertText { line: usize, column: usize, text: String },
    /// Delete text from a range.
    DeleteText {
        line: usize,
        column: usize,
        text: String, // Store the deleted text for undo
    },
    /// Replace text in a range.
    ReplaceText {
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
        old_text: String,
        new_text: String,
    },
    /// Insert a new line.
    InsertLine { line: usize, content: String },
    /// Delete a line.
    DeleteLine {
        line: usize,
        content: String, // Store the deleted line for undo
    },
    /// Move cursor.
    MoveCursor { old_position: Cursor, new_position: Cursor },
    /// Change selection.
    ChangeSelection { old_selection: Option<Selection>, new_selection: Option<Selection> },
    /// Composite command containing multiple sub-commands.
    Composite { commands: Vec<Command>, description: String },
}

impl Command {
    /// Create a new command.
    pub fn new(command_type: CommandType, buffer_path: Option<PathBuf>) -> Self {
        Self { command_type, buffer_path, executed: false, timestamp: std::time::SystemTime::now() }
    }

    /// Create an insert text command.
    pub fn insert_text(
        line: usize,
        column: usize,
        text: String,
        buffer_path: Option<PathBuf>,
    ) -> Self {
        Self::new(CommandType::InsertText { line, column, text }, buffer_path)
    }

    /// Create a delete text command.
    pub fn delete_text(
        line: usize,
        column: usize,
        text: String,
        buffer_path: Option<PathBuf>,
    ) -> Self {
        Self::new(CommandType::DeleteText { line, column, text }, buffer_path)
    }

    /// Create a replace text command.
    pub fn replace_text(
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
        old_text: String,
        new_text: String,
        buffer_path: Option<PathBuf>,
    ) -> Self {
        Self::new(
            CommandType::ReplaceText {
                start_line,
                start_column,
                end_line,
                end_column,
                old_text,
                new_text,
            },
            buffer_path,
        )
    }

    /// Create an insert line command.
    pub fn insert_line(line: usize, content: String, buffer_path: Option<PathBuf>) -> Self {
        Self::new(CommandType::InsertLine { line, content }, buffer_path)
    }

    /// Create a delete line command.
    pub fn delete_line(line: usize, content: String, buffer_path: Option<PathBuf>) -> Self {
        Self::new(CommandType::DeleteLine { line, content }, buffer_path)
    }

    /// Create a move cursor command.
    pub fn move_cursor(
        old_position: Cursor,
        new_position: Cursor,
        buffer_path: Option<PathBuf>,
    ) -> Self {
        Self::new(CommandType::MoveCursor { old_position, new_position }, buffer_path)
    }

    /// Create a change selection command.
    pub fn change_selection(
        old_selection: Option<Selection>,
        new_selection: Option<Selection>,
        buffer_path: Option<PathBuf>,
    ) -> Self {
        Self::new(CommandType::ChangeSelection { old_selection, new_selection }, buffer_path)
    }

    /// Create a composite command.
    pub fn composite(
        commands: Vec<Command>,
        description: String,
        buffer_path: Option<PathBuf>,
    ) -> Self {
        Self::new(CommandType::Composite { commands, description }, buffer_path)
    }

    /// Execute the command on a buffer.
    pub fn execute(&mut self, buffer: &mut Buffer) -> Result<()> {
        if self.executed {
            return Ok(()); // Already executed
        }

        match &self.command_type {
            CommandType::InsertText { line, column, text } => {
                buffer.insert_text(*line, *column, text)?;
            }
            CommandType::DeleteText { line, column, text } => {
                buffer.delete_text(*line, *column, text.len())?;
            }
            CommandType::ReplaceText {
                start_line,
                start_column,
                end_line,
                end_column,
                new_text,
                ..
            } => {
                buffer.delete_range(*start_line, *start_column, *end_line, *end_column)?;
                buffer.insert_text(*start_line, *start_column, new_text)?;
            }
            CommandType::InsertLine { line, content } => {
                buffer.insert_line(*line, content.clone())?;
            }
            CommandType::DeleteLine { line, .. } => {
                buffer.delete_line(*line)?;
            }
            CommandType::MoveCursor { .. } => {
                // Cursor movement doesn't affect buffer
            }
            CommandType::ChangeSelection { .. } => {
                // Selection change doesn't affect buffer
            }
            CommandType::Composite { commands, .. } => {
                for cmd in commands {
                    let mut cmd_clone = cmd.clone();
                    cmd_clone.execute(buffer)?;
                }
            }
        }

        self.executed = true;
        Ok(())
    }

    /// Undo the command on a buffer.
    pub fn undo(&mut self, buffer: &mut Buffer) -> Result<()> {
        if !self.executed {
            return Ok(()); // Not executed, nothing to undo
        }

        match &self.command_type {
            CommandType::InsertText { line, column, text } => {
                buffer.delete_text(*line, *column, text.len())?;
            }
            CommandType::DeleteText { line, column, text } => {
                buffer.insert_text(*line, *column, text)?;
            }
            CommandType::ReplaceText {
                start_line,
                start_column,
                end_line: _,
                end_column: _,
                old_text,
                new_text,
            } => {
                buffer.delete_range(
                    *start_line,
                    *start_column,
                    *start_line,
                    start_column + new_text.len(),
                )?;
                buffer.insert_text(*start_line, *start_column, old_text)?;
            }
            CommandType::InsertLine { line, .. } => {
                buffer.delete_line(*line)?;
            }
            CommandType::DeleteLine { line, content } => {
                buffer.insert_line(*line, content.clone())?;
            }
            CommandType::MoveCursor { .. } => {
                // Cursor movement undo is handled differently
            }
            CommandType::ChangeSelection { .. } => {
                // Selection change undo is handled differently
            }
            CommandType::Composite { commands, .. } => {
                // Undo composite commands in reverse order
                for cmd in commands.iter().rev() {
                    let mut cmd_clone = cmd.clone();
                    cmd_clone.undo(buffer)?;
                }
            }
        }

        self.executed = false;
        Ok(())
    }

    /// Check if this command can be merged with another command.
    pub fn can_merge_with(&self, other: &Command) -> bool {
        if self.buffer_path != other.buffer_path {
            return false;
        }

        // Only merge certain types of commands
        match (&self.command_type, &other.command_type) {
            (
                CommandType::InsertText { line: line1, column: column1, text: text1 },
                CommandType::InsertText { line: line2, column: column2, text: _text2 },
            ) => {
                // Merge if inserting at consecutive positions
                *line1 == *line2 && *column1 + text1.len() == *column2
            }
            (
                CommandType::DeleteText { line: line1, column: column1, .. },
                CommandType::DeleteText { line: line2, column: column2, .. },
            ) => {
                // Merge if deleting at the same position
                *line1 == *line2 && *column1 == *column2
            }
            _ => false,
        }
    }

    /// Merge this command with another command.
    pub fn merge_with(&mut self, other: Command) -> Result<()> {
        if !self.can_merge_with(&other) {
            return Err(XyluxError::syntax_error("Cannot merge incompatible commands".to_string()));
        }

        match (&mut self.command_type, other.command_type) {
            (
                CommandType::InsertText { text: text1, .. },
                CommandType::InsertText { text: text2, .. },
            ) => {
                text1.push_str(&text2);
            }
            (
                CommandType::DeleteText { text: text1, .. },
                CommandType::DeleteText { text: text2, .. },
            ) => {
                text1.push_str(&text2);
            }
            _ => {
                return Err(XyluxError::syntax_error(
                    "Cannot merge incompatible commands".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get a description of the command.
    pub fn description(&self) -> String {
        match &self.command_type {
            CommandType::InsertText { text, .. } => {
                format!("Insert '{}'", text.chars().take(20).collect::<String>())
            }
            CommandType::DeleteText { text, .. } => {
                format!("Delete '{}'", text.chars().take(20).collect::<String>())
            }
            CommandType::ReplaceText { old_text, new_text, .. } => {
                format!(
                    "Replace '{}' with '{}'",
                    old_text.chars().take(10).collect::<String>(),
                    new_text.chars().take(10).collect::<String>()
                )
            }
            CommandType::InsertLine { .. } => "Insert line".to_string(),
            CommandType::DeleteLine { .. } => "Delete line".to_string(),
            CommandType::MoveCursor { .. } => "Move cursor".to_string(),
            CommandType::ChangeSelection { .. } => "Change selection".to_string(),
            CommandType::Composite { description, .. } => description.clone(),
        }
    }

    /// Check if this command affects the buffer content.
    pub fn affects_content(&self) -> bool {
        match &self.command_type {
            CommandType::InsertText { .. }
            | CommandType::DeleteText { .. }
            | CommandType::ReplaceText { .. }
            | CommandType::InsertLine { .. }
            | CommandType::DeleteLine { .. } => true,
            CommandType::MoveCursor { .. } | CommandType::ChangeSelection { .. } => false,
            CommandType::Composite { commands, .. } => {
                commands.iter().any(|cmd| cmd.affects_content())
            }
        }
    }

    /// Get the timestamp of the command.
    pub fn timestamp(&self) -> std::time::SystemTime {
        self.timestamp
    }

    /// Check if the command is executed.
    pub fn is_executed(&self) -> bool {
        self.executed
    }
}

/// Command history manager for undo/redo functionality.
#[derive(Debug)]
pub struct CommandHistory {
    /// List of commands in order.
    commands: Vec<Command>,
    /// Current position in the command history.
    position: usize,
    /// Maximum number of commands to keep.
    max_size: usize,
    /// Whether to merge similar commands.
    merge_similar: bool,
}

impl CommandHistory {
    /// Create a new command history.
    pub fn new(max_size: usize) -> Self {
        Self { commands: Vec::new(), position: 0, max_size, merge_similar: true }
    }

    /// Add a command to the history.
    pub fn add_command(&mut self, command: Command) {
        // Try to merge with the last command if enabled
        if self.merge_similar && !self.commands.is_empty() && self.position > 0 {
            let last_index = self.position - 1;
            if let Some(last_command) = self.commands.get_mut(last_index) {
                if last_command.can_merge_with(&command) {
                    if last_command.merge_with(command.clone()).is_ok() {
                        return; // Successfully merged
                    }
                }
            }
        }

        // Remove any commands after the current position (if we're not at the end)
        self.commands.truncate(self.position);

        // Add the new command
        self.commands.push(command);
        self.position = self.commands.len();

        // Ensure we don't exceed the maximum size
        while self.commands.len() > self.max_size {
            self.commands.remove(0);
            self.position = self.position.saturating_sub(1);
        }
    }

    /// Undo the last command.
    pub fn undo(&mut self, buffer: &mut Buffer) -> Result<Option<Command>> {
        if self.position == 0 {
            return Ok(None); // Nothing to undo
        }

        self.position -= 1;
        if let Some(command) = self.commands.get_mut(self.position) {
            command.undo(buffer)?;
            Ok(Some(command.clone()))
        } else {
            Ok(None)
        }
    }

    /// Redo the next command.
    pub fn redo(&mut self, buffer: &mut Buffer) -> Result<Option<Command>> {
        if self.position >= self.commands.len() {
            return Ok(None); // Nothing to redo
        }

        if let Some(command) = self.commands.get_mut(self.position) {
            command.execute(buffer)?;
            self.position += 1;
            Ok(Some(command.clone()))
        } else {
            Ok(None)
        }
    }

    /// Check if there are commands to undo.
    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    /// Check if there are commands to redo.
    pub fn can_redo(&self) -> bool {
        self.position < self.commands.len()
    }

    /// Get the current position in the history.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the total number of commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the history is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Clear the command history.
    pub fn clear(&mut self) {
        self.commands.clear();
        self.position = 0;
    }

    /// Get a description of the last undoable command.
    pub fn last_undoable_description(&self) -> Option<String> {
        if self.position > 0 {
            self.commands.get(self.position - 1).map(|cmd| cmd.description())
        } else {
            None
        }
    }

    /// Get a description of the next redoable command.
    pub fn next_redoable_description(&self) -> Option<String> {
        if self.position < self.commands.len() {
            self.commands.get(self.position).map(|cmd| cmd.description())
        } else {
            None
        }
    }

    /// Enable or disable command merging.
    pub fn set_merge_similar(&mut self, merge: bool) {
        self.merge_similar = merge;
    }

    /// Get the command history as a slice.
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(1000) // Default to 1000 commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::insert_text(0, 0, "Hello".to_string(), None);
        assert!(!cmd.is_executed());
        assert!(cmd.affects_content());
        assert_eq!(cmd.description(), "Insert 'Hello'");
    }

    #[test]
    fn test_command_execution() {
        let mut buffer = Buffer::new("".to_string(), None);
        let mut cmd = Command::insert_text(0, 0, "Hello".to_string(), None);

        cmd.execute(&mut buffer).unwrap();
        assert!(cmd.is_executed());
        assert_eq!(buffer.get_content(), "Hello");
    }

    #[test]
    fn test_command_undo() {
        let mut buffer = Buffer::new("".to_string(), None);
        let mut cmd = Command::insert_text(0, 0, "Hello".to_string(), None);

        cmd.execute(&mut buffer).unwrap();
        assert_eq!(buffer.get_content(), "Hello");

        cmd.undo(&mut buffer).unwrap();
        assert!(!cmd.is_executed());
        assert_eq!(buffer.get_content(), "");
    }

    #[test]
    fn test_command_merging() {
        let mut cmd1 = Command::insert_text(0, 0, "Hello".to_string(), None);
        let cmd2 = Command::insert_text(0, 5, " World".to_string(), None);

        assert!(cmd1.can_merge_with(&cmd2));
        cmd1.merge_with(cmd2).unwrap();

        if let CommandType::InsertText { text, .. } = &cmd1.command_type {
            assert_eq!(text, "Hello World");
        } else {
            panic!("Expected InsertText command");
        }
    }

    #[test]
    fn test_command_history() {
        let mut history = CommandHistory::new(10);
        let mut buffer = Buffer::new("".to_string(), None);

        let cmd1 = Command::insert_text(0, 0, "Hello".to_string(), None);
        let cmd2 = Command::insert_text(0, 5, " World".to_string(), None);

        history.add_command(cmd1);
        history.add_command(cmd2);

        assert_eq!(history.len(), 2);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        // Undo
        let undone = history.undo(&mut buffer).unwrap();
        assert!(undone.is_some());
        assert!(history.can_redo());

        // Redo
        let redone = history.redo(&mut buffer).unwrap();
        assert!(redone.is_some());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_composite_command() {
        let mut buffer = Buffer::new("Line 1\nLine 2".to_string(), None);

        let cmd1 = Command::insert_text(0, 6, " Modified".to_string(), None);
        let cmd2 = Command::insert_text(1, 6, " Modified".to_string(), None);

        let mut composite =
            Command::composite(vec![cmd1, cmd2], "Modify both lines".to_string(), None);

        composite.execute(&mut buffer).unwrap();
        assert_eq!(buffer.get_content(), "Line 1 Modified\nLine 2 Modified");

        composite.undo(&mut buffer).unwrap();
        assert_eq!(buffer.get_content(), "Line 1\nLine 2");
    }

    #[test]
    fn test_delete_command() {
        let mut buffer = Buffer::new("Hello World".to_string(), None);
        let mut cmd = Command::delete_text(0, 5, " World".to_string(), None);

        cmd.execute(&mut buffer).unwrap();
        assert_eq!(buffer.get_content(), "Hello");

        cmd.undo(&mut buffer).unwrap();
        assert_eq!(buffer.get_content(), "Hello World");
    }

    #[test]
    fn test_replace_command() {
        let mut buffer = Buffer::new("Hello World".to_string(), None);
        let mut cmd =
            Command::replace_text(0, 6, 0, 11, "World".to_string(), "Universe".to_string(), None);

        cmd.execute(&mut buffer).unwrap();
        assert_eq!(buffer.get_content(), "Hello Universe");

        cmd.undo(&mut buffer).unwrap();
        assert_eq!(buffer.get_content(), "Hello World");
    }
}
