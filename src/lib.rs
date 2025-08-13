//! # Xylux IDE
//!
//! A comprehensive IDE for Rust development and Alux scripting with Xylux engine integration.
//!
//! Xylux IDE is designed specifically for game development using the Xylux engine and Alux
//! scripting language. It provides advanced features including:
//!
//! - Full Rust language support with rust-analyzer integration
//! - Native Alux scripting language support
//! - Xylux engine project management
//! - Hot-reload capabilities for rapid development
//! - WebAssembly compilation support
//! - Integrated terminal and build system
//! - Modern UI with multiple themes
//! - Plugin system for extensibility
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use xylux_ide::{Config, XyluxIde, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let config = Config::load().await?;
//!     let mut ide = XyluxIde::new(config).await?;
//!     ide.run().await
//! }
//! ```
//!
//! ## Architecture
//!
//! The IDE is built with a modular architecture:
//!
//! - **Core**: Fundamental systems (config, events, errors)
//! - **Editor**: Text editing and buffer management
//! - **Syntax**: Language support and LSP integration
//! - **UI**: User interface components and layout
//! - **Project**: Project management and file operations
//! - **Build**: Build system integration (Cargo, Xylux CLI)
//! - **Platform**: Platform-specific implementations

#![allow(missing_docs)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

// Re-export core functionality
pub use crate::core::{
    BUILD_INFO, BuildInfo, Config, ConfigLoader, Event, EventBus, EventHandler, EventMessage,
    Result, VERSION, XyluxError, initialize, shutdown,
};

// Re-export main IDE structure
pub use crate::ide::XyluxIde;

// Core modules
pub mod core;

// Main IDE implementation
mod ide;

// Feature modules
pub mod build;
pub mod editor;
pub mod gui;
pub mod platform;
pub mod project;
pub mod syntax;
pub mod ui;

// Conditional compilation for different platforms
#[cfg(unix)]
pub use platform::unix;

#[cfg(windows)]
pub use platform::windows;

#[cfg(target_os = "wasi")]
pub use platform::wasi;

/// Prelude module for common imports.
pub mod prelude {
    //! Common imports for Xylux IDE development.

    pub use crate::XyluxIde;
    pub use crate::core::{
        Config, Event, EventBus, EventHandler, EventMessage, EventPriority, Result, XyluxError,
    };
    pub use crate::editor::{Buffer, Cursor, Editor};
    pub use crate::project::{Project, ProjectManager, ProjectType};
    pub use crate::ui::{Component, Layout, Theme, UiManager};

    // Common async traits
    pub use async_trait::async_trait;

    // Common futures and async utilities
    pub use futures::{Future, Stream, future, stream};
    pub use tokio::{spawn, time};

    // Common error handling
    pub use anyhow::{Context as AnyhowContext, anyhow};
    pub use thiserror::Error;

    // Common serialization
    pub use serde::{Deserialize, Serialize};

    // Common logging
    pub use tracing::{debug, error, info, trace, warn};
}

/// Feature flags and capabilities.
pub mod features {
    //! Runtime feature detection and capabilities.

    /// Check if clipboard support is available.
    pub fn has_clipboard() -> bool {
        cfg!(feature = "clipboard")
    }

    /// Check if network features are available.
    pub fn has_network() -> bool {
        cfg!(feature = "network")
    }

    /// Check if debug features are enabled.
    pub fn has_debug() -> bool {
        cfg!(feature = "debug") || cfg!(debug_assertions)
    }

    /// Get available language servers.
    pub fn available_language_servers() -> Vec<&'static str> {
        let mut servers = vec!["rust-analyzer"];

        // Add Alux LSP if available
        if which::which("alux-lsp").is_ok() {
            servers.push("alux-lsp");
        }

        servers
    }

    /// Get available build tools.
    pub fn available_build_tools() -> Vec<&'static str> {
        let mut tools = vec![];

        if which::which("cargo").is_ok() {
            tools.push("cargo");
        }

        if which::which("xylux").is_ok() {
            tools.push("xylux-cli");
        }

        if which::which("wasm-pack").is_ok() {
            tools.push("wasm-pack");
        }

        tools
    }
}

