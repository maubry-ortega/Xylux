//! # Configuration System
//!
//! Enhanced configuration system for Xylux IDE supporting multiple formats
//! and hierarchical configuration loading.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::core::error::{Result, XyluxError};

/// Main configuration structure for Xylux IDE.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Editor configuration.
    pub editor: EditorConfig,
    /// UI configuration.
    pub ui: UiConfig,
    /// Language Server Protocol configuration.
    pub lsp: LspConfig,
    /// Project management configuration.
    pub project: ProjectConfig,
    /// Build system configuration.
    pub build: BuildConfig,
    /// Alux language configuration.
    pub alux: AluxConfig,
    /// Xylux engine integration configuration.
    pub xylux: XyluxConfig,
    /// Plugin configuration.
    pub plugins: PluginConfig,
    /// Advanced settings.
    pub advanced: AdvancedConfig,
}

/// Editor-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorConfig {
    /// Tab size in spaces.
    pub tab_size: usize,
    /// Whether to use spaces instead of tabs.
    pub use_spaces: bool,
    /// Whether to show line numbers.
    pub show_line_numbers: bool,
    /// Whether to show relative line numbers.
    pub show_relative_line_numbers: bool,
    /// Whether to highlight the current line.
    pub highlight_current_line: bool,
    /// Whether to show whitespace characters.
    pub show_whitespace: bool,
    /// Word wrap configuration.
    pub word_wrap: bool,
    /// Auto-save configuration (in seconds, 0 to disable).
    pub auto_save_interval: u64,
    /// Whether to auto-format on save.
    pub format_on_save: bool,
    /// Maximum number of recent files to remember.
    pub max_recent_files: usize,
    /// Scrolloff - minimum lines to keep above/below cursor.
    pub scroll_offset: usize,
    /// Whether to enable bracket matching.
    pub bracket_matching: bool,
    /// Whether to auto-close brackets.
    pub auto_close_brackets: bool,
}

/// UI configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiConfig {
    /// Color theme name.
    pub theme: String,
    /// Font family.
    pub font_family: String,
    /// Font size.
    pub font_size: u16,
    /// Whether to show the file explorer panel.
    pub show_file_explorer: bool,
    /// Whether to show the terminal panel.
    pub show_terminal: bool,
    /// Whether to show the minimap.
    pub show_minimap: bool,
    /// Status bar configuration.
    pub status_bar: StatusBarConfig,
    /// Message display duration.
    pub message_duration: Duration,
    /// Window transparency (0.0 to 1.0).
    pub transparency: f32,
}

/// Status bar configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusBarConfig {
    /// Whether to show cursor position.
    pub show_cursor_position: bool,
    /// Whether to show file encoding.
    pub show_encoding: bool,
    /// Whether to show line endings.
    pub show_line_endings: bool,
    /// Whether to show file type.
    pub show_file_type: bool,
    /// Whether to show Git branch.
    pub show_git_branch: bool,
    /// Whether to show LSP status.
    pub show_lsp_status: bool,
}

/// Language Server Protocol configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LspConfig {
    /// Whether LSP is enabled.
    pub enabled: bool,
    /// Rust analyzer configuration.
    pub rust_analyzer: RustAnalyzerConfig,
    /// Alux LSP configuration.
    pub alux_lsp: AluxLspConfig,
    /// Timeout for LSP requests (in milliseconds).
    pub request_timeout_ms: u64,
    /// Whether to show LSP diagnostics inline.
    pub show_diagnostics_inline: bool,
    /// Whether to show hover information.
    pub show_hover: bool,
    /// Whether to enable code completion.
    pub enable_completion: bool,
}

/// Rust analyzer specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RustAnalyzerConfig {
    /// Path to rust-analyzer binary (None for auto-detect).
    pub binary_path: Option<PathBuf>,
    /// Additional command line arguments.
    pub args: Vec<String>,
    /// Whether to enable proc macro support.
    pub enable_proc_macros: bool,
    /// Cargo features to enable.
    pub cargo_features: Vec<String>,
    /// Whether to run cargo check on save.
    pub check_on_save: bool,
}

/// Alux LSP configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AluxLspConfig {
    /// Path to alux-lsp binary (None for auto-detect).
    pub binary_path: Option<PathBuf>,
    /// Whether to enable semantic highlighting.
    pub semantic_highlighting: bool,
    /// Whether to enable inline hints.
    pub inline_hints: bool,
}

