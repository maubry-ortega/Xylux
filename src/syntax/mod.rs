//! # Syntax Module
//!
//! Syntax highlighting and language server management for Xylux IDE.

pub mod alux_syntax;
pub mod highlighter;
pub mod lsp_client;
pub mod rust_analyzer;

pub use alux_syntax::AluxSyntax;
pub use highlighter::SyntaxHighlighter;
pub use lsp_client::LspClient;
pub use rust_analyzer::RustAnalyzer;

// Type alias for backward compatibility
pub type HighlightInfo = HighlightToken;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::core::events::{
    CompletionItemKind as CoreCompletionItemKind, DiagnosticSeverity as CoreDiagnosticSeverity,
    LspCompletionItem, LspDiagnostic,
};
use crate::core::{Config, EventBus, Result};

/// Main syntax manager that coordinates syntax highlighting and LSP services.
pub struct SyntaxManager {
    /// IDE configuration.
    config: Arc<RwLock<Config>>,
    /// Event bus for communication.
    event_bus: Arc<EventBus>,
    /// Syntax highlighters by file extension.
    highlighters: Arc<RwLock<HashMap<String, Box<dyn SyntaxHighlighter + Send + Sync>>>>,
    /// LSP clients by language.
    lsp_clients: Arc<RwLock<HashMap<String, Box<dyn LspClient + Send + Sync>>>>,
    /// Current project root.
    project_root: Arc<RwLock<Option<PathBuf>>>,
    /// Whether LSP is enabled.
    lsp_enabled: Arc<RwLock<bool>>,
}

impl SyntaxManager {
    /// Create a new syntax manager.
    pub async fn new(config: Arc<RwLock<Config>>, event_bus: Arc<EventBus>) -> Result<Self> {
        debug!("Initializing syntax manager");

        let lsp_enabled = {
            let config = config.read().await;
            config.lsp.enabled
        };

        let manager = Self {
            config,
            event_bus,
            highlighters: Arc::new(RwLock::new(HashMap::new())),
            lsp_clients: Arc::new(RwLock::new(HashMap::new())),
            project_root: Arc::new(RwLock::new(None)),
            lsp_enabled: Arc::new(RwLock::new(lsp_enabled)),
        };

        manager.initialize_highlighters().await?;

        if lsp_enabled {
            manager.initialize_lsp_clients().await?;
        }

        Ok(manager)
    }

    /// Initialize syntax highlighters for supported languages.
    async fn initialize_highlighters(&self) -> Result<()> {
        debug!("Initializing syntax highlighters");

        let mut highlighters = self.highlighters.write().await;

        // Register Rust highlighter
        highlighters.insert("rs".to_string(), Box::new(RustSyntaxHighlighter::new()));

        // Register TOML highlighter
        highlighters.insert("toml".to_string(), Box::new(TomlSyntaxHighlighter::new()));

        // Register JSON highlighter
        highlighters.insert("json".to_string(), Box::new(JsonSyntaxHighlighter::new()));

        // Register Markdown highlighter
        highlighters.insert("md".to_string(), Box::new(MarkdownSyntaxHighlighter::new()));

        info!("Initialized {} syntax highlighters", highlighters.len());
        Ok(())
    }

    /// Initialize LSP clients for supported languages.
    async fn initialize_lsp_clients(&self) -> Result<()> {
        debug!("Initializing LSP clients");

        let mut clients = self.lsp_clients.write().await;
        let config = self.config.read().await;

        // Initialize rust-analyzer
        if let Ok(rust_analyzer) = RustAnalyzer::new(&config.lsp.rust_analyzer).await {
            clients.insert("rust".to_string(), Box::new(rust_analyzer));
            info!("Initialized rust-analyzer LSP client");
        } else {
            warn!("Failed to initialize rust-analyzer");
        }

        // Initialize Alux LSP if available
        if let Ok(alux_lsp) = AluxLspClient::new(&config.lsp.alux_lsp).await {
            clients.insert("alux".to_string(), Box::new(alux_lsp));
            info!("Initialized Alux LSP client");
        } else {
            warn!("Failed to initialize Alux LSP client");
        }

        info!("Initialized {} LSP clients", clients.len());
        Ok(())
    }

