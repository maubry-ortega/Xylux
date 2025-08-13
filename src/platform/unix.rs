//! # Unix Platform Implementation
//!
//! Unix-specific platform functionality for Xylux IDE.

#![allow(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

use libc::{SA_SIGINFO, STDIN_FILENO, STDOUT_FILENO, TCSADRAIN, TIOCGWINSZ, VMIN, VTIME};
use libc::{c_int, c_void, sigaction, sighandler_t, siginfo_t, winsize};

use crate::core::{Result, XyluxError};
use crate::platform::Platform;

// Re-export termios as TermMode for compatibility
pub use libc::termios as TermMode;

/// Unix platform implementation.
pub struct UnixPlatform;

impl UnixPlatform {
    /// Create a new Unix platform instance.
    pub fn new() -> Self {
        Self
    }
}

impl Platform for UnixPlatform {
    fn name(&self) -> &'static str {
        "unix"
    }

    fn config_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();

        // XDG_CONFIG_HOME or ~/.config
        if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
            if let Some(path) = config_home.to_str() {
                dirs.push(format!("{}/xylux-ide", path));
            }
        } else if let Some(home) = env::var_os("HOME") {
            if let Some(path) = home.to_str() {
                dirs.push(format!("{}/.config/xylux-ide", path));
            }
        }

        // System-wide config
        dirs.push("/etc/xylux-ide".to_string());

        dirs
    }

    fn data_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();

        // XDG_DATA_HOME or ~/.local/share
        if let Some(data_home) = env::var_os("XDG_DATA_HOME") {
            if let Some(path) = data_home.to_str() {
                dirs.push(format!("{}/xylux-ide", path));
            }
        } else if let Some(home) = env::var_os("HOME") {
            if let Some(path) = home.to_str() {
                dirs.push(format!("{}/.local/share/xylux-ide", path));
            }
        }

        // XDG_DATA_DIRS or system defaults
        if let Some(data_dirs) = env::var_os("XDG_DATA_DIRS") {
            if let Some(paths) = data_dirs.to_str() {
                for path in paths.split(':') {
                    if !path.is_empty() {
                        dirs.push(format!("{}/xylux-ide", path));
                    }
                }
            }
        } else {
            dirs.push("/usr/local/share/xylux-ide".to_string());
            dirs.push("/usr/share/xylux-ide".to_string());
        }

        dirs
    }

    fn cache_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();

        // XDG_CACHE_HOME or ~/.cache
        if let Some(cache_home) = env::var_os("XDG_CACHE_HOME") {
            if let Some(path) = cache_home.to_str() {
                dirs.push(format!("{}/xylux-ide", path));
            }
        } else if let Some(home) = env::var_os("HOME") {
            if let Some(path) = home.to_str() {
                dirs.push(format!("{}/.cache/xylux-ide", path));
            }
        }

        dirs
    }

    fn home_dir(&self) -> Option<String> {
        env::var_os("HOME").and_then(|path| path.to_str().map(String::from))
    }

    fn env_var(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }

    fn set_env_var(&self, key: &str, value: &str) -> Result<()> {
        env::set_var(key, value);
        Ok(())
    }

    fn command_exists(&self, command: &str) -> bool {
        which::which(command).is_ok()
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
        '/'
    }

    fn line_separator(&self) -> &'static str {
        "\n"
    }
}

impl Default for UnixPlatform {
    fn default() -> Self {
        Self::new()
    }
}

// Terminal-specific functionality
fn cerr(err: c_int) -> Result<()> {
    match err {
        0..=c_int::MAX => Ok(()),
        _ => Err(XyluxError::io(std::io::Error::last_os_error(), "System call failed")),
    }
}

