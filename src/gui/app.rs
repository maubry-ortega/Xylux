//! # Main GUI Application
//!
//! Composed, production-focused GUI app for Xylux IDE.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{IdTheme, ToolsWindow};
use crate::core::{Config, EventBus};
use crate::editor::Editor;
use crate::project::ProjectManager;
use crate::syntax::SyntaxManager;

use crate::gui::editor::EditorWidget;
use crate::gui::file_tree::FileTreeWidget;
use crate::gui::menu::{MenuAction, MenuBarWidget};
use crate::gui::statusbar::{StatusBarWidget, StatusInfo};

/// Main Xylux IDE application (GUI-only)
pub struct XyluxIdeApp {
    config: Arc<RwLock<Config>>,
    event_bus: Arc<EventBus>,
    editor: Editor,
    syntax_manager: SyntaxManager,
    project_manager: ProjectManager,
    rt: tokio::runtime::Runtime,

    theme: IdTheme,
    show_file_explorer: bool,
    current_directory: PathBuf,
    status_message: String,
    file_dialog_open: bool,
    about_dialog_open: bool,

    // Widgets
    menu: MenuBarWidget,
    status_bar: StatusBarWidget,
    file_tree: FileTreeWidget,
    editor_widget: EditorWidget,
    tools_window: ToolsWindow,
}