    /// Set the project root directory.
    pub async fn set_project_root(&self, root: &PathBuf) -> Result<()> {
        info!("Setting project root: {}", root.display());

        {
            let mut project_root = self.project_root.write().await;
            *project_root = Some(root.clone());
        }

        // Publish project root changed event
        let event = crate::core::EventMessage::from_event(crate::core::Event::Project(
            crate::core::ProjectEvent::Opened {
                path: root.clone(),
                project_type: "Unknown".to_string(),
            },
        ))
        .with_priority(crate::core::EventPriority::Normal)
        .with_source("syntax_manager");
        self.event_bus.publish(event).await?;

        // Notify LSP clients of project root change if LSP is enabled
        let lsp_enabled = { *self.lsp_enabled.read().await };
        if lsp_enabled {
            let clients = self.lsp_clients.read().await;
            for (lang, client) in clients.iter() {
                if let Err(e) = client.set_root_uri(root.to_str().unwrap_or("")).await {
                    error!("Failed to set root URI for {} LSP: {}", lang, e);
                }
            }
        }

        Ok(())
    }

    /// Clear the current project.
    pub async fn clear_project(&self) -> Result<()> {
        debug!("Clearing current project");

        let old_project_root = {
            let mut project_root = self.project_root.write().await;
            let old_root = project_root.clone();
            *project_root = None;
            old_root
        };

        // Publish project closed event if there was a project
        if let Some(root) = old_project_root {
            let event = crate::core::EventMessage::from_event(crate::core::Event::Project(
                crate::core::ProjectEvent::Closed { path: root },
            ))
            .with_priority(crate::core::EventPriority::Normal)
            .with_source("syntax_manager");
            self.event_bus.publish(event).await?;
        }

        // Notify LSP clients if LSP is enabled
        let lsp_enabled = { *self.lsp_enabled.read().await };
        if lsp_enabled {
            let clients = self.lsp_clients.read().await;
            for (lang, client) in clients.iter() {
                if let Err(e) = client.shutdown().await {
                    error!("Failed to shutdown {} LSP: {}", lang, e);
                }
            }
        }

        Ok(())
    }

    /// Get syntax highlighting for a file.
    pub async fn highlight_file(
        &self,
        file_path: &PathBuf,
        content: &str,
    ) -> Result<Vec<HighlightToken>> {
        let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let highlighters = self.highlighters.read().await;
        if let Some(highlighter) = highlighters.get(extension) {
            highlighter.highlight(content).await
        } else {
            // No highlighter available, return plain text
            Ok(vec![HighlightToken::new(0, content.len(), TokenType::Text)])
        }
    }