/// Return the current window size as (rows, columns).
///
/// We use the `TIOCGWINSZ` ioctl to get window size. If it succeeds, a
/// `winsize` struct will be populated.
/// This ioctl is described here: <http://man7.org/linux/man-pages/man4/tty_ioctl.4.html>
pub fn get_window_size() -> Result<(usize, usize)> {
    let mut maybe_ws = std::mem::MaybeUninit::<winsize>::uninit();
    cerr(unsafe { libc::ioctl(STDOUT_FILENO, TIOCGWINSZ, maybe_ws.as_mut_ptr()) })
        .map_or(None, |()| unsafe { Some(maybe_ws.assume_init()) })
        .filter(|ws| ws.ws_col != 0 && ws.ws_row != 0)
        .map_or(Err(XyluxError::platform("Invalid window size")), |ws| {
            Ok((ws.ws_row as usize, ws.ws_col as usize))
        })
}

/// Stores whether the window size has changed since last call to
/// `has_window_size_changed`.
static WSC: AtomicBool = AtomicBool::new(false);

/// Handle a change in window size.
extern "C" fn handle_wsize(_: c_int, _: *mut siginfo_t, _: *mut c_void) {
    WSC.store(true, Relaxed);
}

#[allow(clippy::fn_to_numeric_cast_any)]
/// Register a signal handler that sets a global variable when the window size
/// changes. After calling this function, use `has_window_size_changed` to query
/// the global variable.
pub fn register_winsize_change_signal_handler() -> Result<()> {
    unsafe {
        let mut maybe_sa = std::mem::MaybeUninit::<sigaction>::uninit();
        cerr(libc::sigemptyset(&mut (*maybe_sa.as_mut_ptr()).sa_mask))?;
        // We could use sa_handler here, however, sigaction defined in libc does not
        // have sa_handler field, so we use sa_sigaction instead.
        (*maybe_sa.as_mut_ptr()).sa_flags = SA_SIGINFO;
        (*maybe_sa.as_mut_ptr()).sa_sigaction = handle_wsize as sighandler_t;
        cerr(sigaction(libc::SIGWINCH, maybe_sa.as_ptr(), std::ptr::null_mut()))
    }
}

/// Check if the window size has changed since the last call to this function.
/// The `register_winsize_change_signal_handler` needs to be called before this
/// function.
pub fn has_window_size_changed() -> bool {
    WSC.swap(false, Relaxed)
}

/// Set the terminal mode.
pub fn set_term_mode(term: &TermMode) -> Result<()> {
    cerr(unsafe { libc::tcsetattr(STDIN_FILENO, TCSADRAIN, term) })
}

/// Setup the termios to enable raw mode, and return the original termios.
///
/// termios manual is available at: <http://man7.org/linux/man-pages/man3/termios.3.html>
pub fn enable_raw_mode() -> Result<TermMode> {
    let mut maybe_term = std::mem::MaybeUninit::<TermMode>::uninit();
    cerr(unsafe { libc::tcgetattr(STDIN_FILENO, maybe_term.as_mut_ptr()) })?;
    let orig_term = unsafe { maybe_term.assume_init() };
    let mut term = orig_term;
    unsafe { libc::cfmakeraw(&mut term) };
    // First sets the minimum number of characters for non-canonical reads
    // Second sets the timeout in deciseconds for non-canonical reads
    (term.c_cc[VMIN], term.c_cc[VTIME]) = (0, 1);
    set_term_mode(&term)?;
    Ok(orig_term)
}

/// Get stdin handle.
pub fn stdin() -> std::io::Result<std::io::Stdin> {
    Ok(std::io::stdin())
}

/// Create a path from filename.
pub fn path(filename: &str) -> PathBuf {
    PathBuf::from(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_implementation() {
        let platform = UnixPlatform::new();

        assert_eq!(platform.name(), "unix");
        assert_eq!(platform.path_separator(), '/');
        assert_eq!(platform.line_separator(), "\n");

        // Test directory functions
        let config_dirs = platform.config_dirs();
        assert!(!config_dirs.is_empty());

        let data_dirs = platform.data_dirs();
        assert!(!data_dirs.is_empty());

        // Test current dir
        let current = platform.current_dir();
        assert!(current.is_ok());
    }

    #[test]
    fn test_window_size() {
        // This might fail in CI environments without a terminal
        let _unused = get_window_size();
    }

    #[test]
    fn test_signal_handler() {
        // Test that we can register the signal handler without panic
        let _unused = register_winsize_change_signal_handler();
    }
}
