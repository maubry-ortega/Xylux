//! # Windows Platform Implementation
//!
//! Windows-specific platform functionality for Xylux IDE.

#![allow(clippy::wildcard_imports)]

use std::{env, io};  // Eliminada la importación redundante de TryInto

use winapi::um::wincon::*;
use winapi_util::{HandleRef, console as cons};

use crate::core::{Result, XyluxError};
use crate::platform::Platform;

// On Windows systems, the terminal mode is represented as 2 unsigned integers
// (one for stdin, one for stdout).
pub type TermMode = (u32, u32);

/// Windows platform implementation.
pub struct WindowsPlatform;

impl WindowsPlatform {
    /// Create a new Windows platform instance.
    pub fn new() -> Self {
        Self
    }
}

impl Platform for WindowsPlatform {
    fn name(&self) -> &'static str {
        "windows"
    }

    fn config_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();

        // APPDATA/xylux-ide
        if let Some(appdata) = env::var_os("APPDATA") {
            if let Some(path) = appdata.to_str() {
                dirs.push(format!("{}\\xylux-ide", path));
            }
        }

        // LOCALAPPDATA/xylux-ide
        if let Some(local_appdata) = env::var_os("LOCALAPPDATA") {
            if let Some(path) = local_appdata.to_str() {
                dirs.push(format!("{}\\xylux-ide", path));
            }
        }

        // ProgramData/xylux-ide (system-wide)
        if let Some(program_data) = env::var_os("ProgramData") {
            if let Some(path) = program_data.to_str() {
                dirs.push(format!("{}\\xylux-ide", path));
            }
        }

        dirs
    }

    fn data_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();

        // LOCALAPPDATA/xylux-ide
        if let Some(local_appdata) = env::var_os("LOCALAPPDATA") {
            if let Some(path) = local_appdata.to_str() {
                dirs.push(format!("{}\\xylux-ide", path));
            }
        }

        // APPDATA/xylux-ide
        if let Some(appdata) = env::var_os("APPDATA") {
            if let Some(path) = appdata.to_str() {
                dirs.push(format!("{}\\xylux-ide", path));
            }
        }

        // ProgramData/xylux-ide (system-wide)
        if let Some(program_data) = env::var_os("ProgramData") {
            if let Some(path) = program_data.to_str() {
                dirs.push(format!("{}\\xylux-ide", path));
            }
        }

        dirs
    }

    fn cache_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();

        // LOCALAPPDATA/xylux-ide/cache
        if let Some(local_appdata) = env::var_os("LOCALAPPDATA") {
            if let Some(path) = local_appdata.to_str() {
                dirs.push(format!("{}\\xylux-ide\\cache", path));
            }
        }

        // TEMP/xylux-ide
        if let Some(temp) = env::var_os("TEMP") {
            if let Some(path) = temp.to_str() {
                dirs.push(format!("{}\\xylux-ide", path));
            }
        }

        dirs
    }

    fn home_dir(&self) -> Option<String> {
        env::var_os("USERPROFILE")
            .or_else(|| {
                // Fallback to HOMEDRIVE + HOMEPATH
                env::var_os("HOMEDRIVE").and_then(|drive| {
                    env::var_os("HOMEPATH").map(|path| {
                        let mut full_path = drive;
                        full_path.push(path);
                        full_path
                    })
                })
            })
            .and_then(|path| path.to_str().map(String::from))
    }

    fn env_var(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }

    fn set_env_var(&self, key: &str, value: &str) -> Result<()> {
        unsafe {
            env::set_var(key, value);
        }
        Ok(())
    }

    fn command_exists(&self, command: &str) -> bool {
        // Try with .exe extension first
        let exe_command = if command.ends_with(".exe") {
            command.to_string()
        } else {
            format!("{}.exe", command)
        };

        which::which(&exe_command).is_ok() || which::which(command).is_ok()
    }

    fn current_dir(&self) -> Result<String> {
        env::current_dir()
            .map_err(|e| XyluxError::io(e, "Failed to get current directory"))
            .and_then(|path| {
                path.to_str()
                    .ok_or_else(|| {
                        XyluxError::invalid_data("Current directory path is not valid UTF-8")
                    })
                    .map(String::from)
            })
    }

    fn path_separator(&self) -> char {
        '\\'
    }

    fn line_separator(&self) -> &'static str {
        "\r\n"
    }
}