/// Utility functions and helpers.
pub mod utils {
    //! Utility functions and helpers.

    use std::path::{Path, PathBuf};

    /// Check if a path is likely a Rust project.
    pub fn is_rust_project<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();
        path.join("Cargo.toml").exists() || path.join("src").join("main.rs").exists()
    }

    /// Check if a path is likely a Xylux project.
    pub fn is_xylux_project<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();
        path.join("xylux.toml").exists() || path.join("scripts").exists()
    }

    /// Get the project root directory from a given path.
    pub fn find_project_root<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
        let mut current = path.as_ref().to_path_buf();

        loop {
            if is_rust_project(&current) || is_xylux_project(&current) {
                return Some(current);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Format file size in human-readable format.
    pub fn format_file_size(bytes: u64) -> String {
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

    /// Escape text for display in terminal.
    pub fn escape_text(text: &str) -> String {
        text.chars()
            .map(|c| match c {
                '\t' => "→".to_string(),
                '\r' => "↵".to_string(),
                '\n' => "⏎".to_string(),
                c if c.is_control() => format!("\\u{:04x}", c as u32),
                c => c.to_string(),
            })
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use tempfile::TempDir;

        #[test]
        fn test_format_file_size() {
            assert_eq!(format_file_size(0), "0 B");
            assert_eq!(format_file_size(512), "512 B");
            assert_eq!(format_file_size(1024), "1.0 KB");
            assert_eq!(format_file_size(1536), "1.5 KB");
            assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        }

        #[test]
        fn test_project_detection() {
            let temp_dir = TempDir::new().unwrap();
            let project_dir = temp_dir.path().join("test_project");
            std::fs::create_dir_all(&project_dir).unwrap();

            // Not a project yet
            assert!(!is_rust_project(&project_dir));
            assert!(!is_xylux_project(&project_dir));

            // Make it a Rust project
            std::fs::write(project_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
            assert!(is_rust_project(&project_dir));

            // Make it also a Xylux project
            std::fs::write(project_dir.join("xylux.toml"), "[project]\nname = \"test\"").unwrap();
            assert!(is_xylux_project(&project_dir));
        }

        #[test]
        fn test_find_project_root() {
            let temp_dir = TempDir::new().unwrap();
            let project_dir = temp_dir.path().join("project");
            let sub_dir = project_dir.join("src").join("deep");
            std::fs::create_dir_all(&sub_dir).unwrap();

            // Create Cargo.toml in project root
            std::fs::write(project_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

            // Should find project root from subdirectory
            let found_root = find_project_root(&sub_dir).unwrap();
            assert_eq!(found_root, project_dir);

            // Should return None for non-project directory
            let non_project = temp_dir.path().join("not_a_project");
            std::fs::create_dir_all(&non_project).unwrap();
            assert!(find_project_root(&non_project).is_none());
        }

        #[test]
        fn test_escape_text() {
            assert_eq!(escape_text("hello"), "hello");
            assert_eq!(escape_text("hello\tworld"), "hello→world");
            assert_eq!(escape_text("line1\nline2"), "line1⏎line2");
            assert_eq!(escape_text("with\rcarriage"), "with↵carriage");
        }
    }
}

// Error handling helpers - From implementations moved to core/error.rs
impl From<std::fmt::Error> for XyluxError {
    fn from(_: std::fmt::Error) -> Self {
        Self::critical("Formatting error occurred")
    }
}

// Version and build information - Display implementation moved to core/mod.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_build_info_display() {
        let info_str = format!("{}", BUILD_INFO);
        assert!(info_str.contains("Xylux IDE"));
        assert!(info_str.contains(VERSION));
    }

    #[test]
    fn test_features() {
        // These should not panic
        features::has_clipboard();
        features::has_network();
        features::has_debug();
        features::available_language_servers();
        features::available_build_tools();
    }
}
