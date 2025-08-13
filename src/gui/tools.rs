//! # Specialized Tools Window
//!
//! Tools and utilities specifically designed for Rust and Alux development

use std::path::PathBuf;

/// Rust-specific tools and information
#[derive(Clone, Debug, Default)]
pub struct RustTools {
    pub cargo_info: CargoInfo,
    pub dependencies: Vec<CrateDependency>,
    pub build_targets: Vec<BuildTarget>,
    pub test_results: Vec<TestResult>,
    pub clippy_warnings: Vec<ClippyWarning>,
    pub rustfmt_config: RustfmtConfig,
}

/// Alux-specific tools and information
#[derive(Clone, Debug, Default)]
pub struct AluxTools {
    pub script_info: AluxScriptInfo,
    pub modules: Vec<AluxModule>,
    pub functions: Vec<AluxFunction>,
    pub variables: Vec<AluxVariable>,
    pub syntax_errors: Vec<AluxSyntaxError>,
    pub runtime_info: AluxRuntimeInfo,
}

/// Cargo project information
#[derive(Clone, Debug, Default)]
pub struct CargoInfo {
    pub project_name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub edition: String,
    pub features: Vec<String>,
    pub manifest_path: Option<PathBuf>,
}

/// Crate dependency information
#[derive(Clone, Debug)]
pub struct CrateDependency {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub dev_dependency: bool,
}

/// Build target information
#[derive(Clone, Debug)]
pub struct BuildTarget {
    pub name: String,
    pub target_type: BuildTargetType,
    pub path: PathBuf,
    pub features: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum BuildTargetType {
    Binary,
    Library,
    Example,
    Test,
    Benchmark,
}

/// Test result information
#[derive(Clone, Debug)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration: std::time::Duration,
    pub output: String,
}

#[derive(Clone, Debug)]
pub enum TestStatus {
    Passed,
    Failed,
    Ignored,
    Running,
}

/// Clippy warning information
#[derive(Clone, Debug)]
pub struct ClippyWarning {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub lint_name: String,
    pub severity: ClippySeverity,
}

#[derive(Clone, Debug)]
pub enum ClippySeverity {
    Error,
    Warning,
    Note,
    Help,
}

/// Rustfmt configuration
#[derive(Clone, Debug, Default)]
pub struct RustfmtConfig {
    pub edition: String,
    pub max_width: usize,
    pub tab_spaces: usize,
    pub use_small_heuristics: bool,
    pub newline_style: String,
}

/// Alux script information
#[derive(Clone, Debug, Default)]
pub struct AluxScriptInfo {
    pub script_name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry_point: Option<PathBuf>,
    pub dependencies: Vec<String>,
}

/// Alux module information
#[derive(Clone, Debug)]
pub struct AluxModule {
    pub name: String,
    pub path: PathBuf,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
    pub line_count: usize,
}

/// Alux function information
#[derive(Clone, Debug)]
pub struct AluxFunction {
    pub name: String,
    pub parameters: Vec<AluxParameter>,
    pub return_type: String,
    pub visibility: AluxVisibility,
    pub line: usize,
    pub module: String,
}

/// Alux function parameter
#[derive(Clone, Debug)]
pub struct AluxParameter {
    pub name: String,
    pub param_type: String,
    pub default_value: Option<String>,
}

/// Alux variable information
#[derive(Clone, Debug)]
pub struct AluxVariable {
    pub name: String,
    pub var_type: String,
    pub value: String,
    pub scope: AluxScope,
    pub line: usize,
    pub mutable: bool,
}

/// Alux visibility levels
#[derive(Clone, Debug)]
pub enum AluxVisibility {
    Public,
    Private,
    Module,
}

/// Alux variable scope
#[derive(Clone, Debug)]
pub enum AluxScope {
    Global,
    Function,
    Block,
    Module,
}

/// Alux syntax error information
#[derive(Clone, Debug)]
pub struct AluxSyntaxError {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub error_type: AluxErrorType,
    pub suggestion: Option<String>,
}

#[derive(Clone, Debug)]
pub enum AluxErrorType {
    SyntaxError,
    TypeError,
    NameError,
    RuntimeError,
    Warning,
}

/// Alux runtime information
#[derive(Clone, Debug, Default)]
pub struct AluxRuntimeInfo {
    pub memory_usage: u64,
    pub execution_time: std::time::Duration,
    pub active_objects: usize,
    pub call_stack_depth: usize,
    pub gc_collections: usize,
}

