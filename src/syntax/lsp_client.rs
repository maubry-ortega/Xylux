//! # LSP Client Trait
//!
//! Language Server Protocol client trait and interfaces.

use std::collections::HashMap;

use crate::core::Result;
use crate::syntax::{CompletionItem, Diagnostic};

/// Trait for Language Server Protocol clients.
#[async_trait::async_trait]
pub trait LspClient {
    /// Set the root URI for the workspace.
    async fn set_root_uri(&self, uri: &str) -> Result<()>;

    /// Get diagnostics for a file.
    async fn get_diagnostics(&self, file_path: &str) -> Result<Vec<Diagnostic>>;

    /// Get code completion at a specific position.
    async fn get_completion(
        &self,
        file_path: &str,
        line: usize,
        column: usize,
    ) -> Result<Vec<CompletionItem>>;

    /// Get hover information at a specific position.
    async fn get_hover(
        &self,
        file_path: &str,
        line: usize,
        column: usize,
    ) -> Result<Option<String>>;

    /// Get code actions for a range.
    async fn get_code_actions(
        &self,
        file_path: &str,
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
    ) -> Result<Vec<CodeAction>> {
        let _ = (file_path, start_line, start_column, end_line, end_column);
        Ok(Vec::new())
    }

    /// Go to definition.
    async fn goto_definition(
        &self,
        file_path: &str,
        line: usize,
        column: usize,
    ) -> Result<Option<Location>> {
        let _ = (file_path, line, column);
        Ok(None)
    }

    /// Find references.
    async fn find_references(
        &self,
        file_path: &str,
        line: usize,
        column: usize,
    ) -> Result<Vec<Location>> {
        let _ = (file_path, line, column);
        Ok(Vec::new())
    }

    /// Rename symbol.
    async fn rename_symbol(
        &self,
        file_path: &str,
        line: usize,
        column: usize,
        new_name: &str,
    ) -> Result<Option<WorkspaceEdit>> {
        let _ = (file_path, line, column, new_name);
        Ok(None)
    }

    /// Format document.
    async fn format_document(&self, file_path: &str) -> Result<Option<Vec<TextEdit>>> {
        let _ = file_path;
        Ok(None)
    }

    /// Shutdown the LSP client.
    async fn shutdown(&self) -> Result<()>;

    /// Get the language this client supports.
    fn language(&self) -> &str {
        "unknown"
    }

    /// Check if the client is running.
    fn is_running(&self) -> bool {
        false
    }

    /// Get server capabilities.
    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities::default()
    }
}

/// Represents a code action.
#[derive(Debug, Clone)]
pub struct CodeAction {
    pub title: String,
    pub kind: Option<String>,
    pub edit: Option<WorkspaceEdit>,
    pub command: Option<Command>,
}

/// Represents a location in the workspace.
#[derive(Debug, Clone)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

/// Represents a range in a document.
#[derive(Debug, Clone)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Represents a position in a document.
#[derive(Debug, Clone)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

/// Represents a workspace edit.
#[derive(Debug, Clone)]
pub struct WorkspaceEdit {
    pub changes: HashMap<String, Vec<TextEdit>>,
}

/// Represents a text edit.
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Represents an LSP command.
#[derive(Debug, Clone)]
pub struct Command {
    pub title: String,
    pub command: String,
    pub arguments: Option<Vec<serde_json::Value>>,
}

/// Server capabilities.
#[derive(Debug, Clone)]
pub struct ServerCapabilities {
    pub text_document_sync: Option<TextDocumentSyncCapability>,
    pub hover_provider: bool,
    pub completion_provider: Option<CompletionOptions>,
    pub definition_provider: bool,
    pub references_provider: bool,
    pub document_formatting_provider: bool,
    pub rename_provider: bool,
    pub code_action_provider: bool,
    pub document_symbol_provider: bool,
    pub workspace_symbol_provider: bool,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            text_document_sync: None,
            hover_provider: false,
            completion_provider: None,
            definition_provider: false,
            references_provider: false,
            document_formatting_provider: false,
            rename_provider: false,
            code_action_provider: false,
            document_symbol_provider: false,
            workspace_symbol_provider: false,
        }
    }
}

/// Text document sync capability.
#[derive(Debug, Clone)]
pub enum TextDocumentSyncCapability {
    None,
    Full,
    Incremental,
}

