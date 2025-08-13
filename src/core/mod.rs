//! # Core Module
//!
//! Core functionality and shared components for Xylux IDE.

pub mod config;
pub mod error;
pub mod events;

pub use config::{Config, ConfigLoader};
pub use error::{Result, ResultExt, XyluxError};
pub use events::{
    AluxEvent, BuildEvent, EditorEvent, Event, EventBus, EventHandler, EventMessage, EventPriority,
    EventSubscription, FileSystemEvent, LspEvent, PluginEvent, ProjectEvent, SystemEvent, UiEvent,
    XyluxEvent,
};

/// Version information for Xylux IDE.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information.
pub const BUILD_INFO: BuildInfo = BuildInfo {
    version: VERSION,
    git_hash: option_env!("GIT_HASH"),
    build_date: option_env!("BUILD_DATE"),
    target: env!("TARGET"),
    profile: if cfg!(debug_assertions) { "debug" } else { "release" },
};

/// Build information structure.
#[derive(Debug, Clone)]
pub struct BuildInfo {
    /// Version string of the IDE.
    pub version: &'static str,
    /// Git commit hash if available.
    pub git_hash: Option<&'static str>,
    /// Build date if available.
    pub build_date: Option<&'static str>,
    /// Target architecture.
    pub target: &'static str,
    /// Build profile (debug/release).
    pub profile: &'static str,
}

impl std::fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Xylux IDE {}", self.version)?;

        if let Some(git_hash) = self.git_hash {
            write!(f, " ({})", &git_hash[..8.min(git_hash.len())])?;
        }

        if let Some(build_date) = self.build_date {
            write!(f, " built on {}", build_date)?;
        }

        write!(f, " [{}] [{}]", self.target, self.profile)?;

        Ok(())
    }
}

/// Initialize the core systems.
pub async fn initialize() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Initializing Xylux IDE core systems");
    tracing::info!("{}", BUILD_INFO);

    Ok(())
}

/// Shutdown the core systems.
pub async fn shutdown() -> Result<()> {
    tracing::info!("Shutting down Xylux IDE core systems");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info() {
        let build_info = BUILD_INFO;
        assert_eq!(build_info.version, VERSION);
        assert!(!build_info.target.is_empty());
        assert!(build_info.profile == "debug" || build_info.profile == "release");

        // Test display formatting
        let display_str = format!("{}", build_info);
        assert!(display_str.contains("Xylux IDE"));
        assert!(display_str.contains(VERSION));
    }

    #[tokio::test]
    async fn test_initialize_and_shutdown() {
        // These should not panic
        initialize().await.unwrap();
        shutdown().await.unwrap();
    }
}
