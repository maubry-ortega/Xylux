//! # Rust Analyzer LSP Client
//!
//! Rust analyzer integration for Rust language support.

use std::process::Stdio;

use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::process::{Child, Command};
use tracing::{debug, error, info};

use crate::core::{Result, XyluxError};
use crate::syntax::{
    CompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity, LspClient,
};

/// Rust analyzer LSP client.
pub struct RustAnalyzer {
    /// rust-analyzer process.
    process: Option<Child>,
    /// Configuration.
    config: RustAnalyzerConfig,
    /// Whether the LSP is initialized.
    initialized: bool,
    /// Request ID counter.
    request_id: u64,
}

/// Configuration for rust-analyzer.
#[derive(Debug, Clone)]
pub struct RustAnalyzerConfig {
    /// Path to rust-analyzer binary.
    pub binary_path: String,
    /// Additional arguments.
    pub args: Vec<String>,
    /// Enable proc macros.
    pub enable_proc_macros: bool,
    /// Cargo features to enable.
    pub cargo_features: Vec<String>,
    /// Check on save.
    pub check_on_save: bool,
}

impl Default for RustAnalyzerConfig {
    fn default() -> Self {
        Self {
            binary_path: "rust-analyzer".to_string(),
            args: Vec::new(),
            enable_proc_macros: true,
            cargo_features: Vec::new(),
            check_on_save: true,
        }
    }
}

impl RustAnalyzer {
    /// Create a new rust-analyzer client.
    pub async fn new(config: &crate::core::config::RustAnalyzerConfig) -> Result<Self> {
        let ra_config = RustAnalyzerConfig {
            binary_path: config
                .binary_path
                .clone()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "rust-analyzer".to_string()),
            args: config.args.clone(),
            enable_proc_macros: config.enable_proc_macros,
            cargo_features: config.cargo_features.clone(),
            check_on_save: config.check_on_save,
        };

        let mut analyzer =
            Self { process: None, config: ra_config, initialized: false, request_id: 0 };

