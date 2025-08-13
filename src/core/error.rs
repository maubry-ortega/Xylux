//! # Error Handling
//!
//! Comprehensive error handling system for Xylux IDE.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for Xylux IDE operations.
#[derive(Error, Debug)]
pub enum XyluxError {
    /// IO related errors.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration parsing errors.
    #[error("Configuration error in {file}:{line}: {message}")]
    Config {
        /// Configuration file path.
        file: PathBuf,
        /// Line number where error occurred.
        line: usize,
        /// Error message.
        message: String,
    },

    /// Project management errors.
    #[error("Project error: {0}")]
    Project(String),

    /// Language Server Protocol errors.
    #[error("LSP error: {0}")]
    Lsp(String),

    /// Syntax highlighting errors.
    #[error("Syntax error: {0}")]
    Syntax(String),

    /// Build system errors.
    #[error("Build error: {0}")]
    Build(String),

    /// File watching errors.
    #[error("File watcher error: {0}")]
    FileWatcher(#[from] notify::Error),

    /// Terminal/UI errors.
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// JSON serialization/deserialization errors.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parsing errors.
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Command line argument errors.
    #[error("Invalid command line arguments: {0}")]
    Arguments(String),

    /// Plugin/extension errors.
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// Alux language specific errors.
    #[error("Alux error: {0}")]
    Alux(String),

    /// Xylux engine integration errors.
    #[error("Xylux engine error: {0}")]
    XyluxEngine(String),

    /// Network/HTTP errors (for updates, extensions, etc.).
    #[cfg(feature = "network")]
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Clipboard errors.
    #[cfg(feature = "clipboard")]
    #[error("Clipboard error: {0}")]
    Clipboard(#[from] arboard::Error),

    /// Generic errors with context.
    #[error("Error: {context}: {source}")]
    WithContext {
        /// Context describing what operation failed.
        context: String,
        #[source]
        /// The underlying error.
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Critical errors that should cause immediate shutdown.
    #[error("Critical error: {0}")]
    Critical(String),
}

impl XyluxError {
    /// Create a configuration error.
    pub fn config_error<P, S>(file: P, line: usize, message: S) -> Self
    where
        P: Into<PathBuf>,
        S: Into<String>,
    {
        Self::Config { file: file.into(), line, message: message.into() }
    }

    /// Create a project error.
    pub fn project_error<S: Into<String>>(message: S) -> Self {
        Self::Project(message.into())
    }

    /// Create an LSP error.
    pub fn lsp_error<S: Into<String>>(message: S) -> Self {
        Self::Lsp(message.into())
    }

    /// Create a syntax error.
    pub fn syntax_error<S: Into<String>>(message: S) -> Self {
        Self::Syntax(message.into())
    }

    /// Create a build error.
    pub fn build_error<S: Into<String>>(message: S) -> Self {
        Self::Build(message.into())
    }

    /// Create a plugin error.
    pub fn plugin_error<S: Into<String>>(message: S) -> Self {
        Self::Plugin(message.into())
    }

    /// Create an Alux error.
    pub fn alux_error<S: Into<String>>(message: S) -> Self {
        Self::Alux(message.into())
    }

    /// Create a Xylux engine error.
    pub fn xylux_engine_error<S: Into<String>>(message: S) -> Self {
        Self::XyluxEngine(message.into())
    }

    /// Create an IO error with context.
    pub fn io<E: Into<std::io::Error>, S: Into<String>>(error: E, context: S) -> Self {
        Self::with_context(context, error.into())
    }

    /// Create an invalid data error.
    pub fn invalid_data<S: Into<String>>(message: S) -> Self {
        Self::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()))
    }

    /// Create an invalid input error.
    pub fn invalid_input<S: Into<String>>(message: S) -> Self {
        Self::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, message.into()))
    }

    /// Create a not found error.
    pub fn not_found<S: Into<String>>(message: S) -> Self {
        Self::Io(std::io::Error::new(std::io::ErrorKind::NotFound, message.into()))
    }

    /// Create a permission denied error.
    pub fn permission_denied<S: Into<String>>(message: S) -> Self {
        Self::Io(std::io::Error::new(std::io::ErrorKind::PermissionDenied, message.into()))
    }

    /// Create a terminal error.
    pub fn terminal<S: Into<String>>(message: S) -> Self {
        Self::Terminal(message.into())
    }

    /// Create a config error with just a message.
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config { file: PathBuf::from("unknown"), line: 0, message: message.into() }
    }

    /// Create a platform-specific error.
    pub fn platform<S: Into<String>>(message: S) -> Self {
        Self::Io(std::io::Error::new(std::io::ErrorKind::Other, message.into()))
    }

    /// Create a parse error.
    pub fn parse<S: Into<String>>(message: S) -> Self {
        Self::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()))
    }

    /// Create a serialization error.
    pub fn serialize<S: Into<String>>(message: S) -> Self {
        Self::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()))
    }

    /// Create an error with additional context.
    pub fn with_context<C, E>(context: C, error: E) -> Self
    where
        C: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::WithContext { context: context.into(), source: Box::new(error) }
    }

    /// Create a critical error.
    pub fn critical<S: Into<String>>(message: S) -> Self {
        Self::Critical(message.into())
    }

    /// Check if this error is critical and should cause shutdown.
    pub fn is_critical(&self) -> bool {
        matches!(self, Self::Critical(_))
    }

    /// Get the error category for logging and metrics.
    pub fn category(&self) -> &'static str {
        match self {
            Self::Io(_) => "io",
            Self::Config { .. } => "config",
            Self::Project(_) => "project",
            Self::Lsp(_) => "lsp",
            Self::Syntax(_) => "syntax",
            Self::Build(_) => "build",
            Self::FileWatcher(_) => "file_watcher",
            Self::Terminal(_) => "terminal",
            Self::Json(_) => "json",
            Self::Toml(_) => "toml",
            Self::Arguments(_) => "arguments",
            Self::Plugin(_) => "plugin",
            Self::Alux(_) => "alux",
            Self::XyluxEngine(_) => "xylux_engine",
            #[cfg(feature = "network")]
            Self::Network(_) => "network",
            #[cfg(feature = "clipboard")]
            Self::Clipboard(_) => "clipboard",
            Self::WithContext { .. } => "context",
            Self::Critical(_) => "critical",
        }
    }
}

