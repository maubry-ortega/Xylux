//! # Xylux Project Implementation
//!
//! Xylux-specific project functionality and configuration.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
// No tracing imports needed currently

use crate::core::{Result, XyluxError};

/// Xylux project configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XyluxProject {
    /// Project metadata.
    pub project: ProjectMetadata,
    /// Build configuration.
    pub build: BuildConfig,
    /// Runtime configuration.
    pub runtime: RuntimeConfig,
    /// Asset configuration.
    pub assets: Option<AssetConfig>,
    /// Dependency configuration.
    pub dependencies: Option<HashMap<String, Dependency>>,
}

/// Project metadata section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Project name.
    pub name: String,
    /// Project version.
    pub version: String,
    /// Xylux engine version requirement.
    pub engine_version: String,
    /// Project description.
    pub description: Option<String>,
    /// Project authors.
    pub authors: Option<Vec<String>>,
    /// Project license.
    pub license: Option<String>,
    /// Project repository URL.
    pub repository: Option<String>,
}

/// Build configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Build target (native, wasm, android, ios).
    pub target: String,
    /// Assets directory.
    pub assets_dir: String,
    /// Scripts directory.
    pub scripts_dir: String,
    /// Shaders directory.
    pub shaders_dir: Option<String>,
    /// Output directory.
    pub output_dir: Option<String>,
    /// Optimization level.
    pub optimization: Option<String>,
    /// Debug symbols.
    pub debug_symbols: Option<bool>,
}

/// Runtime configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Enable hot reload.
    pub hot_reload: bool,
    /// Log level.
    pub log_level: String,
    /// Window configuration.
    pub window: Option<WindowConfig>,
    /// Renderer configuration.
    pub renderer: Option<RendererConfig>,
    /// Audio configuration.
    pub audio: Option<AudioConfig>,
}

/// Window configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window title.
    pub title: Option<String>,
    /// Window width.
    pub width: Option<u32>,
    /// Window height.
    pub height: Option<u32>,
    /// Window resizable.
    pub resizable: Option<bool>,
    /// Window fullscreen.
    pub fullscreen: Option<bool>,
    /// Window vsync.
    pub vsync: Option<bool>,
}

/// Renderer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererConfig {
    /// Renderer backend (vulkan, metal, dx12, opengl).
    pub backend: Option<String>,
    /// MSAA samples.
    pub msaa_samples: Option<u32>,
    /// Anisotropic filtering.
    pub anisotropic_filtering: Option<u32>,
    /// Shadow quality.
    pub shadow_quality: Option<String>,
}

/// Audio configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Master volume.
    pub master_volume: Option<f32>,
    /// Music volume.
    pub music_volume: Option<f32>,
    /// SFX volume.
    pub sfx_volume: Option<f32>,
    /// Audio device.
    pub device: Option<String>,
}

/// Asset configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    /// Asset pipeline configuration.
    pub pipeline: Option<HashMap<String, AssetPipelineConfig>>,
    /// Texture compression settings.
    pub textures: Option<TextureConfig>,
    /// Audio compression settings.
    pub audio: Option<AudioAssetConfig>,
    /// Model optimization settings.
    pub models: Option<ModelConfig>,
}

/// Asset pipeline configuration for specific file types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPipelineConfig {
    /// Asset processor to use.
    pub processor: String,
    /// Processor-specific options.
    pub options: Option<HashMap<String, serde_json::Value>>,
}

/// Texture asset configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureConfig {
    /// Default texture format.
    pub format: Option<String>,
    /// Compression quality.
    pub quality: Option<f32>,
    /// Generate mipmaps.
    pub mipmaps: Option<bool>,
}

/// Audio asset configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAssetConfig {
    /// Default audio format.
    pub format: Option<String>,
    /// Compression quality.
    pub quality: Option<f32>,
    /// Sample rate.
    pub sample_rate: Option<u32>,
}

/// Model asset configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Optimize meshes.
    pub optimize_meshes: Option<bool>,
    /// Generate tangents.
    pub generate_tangents: Option<bool>,
    /// Split large meshes.
    pub split_large_meshes: Option<bool>,
}

/// Dependency specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Simple version string.
    Simple(String),
    /// Detailed dependency configuration.
    Detailed(DetailedDependency),
}