    /// Get diagnostics for a file.
    pub async fn get_diagnostics(&self, file_path: &PathBuf) -> Result<Vec<Diagnostic>> {
        let lsp_enabled = { *self.lsp_enabled.read().await };
        if !lsp_enabled {
            return Ok(Vec::new());
        }

        let language = self.detect_language(file_path);

        let clients = self.lsp_clients.read().await;
        if let Some(client) = clients.get(&language) {
            let diagnostics = client.get_diagnostics(file_path.to_str().unwrap_or("")).await?;

            // Publish diagnostics received event
            if !diagnostics.is_empty() {
                let lsp_diagnostics: Vec<LspDiagnostic> = diagnostics
                    .iter()
                    .map(|d| LspDiagnostic {
                        line: d.line,
                        column: d.column,
                        end_line: d.line,
                        end_column: d.column + d.length,
                        severity: match d.severity {
                            DiagnosticSeverity::Error => CoreDiagnosticSeverity::Error,
                            DiagnosticSeverity::Warning => CoreDiagnosticSeverity::Warning,
                            DiagnosticSeverity::Info => CoreDiagnosticSeverity::Info,
                            DiagnosticSeverity::Hint => CoreDiagnosticSeverity::Hint,
                        },
                        message: d.message.clone(),
                        source: d.source.clone(),
                    })
                    .collect();

                let event = crate::core::EventMessage::from_event(crate::core::Event::Lsp(
                    crate::core::LspEvent::DiagnosticsReceived {
                        path: file_path.clone(),
                        diagnostics: lsp_diagnostics,
                    },
                ))
                .with_priority(crate::core::EventPriority::Normal)
                .with_source("syntax_manager");
                self.event_bus.publish(event).await?;
            }

            Ok(diagnostics)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get code completion for a position in a file.
    pub async fn get_completion(
        &self,
        file_path: &PathBuf,
        line: usize,
        column: usize,
    ) -> Result<Vec<CompletionItem>> {
        let lsp_enabled = { *self.lsp_enabled.read().await };
        if !lsp_enabled {
            return Ok(Vec::new());
        }

        let language = self.detect_language(file_path);

        let clients = self.lsp_clients.read().await;
        if let Some(client) = clients.get(&language) {
            let completions =
                client.get_completion(file_path.to_str().unwrap_or(""), line, column).await?;

            // Publish completion received event
            if !completions.is_empty() {
                let lsp_completions: Vec<LspCompletionItem> = completions
                    .iter()
                    .map(|c| LspCompletionItem {
                        label: c.label.clone(),
                        kind: Some(match c.kind {
                            CompletionItemKind::Text => CoreCompletionItemKind::Text,
                            CompletionItemKind::Method => CoreCompletionItemKind::Method,
                            CompletionItemKind::Function => CoreCompletionItemKind::Function,
                            CompletionItemKind::Constructor => CoreCompletionItemKind::Constructor,
                            CompletionItemKind::Field => CoreCompletionItemKind::Field,
                            CompletionItemKind::Variable => CoreCompletionItemKind::Variable,
                            CompletionItemKind::Class => CoreCompletionItemKind::Class,
                            CompletionItemKind::Interface => CoreCompletionItemKind::Interface,
                            CompletionItemKind::Module => CoreCompletionItemKind::Module,
                            CompletionItemKind::Property => CoreCompletionItemKind::Property,
                            CompletionItemKind::Unit => CoreCompletionItemKind::Unit,
                            CompletionItemKind::Value => CoreCompletionItemKind::Value,
                            CompletionItemKind::Enum => CoreCompletionItemKind::Enum,
                            CompletionItemKind::Keyword => CoreCompletionItemKind::Keyword,
                            CompletionItemKind::Snippet => CoreCompletionItemKind::Snippet,
                            CompletionItemKind::Color => CoreCompletionItemKind::Color,
                            CompletionItemKind::File => CoreCompletionItemKind::File,
                            CompletionItemKind::Reference => CoreCompletionItemKind::Reference,
                        }),
                        detail: c.detail.clone(),
                        documentation: c.documentation.clone(),
                        insert_text: c.insert_text.clone(),
                    })
                    .collect();

                let event = crate::core::EventMessage::from_event(crate::core::Event::Lsp(
                    crate::core::LspEvent::CompletionReceived { items: lsp_completions },
                ))
                .with_priority(crate::core::EventPriority::Low)
                .with_source("syntax_manager");
                self.event_bus.publish(event).await?;
            }

            Ok(completions)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get hover information for a position in a file.
    pub async fn get_hover(
        &self,
        file_path: &PathBuf,
        line: usize,
        column: usize,
    ) -> Result<Option<String>> {
        let lsp_enabled = { *self.lsp_enabled.read().await };
        if !lsp_enabled {
            return Ok(None);
        }

        let language = self.detect_language(file_path);

        let clients = self.lsp_clients.read().await;
        if let Some(client) = clients.get(&language) {
            let hover_content =
                client.get_hover(file_path.to_str().unwrap_or(""), line, column).await?;

            // Publish hover received event
            if let Some(content) = &hover_content {
                let event = crate::core::EventMessage::from_event(crate::core::Event::Lsp(
                    crate::core::LspEvent::HoverReceived { content: content.clone() },
                ))
                .with_priority(crate::core::EventPriority::Low)
                .with_source("syntax_manager");
                self.event_bus.publish(event).await?;
            }

            Ok(hover_content)
        } else {
            Ok(None)
        }
    }

    /// Detect language from file path.
    fn detect_language(&self, file_path: &PathBuf) -> String {
        let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        match extension {
            "rs" => "rust".to_string(),
            "toml" => "toml".to_string(),
            "json" => "json".to_string(),
            "md" => "markdown".to_string(),
            _ => "text".to_string(),
        }
    }

    /// Enable or disable LSP functionality.
    pub async fn set_lsp_enabled(&self, enabled: bool) -> Result<()> {
        let mut lsp_enabled = self.lsp_enabled.write().await;
        let was_enabled = *lsp_enabled;
        *lsp_enabled = enabled;

        if enabled && !was_enabled {
            // Re-initialize LSP clients
            self.initialize_lsp_clients().await?;
        } else if !enabled && was_enabled {
            // Shutdown LSP clients
            let clients = self.lsp_clients.read().await;
            for (lang, client) in clients.iter() {
                if let Err(e) = client.shutdown().await {
                    error!("Failed to shutdown {} LSP: {}", lang, e);
                }
            }
        }

        // Publish LSP status change event
        let event = crate::core::EventMessage::from_event(crate::core::Event::System(
            crate::core::SystemEvent::ConfigReloaded,
        ))
        .with_priority(crate::core::EventPriority::Normal)
        .with_source("syntax_manager");
        self.event_bus.publish(event).await?;

        info!("LSP functionality {}", if enabled { "enabled" } else { "disabled" });
        Ok(())
    }

    /// Check if LSP is currently enabled.
    pub async fn is_lsp_enabled(&self) -> bool {
        *self.lsp_enabled.read().await
    }

    /// Shutdown the syntax manager.
    pub async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down syntax manager");

        // Publish shutdown event
        let event = crate::core::EventMessage::from_event(crate::core::Event::System(
            crate::core::SystemEvent::ShutdownRequested,
        ))
        .with_priority(crate::core::EventPriority::Critical)
        .with_source("syntax_manager");
        self.event_bus.publish(event).await?;

        // Shutdown all LSP clients
        let lsp_enabled = { *self.lsp_enabled.read().await };
        if lsp_enabled {
            let clients = self.lsp_clients.read().await;
            for (lang, client) in clients.iter() {
                if let Err(e) = client.shutdown().await {
                    error!("Failed to shutdown {} LSP: {}", lang, e);
                }
            }
        }

        Ok(())
    }
}

/// Represents a syntax highlight token.
#[derive(Debug, Clone)]
pub struct HighlightToken {
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
}

impl HighlightToken {
    pub fn new(start: usize, end: usize, token_type: TokenType) -> Self {
        Self { start, end, token_type }
    }
}

/// Types of syntax tokens.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Text,
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Variable,
    Type,
    Operator,
    Punctuation,
}

/// Represents a diagnostic message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>,
}

/// Diagnostic severity levels.
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Represents a code completion item.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

/// Types of completion items.
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionItemKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
}