impl Default for WindowsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

// Terminal-specific functionality

/// Return the current window size as (rows, columns).
pub fn get_window_size() -> Result<(usize, usize)> {
    let rect = cons::screen_buffer_info(HandleRef::stdout())
        .map_err(|e| XyluxError::io(e, "Failed to get screen buffer info"))?
        .window_rect();

    match ((rect.bottom - rect.top + 1).try_into(), (rect.right - rect.left + 1).try_into()) {
        (Ok(rows), Ok(cols)) => Ok((rows, cols)),
        _ => Err(XyluxError::platform("Invalid window size")),
    }
}

/// Register signal handler for window size changes (no-op on Windows).
pub fn register_winsize_change_signal_handler() -> Result<()> {
    // Windows doesn't use signals in the same way as Unix
    Ok(())
}

/// Check if window size has changed (always false on Windows).
pub fn has_window_size_changed() -> bool {
    // Windows doesn't have the same signal-based mechanism
    false
}

/// Set the terminal mode.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn set_term_mode((stdin_mode, stdout_mode): &TermMode) -> Result<()> {
    cons::set_mode(HandleRef::stdin(), *stdin_mode)
        .map_err(|e| XyluxError::io(e, "Failed to set stdin mode"))?;
    cons::set_mode(HandleRef::stdout(), *stdout_mode)
        .map_err(|e| XyluxError::io(e, "Failed to set stdout mode"))?;
    Ok(())
}

/// Enable raw mode, and return the original terminal mode.
///
/// Documentation for console modes is available at:
/// <https://docs.microsoft.com/en-us/windows/console/setconsolemode>
pub fn enable_raw_mode() -> Result<TermMode> {
    // Get original terminal modes
    let mode_in0 = cons::mode(HandleRef::stdin())
        .map_err(|e| XyluxError::io(e, "Failed to get stdin mode"))?;
    let mode_out0 = cons::mode(HandleRef::stdout())
        .map_err(|e| XyluxError::io(e, "Failed to get stdout mode"))?;

    // Calculate new terminal modes
    let mode_in = (mode_in0 | ENABLE_VIRTUAL_TERMINAL_INPUT)
        & !(ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);
    let mode_out = (mode_out0 | ENABLE_VIRTUAL_TERMINAL_PROCESSING)
        | (DISABLE_NEWLINE_AUTO_RETURN | ENABLE_PROCESSED_OUTPUT);

    set_term_mode(&(mode_in, mode_out))?;
    Ok((mode_in0, mode_out0))
}

/// Get stdin handle.
pub fn stdin() -> io::Result<io::Stdin> {
    Ok(io::stdin())
}

/// Create a path from filename.
pub fn path(filename: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_implementation() {
        let platform = WindowsPlatform::new();

        assert_eq!(platform.name(), "windows");
        assert_eq!(platform.path_separator(), '\\');
        assert_eq!(platform.line_separator(), "\r\n");

        // Test directory functions
        let config_dirs = platform.config_dirs();
        // Should have at least one directory on most Windows systems
        assert!(!config_dirs.is_empty());

        let data_dirs = platform.data_dirs();
        assert!(!data_dirs.is_empty());

        // Test current dir
        let current = platform.current_dir();
        assert!(current.is_ok());
    }

    #[test]
    fn test_command_exists() {
        let platform = WindowsPlatform::new();

        // cmd.exe should exist on all Windows systems
        assert!(platform.command_exists("cmd"));
        assert!(platform.command_exists("cmd.exe"));

        // This command should not exist
        assert!(!platform.command_exists("definitely_not_a_real_command_12345"));
    }

    #[test]
    fn test_home_dir() {
        let platform = WindowsPlatform::new();
        let home = platform.home_dir();

        // Should have a home directory on Windows
        assert!(home.is_some());
        if let Some(home_path) = home {
            assert!(!home_path.is_empty());
        }
    }

    #[test]
    fn test_window_size() {
        // This might fail in CI environments without a console
        let _ = get_window_size();
    }

    #[test]
    fn test_signal_handler() {
        // Should not fail on Windows
        let result = register_winsize_change_signal_handler();
        assert!(result.is_ok());

        // Should always return false on Windows
        assert!(!has_window_size_changed());
    }
}