/// Completion options.
#[derive(Debug, Clone)]
pub struct CompletionOptions {
    pub trigger_characters: Vec<String>,
    pub resolve_provider: bool,
}

impl Position {
    pub fn new(line: usize, character: usize) -> Self {
        Self { line, character }
    }
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

impl Location {
    pub fn new(uri: String, range: Range) -> Self {
        Self { uri, range }
    }
}

impl TextEdit {
    pub fn new(range: Range, new_text: String) -> Self {
        Self { range, new_text }
    }
}

impl CodeAction {
    pub fn new(title: String) -> Self {
        Self { title, kind: None, edit: None, command: None }
    }

    pub fn with_edit(mut self, edit: WorkspaceEdit) -> Self {
        self.edit = Some(edit);
        self
    }

    pub fn with_command(mut self, command: Command) -> Self {
        self.command = Some(command);
        self
    }
}

/// LSP client configuration.
#[derive(Debug, Clone)]
pub struct LspClientConfig {
    /// Path to the LSP server binary.
    pub server_path: String,
    /// Arguments to pass to the server.
    pub server_args: Vec<String>,
    /// Environment variables for the server.
    pub env_vars: HashMap<String, String>,
    /// Working directory for the server.
    pub working_directory: Option<String>,
    /// Timeout for requests in milliseconds.
    pub request_timeout_ms: u64,
    /// Whether to enable logging.
    pub enable_logging: bool,
    /// Log file path.
    pub log_file: Option<String>,
}

impl Default for LspClientConfig {
    fn default() -> Self {
        Self {
            server_path: String::new(),
            server_args: Vec::new(),
            env_vars: HashMap::new(),
            working_directory: None,
            request_timeout_ms: 5000,
            enable_logging: false,
            log_file: None,
        }
    }
}

/// LSP message types.
#[derive(Debug, Clone)]
pub enum LspMessage {
    Request { id: i32, method: String, params: serde_json::Value },
    Response { id: i32, result: Option<serde_json::Value>, error: Option<LspError> },
    Notification { method: String, params: serde_json::Value },
}

/// LSP error.
#[derive(Debug, Clone)]
pub struct LspError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Common LSP methods.
pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const INITIALIZED: &str = "initialized";
    pub const SHUTDOWN: &str = "shutdown";
    pub const EXIT: &str = "exit";

    pub const TEXT_DOCUMENT_DID_OPEN: &str = "textDocument/didOpen";
    pub const TEXT_DOCUMENT_DID_CHANGE: &str = "textDocument/didChange";
    pub const TEXT_DOCUMENT_DID_CLOSE: &str = "textDocument/didClose";
    pub const TEXT_DOCUMENT_DID_SAVE: &str = "textDocument/didSave";

    pub const TEXT_DOCUMENT_HOVER: &str = "textDocument/hover";
    pub const TEXT_DOCUMENT_COMPLETION: &str = "textDocument/completion";
    pub const TEXT_DOCUMENT_DEFINITION: &str = "textDocument/definition";
    pub const TEXT_DOCUMENT_REFERENCES: &str = "textDocument/references";
    pub const TEXT_DOCUMENT_RENAME: &str = "textDocument/rename";
    pub const TEXT_DOCUMENT_FORMATTING: &str = "textDocument/formatting";
    pub const TEXT_DOCUMENT_CODE_ACTION: &str = "textDocument/codeAction";

    pub const TEXT_DOCUMENT_PUBLISH_DIAGNOSTICS: &str = "textDocument/publishDiagnostics";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(10, 5);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn test_range_creation() {
        let start = Position::new(0, 0);
        let end = Position::new(1, 10);
        let range = Range::new(start, end);

        assert_eq!(range.start.line, 0);
        assert_eq!(range.end.line, 1);
    }

    #[test]
    fn test_code_action_builder() {
        let action = CodeAction::new("Fix issue".to_string());
        assert_eq!(action.title, "Fix issue");
        assert!(action.edit.is_none());
        assert!(action.command.is_none());
    }

    #[test]
    fn test_server_capabilities() {
        let caps = ServerCapabilities::default();
        assert!(!caps.hover_provider);
        assert!(!caps.definition_provider);
        assert!(caps.completion_provider.is_none());
    }

    #[test]
    fn test_lsp_client_config() {
        let config = LspClientConfig::default();
        assert_eq!(config.request_timeout_ms, 5000);
        assert!(!config.enable_logging);
        assert!(config.server_path.is_empty());
    }
}