// Basic syntax highlighter implementations

pub struct RustSyntaxHighlighter {
    keywords: Vec<String>,
}

impl RustSyntaxHighlighter {
    pub fn new() -> Self {
        // Try to load config from assets/syntax_config.json
        let mut keywords: Vec<String> = vec!["fn".to_string()];

        let config_path = std::path::Path::new("assets").join("syntax_config.json");
        if let Ok(content) = std::fs::read_to_string(config_path) {
            #[derive(serde::Deserialize)]
            struct RustCfg { keywords: Option<Vec<String>> }
            #[derive(serde::Deserialize)]
            struct Cfg { rust: Option<RustCfg> }
            if let Ok(cfg) = serde_json::from_str::<Cfg>(&content) {
                if let Some(r) = cfg.rust {
                    if let Some(kw) = r.keywords {
                        if !kw.is_empty() { keywords = kw; }
                    }
                }
            }
        }

        Self { keywords }
    }
}

#[async_trait::async_trait]
impl SyntaxHighlighter for RustSyntaxHighlighter {
    async fn highlight(&self, content: &str) -> Result<Vec<HighlightToken>> {
        // Keyword + strings + numbers + line comments
        let mut tokens: Vec<HighlightToken> = Vec::new();

        // Keywords (whole word)
        let keyword_refs: Vec<&str> = self.keywords.iter().map(|s| s.as_str()).collect();
        let mut kw = highlighter::utils::extract_keywords(content, &keyword_refs);
        tokens.append(&mut kw);

        // Strings ("..." and '...')
        let mut str_tokens = highlighter::utils::extract_strings(content, &['"', '\'']);
        tokens.append(&mut str_tokens);

        // Numbers
        let mut nums = highlighter::utils::extract_numbers(content);
        tokens.append(&mut nums);

        // Line comments
        let mut comments = highlighter::utils::extract_line_comments(content, "//");
        tokens.append(&mut comments);

        Ok(highlighter::utils::merge_tokens(tokens))
    }
}

pub struct AluxSyntaxHighlighter;