/// Detailed dependency configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedDependency {
    /// Version requirement.
    pub version: Option<String>,
    /// Git repository URL.
    pub git: Option<String>,
    /// Git branch.
    pub branch: Option<String>,
    /// Git tag.
    pub tag: Option<String>,
    /// Git revision.
    pub rev: Option<String>,
    /// Local path.
    pub path: Option<String>,
    /// Optional dependency.
    pub optional: Option<bool>,
    /// Default features.
    pub default_features: Option<bool>,
    /// Specific features to enable.
    pub features: Option<Vec<String>>,
}

impl XyluxProject {
    /// Create a new Xylux project configuration.
    pub fn new(name: String) -> Self {
        Self {
            project: ProjectMetadata {
                name: name.clone(),
                version: "0.1.0".to_string(),
                engine_version: "0.1.0".to_string(),
                description: None,
                authors: None,
                license: None,
                repository: None,
            },
            build: BuildConfig {
                target: "native".to_string(),
                assets_dir: "assets".to_string(),
                scripts_dir: "scripts".to_string(),
                shaders_dir: Some("shaders".to_string()),
                output_dir: Some("target".to_string()),
                optimization: None,
                debug_symbols: None,
            },
            runtime: RuntimeConfig {
                hot_reload: true,
                log_level: "info".to_string(),
                window: None,
                renderer: None,
                audio: None,
            },
            assets: None,
            dependencies: None,
        }
    }

    /// Load Xylux project from file.
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(path.as_ref())
            .await
            .map_err(|e| XyluxError::io(e, "Failed to read xylux.toml"))?;

        let project: XyluxProject = toml::from_str(&content)
            .map_err(|e| XyluxError::parse(format!("Failed to parse xylux.toml: {}", e)))?;

