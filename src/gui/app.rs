//! # Main GUI Application
//!
//! Main application structure for Xylux IDE GUI

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::{FileBuffer, IdTheme, ToolsWindow};
use crate::core::Config;

/// Main Xylux IDE application
pub struct XyluxIdeApp {
    /// Current open files
    open_files: HashMap<usize, FileBuffer>,
    /// Currently active file tab
    active_file: Option<usize>,
    /// Next file ID counter
    next_file_id: usize,
    /// IDE theme
    theme: IdTheme,
    /// Configuration
    config: Config,
    /// Show file explorer
    show_file_explorer: bool,
    /// Current directory for file explorer
    current_directory: PathBuf,
    /// Status message
    status_message: String,
    /// File dialog state
    file_dialog_open: bool,
    /// About dialog state
    about_dialog_open: bool,
    /// Specialized tools window
    tools_window: ToolsWindow,
}

impl XyluxIdeApp {
    /// Create a new IDE application
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        // Setup egui style
        Self::configure_style(&cc.egui_ctx);

        let mut app = Self {
            open_files: HashMap::new(),
            active_file: None,
            next_file_id: 0,
            theme: IdTheme::default(),
            config,
            show_file_explorer: true,
            current_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            status_message: "Ready".to_string(),
            file_dialog_open: false,
            about_dialog_open: false,
            tools_window: ToolsWindow::new(),
        };

