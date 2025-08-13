//! # WASI Platform Implementation
//!
//! WebAssembly System Interface (WASI) platform functionality for Xylux IDE.

use std::env;
use std::path::PathBuf;

use crate::core::{Result, XyluxError};
use crate::platform::Platform;

/// WASI platform implementation.
pub struct WasiPlatform;

impl WasiPlatform {
    /// Create a new WASI platform instance.
    pub fn new() -> Self {
        Self
    }
}

impl Platform for WasiPlatform {
    fn name(&self) -> &'static str {
        "wasi"
    }

    fn config_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();

        // Try to use XDG-style directories even in WASI
        if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
            if let Some(path) = config_home.to_str() {
                dirs.push(format!("{}/xylux-ide", path));
            }
        } else {
            // Fallback to a reasonable default
            dirs.push("/config/xylux-ide".to_string());
        }

        dirs
    }

    fn data_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();

        // Try to use XDG-style directories
        if let Some(data_home) = env::var_os("XDG_DATA_HOME") {
            if let Some(path) = data_home.to_str() {
                dirs.push(format!("{}/xylux-ide", path));
            }
        } else {
            // Fallback to reasonable defaults
            dirs.push("/data/xylux-ide".to_string());
            dirs.push("/usr/share/xylux-ide".to_string());
        }

        dirs
    }

    fn cache_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();

        // Try to use XDG-style cache directory
        if let Some(cache_home) = env::var_os("XDG_CACHE_HOME") {
            if let Some(path) = cache_home.to_str() {
                dirs.push(format!("{}/xylux-ide", path));
            }
        } else {
            // Fallback to a reasonable default
            dirs.push("/cache/xylux-ide".to_string());
        }

        dirs
    }

    fn home_dir(&self) -> Option<String> {
        // WASI doesn't have a traditional home directory concept
        // Try common environment variables
        env::var_os("HOME")
            .or_else(|| env::var_os("PWD"))
            .and_then(|path| path.to_str().map(String::from))
            .or_else(|| Some("/".to_string())) // Fallback to root
    }

    fn env_var(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }

    fn set_env_var(&self, key: &str, value: &str) -> Result<()> {
        env::set_var(key, value);
        Ok(())
    }

    fn command_exists(&self, command: &str) -> bool {
        // WASI has limited command execution capabilities
        // For now, assume common tools might be available
        matches!(command, "rust-analyzer" | "cargo" | "wasm-pack")
    }

    fn current_dir(&self) -> Result<String> {
        // WASI doesn't have a traditional current directory
        // Try PWD environment variable first
        if let Some(pwd) = env::var_os("PWD") {
            if let Some(path) = pwd.to_str() {
                return Ok(path.to_string());
            }
        }

        // Try std::env::current_dir() as fallback
        env::current_dir()
            .map_err(|e| XyluxError::io(e, "Failed to get current directory"))
            .and_then(|path| {
                path.to_str()
                    .ok_or_else(|| {
                        XyluxError::invalid_data("Current directory path is not valid UTF-8")
                    })
                    .map(String::from)
            })
            .or_else(|_| {
                // Final fallback to root
                Ok("/".to_string())
            })
    }

    fn path_separator(&self) -> char {
        '/'
    }

    fn line_separator(&self) -> &'static str {
        "\n"
    }
}

impl Default for WasiPlatform {
    fn default() -> Self {
        Self::new()
    }
}

// Terminal-specific functionality for WASI

/// Terminal mode placeholder for WASI.
pub struct TermMode;

/// Return the current window size as (rows, columns).
/// WASI doesn't have terminal size detection, so we return a reasonable default.
pub fn get_window_size() -> Result<(usize, usize)> {
    // Return a reasonable default size for WASI environments
    Ok((24, 80))
}

/// Register a signal handler for window size changes.
/// This is a no-op on WASI as signals aren't supported.
pub fn register_winsize_change_signal_handler() -> Result<()> {
    // WASI doesn't support signals
    Ok(())
}

/// Check if the window size has changed.
/// Always returns false on WASI.
pub fn has_window_size_changed() -> bool {
    // WASI doesn't have signal-based window size detection
    false
}

/// Set the terminal mode.
/// This is a no-op on WASI.
pub fn set_term_mode(_term: &TermMode) -> Result<()> {
    // WASI doesn't have terminal mode control
    Ok(())
}

/// Enable raw mode and return the original terminal mode.
/// This is a no-op on WASI, returning a placeholder.
pub fn enable_raw_mode() -> Result<TermMode> {
    // WASI doesn't have raw mode concept
    Ok(TermMode)
}

/// Get stdin handle.
/// On WASI, we try to open /dev/tty as a fallback.
pub fn stdin() -> std::io::Result<std::fs::File> {
    std::fs::File::open("/dev/tty")
}

/// Create a path from filename, handling WASI-specific path resolution.
pub fn path(filename: &str) -> PathBuf {
    // If the filename is absolute (starts with '/'), use it as-is
    if filename.starts_with('/') {
        PathBuf::from(filename)
    } else {
        // For relative paths, join with current directory
        // WASI doesn't have a standard current directory concept,
        // so we use PWD environment variable or fallback to root
        let base_dir = env::var_os("PWD")
            .and_then(|path| path.to_str().map(String::from))
            .unwrap_or_else(|| "/".to_string());

        PathBuf::from(base_dir).join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_implementation() {
        let platform = WasiPlatform::new();

        assert_eq!(platform.name(), "wasi");
        assert_eq!(platform.path_separator(), '/');
        assert_eq!(platform.line_separator(), "\n");

        // Test directory functions
        let config_dirs = platform.config_dirs();
        assert!(!config_dirs.is_empty());

        let data_dirs = platform.data_dirs();
        assert!(!data_dirs.is_empty());

        // Test current dir (should not fail)
        let current = platform.current_dir();
        assert!(current.is_ok());
    }

    #[test]
    fn test_window_size() {
        let size = get_window_size();
        assert!(size.is_ok());
        let (rows, cols) = size.unwrap();
        assert_eq!(rows, 24);
        assert_eq!(cols, 80);
    }

    #[test]
    fn test_signal_handlers() {
        // Should not fail
        let result = register_winsize_change_signal_handler();
        assert!(result.is_ok());

        // Should always return false
        assert!(!has_window_size_changed());
    }

    #[test]
    fn test_terminal_mode() {
        // Should not fail
        let mode_result = enable_raw_mode();
        assert!(mode_result.is_ok());

        let mode = mode_result.unwrap();
        let set_result = set_term_mode(&mode);
        assert!(set_result.is_ok());
    }

    #[test]
    fn test_path_resolution() {
        // Test absolute path
        let abs_path = path("/absolute/path/file.txt");
        assert_eq!(abs_path.to_str().unwrap(), "/absolute/path/file.txt");

        // Test relative path (result depends on PWD env var)
        let rel_path = path("relative/file.txt");
        assert!(rel_path.to_str().unwrap().ends_with("relative/file.txt"));
    }

    #[test]
    fn test_home_dir() {
        let platform = WasiPlatform::new();
        let home = platform.home_dir();

        // Should have some home directory (at least "/" as fallback)
        assert!(home.is_some());
        assert!(!home.unwrap().is_empty());
    }

    #[test]
    fn test_command_exists() {
        let platform = WasiPlatform::new();

        // Known commands that might be available
        assert!(platform.command_exists("rust-analyzer"));
        assert!(platform.command_exists("cargo"));
        assert!(platform.command_exists("wasm-pack"));

        // Unknown command
        assert!(!platform.command_exists("definitely_not_a_real_command"));
    }
}