/// Main tools window state
#[derive(Clone, Debug, Default)]
pub struct ToolsWindow {
    pub rust_tools: RustTools,
    pub alux_tools: AluxTools,
    pub show_rust_panel: bool,
    pub show_alux_panel: bool,
    pub show_cargo_info: bool,
    pub show_dependencies: bool,
    pub show_build_targets: bool,
    pub show_test_results: bool,
    pub show_clippy_warnings: bool,
    pub show_alux_modules: bool,
    pub show_alux_functions: bool,
    pub show_alux_variables: bool,
    pub show_alux_errors: bool,
    pub show_runtime_info: bool,
    pub window_open: bool,
}

impl ToolsWindow {
    /// Create a new tools window
    pub fn new() -> Self {
        Self {
            rust_tools: RustTools::default(),
            alux_tools: AluxTools::default(),
            show_rust_panel: true,
            show_alux_panel: true,
            show_cargo_info: true,
            show_dependencies: true,
            show_build_targets: false,
            show_test_results: false,
            show_clippy_warnings: true,
            show_alux_modules: true,
            show_alux_functions: false,
            show_alux_variables: false,
            show_alux_errors: true,
            show_runtime_info: false,
            window_open: false,
        }
    }

    /// Toggle tools window visibility
    pub fn toggle(&mut self) {
        self.window_open = !self.window_open;
    }

    /// Show the tools window
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.window_open {
            return;
        }