        analyzer.start().await?;
        Ok(analyzer)
    }

    /// Start the rust-analyzer process.
    async fn start(&mut self) -> Result<()> {
        debug!("Starting rust-analyzer process");

        let mut command = Command::new(&self.config.binary_path);
        command.args(&self.config.args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let child = command
            .spawn()
            .map_err(|e| XyluxError::lsp_error(format!("Failed to start rust-analyzer: {}", e)))?;

        self.process = Some(child);
        info!("rust-analyzer process started");

        // Initialize the LSP
        self.initialize().await?;

        Ok(())
    }

    /// Initialize the LSP connection.
    async fn initialize(&mut self) -> Result<()> {
        debug!("Initializing rust-analyzer LSP");

        let _initialize_request = json!({
            "jsonrpc": "2.0",
            "id": self.next_request_id(),
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": null,
                "capabilities": {
                    "textDocument": {
                        "completion": {
                            "completionItem": {
                                "snippetSupport": true,
                                "resolveSupport": {
                                    "properties": ["documentation", "detail"]
                                }
                            }
                        },
                        "hover": {
                            "contentFormat": ["markdown", "plaintext"]
                        },
                        "publishDiagnostics": {
                            "relatedInformation": true,
                            "tagSupport": {
                                "valueSet": [1, 2]
                            }
                        }
                    },
                    "workspace": {
                        "workspaceFolders": true,
                        "configuration": true
                    }
                },
                "initializationOptions": {
                    "procMacro": {
                        "enable": self.config.enable_proc_macros
                    },
                    "cargo": {
                        "features": self.config.cargo_features
                    },
                    "checkOnSave": {
                        "enable": self.config.check_on_save
                    }
                }
            }
        });

        // Send initialize request (simplified for now)
        self.initialized = true;
        info!("rust-analyzer LSP initialized");

        Ok(())
    }

    /// Get the next request ID.
    fn next_request_id(&mut self) -> u64 {
        self.request_id += 1;
        self.request_id
    }

    /// Send a request to rust-analyzer.
    #[allow(dead_code)]
    async fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        if !self.initialized {
            return Err(XyluxError::lsp_error("LSP not initialized"));
        }

        let _request = json!({
            "jsonrpc": "2.0",
            "id": self.next_request_id(),
            "method": method,
            "params": params
        });

        // For now, return a mock response
        // In a real implementation, we would send this to the process and wait for response
        debug!("Would send request: {}", method);
        Ok(json!({}))
    }

    /// Check if rust-analyzer is available.
    pub async fn check_availability() -> Result<String> {
        let output = Command::new("rust-analyzer")
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| XyluxError::lsp_error(format!("rust-analyzer not found: {}", e)))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(version.trim().to_string())
        } else {
            Err(XyluxError::lsp_error("rust-analyzer is not available or not working"))
        }
    }

    /// Convert LSP completion item kind to our enum.
    #[allow(dead_code)]
    fn convert_completion_kind(lsp_kind: Option<i32>) -> CompletionItemKind {
        match lsp_kind {
            Some(1) => CompletionItemKind::Text,
            Some(2) => CompletionItemKind::Method,
            Some(3) => CompletionItemKind::Function,
            Some(4) => CompletionItemKind::Constructor,
            Some(5) => CompletionItemKind::Field,
            Some(6) => CompletionItemKind::Variable,
            Some(7) => CompletionItemKind::Class,
            Some(8) => CompletionItemKind::Interface,
            Some(9) => CompletionItemKind::Module,
            Some(10) => CompletionItemKind::Property,
            Some(11) => CompletionItemKind::Unit,
            Some(12) => CompletionItemKind::Value,
            Some(13) => CompletionItemKind::Enum,
            Some(14) => CompletionItemKind::Keyword,
            Some(15) => CompletionItemKind::Snippet,
            Some(16) => CompletionItemKind::Color,
            Some(17) => CompletionItemKind::File,
            Some(18) => CompletionItemKind::Reference,
            _ => CompletionItemKind::Text,
        }
    }

    /// Convert LSP diagnostic severity to our enum.
    #[allow(dead_code)]
    fn convert_diagnostic_severity(lsp_severity: Option<i32>) -> DiagnosticSeverity {
        match lsp_severity {
            Some(1) => DiagnosticSeverity::Error,
            Some(2) => DiagnosticSeverity::Warning,
            Some(3) => DiagnosticSeverity::Info,
            Some(4) => DiagnosticSeverity::Hint,
            _ => DiagnosticSeverity::Error,
        }
    }

    /// Parse completion items from LSP response.
    #[allow(dead_code)]
    fn parse_completion_items(&self, response: &Value) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        if let Some(result) = response.get("result") {
            let empty_vec = Vec::new();
            let completion_items = if let Some(list) = result.get("items") {
                list.as_array().unwrap_or(&empty_vec)
            } else if let Some(array) = result.as_array() {
                array
            } else {
                return items;
            };

            for item in completion_items {
                if let Some(label) = item.get("label").and_then(|l| l.as_str()) {
                    let kind = item.get("kind").and_then(|k| k.as_i64()).map(|k| k as i32);

                    let detail = item.get("detail").and_then(|d| d.as_str()).map(String::from);

                    let documentation = item.get("documentation").and_then(|d| {
                        if let Some(s) = d.as_str() {
                            Some(s.to_string())
                        } else if let Some(obj) = d.as_object() {
                            obj.get("value").and_then(|v| v.as_str()).map(String::from)
                        } else {
                            None
                        }
                    });

                    let insert_text = item
                        .get("insertText")
                        .and_then(|t| t.as_str())
                        .map(String::from)
                        .or_else(|| Some(label.to_string()));

                    items.push(CompletionItem {
                        label: label.to_string(),
                        kind: Self::convert_completion_kind(kind),
                        detail,
                        documentation,
                        insert_text,
                    });
                }
            }
        }

        items
    }

    /// Parse diagnostics from LSP response.
    #[allow(dead_code)]
    fn parse_diagnostics(&self, response: &Value) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if let Some(params) = response.get("params") {
            if let Some(diag_array) = params.get("diagnostics").and_then(|d| d.as_array()) {
                for diag in diag_array {
                    if let Some(range) = diag.get("range") {
                        let start = range.get("start");
                        let end = range.get("end");

                        if let (Some(start_line), Some(start_char), Some(end_char)) = (
                            start.and_then(|s| s.get("line")).and_then(|l| l.as_u64()),
                            start.and_then(|s| s.get("character")).and_then(|c| c.as_u64()),
                            end.and_then(|e| e.get("character")).and_then(|c| c.as_u64()),
                        ) {
                            let severity =
                                diag.get("severity").and_then(|s| s.as_i64()).map(|s| s as i32);

                            let message = diag
                                .get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("Unknown error")
                                .to_string();

                            let source =
                                diag.get("source").and_then(|s| s.as_str()).map(String::from);

                            diagnostics.push(Diagnostic {
                                line: start_line as usize,
                                column: start_char as usize,
                                length: (end_char - start_char) as usize,
                                severity: Self::convert_diagnostic_severity(severity),
                                message,
                                source,
                            });
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

#[async_trait]
impl LspClient for RustAnalyzer {
    async fn set_root_uri(&self, uri: &str) -> Result<()> {
        debug!("Setting root URI for rust-analyzer: {}", uri);
        // In a real implementation, we would send a workspace/didChangeWorkspaceFolders notification
        Ok(())
    }

    async fn get_diagnostics(&self, file_path: &str) -> Result<Vec<Diagnostic>> {
        debug!("Getting diagnostics for: {}", file_path);

        if !self.initialized {
            return Ok(Vec::new());
        }

        // In a real implementation, we would maintain a cache of diagnostics
        // that get updated via publishDiagnostics notifications

        // For now, return mock diagnostics for demonstration
        if file_path.ends_with(".rs") {
            Ok(vec![Diagnostic {
                line: 0,
                column: 0,
                length: 10,
                severity: DiagnosticSeverity::Warning,
                message: "Mock diagnostic from rust-analyzer".to_string(),
                source: Some("rust-analyzer".to_string()),
            }])
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_completion(
        &self,
        file_path: &str,
        line: usize,
        column: usize,
    ) -> Result<Vec<CompletionItem>> {
        debug!("Getting completion for {}:{}:{}", file_path, line, column);

        if !self.initialized {
            return Ok(Vec::new());
        }

        // Mock completion items for demonstration
        if file_path.ends_with(".rs") {
            Ok(vec![
                CompletionItem {
                    label: "println!".to_string(),
                    kind: CompletionItemKind::Function,
                    detail: Some("macro".to_string()),
                    documentation: Some("Prints to stdout".to_string()),
                    insert_text: Some("println!(\"{}\", )".to_string()),
                },
                CompletionItem {
                    label: "Vec".to_string(),
                    kind: CompletionItemKind::Class,
                    detail: Some("struct std::vec::Vec".to_string()),
                    documentation: Some("A contiguous growable array type".to_string()),
                    insert_text: Some("Vec".to_string()),
                },
                CompletionItem {
                    label: "String".to_string(),
                    kind: CompletionItemKind::Class,
                    detail: Some("struct std::string::String".to_string()),
                    documentation: Some("A UTF-8 encoded string".to_string()),
                    insert_text: Some("String".to_string()),
                },
            ])
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_hover(
        &self,
        file_path: &str,
        line: usize,
        column: usize,
    ) -> Result<Option<String>> {
        debug!("Getting hover for {}:{}:{}", file_path, line, column);

        if !self.initialized {
            return Ok(None);
        }

        // Mock hover information for demonstration
        if file_path.ends_with(".rs") {
            Ok(Some("Hover information from rust-analyzer".to_string()))
        } else {
            Ok(None)
        }
    }

    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down rust-analyzer");

        // In a real implementation, we would send a shutdown request
        // followed by an exit notification to gracefully close the LSP server

        Ok(())
    }
}

impl Drop for RustAnalyzer {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            if let Err(e) = process.start_kill() {
                error!("Failed to kill rust-analyzer process: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rust_analyzer_availability() {
        // This test may fail in environments without rust-analyzer
        let _unused = RustAnalyzer::check_availability().await;
    }

    #[test]
    fn test_completion_kind_conversion() {
        assert_eq!(RustAnalyzer::convert_completion_kind(Some(3)), CompletionItemKind::Function);
        assert_eq!(RustAnalyzer::convert_completion_kind(Some(7)), CompletionItemKind::Class);
        assert_eq!(RustAnalyzer::convert_completion_kind(None), CompletionItemKind::Text);
    }

    #[test]
    fn test_diagnostic_severity_conversion() {
        assert_eq!(RustAnalyzer::convert_diagnostic_severity(Some(1)), DiagnosticSeverity::Error);
        assert_eq!(RustAnalyzer::convert_diagnostic_severity(Some(2)), DiagnosticSeverity::Warning);
        assert_eq!(RustAnalyzer::convert_diagnostic_severity(None), DiagnosticSeverity::Error);
    }

    #[test]
    fn test_parse_completion_items() {
        let analyzer = RustAnalyzer {
            process: None,
            config: RustAnalyzerConfig::default(),
            initialized: true,
            request_id: 0,
        };

        let response = json!({
            "result": {
                "items": [
                    {
                        "label": "test_function",
                        "kind": 3,
                        "detail": "fn test_function()",
                        "documentation": "A test function"
                    }
                ]
            }
        });

        let items = analyzer.parse_completion_items(&response);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "test_function");
        assert_eq!(items[0].kind, CompletionItemKind::Function);
    }

    #[test]
    fn test_parse_diagnostics() {
        let analyzer = RustAnalyzer {
            process: None,
            config: RustAnalyzerConfig::default(),
            initialized: true,
            request_id: 0,
        };

        let response = json!({
            "params": {
                "diagnostics": [
                    {
                        "range": {
                            "start": { "line": 0, "character": 0 },
                            "end": { "line": 0, "character": 10 }
                        },
                        "severity": 1,
                        "message": "Test error",
                        "source": "rust-analyzer"
                    }
                ]
            }
        });

        let diagnostics = analyzer.parse_diagnostics(&response);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 0);
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].message, "Test error");
    }

    #[test]
    fn test_config_default() {
        let config = RustAnalyzerConfig::default();
        assert_eq!(config.binary_path, "rust-analyzer");
        assert!(config.enable_proc_macros);
        assert!(config.check_on_save);
    }
}
