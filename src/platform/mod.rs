//! # Platform Module
//!
//! Platform-specific implementations for Xylux IDE.

#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "wasi")]
pub mod wasi;

use crate::core::Result;

/// Platform-specific functionality trait.
pub trait Platform {
    /// Get the platform name.
    fn name(&self) -> &'static str;

    /// Get configuration directories.
    fn config_dirs(&self) -> Vec<String>;

    /// Get data directories.
    fn data_dirs(&self) -> Vec<String>;

    /// Get cache directories.
    fn cache_dirs(&self) -> Vec<String>;

    /// Get the home directory.
    fn home_dir(&self) -> Option<String>;

    /// Get environment variable.
    fn env_var(&self, key: &str) -> Option<String>;

    /// Set environment variable.
    fn set_env_var(&self, key: &str, value: &str) -> Result<()>;

    /// Check if a command exists in PATH.
    fn command_exists(&self, command: &str) -> bool;

    /// Get the current working directory.
    fn current_dir(&self) -> Result<String>;

    /// Get the path separator.
    fn path_separator(&self) -> char;

    /// Get the line separator.
    fn line_separator(&self) -> &'static str;
}

/// Get the current platform implementation.
pub fn current_platform() -> Box<dyn Platform> {
    #[cfg(unix)]
    {
        Box::new(unix::UnixPlatform::new())
    }
    #[cfg(windows)]
    {
        Box::new(windows::WindowsPlatform::new())
    }
    #[cfg(target_os = "wasi")]
    {
        Box::new(wasi::WasiPlatform::new())
    }
    #[cfg(not(any(unix, windows, target_os = "wasi")))]
    {
        compile_error!("Unsupported platform")
    }
}

/// Get platform-specific configuration directories.
pub fn config_dirs() -> Vec<String> {
    current_platform().config_dirs()
}

/// Get platform-specific data directories.
pub fn data_dirs() -> Vec<String> {
    current_platform().data_dirs()
}

/// Check if a command exists on the current platform.
pub fn command_exists(command: &str) -> bool {
    current_platform().command_exists(command)
}

/// Get the current working directory.
pub fn current_dir() -> Result<String> {
    current_platform().current_dir()
}

/// Platform-specific constants.
pub mod constants {
    #[cfg(unix)]
    pub const PATH_SEPARATOR: char = '/';
    #[cfg(windows)]
    pub const PATH_SEPARATOR: char = '\\';
    #[cfg(target_os = "wasi")]
    pub const PATH_SEPARATOR: char = '/';

    #[cfg(unix)]
    pub const LINE_SEPARATOR: &str = "\n";
    #[cfg(windows)]
    pub const LINE_SEPARATOR: &str = "\r\n";
    #[cfg(target_os = "wasi")]
    pub const LINE_SEPARATOR: &str = "\n";

    #[cfg(unix)]
    pub const EXECUTABLE_EXTENSION: &str = "";
    #[cfg(windows)]
    pub const EXECUTABLE_EXTENSION: &str = ".exe";
    #[cfg(target_os = "wasi")]
    pub const EXECUTABLE_EXTENSION: &str = ".wasm";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_creation() {
        let platform = current_platform();
        assert!(!platform.name().is_empty());
    }

    #[test]
    fn test_config_dirs() {
        let dirs = config_dirs();
        // Should return at least one directory on most platforms
        #[cfg(not(target_os = "wasi"))]
        assert!(!dirs.is_empty());
    }

    #[test]
    fn test_current_dir() {
        let dir = current_dir();
        assert!(dir.is_ok());
        assert!(!dir.unwrap().is_empty());
    }

    #[test]
    fn test_constants() {
        use constants::*;

        #[cfg(unix)]
        {
            assert_eq!(PATH_SEPARATOR, '/');
            assert_eq!(LINE_SEPARATOR, "\n");
            assert_eq!(EXECUTABLE_EXTENSION, "");
        }

        #[cfg(windows)]
        {
            assert_eq!(PATH_SEPARATOR, '\\');
            assert_eq!(LINE_SEPARATOR, "\r\n");
            assert_eq!(EXECUTABLE_EXTENSION, ".exe");
        }
    }
}