impl AluxSyntaxHighlighter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl SyntaxHighlighter for AluxSyntaxHighlighter {
    async fn highlight(&self, content: &str) -> Result<Vec<HighlightToken>> {
        // Simple Alux syntax highlighting
        let mut tokens = Vec::new();
        let keywords = ["fn", "let", "if", "else", "while", "for", "return", "task"];

        for keyword in &keywords {
            for (start, _) in content.match_indices(keyword) {
                tokens.push(HighlightToken::new(start, start + keyword.len(), TokenType::Keyword));
            }
        }

        Ok(tokens)
    }
}

pub struct TomlSyntaxHighlighter;

impl TomlSyntaxHighlighter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl SyntaxHighlighter for TomlSyntaxHighlighter {
    async fn highlight(&self, _content: &str) -> Result<Vec<HighlightToken>> {
        // Basic TOML highlighting would go here
        Ok(Vec::new())
    }
}

pub struct JsonSyntaxHighlighter;

impl JsonSyntaxHighlighter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl SyntaxHighlighter for JsonSyntaxHighlighter {
    async fn highlight(&self, _content: &str) -> Result<Vec<HighlightToken>> {
        // Basic JSON highlighting would go here
        Ok(Vec::new())
    }
}

pub struct MarkdownSyntaxHighlighter;

impl MarkdownSyntaxHighlighter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl SyntaxHighlighter for MarkdownSyntaxHighlighter {
    async fn highlight(&self, content: &str) -> Result<Vec<HighlightToken>> {
        // Simple markdown: headings and code fences
        let mut tokens = Vec::new();
        for (line_idx, line) in content.lines().enumerate() {
            let line_start: usize = content
                .lines()
                .take(line_idx)
                .map(|l| l.len() + 1)
                .sum();
            if line.trim_start().starts_with('#') {
                tokens.push(HighlightToken::new(
                    line_start,
                    line_start + line.len(),
                    TokenType::Keyword,
                ));
            }
            if line.trim_start().starts_with("```") {
                tokens.push(HighlightToken::new(
                    line_start,
                    line_start + line.len(),
                    TokenType::Punctuation,
                ));
            }
        }
        Ok(tokens)
    }
}

// Placeholder LSP client
pub struct AluxLspClient;

impl AluxLspClient {
    pub async fn new(_config: &crate::core::config::AluxLspConfig) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait::async_trait]
impl LspClient for AluxLspClient {
    async fn set_root_uri(&self, _uri: &str) -> Result<()> {
        Ok(())
    }

    async fn get_diagnostics(&self, _file_path: &str) -> Result<Vec<Diagnostic>> {
        Ok(Vec::new())
    }

    async fn get_completion(
        &self,
        _file_path: &str,
        _line: usize,
        _column: usize,
    ) -> Result<Vec<CompletionItem>> {
        Ok(Vec::new())
    }

    async fn get_hover(
        &self,
        _file_path: &str,
        _line: usize,
        _column: usize,
    ) -> Result<Option<String>> {
        Ok(None)
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_syntax_manager_creation() {
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = Arc::new(EventBus::new());

        let syntax_manager = SyntaxManager::new(config, event_bus).await;
        assert!(syntax_manager.is_ok());
    }

    #[test]
    fn test_language_detection() {
        let manager = SyntaxManager {
            config: Arc::new(RwLock::new(Config::default())),
            event_bus: Arc::new(EventBus::new()),
            highlighters: Arc::new(RwLock::new(HashMap::new())),
            lsp_clients: Arc::new(RwLock::new(HashMap::new())),
            project_root: Arc::new(RwLock::new(None)),
            lsp_enabled: Arc::new(RwLock::new(true)),
        };

        assert_eq!(manager.detect_language(&PathBuf::from("test.rs")), "rust");
        assert_eq!(manager.detect_language(&PathBuf::from("script.aux")), "alux");
        assert_eq!(manager.detect_language(&PathBuf::from("config.toml")), "toml");
        assert_eq!(manager.detect_language(&PathBuf::from("data.json")), "json");
        assert_eq!(manager.detect_language(&PathBuf::from("shader.wgsl")), "wgsl");
    }

    #[tokio::test]
    async fn test_rust_syntax_highlighting() {
        let highlighter = RustSyntaxHighlighter::new();
        let content = "fn main() { let x = 42; }";

        let tokens = highlighter.highlight(content).await.unwrap();
        assert!(!tokens.is_empty());

        // Should have highlighted "fn" and "let"
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Keyword));
    }
}
