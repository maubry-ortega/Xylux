//! # Menu Module
//!
//! Menu bar and context menu components for the GUI interface

/// Menu bar widget for the main application
pub struct MenuBarWidget {
    /// Whether file menu is open
    file_menu_open: bool,
    /// Whether edit menu is open
    edit_menu_open: bool,
    /// Whether view menu is open
    view_menu_open: bool,
    /// Whether help menu is open
    help_menu_open: bool,
}

impl MenuBarWidget {
    /// Create a new menu bar widget
    pub fn new() -> Self {
        Self {
            file_menu_open: false,
            edit_menu_open: false,
            view_menu_open: false,
            help_menu_open: false,
        }
    }

    /// Draw the menu bar
    pub fn show(&mut self, ui: &mut egui::Ui) -> MenuAction {
        let mut action = MenuAction::None;

        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New File").clicked() {
                    action = MenuAction::NewFile;
                    ui.close_menu();
                }
                if ui.button("Open File...").clicked() {
                    action = MenuAction::OpenFile;
                    ui.close_menu();
                }
                if ui.button("Open Folder...").clicked() {
                    action = MenuAction::OpenFolder;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Save").clicked() {
                    action = MenuAction::Save;
                    ui.close_menu();
                }
                if ui.button("Save As...").clicked() {
                    action = MenuAction::SaveAs;
                    ui.close_menu();
                }
                if ui.button("Save All").clicked() {
                    action = MenuAction::SaveAll;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Close File").clicked() {
                    action = MenuAction::CloseFile;
                    ui.close_menu();
                }
                if ui.button("Close All").clicked() {
                    action = MenuAction::CloseAll;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    action = MenuAction::Exit;
                    ui.close_menu();
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    action = MenuAction::Undo;
                    ui.close_menu();
                }
                if ui.button("Redo").clicked() {
                    action = MenuAction::Redo;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Cut").clicked() {
                    action = MenuAction::Cut;
                    ui.close_menu();
                }
                if ui.button("Copy").clicked() {
                    action = MenuAction::Copy;
                    ui.close_menu();
                }
                if ui.button("Paste").clicked() {
                    action = MenuAction::Paste;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Select All").clicked() {
                    action = MenuAction::SelectAll;
                    ui.close_menu();
                }
                if ui.button("Find...").clicked() {
                    action = MenuAction::Find;
                    ui.close_menu();
                }
                if ui.button("Replace...").clicked() {
                    action = MenuAction::Replace;
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("File Explorer").clicked() {
                    action = MenuAction::ToggleFileExplorer;
                    ui.close_menu();
                }
                if ui.button("Terminal").clicked() {
                    action = MenuAction::ToggleTerminal;
                    ui.close_menu();
                }
                if ui.button("Output Panel").clicked() {
                    action = MenuAction::ToggleOutput;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Zoom In").clicked() {
                    action = MenuAction::ZoomIn;
                    ui.close_menu();
                }
                if ui.button("Zoom Out").clicked() {
                    action = MenuAction::ZoomOut;
                    ui.close_menu();
                }
                if ui.button("Reset Zoom").clicked() {
                    action = MenuAction::ResetZoom;
                    ui.close_menu();
                }
            });

            ui.menu_button("Run", |ui| {
                if ui.button("Build Project").clicked() {
                    action = MenuAction::Build;
                    ui.close_menu();
                }
                if ui.button("Run Project").clicked() {
                    action = MenuAction::Run;
                    ui.close_menu();
                }
                if ui.button("Test Project").clicked() {
                    action = MenuAction::Test;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Clean Build").clicked() {
                    action = MenuAction::Clean;
                    ui.close_menu();
                }
            });

            ui.menu_button("Tools", |ui| {
                if ui.button("🔧 Specialized Tools").clicked() {
                    action = MenuAction::ToggleSpecializedTools;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Command Palette").clicked() {
                    action = MenuAction::CommandPalette;
                    ui.close_menu();
                }
                if ui.button("Settings").clicked() {
                    action = MenuAction::Settings;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Format Document").clicked() {
                    action = MenuAction::FormatDocument;
                    ui.close_menu();
                }
                if ui.button("Go to Line...").clicked() {
                    action = MenuAction::GoToLine;
                    ui.close_menu();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("Documentation").clicked() {
                    action = MenuAction::ShowDocumentation;
                    ui.close_menu();
                }
                if ui.button("Keyboard Shortcuts").clicked() {
                    action = MenuAction::ShowShortcuts;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Check for Updates").clicked() {
                    action = MenuAction::CheckUpdates;
                    ui.close_menu();
                }
                if ui.button("About").clicked() {
                    action = MenuAction::About;
                    ui.close_menu();
                }
            });
        });

        action
    }
}

impl Default for MenuBarWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions that can be triggered from the menu
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    None,
    // File menu actions
    NewFile,
    OpenFile,
    OpenFolder,
    Save,
    SaveAs,
    SaveAll,
    CloseFile,
    CloseAll,
    Exit,
    // Edit menu actions
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Find,
    Replace,
    // View menu actions
    ToggleFileExplorer,
    ToggleTerminal,
    ToggleOutput,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    // Run menu actions
    Build,
    Run,
    Test,
    Clean,
    // Tools menu actions
    ToggleSpecializedTools,
    CommandPalette,
    Settings,
    FormatDocument,
    GoToLine,
    // Help menu actions
    ShowDocumentation,
    ShowShortcuts,
    CheckUpdates,
    About,
}

/// Context menu widget for right-click menus
pub struct ContextMenuWidget;

impl ContextMenuWidget {
    /// Show a context menu for the editor
    pub fn show_editor_context_menu(ui: &mut egui::Ui, pos: egui::Pos2) -> MenuAction {
        let mut action = MenuAction::None;

        ui.allocate_ui_at_rect(egui::Rect::from_min_size(pos, egui::Vec2::splat(1.0)), |ui| {
            egui::menu::menu_button(ui, "Context", |ui| {
                if ui.button("Cut").clicked() {
                    action = MenuAction::Cut;
                    ui.close_menu();
                }
                if ui.button("Copy").clicked() {
                    action = MenuAction::Copy;
                    ui.close_menu();
                }
                if ui.button("Paste").clicked() {
                    action = MenuAction::Paste;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Select All").clicked() {
                    action = MenuAction::SelectAll;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Go to Line...").clicked() {
                    action = MenuAction::GoToLine;
                    ui.close_menu();
                }
                if ui.button("Format Document").clicked() {
                    action = MenuAction::FormatDocument;
                    ui.close_menu();
                }
            });
        });

        action
    }

    /// Show a context menu for the file explorer
    pub fn show_file_context_menu(ui: &mut egui::Ui, pos: egui::Pos2) -> MenuAction {
        let mut action = MenuAction::None;

        ui.allocate_ui_at_rect(egui::Rect::from_min_size(pos, egui::Vec2::splat(1.0)), |ui| {
            egui::menu::menu_button(ui, "File Context", |ui| {
                if ui.button("Open").clicked() {
                    action = MenuAction::OpenFile;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("New File").clicked() {
                    action = MenuAction::NewFile;
                    ui.close_menu();
                }
                if ui.button("New Folder").clicked() {
                    // TODO: Implement new folder action
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Rename").clicked() {
                    // TODO: Implement rename action
                    ui.close_menu();
                }
                if ui.button("Delete").clicked() {
                    // TODO: Implement delete action
                    ui.close_menu();
                }
            });
        });

        action
    }
}