/// Project management configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectConfig {
    /// Whether to auto-detect project type.
    pub auto_detect_type: bool,
    /// Default project templates.
    pub default_templates: HashMap<String, PathBuf>,
    /// Whether to watch for file changes.
    pub file_watching: bool,
    /// Patterns to ignore when watching files.
    pub ignore_patterns: Vec<String>,
    /// Whether to auto-reload changed files.
    pub auto_reload_files: bool,
    /// Git integration settings.
    pub git: GitConfig,
}

/// Git integration configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitConfig {
    /// Whether Git integration is enabled.
    pub enabled: bool,
    /// Whether to show Git status in file explorer.
    pub show_status_in_explorer: bool,
    /// Whether to show Git blame information.
    pub show_blame: bool,
}

/// Build system configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildConfig {
    /// Default build command for Rust projects.
    pub rust_build_command: String,
    /// Default run command for Rust projects.
    pub rust_run_command: String,
    /// Default test command for Rust projects.
    pub rust_test_command: String,
    /// Xylux CLI path (None for auto-detect).
    pub xylux_cli_path: Option<PathBuf>,
    /// Whether to show build output in terminal.
    pub show_build_output: bool,
    /// Whether to auto-build on save.
    pub auto_build_on_save: bool,
    /// Build environment variables.
    pub env_vars: HashMap<String, String>,
}

/// Alux language configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AluxConfig {
    /// Path to Alux compiler (None for auto-detect).
    pub compiler_path: Option<PathBuf>,
    /// Path to Alux VM (None for auto-detect).
    pub vm_path: Option<PathBuf>,
    /// Whether to enable hot reload for Alux scripts.
    pub hot_reload: bool,
    /// Alux VM log level.
    pub vm_log_level: String,
    /// Whether to compile to bytecode automatically.
    pub auto_compile_bytecode: bool,
    /// Optimization level for Alux compilation.
    pub optimization_level: u8,
}

/// Xylux engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct XyluxConfig {
    /// Path to Xylux engine (None for auto-detect).
    pub engine_path: Option<PathBuf>,
    /// Default target platform.
    pub default_target: String,
    /// Whether to enable headless mode for testing.
    pub headless_testing: bool,
    /// WebAssembly compilation settings.
    pub wasm: WasmConfig,
    /// Shader development settings.
    pub shaders: ShaderConfig,
}

/// WebAssembly compilation configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WasmConfig {
    /// Whether WASM compilation is enabled.
    pub enabled: bool,
    /// Whether to optimize WASM output.
    pub optimize: bool,
    /// Path to wasm-pack (None for auto-detect).
    pub wasm_pack_path: Option<PathBuf>,
}

/// Shader development configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShaderConfig {
    /// Whether to enable shader hot reload.
    pub hot_reload: bool,
    /// Whether to validate shaders on save.
    pub validate_on_save: bool,
    /// Shader optimization level.
    pub optimization_level: u8,
}

/// Plugin system configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginConfig {
    /// Whether plugins are enabled.
    pub enabled: bool,
    /// Directories to search for plugins.
    pub plugin_directories: Vec<PathBuf>,
    /// List of enabled plugins.
    pub enabled_plugins: Vec<String>,
    /// Plugin-specific configurations.
    pub plugin_configs: HashMap<String, serde_json::Value>,
}

/// Advanced configuration options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdvancedConfig {
    /// Number of quit confirmations needed.
    pub quit_times: usize,
    /// Whether to enable debug logging.
    pub debug_logging: bool,
    /// Log level (trace, debug, info, warn, error).
    pub log_level: String,
    /// Performance monitoring settings.
    pub performance: PerformanceConfig,
    /// Memory management settings.
    pub memory: MemoryConfig,
}

/// Performance monitoring configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceConfig {
    /// Whether to enable performance monitoring.
    pub enabled: bool,
    /// Whether to show performance metrics in status bar.
    pub show_metrics: bool,
    /// Performance sampling interval (in milliseconds).
    pub sampling_interval_ms: u64,
}