        project.validate()?;
        Ok(project)
    }

    /// Save Xylux project to file.
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.validate()?;

        let content = toml::to_string_pretty(self)
            .map_err(|e| XyluxError::serialize(format!("Failed to serialize xylux.toml: {}", e)))?;

        tokio::fs::write(path.as_ref(), content)
            .await
            .map_err(|e| XyluxError::io(e, "Failed to write xylux.toml"))?;

        Ok(())
    }

    /// Validate the project configuration.
    pub fn validate(&self) -> Result<()> {
        // Validate project name
        if self.project.name.is_empty() {
            return Err(XyluxError::invalid_data("Project name cannot be empty"));
        }

        // Validate version format
        if !self.is_valid_version(&self.project.version) {
            return Err(XyluxError::invalid_data("Invalid project version format"));
        }

        if !self.is_valid_version(&self.project.engine_version) {
            return Err(XyluxError::invalid_data("Invalid engine version format"));
        }

        // Validate build target
        let valid_targets = ["native", "wasm", "android", "ios"];
        if !valid_targets.contains(&self.build.target.as_str()) {
            return Err(XyluxError::invalid_data(&format!(
                "Invalid build target: {}. Must be one of: {}",
                self.build.target,
                valid_targets.join(", ")
            )));
        }

        // Validate log level
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.runtime.log_level.as_str()) {
            return Err(XyluxError::invalid_data(&format!(
                "Invalid log level: {}. Must be one of: {}",
                self.runtime.log_level,
                valid_log_levels.join(", ")
            )));
        }

        Ok(())
    }

    /// Check if a version string is valid (simplified semver check).
    fn is_valid_version(&self, version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }

        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }

    /// Get the project root directory from the config file path.
    pub fn project_root<P: AsRef<Path>>(config_path: P) -> Option<PathBuf> {
        config_path.as_ref().parent().map(|p| p.to_path_buf())
    }

    /// Get the assets directory path.
    pub fn assets_dir<P: AsRef<Path>>(&self, project_root: P) -> PathBuf {
        project_root.as_ref().join(&self.build.assets_dir)
    }

    /// Get the scripts directory path.
    pub fn scripts_dir<P: AsRef<Path>>(&self, project_root: P) -> PathBuf {
        project_root.as_ref().join(&self.build.scripts_dir)
    }

    /// Get the shaders directory path.
    pub fn shaders_dir<P: AsRef<Path>>(&self, project_root: P) -> Option<PathBuf> {
        self.build.shaders_dir.as_ref().map(|dir| project_root.as_ref().join(dir))
    }

    /// Get the output directory path.
    pub fn output_dir<P: AsRef<Path>>(&self, project_root: P) -> PathBuf {
        let output_dir = self.build.output_dir.as_deref().unwrap_or("target");
        project_root.as_ref().join(output_dir)
    }

    /// Check if hot reload is enabled.
    pub fn hot_reload_enabled(&self) -> bool {
        self.runtime.hot_reload
    }

    /// Get build command for this project.
    pub fn build_command(&self) -> String {
        match self.build.target.as_str() {
            "wasm" => "xylux build --target wasm".to_string(),
            "android" => "xylux build --target android".to_string(),
            "ios" => "xylux build --target ios".to_string(),
            _ => "xylux build".to_string(),
        }
    }

    /// Get run command for this project.
    pub fn run_command(&self) -> String {
        match self.build.target.as_str() {
            "wasm" => "xylux serve".to_string(),
            "android" => "xylux run --target android".to_string(),
            "ios" => "xylux run --target ios".to_string(),
            _ => "xylux run".to_string(),
        }
    }

    /// Get clean command for this project.
    pub fn clean_command(&self) -> String {
        "xylux clean".to_string()
    }

    /// Add a dependency to the project.
    pub fn add_dependency(&mut self, name: String, dependency: Dependency) {
        if self.dependencies.is_none() {
            self.dependencies = Some(HashMap::new());
        }

        if let Some(ref mut deps) = self.dependencies {
            deps.insert(name, dependency);
        }
    }

    /// Remove a dependency from the project.
    pub fn remove_dependency(&mut self, name: &str) -> bool {
        if let Some(ref mut deps) = self.dependencies { deps.remove(name).is_some() } else { false }
    }

    /// Get all dependencies.
    pub fn dependencies(&self) -> Option<&HashMap<String, Dependency>> {
        self.dependencies.as_ref()
    }

    /// Create a minimal project configuration.
    pub fn minimal(name: String) -> Self {
        Self {
            project: ProjectMetadata {
                name: name.clone(),
                version: "0.1.0".to_string(),
                engine_version: "0.1.0".to_string(),
                description: None,
                authors: None,
                license: None,
                repository: None,
            },
            build: BuildConfig {
                target: "native".to_string(),
                assets_dir: "assets".to_string(),
                scripts_dir: "scripts".to_string(),
                shaders_dir: None,
                output_dir: None,
                optimization: None,
                debug_symbols: None,
            },
            runtime: RuntimeConfig {
                hot_reload: true,
                log_level: "info".to_string(),
                window: None,
                renderer: None,
                audio: None,
            },
            assets: None,
            dependencies: None,
        }
    }

    /// Create a full project configuration with all options.
    pub fn full(name: String) -> Self {
        let mut project = Self::new(name.clone());

        // Add window configuration
        project.runtime.window = Some(WindowConfig {
            title: Some(name.clone()),
            width: Some(1920),
            height: Some(1080),
            resizable: Some(true),
            fullscreen: Some(false),
            vsync: Some(true),
        });

        // Add renderer configuration
        project.runtime.renderer = Some(RendererConfig {
            backend: Some("vulkan".to_string()),
            msaa_samples: Some(4),
            anisotropic_filtering: Some(16),
            shadow_quality: Some("high".to_string()),
        });

        // Add audio configuration
        project.runtime.audio = Some(AudioConfig {
            master_volume: Some(1.0),
            music_volume: Some(0.8),
            sfx_volume: Some(1.0),
            device: None,
        });

        // Add asset configuration
        let mut asset_pipeline = HashMap::new();
        asset_pipeline.insert(
            "png".to_string(),
            AssetPipelineConfig { processor: "texture".to_string(), options: None },
        );
        asset_pipeline.insert(
            "jpg".to_string(),
            AssetPipelineConfig { processor: "texture".to_string(), options: None },
        );
        asset_pipeline.insert(
            "gltf".to_string(),
            AssetPipelineConfig { processor: "model".to_string(), options: None },
        );

        project.assets = Some(AssetConfig {
            pipeline: Some(asset_pipeline),
            textures: Some(TextureConfig {
                format: Some("bc7".to_string()),
                quality: Some(0.9),
                mipmaps: Some(true),
            }),
            audio: Some(AudioAssetConfig {
                format: Some("ogg".to_string()),
                quality: Some(0.8),
                sample_rate: Some(44100),
            }),
            models: Some(ModelConfig {
                optimize_meshes: Some(true),
                generate_tangents: Some(true),
                split_large_meshes: Some(true),
            }),
        });

        project
    }
}