        egui::Window::new("🔧 Specialized Tools")
            .default_width(400.0)
            .default_height(600.0)
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.show_rust_panel, true, "🦀 Rust");
                    ui.selectable_value(&mut self.show_alux_panel, true, "⚡ Alux");
                });

                ui.separator();

                egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                    if self.show_rust_panel {
                        self.draw_rust_panel(ui);
                    }

                    if self.show_alux_panel {
                        self.draw_alux_panel(ui);
                    }
                });
            });
    }

    /// Draw Rust-specific tools panel
    fn draw_rust_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("🦀 Rust Tools", |ui| {
            // Cargo Information
            ui.checkbox(&mut self.show_cargo_info, "📦 Cargo Information");
            if self.show_cargo_info {
                ui.indent("cargo_info", |ui| {
                    self.draw_cargo_info(ui);
                });
            }

            // Dependencies
            ui.checkbox(&mut self.show_dependencies, "📚 Dependencies");
            if self.show_dependencies {
                ui.indent("dependencies", |ui| {
                    self.draw_dependencies(ui);
                });
            }

            // Build Targets
            ui.checkbox(&mut self.show_build_targets, "🎯 Build Targets");
            if self.show_build_targets {
                ui.indent("build_targets", |ui| {
                    self.draw_build_targets(ui);
                });
            }

            // Test Results
            ui.checkbox(&mut self.show_test_results, "🧪 Test Results");
            if self.show_test_results {
                ui.indent("test_results", |ui| {
                    self.draw_test_results(ui);
                });
            }

            // Clippy Warnings
            ui.checkbox(&mut self.show_clippy_warnings, "📋 Clippy Warnings");
            if self.show_clippy_warnings {
                ui.indent("clippy_warnings", |ui| {
                    self.draw_clippy_warnings(ui);
                });
            }
        });
    }

    /// Draw Alux-specific tools panel
    fn draw_alux_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("⚡ Alux Tools", |ui| {
            // Script Information
            ui.label("📄 Script Information");
            ui.indent("alux_script_info", |ui| {
                self.draw_alux_script_info(ui);
            });

            // Modules
            ui.checkbox(&mut self.show_alux_modules, "📦 Modules");
            if self.show_alux_modules {
                ui.indent("alux_modules", |ui| {
                    self.draw_alux_modules(ui);
                });
            }

            // Functions
            ui.checkbox(&mut self.show_alux_functions, "🔧 Functions");
            if self.show_alux_functions {
                ui.indent("alux_functions", |ui| {
                    self.draw_alux_functions(ui);
                });
            }

            // Variables
            ui.checkbox(&mut self.show_alux_variables, "💾 Variables");
            if self.show_alux_variables {
                ui.indent("alux_variables", |ui| {
                    self.draw_alux_variables(ui);
                });
            }

            // Syntax Errors
            ui.checkbox(&mut self.show_alux_errors, "❌ Syntax Errors");
            if self.show_alux_errors {
                ui.indent("alux_errors", |ui| {
                    self.draw_alux_errors(ui);
                });
            }

            // Runtime Information
            ui.checkbox(&mut self.show_runtime_info, "⚙️ Runtime Info");
            if self.show_runtime_info {
                ui.indent("runtime_info", |ui| {
                    self.draw_runtime_info(ui);
                });
            }
        });
    }

    /// Draw Cargo project information
    fn draw_cargo_info(&self, ui: &mut egui::Ui) {
        let cargo = &self.rust_tools.cargo_info;

        ui.horizontal(|ui| {
            ui.label("Project:");
            ui.label(&cargo.project_name);
        });

        ui.horizontal(|ui| {
            ui.label("Version:");
            ui.label(&cargo.version);
        });

        ui.horizontal(|ui| {
            ui.label("Edition:");
            ui.label(&cargo.edition);
        });

        if !cargo.features.is_empty() {
            ui.horizontal(|ui| {
                ui.label("Features:");
                ui.label(cargo.features.join(", "));
            });
        }

        if let Some(manifest) = &cargo.manifest_path {
            ui.horizontal(|ui| {
                ui.label("Manifest:");
                ui.label(manifest.display().to_string());
            });
        }
    }

    /// Draw dependencies list
    fn draw_dependencies(&self, ui: &mut egui::Ui) {
        if self.rust_tools.dependencies.is_empty() {
            ui.label("No dependencies found");
            return;
        }

        for dep in &self.rust_tools.dependencies {
            ui.horizontal(|ui| {
                let color = if dep.dev_dependency {
                    egui::Color32::YELLOW
                } else if dep.optional {
                    egui::Color32::LIGHT_BLUE
                } else {
                    egui::Color32::WHITE
                };

                ui.colored_label(color, &dep.name);
                ui.label(&dep.version);

                if !dep.features.is_empty() {
                    ui.label(format!("[{}]", dep.features.join(", ")));
                }
            });
        }
    }

    /// Draw build targets
    fn draw_build_targets(&self, ui: &mut egui::Ui) {
        if self.rust_tools.build_targets.is_empty() {
            ui.label("No build targets found");
            return;
        }

        for target in &self.rust_tools.build_targets {
            ui.horizontal(|ui| {
                let icon = match target.target_type {
                    BuildTargetType::Binary => "🎯",
                    BuildTargetType::Library => "📚",
                    BuildTargetType::Example => "📝",
                    BuildTargetType::Test => "🧪",
                    BuildTargetType::Benchmark => "📊",
                };

                ui.label(icon);
                ui.label(&target.name);
                ui.label(target.path.display().to_string());
            });
        }
    }

    /// Draw test results
    fn draw_test_results(&self, ui: &mut egui::Ui) {
        if self.rust_tools.test_results.is_empty() {
            ui.label("No test results available");
            return;
        }

        for test in &self.rust_tools.test_results {
            ui.horizontal(|ui| {
                let (icon, color) = match test.status {
                    TestStatus::Passed => ("✅", egui::Color32::GREEN),
                    TestStatus::Failed => ("❌", egui::Color32::RED),
                    TestStatus::Ignored => ("⏭️", egui::Color32::YELLOW),
                    TestStatus::Running => ("⏳", egui::Color32::BLUE),
                };

                ui.colored_label(color, icon);
                ui.label(&test.name);
                ui.label(format!("{:.2}ms", test.duration.as_millis()));
            });
        }
    }

    /// Draw Clippy warnings
    fn draw_clippy_warnings(&self, ui: &mut egui::Ui) {
        if self.rust_tools.clippy_warnings.is_empty() {
            ui.colored_label(egui::Color32::GREEN, "No Clippy warnings! 🎉");
            return;
        }

        for warning in &self.rust_tools.clippy_warnings {
            ui.horizontal(|ui| {
                let (icon, color) = match warning.severity {
                    ClippySeverity::Error => ("❌", egui::Color32::RED),
                    ClippySeverity::Warning => ("⚠️", egui::Color32::YELLOW),
                    ClippySeverity::Note => ("ℹ️", egui::Color32::BLUE),
                    ClippySeverity::Help => ("💡", egui::Color32::GREEN),
                };

                ui.colored_label(color, icon);
                ui.label(format!("{}:{}", warning.line, warning.column));
                ui.label(&warning.lint_name);
            });

            ui.label(&warning.message);
            ui.separator();
        }
    }

    /// Draw Alux script information
    fn draw_alux_script_info(&self, ui: &mut egui::Ui) {
        let script = &self.alux_tools.script_info;

        ui.horizontal(|ui| {
            ui.label("Script:");
            ui.label(&script.script_name);
        });

        ui.horizontal(|ui| {
            ui.label("Version:");
            ui.label(&script.version);
        });

        ui.horizontal(|ui| {
            ui.label("Author:");
            ui.label(&script.author);
        });

        if !script.description.is_empty() {
            ui.label("Description:");
            ui.label(&script.description);
        }

        if let Some(entry) = &script.entry_point {
            ui.horizontal(|ui| {
                ui.label("Entry:");
                ui.label(entry.display().to_string());
            });
        }
    }

    /// Draw Alux modules
    fn draw_alux_modules(&self, ui: &mut egui::Ui) {
        if self.alux_tools.modules.is_empty() {
            ui.label("No Alux modules found");
            return;
        }

        for module in &self.alux_tools.modules {
            ui.collapsing(&module.name, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    ui.label(module.path.display().to_string());
                });

                ui.horizontal(|ui| {
                    ui.label("Lines:");
                    ui.label(module.line_count.to_string());
                });

                if !module.exports.is_empty() {
                    ui.label("Exports:");
                    for export in &module.exports {
                        ui.label(format!("  • {}", export));
                    }
                }

                if !module.imports.is_empty() {
                    ui.label("Imports:");
                    for import in &module.imports {
                        ui.label(format!("  • {}", import));
                    }
                }
            });
        }
    }

    /// Draw Alux functions
    fn draw_alux_functions(&self, ui: &mut egui::Ui) {
        if self.alux_tools.functions.is_empty() {
            ui.label("No Alux functions found");
            return;
        }

        for function in &self.alux_tools.functions {
            ui.collapsing(&function.name, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Module:");
                    ui.label(&function.module);
                });

                ui.horizontal(|ui| {
                    ui.label("Line:");
                    ui.label(function.line.to_string());
                });

                ui.horizontal(|ui| {
                    ui.label("Returns:");
                    ui.label(&function.return_type);
                });

                if !function.parameters.is_empty() {
                    ui.label("Parameters:");
                    for param in &function.parameters {
                        let param_text = if let Some(default) = &param.default_value {
                            format!("  • {}: {} = {}", param.name, param.param_type, default)
                        } else {
                            format!("  • {}: {}", param.name, param.param_type)
                        };
                        ui.label(param_text);
                    }
                }
            });
        }
    }

    /// Draw Alux variables
    fn draw_alux_variables(&self, ui: &mut egui::Ui) {
        if self.alux_tools.variables.is_empty() {
            ui.label("No Alux variables found");
            return;
        }

        for variable in &self.alux_tools.variables {
            ui.horizontal(|ui| {
                let mutability_icon = if variable.mutable { "🔄" } else { "🔒" };
                ui.label(mutability_icon);
                ui.label(&variable.name);
                ui.label(&variable.var_type);
                ui.label(&variable.value);
            });
        }
    }

    /// Draw Alux syntax errors
    fn draw_alux_errors(&self, ui: &mut egui::Ui) {
        if self.alux_tools.syntax_errors.is_empty() {
            ui.colored_label(egui::Color32::GREEN, "No Alux errors! ⚡");
            return;
        }

        for error in &self.alux_tools.syntax_errors {
            ui.horizontal(|ui| {
                let (icon, color) = match error.error_type {
                    AluxErrorType::SyntaxError => ("❌", egui::Color32::RED),
                    AluxErrorType::TypeError => ("🔢", egui::Color32::RED),
                    AluxErrorType::NameError => ("📛", egui::Color32::RED),
                    AluxErrorType::RuntimeError => ("💥", egui::Color32::RED),
                    AluxErrorType::Warning => ("⚠️", egui::Color32::YELLOW),
                };

                ui.colored_label(color, icon);
                ui.label(format!("{}:{}", error.line, error.column));
            });

            ui.label(&error.message);

            if let Some(suggestion) = &error.suggestion {
                ui.colored_label(egui::Color32::GREEN, format!("💡 {}", suggestion));
            }

            ui.separator();
        }
    }

    /// Draw Alux runtime information
    fn draw_runtime_info(&self, ui: &mut egui::Ui) {
        let runtime = &self.alux_tools.runtime_info;

        ui.horizontal(|ui| {
            ui.label("Memory:");
            ui.label(format!("{} KB", runtime.memory_usage / 1024));
        });

        ui.horizontal(|ui| {
            ui.label("Execution Time:");
            ui.label(format!("{:.2}ms", runtime.execution_time.as_millis()));
        });

        ui.horizontal(|ui| {
            ui.label("Active Objects:");
            ui.label(runtime.active_objects.to_string());
        });

        ui.horizontal(|ui| {
            ui.label("Call Stack:");
            ui.label(runtime.call_stack_depth.to_string());
        });

        ui.horizontal(|ui| {
            ui.label("GC Collections:");
            ui.label(runtime.gc_collections.to_string());
        });
    }

    /// Update tools data (to be called periodically)
    pub fn update_rust_tools(&mut self) {
        // TODO: Implement actual data collection from cargo, clippy, etc.
        // This is a placeholder for demonstration

        // Mock cargo info
        self.rust_tools.cargo_info = CargoInfo {
            project_name: "xylux-ide".to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["Equipo Xylux".to_string()],
            edition: "2021".to_string(),
            features: vec!["default".to_string(), "clipboard".to_string()],
            manifest_path: Some(PathBuf::from("Cargo.toml")),
        };

        // Mock dependencies
        self.rust_tools.dependencies = vec![
            CrateDependency {
                name: "eframe".to_string(),
                version: "0.28".to_string(),
                features: vec![],
                optional: false,
                dev_dependency: false,
            },
            CrateDependency {
                name: "egui".to_string(),
                version: "0.28".to_string(),
                features: vec![],
                optional: false,
                dev_dependency: false,
            },
        ];
    }

    /// Update Alux tools data
    pub fn update_alux_tools(&mut self) {
        // TODO: Implement actual data collection from Alux scripts
        // This is a placeholder for demonstration

        self.alux_tools.script_info = AluxScriptInfo {
            script_name: "main".to_string(),
            version: "1.0.0".to_string(),
            author: "Developer".to_string(),
            description: "Main Alux script".to_string(),
            entry_point: Some(PathBuf::from("main.alux")),
            dependencies: vec!["core".to_string(), "math".to_string()],
        };
    }

    /// Update Rust tools with real project data
    pub fn update_rust_tools_from_project(
        &mut self,
        project: Option<&crate::project::Project>,
        _active_file: &PathBuf,
        open_files: &[PathBuf],
    ) {
        if let Some(proj) = project {
            // Update with real project data
            self.rust_tools.cargo_info = CargoInfo {
                project_name: proj.name.clone(),
                version: "0.1.0".to_string(), // TODO: Parse from Cargo.toml
                authors: vec!["Project Author".to_string()],
                edition: "2021".to_string(),
                features: vec!["default".to_string()],
                manifest_path: proj.config_path.clone(),
            };

            // Update build targets from open files
            self.rust_tools.build_targets = open_files
                .iter()
                .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("rs"))
                .map(|path| BuildTarget {
                    name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                    target_type: if path.file_name().unwrap_or_default() == "main.rs" {
                        BuildTargetType::Binary
                    } else if path.file_name().unwrap_or_default() == "lib.rs" {
                        BuildTargetType::Library
                    } else {
                        BuildTargetType::Binary
                    },
                    path: path.clone(),
                    features: vec![],
                })
                .collect();
        } else {
            // Fallback to mock data
            self.update_rust_tools();
        }
    }

    /// Update Alux tools with real project data
    pub fn update_alux_tools_from_project(
        &mut self,
        project: Option<&crate::project::Project>,
        _active_file: &PathBuf,
        open_files: &[PathBuf],
    ) {
        if let Some(proj) = project {
            // Update with real project data
            self.alux_tools.script_info = AluxScriptInfo {
                script_name: proj.name.clone(),
                version: "1.0.0".to_string(),
                author: "Project Author".to_string(),
                description: "Alux script project".to_string(),
                entry_point: open_files
                    .iter()
                    .find(|path| {
                        path.file_name().unwrap_or_default().to_string_lossy().contains("main")
                            && path.extension().and_then(|e| e.to_str()) == Some("alux")
                    })
                    .cloned(),
                dependencies: vec!["core".to_string(), "math".to_string()],
            };

            // Update modules from open files
            self.alux_tools.modules = open_files
                .iter()
                .filter(|path| {
                    matches!(path.extension().and_then(|e| e.to_str()), Some("alux") | Some("alx"))
                })
                .map(|path| AluxModule {
                    name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                    path: path.clone(),
                    exports: vec!["main".to_string(), "init".to_string()], // TODO: Parse from file
                    imports: vec!["core".to_string(), "math".to_string()], // TODO: Parse from file
                    line_count: 100, // TODO: Count actual lines
                })
                .collect();
        } else {
            // Fallback to mock data
            self.update_alux_tools();
        }
    }
}