        // Create initial empty file
        app.new_file();
        app
    }

    /// Configure egui visual style
    fn configure_style(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // Dark theme
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 220));
        style.visuals.panel_fill = egui::Color32::from_rgb(30, 30, 30);
        style.visuals.window_fill = egui::Color32::from_rgb(40, 40, 40);
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(20, 20, 20);

        // Spacing
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.indent = 20.0;

        ctx.set_style(style);
    }

    /// Create a new file
    pub fn new_file(&mut self) {
        let file_id = self.next_file_id;
        self.next_file_id += 1;

        let buffer = FileBuffer::new();
        self.open_files.insert(file_id, buffer);
        self.active_file = Some(file_id);
        self.status_message = "New file created".to_string();
    }

    /// Open a file
    pub fn open_file(&mut self, path: PathBuf) {
        match fs::read_to_string(&path) {
            Ok(content) => {
                let file_id = self.next_file_id;
                self.next_file_id += 1;

                let buffer = FileBuffer::from_file(path.clone(), content);
                self.open_files.insert(file_id, buffer);
                self.active_file = Some(file_id);
                self.status_message = format!("Opened: {}", path.display());
            }
            Err(e) => {
                self.status_message = format!("Error opening file: {}", e);
            }
        }
    }

    /// Save current file
    pub fn save_file(&mut self) {
        if let Some(file_id) = self.active_file {
            if let Some(buffer) = self.open_files.get_mut(&file_id) {
                if let Some(path) = &buffer.path {
                    match fs::write(path, &buffer.content) {
                        Ok(()) => {
                            buffer.modified = false;
                            self.status_message = format!("Saved: {}", path.display());
                        }
                        Err(e) => {
                            self.status_message = format!("Error saving file: {}", e);
                        }
                    }
                } else {
                    // TODO: Implement save as dialog
                    self.status_message = "Please use Save As for untitled files".to_string();
                }
            }
        }
    }

    /// Close current file
    pub fn close_file(&mut self) {
        if let Some(file_id) = self.active_file {
            self.open_files.remove(&file_id);

            // Find next active file
            self.active_file = self.open_files.keys().next().copied();

            // If no files left, create a new one
            if self.active_file.is_none() {
                self.new_file();
            }

            self.status_message = "File closed".to_string();
        }
    }

    /// Get the currently active file buffer
    pub fn get_active_buffer(&self) -> Option<&FileBuffer> {
        self.active_file.and_then(|id| self.open_files.get(&id))
    }

    /// Get the currently active file buffer mutably
    pub fn get_active_buffer_mut(&mut self) -> Option<&mut FileBuffer> {
        self.active_file.and_then(move |id| self.open_files.get_mut(&id))
    }

    /// Draw the menu bar
    fn draw_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.new_file();
                        ui.close_menu();
                    }
                    if ui.button("Open...").clicked() {
                        self.file_dialog_open = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        // TODO: Implement save as
                        self.status_message = "Save As not yet implemented".to_string();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.close_file();
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close_menu();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Cut").clicked() {
                        self.status_message = "Cut not yet implemented".to_string();
                        ui.close_menu();
                    }
                    if ui.button("Copy").clicked() {
                        self.status_message = "Copy not yet implemented".to_string();
                        ui.close_menu();
                    }
                    if ui.button("Paste").clicked() {
                        self.status_message = "Paste not yet implemented".to_string();
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.checkbox(&mut self.show_file_explorer, "File Explorer").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("Tools", |ui| {
                    if ui.button("🔧 Specialized Tools").clicked() {
                        self.tools_window.toggle();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.about_dialog_open = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    /// Draw file tabs
    fn draw_file_tabs(&mut self, ctx: &egui::Context) {
        if self.open_files.len() > 1 {
            egui::TopBottomPanel::top("file_tabs").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let mut files_to_remove = Vec::new();

                    for (&file_id, buffer) in &self.open_files {
                        let is_active = self.active_file == Some(file_id);

                        let button_text = buffer.get_display_name();
                        let button = if is_active {
                            egui::Button::new(&button_text)
                                .fill(egui::Color32::from_rgb(80, 80, 80))
                        } else {
                            egui::Button::new(&button_text)
                                .fill(egui::Color32::from_rgb(50, 50, 50))
                        };

                        if ui.add(button).clicked() {
                            self.active_file = Some(file_id);
                        }

                        // Close button for tab
                        if ui.small_button("×").clicked() {
                            files_to_remove.push(file_id);
                        }
                    }

                    // Remove closed files
                    for file_id in files_to_remove {
                        self.open_files.remove(&file_id);
                        if self.active_file == Some(file_id) {
                            self.active_file = self.open_files.keys().next().copied();
                            if self.active_file.is_none() {
                                self.new_file();
                            }
                        }
                    }
                });
            });
        }
    }

    /// Draw the main editor area
    fn draw_editor(&mut self, ctx: &egui::Context) {
        let mut save_requested = false;
        let mut new_file_requested = false;
        let mut file_modified = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(buffer) = self.get_active_buffer_mut() {
                let _lines = buffer.get_lines();

                egui::ScrollArea::vertical().id_source("editor_scroll").show(ui, |ui| {
                    // Create text editor
                    let response = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut buffer.content)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(30),
                    );

                    // Handle keyboard input
                    if response.has_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
                            save_requested = true;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.ctrl) {
                            new_file_requested = true;
                        }
                    }

                    if response.changed() {
                        buffer.modified = true;
                        file_modified = true;
                    }
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No file open. Use Ctrl+N to create a new file or Ctrl+O to open an existing file.");
                });
            }
        });

        // Handle actions after the borrow is released
        if save_requested {
            self.save_file();
        }
        if new_file_requested {
            self.new_file();
        }
        if file_modified {
            self.status_message = "File modified".to_string();
        }
    }

    /// Draw file explorer
    fn draw_file_explorer(&mut self, ctx: &egui::Context) {
        if self.show_file_explorer {
            egui::SidePanel::left("file_explorer").resizable(true).default_width(200.0).show(
                ctx,
                |ui| {
                    ui.label("File Explorer");
                    ui.separator();

                    if ui.button("📁 Open Folder").clicked() {
                        // TODO: Implement folder selection
                        self.status_message = "Folder selection not yet implemented".to_string();
                    }

                    ui.separator();
                    ui.label(format!("Current: {}", self.current_directory.display()));

                    // List files in current directory
                    if let Ok(entries) = fs::read_dir(&self.current_directory) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

                            if path.is_dir() {
                                if ui.button(format!("📁 {}", name)).clicked() {
                                    self.current_directory = path;
                                }
                            } else {
                                if ui.button(format!("📄 {}", name)).clicked() {
                                    self.open_file(path);
                                }
                            }
                        }
                    }
                },
            );
        }
    }

    /// Draw status bar
    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(buffer) = self.get_active_buffer() {
                        ui.label(format!(
                            "Line: {}, Col: {}",
                            buffer.cursor_line + 1,
                            buffer.cursor_column + 1
                        ));
                    }
                });
            });
        });
    }

    /// Draw dialogs
    fn draw_dialogs(&mut self, ctx: &egui::Context) {
        // About dialog
        if self.about_dialog_open {
            egui::Window::new("About Xylux IDE").collapsible(false).resizable(false).show(
                ctx,
                |ui| {
                    ui.label("Xylux IDE");
                    ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                    ui.label("A modern IDE for Rust development and Alux scripting");
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            self.about_dialog_open = false;
                        }
                    });
                },
            );
        }

        // Simple file dialog (basic implementation)
        if self.file_dialog_open {
            egui::Window::new("Open File").collapsible(false).resizable(true).show(ctx, |ui| {
                ui.label("Select a file to open:");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Ok(entries) = fs::read_dir(&self.current_directory) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

                            if path.is_dir() {
                                if ui.button(format!("📁 {}", name)).clicked() {
                                    self.current_directory = path;
                                }
                            } else {
                                if ui.button(format!("📄 {}", name)).clicked() {
                                    self.open_file(path);
                                    self.file_dialog_open = false;
                                }
                            }
                        }
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.file_dialog_open = false;
                    }
                    if ui.button("Up").clicked() {
                        if let Some(parent) = self.current_directory.parent() {
                            self.current_directory = parent.to_path_buf();
                        }
                    }
                });
            });
        }
    }

    /// Update tools data based on current project and files
    fn update_tools_data(&mut self) {
        // Update Rust tools data
        self.tools_window.update_rust_tools();

        // Update Alux tools data
        self.tools_window.update_alux_tools();

        // Check if current file is Rust or Alux to provide specific context
        if let Some(buffer) = self.get_active_buffer() {
            if let Some(path) = &buffer.path {
                let extension = path.extension().and_then(|ext| ext.to_str());
                match extension {
                    Some("rs") => {
                        // Rust file specific updates
                        self.status_message = "Rust tools updated".to_string();
                    }
                    Some("alux") | Some("alx") => {
                        // Alux file specific updates
                        self.status_message = "Alux tools updated".to_string();
                    }
                    _ => {}
                }
            }
        }
    }
}

impl eframe::App for XyluxIdeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle global shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::Q) && i.modifiers.ctrl) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Draw UI components
        self.draw_menu_bar(ctx);
        self.draw_file_tabs(ctx);
        self.draw_file_explorer(ctx);
        self.draw_status_bar(ctx);
        self.draw_editor(ctx);
        self.draw_dialogs(ctx);

        // Update and draw specialized tools window
        self.update_tools_data();
        self.tools_window.show(ctx);

        // Clear status message after a while
        ctx.request_repaint_after(std::time::Duration::from_secs(3));
    }
}