/// Memory management configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryConfig {
    /// Maximum memory usage for text buffers (in MB).
    pub max_buffer_memory_mb: usize,
    /// Whether to enable memory-mapped file reading.
    pub use_memory_mapping: bool,
    /// Cache size for syntax highlighting (in MB).
    pub syntax_cache_size_mb: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            ui: UiConfig::default(),
            lsp: LspConfig::default(),
            project: ProjectConfig::default(),
            build: BuildConfig::default(),
            alux: AluxConfig::default(),
            xylux: XyluxConfig::default(),
            plugins: PluginConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            use_spaces: true,
            show_line_numbers: true,
            show_relative_line_numbers: false,
            highlight_current_line: true,
            show_whitespace: false,
            word_wrap: false,
            auto_save_interval: 0, // Disabled
            format_on_save: false,
            max_recent_files: 10,
            scroll_offset: 3,
            bracket_matching: true,
            auto_close_brackets: true,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_family: "JetBrains Mono".to_string(),
            font_size: 14,
            show_file_explorer: true,
            show_terminal: true,
            show_minimap: false,
            status_bar: StatusBarConfig::default(),
            message_duration: Duration::from_secs(3),
            transparency: 1.0,
        }
    }
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            show_cursor_position: true,
            show_encoding: true,
            show_line_endings: false,
            show_file_type: true,
            show_git_branch: true,
            show_lsp_status: true,
        }
    }
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rust_analyzer: RustAnalyzerConfig::default(),
            alux_lsp: AluxLspConfig::default(),
            request_timeout_ms: 5000,
            show_diagnostics_inline: true,
            show_hover: true,
            enable_completion: true,
        }
    }
}

impl Default for RustAnalyzerConfig {
    fn default() -> Self {
        Self {
            binary_path: None,
            args: vec![],
            enable_proc_macros: true,
            cargo_features: vec![],
            check_on_save: true,
        }
    }
}

impl Default for AluxLspConfig {
    fn default() -> Self {
        Self { binary_path: None, semantic_highlighting: true, inline_hints: true }
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            auto_detect_type: true,
            default_templates: HashMap::new(),
            file_watching: true,
            ignore_patterns: vec![
                "target/".to_string(),
                "node_modules/".to_string(),
                ".git/".to_string(),
                "*.tmp".to_string(),
                "*.log".to_string(),
            ],
            auto_reload_files: true,
            git: GitConfig::default(),
        }
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        Self { enabled: true, show_status_in_explorer: true, show_blame: false }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            rust_build_command: "cargo build".to_string(),
            rust_run_command: "cargo run".to_string(),
            rust_test_command: "cargo test".to_string(),
            xylux_cli_path: None,
            show_build_output: true,
            auto_build_on_save: false,
            env_vars: HashMap::new(),
        }
    }
}

impl Default for AluxConfig {
    fn default() -> Self {
        Self {
            compiler_path: None,
            vm_path: None,
            hot_reload: true,
            vm_log_level: "warn".to_string(),
            auto_compile_bytecode: true,
            optimization_level: 1,
        }
    }
}

impl Default for XyluxConfig {
    fn default() -> Self {
        Self {
            engine_path: None,
            default_target: "native".to_string(),
            headless_testing: false,
            wasm: WasmConfig::default(),
            shaders: ShaderConfig::default(),
        }
    }
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self { enabled: true, optimize: true, wasm_pack_path: None }
    }
}

impl Default for ShaderConfig {
    fn default() -> Self {
        Self { hot_reload: true, validate_on_save: true, optimization_level: 1 }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            plugin_directories: vec![],
            enabled_plugins: vec![],
            plugin_configs: HashMap::new(),
        }
    }
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            quit_times: 2,
            debug_logging: false,
            log_level: "info".to_string(),
            performance: PerformanceConfig::default(),
            memory: MemoryConfig::default(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self { enabled: false, show_metrics: false, sampling_interval_ms: 1000 }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self { max_buffer_memory_mb: 256, use_memory_mapping: true, syntax_cache_size_mb: 32 }
    }
}

/// Configuration loader that supports multiple formats and sources.
pub struct ConfigLoader {
    config_dirs: Vec<PathBuf>,
}

impl ConfigLoader {
    /// Create a new configuration loader.
    pub fn new() -> Result<Self> {
        let config_dirs = Self::get_config_directories()?;
        Ok(Self { config_dirs })
    }

    /// Load configuration from all available sources.
    pub fn load(&self) -> Result<Config> {
        let mut config = Config::default();

        // Load from system-wide configuration
        for dir in &self.config_dirs {
            let config_path = dir.join("xylux-ide");
            if config_path.exists() {
                self.load_from_directory(&config_path, &mut config)?;
            }
        }

        // Load from user-specific configuration
        if let Some(user_config_dir) = dirs::config_dir() {
            let user_config_path = user_config_dir.join("xylux-ide");
            if user_config_path.exists() {
                self.load_from_directory(&user_config_path, &mut config)?;
            }
        }

        // Load from current project configuration
        if let Ok(current_dir) = std::env::current_dir() {
            let project_config = current_dir.join(".xylux-ide");
            if project_config.exists() {
                self.load_from_directory(&project_config, &mut config)?;
            }
        }

        Ok(config)
    }