impl XyluxIdeApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        Self::configure_style(&cc.egui_ctx);

        let config = Arc::new(RwLock::new(config));
        let event_bus = Arc::new(EventBus::new());
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

        let editor = rt
            .block_on(Editor::new(config.clone(), event_bus.clone()))
            .expect("editor init");
        let syntax_manager = rt
            .block_on(SyntaxManager::new(config.clone(), event_bus.clone()))
            .expect("syntax init");
        let project_manager = ProjectManager::new(config.clone(), event_bus.clone());

        Self {
            config,
            event_bus,
            editor,
            syntax_manager,
            project_manager,
            rt,
            theme: IdTheme::default(),
            show_file_explorer: true,
            current_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            status_message: "Ready".into(),
            file_dialog_open: false,
            about_dialog_open: false,
            menu: MenuBarWidget::new(),
            status_bar: StatusBarWidget::new(),
            file_tree: FileTreeWidget::default(),
            editor_widget: EditorWidget::new(),
            tools_window: ToolsWindow::new(),
        }
    }

    fn configure_style(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 220));
        style.visuals.panel_fill = egui::Color32::from_rgb(30, 30, 30);
        style.visuals.window_fill = egui::Color32::from_rgb(40, 40, 40);
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(20, 20, 20);
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.indent = 20.0;
        ctx.set_style(style);
    }

    fn handle_menu_action(&mut self, ctx: &egui::Context, action: MenuAction) {
        match action {
            MenuAction::NewFile => self.new_file(),
            MenuAction::OpenFile => self.file_dialog_open = true,
            MenuAction::OpenFolder => {
                // Basic: switch file tree root to current dir's parent
                if let Some(parent) = self.current_directory.parent() {
                    self.current_directory = parent.to_path_buf();
                    self.file_tree.set_root_directory(self.current_directory.clone());
                }
            }
            MenuAction::Save => self.save_file(),
            MenuAction::CloseFile => self.close_file(),
            MenuAction::Exit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            MenuAction::ToggleSpecializedTools => self.tools_window.toggle(),
            _ => {}
        }
    }

    fn new_file(&mut self) {
        if self.rt.block_on(self.editor.open_file(&PathBuf::from(""))).is_err() {
            self.status_message = "Error creating new file".into();
        } else {
            self.editor_widget.set_buffer(super::FileBuffer::new());
            self.status_message = "New file".into();
        }
    }

    pub fn open_file(&mut self, path: PathBuf) {
        match fs::read_to_string(&path) {
            Ok(content) => {
                self.editor_widget
                    .set_buffer(super::FileBuffer::from_file(path.clone(), content));
                if let Err(e) = self.rt.block_on(self.editor.open_file(&path)) {
                    self.status_message = format!("Open error: {}", e);
                } else {
                    let _unused = self
                        .rt
                        .block_on(self.syntax_manager.highlight_file(&path, &self.editor_widget.get_buffer().content));
                    self.status_message = format!("Opened: {}", path.display());
                }
            }
            Err(e) => self.status_message = format!("Read error: {}", e),
        }
    }

    fn save_file(&mut self) {
        let content = self.editor_widget.get_buffer().content.clone();
        if let Err(e) = self.rt.block_on(self.editor.insert_text(&content)) {
            self.status_message = format!("Update error: {}", e);
            return;
        }
        if let Err(e) = self.rt.block_on(self.editor.save_current()) {
            self.status_message = format!("Save error: {}", e);
        } else {
            self.status_message = "Saved".into();
        }
    }

    fn close_file(&mut self) {
        let files = self.rt.block_on(self.editor.get_open_files());
        if let Some(path) = files.first() {
            if let Err(e) = self.rt.block_on(self.editor.close_file(path)) {
                self.status_message = format!("Close error: {}", e);
            } else {
                self.editor_widget.set_buffer(super::FileBuffer::new());
                self.status_message = "Closed".into();
            }
        }
    }

    fn update_status_from_buffer(&mut self) {
        let buf = self.editor_widget.get_buffer();
        let mut info = StatusInfo::new();
        info.update_from_buffer(&buf.content, buf.cursor_line, buf.cursor_column, buf.modified);
        info.set_file_path(buf.path.clone());
        self.status_bar.set_status_message(self.status_message.clone());
        self.status_bar.set_current_file(buf.path.clone());
        self.status_bar.set_cursor_position(info.cursor_line, info.cursor_column);
        self.status_bar.set_modified(buf.modified);
    }

    fn update_tools_data(&mut self) {
        let current_project = self.rt.block_on(self.project_manager.active_project());
        if let Some(active_path) = self.editor_widget.get_buffer().path.clone() {
            let ext = active_path.extension().and_then(|e| e.to_str());
            match ext {
                Some("rs") => self.tools_window.update_rust_tools_from_project(
                    current_project.as_ref(),
                    &active_path,
                    &self.rt.block_on(self.editor.get_open_files()),
                ),
                Some("alux") | Some("alx") => self.tools_window.update_alux_tools_from_project(
                    current_project.as_ref(),
                    &active_path,
                    &self.rt.block_on(self.editor.get_open_files()),
                ),
                _ => {
                    self.tools_window.update_rust_tools();
                    self.tools_window.update_alux_tools();
                }
            }
        } else {
            self.tools_window.update_rust_tools();
            self.tools_window.update_alux_tools();
        }
    }

    fn draw_file_dialog(&mut self, ctx: &egui::Context) {
        if !self.file_dialog_open { return; }
        egui::Window::new("Open File").collapsible(false).resizable(true).show(ctx, |ui| {
            ui.label("Select a file:");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Ok(entries) = fs::read_dir(&self.current_directory) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                        if path.is_dir() {
                            if ui.button(format!("📁 {}", name)).clicked() {
                                self.current_directory = path;
                                self.file_tree.set_root_directory(self.current_directory.clone());
                            }
                        } else if ui.button(format!("📄 {}", name)).clicked() {
                            self.open_file(path);
                            self.file_dialog_open = false;
                        }
                    }
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() { self.file_dialog_open = false; }
                if ui.button("Up").clicked() {
                    if let Some(parent) = self.current_directory.parent() {
                        self.current_directory = parent.to_path_buf();
                        self.file_tree.set_root_directory(self.current_directory.clone());
                    }
                }
            });
        });
    }
}

impl eframe::App for XyluxIdeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Q) && i.modifiers.ctrl) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            let action = self.menu.show(ui);
            self.handle_menu_action(ctx, action);
        });

        if self.show_file_explorer {
            egui::SidePanel::left("explorer").resizable(true).default_width(220.0).show(ctx, |ui| {
                if let Some(path) = self.file_tree.show(ui) { self.open_file(path); }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let resp = self.editor_widget.show(ui);
            if resp.changed() { self.status_message = "Modified".into(); }
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            self.update_status_from_buffer();
            self.status_bar.show(ui);
        });

        // Dialogs and tools
        if self.about_dialog_open {
            egui::Window::new("About Xylux IDE").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.label("Xylux IDE");
                ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                if ui.button("Close").clicked() { self.about_dialog_open = false; }
            });
        }
        self.draw_file_dialog(ctx);
        self.update_tools_data();
        self.tools_window.show(ctx);

        // Shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) { self.save_file(); }
        if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) { self.file_dialog_open = true; }
        if ctx.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.ctrl) { self.new_file(); }

        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}