/// Result type alias for Xylux IDE operations.
pub type Result<T> = std::result::Result<T, XyluxError>;

/// Extension trait for Result to add context easily.
pub trait ResultExt<T> {
    /// Add context to an error.
    fn with_context<C: Into<String>>(self, context: C) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<C: Into<String>>(self, context: C) -> Result<T> {
        self.map_err(|e| XyluxError::with_context(context, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let config_err = XyluxError::config_error("/path/to/config.toml", 42, "Invalid syntax");
        assert!(matches!(config_err, XyluxError::Config { .. }));

        let project_err = XyluxError::project_error("Project not found");
        assert!(matches!(project_err, XyluxError::Project(_)));

        let critical_err = XyluxError::critical("System failure");
        assert!(critical_err.is_critical());
    }

    #[test]
    fn test_error_categories() {
        let io_err = XyluxError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert_eq!(io_err.category(), "io");

        let config_err = XyluxError::config_error("test.toml", 1, "test");
        assert_eq!(config_err.category(), "config");

        let critical_err = XyluxError::critical("test");
        assert_eq!(critical_err.category(), "critical");
    }

    #[test]
    fn test_result_ext() {
        let result: std::result::Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));

        let with_context = result.with_context("Testing context");
        assert!(with_context.is_err());

        if let Err(XyluxError::WithContext { context, .. }) = with_context {
            assert_eq!(context, "Testing context");
        } else {
            panic!("Expected WithContext error");
        }
    }
}