    /// Load configuration from a specific directory.
    fn load_from_directory(&self, dir: &Path, config: &mut Config) -> Result<()> {
        // Try different configuration file formats
        let config_files =
            ["config.toml", "config.json", "config.ini", "xylux-ide.toml", "xylux-ide.json"];

        for file_name in &config_files {
            let config_path = dir.join(file_name);
            if config_path.exists() {
                self.load_from_file(&config_path, config)?;
                break; // Use the first available format
            }
        }

        Ok(())
    }

    /// Load configuration from a specific file.
    fn load_from_file(&self, path: &Path, config: &mut Config) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| XyluxError::config_error(path, 0, e.to_string()))?;

        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let loaded_config: Config = match extension {
            "toml" => toml::from_str(&content)
                .map_err(|e| XyluxError::config_error(path, 0, e.to_string()))?,
            "json" => serde_json::from_str(&content)
                .map_err(|e| XyluxError::config_error(path, 0, e.to_string()))?,
            "ini" => self.load_ini_config(path, &content)?,
            _ => return Err(XyluxError::config_error(path, 0, "Unsupported config format")),
        };

        self.merge_configs(config, loaded_config);
        Ok(())
    }

    /// Load configuration from INI format (legacy support).
    fn load_ini_config(&self, path: &Path, content: &str) -> Result<Config> {
        // For now, just return default config with basic INI support
        // This can be expanded to parse specific INI sections
        let mut config = Config::default();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "tab_size" => {
                        config.editor.tab_size = value.parse().map_err(|_| {
                            XyluxError::config_error(path, line_num + 1, "Invalid tab_size")
                        })?;
                    }
                    "theme" => config.ui.theme = value.to_string(),
                    "font_size" => {
                        config.ui.font_size = value.parse().map_err(|_| {
                            XyluxError::config_error(path, line_num + 1, "Invalid font_size")
                        })?;
                    }
                    _ => {} // Ignore unknown keys for compatibility
                }
            }
        }

        Ok(config)
    }

    /// Merge two configurations, with the second one taking precedence.
    fn merge_configs(&self, base: &mut Config, override_config: Config) {
        // For now, completely replace sections
        // TODO: Implement field-by-field merging
        *base = override_config;
    }

    /// Get system configuration directories.
    fn get_config_directories() -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();

        #[cfg(unix)]
        {
            dirs.push(PathBuf::from("/etc"));
            if let Ok(xdg_config_dirs) = std::env::var("XDG_CONFIG_DIRS") {
                for dir in xdg_config_dirs.split(':') {
                    dirs.push(PathBuf::from(dir));
                }
            }
        }

        #[cfg(windows)]
        {
            if let Ok(program_data) = std::env::var("PROGRAMDATA") {
                dirs.push(PathBuf::from(program_data));
            }
        }

        Ok(dirs)
    }

    /// Save configuration to the user's config directory.
    pub fn save(&self, config: &Config) -> Result<()> {
        let user_config_dir = dirs::config_dir().ok_or_else(|| {
            XyluxError::config_error("", 0, "Cannot determine user config directory")
        })?;

        let xylux_config_dir = user_config_dir.join("xylux-ide");
        std::fs::create_dir_all(&xylux_config_dir)?;

        let config_path = xylux_config_dir.join("config.toml");
        let content = toml::to_string_pretty(config)
            .map_err(|e| XyluxError::config_error(&config_path, 0, e.to_string()))?;

        std::fs::write(&config_path, content)?;
        Ok(())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create ConfigLoader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.editor.tab_size, 4);
        assert_eq!(config.ui.theme, "dark");
        assert!(config.lsp.enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();

        // Test TOML serialization
        let toml_content = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_content).unwrap();
        assert_eq!(config, deserialized);

        // Test JSON serialization
        let json_content = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json_content).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_loader() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("xylux-ide");
        std::fs::create_dir_all(&config_dir).unwrap();

        let mut config = Config::default();
        config.editor.tab_size = 8;
        config.ui.theme = "light".to_string();

        let config_content = toml::to_string(&config).unwrap();
        std::fs::write(config_dir.join("config.toml"), config_content).unwrap();

        // Test would require mocking the config directories
        // For now, just test that the ConfigLoader can be created
        assert!(ConfigLoader::new().is_ok());
    }
}