impl Default for XyluxProject {
    fn default() -> Self {
        Self::new("xylux-project".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_xylux_project_creation() {
        let project = XyluxProject::new("test-game".to_string());

        assert_eq!(project.project.name, "test-game");
        assert_eq!(project.project.version, "0.1.0");
        assert_eq!(project.build.target, "native");
        assert!(project.runtime.hot_reload);
    }

    #[test]
    fn test_project_validation() {
        let mut project = XyluxProject::new("test".to_string());

        // Valid project should pass
        assert!(project.validate().is_ok());

        // Empty name should fail
        project.project.name = String::new();
        assert!(project.validate().is_err());

        // Invalid version should fail
        project.project.name = "test".to_string();
        project.project.version = "invalid".to_string();
        assert!(project.validate().is_err());

        // Invalid target should fail
        project.project.version = "1.0.0".to_string();
        project.build.target = "invalid".to_string();
        assert!(project.validate().is_err());
    }

    #[tokio::test]
    async fn test_save_load_project() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("xylux.toml");

        let original_project = XyluxProject::new("test-project".to_string());

        // Save project
        original_project.save_to_file(&config_path).await.unwrap();

        // Load project
        let loaded_project = XyluxProject::load_from_file(&config_path).await.unwrap();

        assert_eq!(original_project.project.name, loaded_project.project.name);
        assert_eq!(original_project.project.version, loaded_project.project.version);
        assert_eq!(original_project.build.target, loaded_project.build.target);
    }

    #[test]
    fn test_project_paths() {
        let project = XyluxProject::new("test".to_string());
        let project_root = PathBuf::from("/project");

        assert_eq!(project.assets_dir(&project_root), project_root.join("assets"));
        assert_eq!(project.scripts_dir(&project_root), project_root.join("scripts"));
        assert_eq!(project.output_dir(&project_root), project_root.join("target"));
    }

    #[test]
    fn test_dependencies() {
        let mut project = XyluxProject::new("test".to_string());

        // Add simple dependency
        project.add_dependency("math".to_string(), Dependency::Simple("1.0.0".to_string()));

        // Add detailed dependency
        project.add_dependency(
            "graphics".to_string(),
            Dependency::Detailed(DetailedDependency {
                version: Some("2.0.0".to_string()),
                git: Some("https://github.com/xylux/graphics.git".to_string()),
                branch: Some("main".to_string()),
                tag: None,
                rev: None,
                path: None,
                optional: Some(false),
                default_features: Some(true),
                features: Some(vec!["vulkan".to_string()]),
            }),
        );

        assert!(project.dependencies().is_some());
        assert_eq!(project.dependencies().unwrap().len(), 2);

        // Remove dependency
        assert!(project.remove_dependency("math"));
        assert_eq!(project.dependencies().unwrap().len(), 1);
    }

    #[test]
    fn test_commands() {
        let project = XyluxProject::new("test".to_string());

        assert_eq!(project.build_command(), "xylux build");
        assert_eq!(project.run_command(), "xylux run");
        assert_eq!(project.clean_command(), "xylux clean");

        // Test WASM target
        let mut wasm_project = project.clone();
        wasm_project.build.target = "wasm".to_string();
        assert_eq!(wasm_project.build_command(), "xylux build --target wasm");
        assert_eq!(wasm_project.run_command(), "xylux serve");
    }

    #[test]
    fn test_minimal_vs_full_project() {
        let minimal = XyluxProject::minimal("minimal".to_string());
        let full = XyluxProject::full("full".to_string());

        assert!(minimal.runtime.window.is_none());
        assert!(full.runtime.window.is_some());

        assert!(minimal.assets.is_none());
        assert!(full.assets.is_some());
    }
}
