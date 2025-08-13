//! # File Tree Module
//!
//! File explorer tree widget for the GUI interface

use std::fs;
use std::path::{Path, PathBuf};

/// File tree widget for the file explorer
pub struct FileTreeWidget {
    /// Current root directory
    root_directory: PathBuf,
    /// Expanded directories
    expanded_dirs: std::collections::HashSet<PathBuf>,
    /// Selected file
    selected_file: Option<PathBuf>,
}

impl FileTreeWidget {
    /// Create a new file tree widget
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            root_directory: root_dir,
            expanded_dirs: std::collections::HashSet::new(),
            selected_file: None,
        }
    }

    /// Set the root directory
    pub fn set_root_directory(&mut self, path: PathBuf) {
        self.root_directory = path;
        self.expanded_dirs.clear();
        self.selected_file = None;
    }

    /// Get the selected file
    pub fn get_selected_file(&self) -> Option<&PathBuf> {
        self.selected_file.as_ref()
    }

    /// Draw the file tree widget
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        let mut file_to_open = None;

        ui.label("📁 File Explorer");
        ui.separator();

        // Current directory display
        ui.horizontal(|ui| {
            ui.label("Current:");
            ui.label(self.root_directory.display().to_string());
        });

        ui.separator();

        // Navigate up button
        if ui.button("⬆ Up").clicked() {
            if let Some(parent) = self.root_directory.parent() {
                self.set_root_directory(parent.to_path_buf());
            }
        }

        ui.separator();

        // File tree
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Ok(entries) = fs::read_dir(&self.root_directory) {
                let mut entries: Vec<_> = entries.collect();
                entries.sort_by(|a, b| match (a.as_ref().ok(), b.as_ref().ok()) {
                    (Some(a), Some(b)) => {
                        let a_is_dir = a.path().is_dir();
                        let b_is_dir = b.path().is_dir();

                        if a_is_dir && !b_is_dir {
                            std::cmp::Ordering::Less
                        } else if !a_is_dir && b_is_dir {
                            std::cmp::Ordering::Greater
                        } else {
                            a.file_name().cmp(&b.file_name())
                        }
                    }
                    _ => std::cmp::Ordering::Equal,
                });

                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

                        if path.is_dir() {
                            let is_expanded = self.expanded_dirs.contains(&path);
                            let icon = if is_expanded { "📂" } else { "📁" };

                            if ui.button(format!("{} {}", icon, name)).clicked() {
                                if is_expanded {
                                    self.expanded_dirs.remove(&path);
                                } else {
                                    self.expanded_dirs.insert(path.clone());
                                }
                            }

                            if is_expanded {
                                ui.indent("subdir", |ui| {
                                    self.show_directory_contents(ui, &path, &mut file_to_open);
                                });
                            }
                        } else {
                            let icon = self.get_file_icon(&path);
                            let is_selected = self.selected_file.as_ref() == Some(&path);

                            let button = if is_selected {
                                egui::Button::new(format!("{} {}", icon, name))
                                    .fill(egui::Color32::from_rgb(80, 120, 200))
                            } else {
                                egui::Button::new(format!("{} {}", icon, name))
                            };

                            if ui.add(button).clicked() {
                                self.selected_file = Some(path.clone());
                                file_to_open = Some(path);
                            }
                        }
                    }
                }
            }
        });

        file_to_open
    }

    /// Show contents of a directory (for expanded folders)
    fn show_directory_contents(
        &mut self,
        ui: &mut egui::Ui,
        dir_path: &Path,
        file_to_open: &mut Option<PathBuf>,
    ) {
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

                if path.is_dir() {
                    if ui.button(format!("📁 {}", name)).clicked() {
                        self.set_root_directory(path);
                    }
                } else {
                    let icon = self.get_file_icon(&path);
                    if ui.button(format!("{} {}", icon, name)).clicked() {
                        self.selected_file = Some(path.clone());
                        *file_to_open = Some(path);
                    }
                }
            }
        }
    }

    /// Get appropriate icon for file type
    fn get_file_icon(&self, path: &Path) -> &'static str {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
                "rs" => "🦀",
                "toml" => "⚙️",
                "json" => "📋",
                "md" | "txt" => "📄",
                "png" | "jpg" | "jpeg" | "gif" => "🖼️",
                "mp3" | "wav" | "ogg" => "🎵",
                "mp4" | "avi" | "mkv" => "🎬",
                "zip" | "tar" | "gz" => "📦",
                "exe" | "bin" => "⚡",
                _ => "📄",
            }
        } else {
            "📄"
        }
    }
}

impl Default for FileTreeWidget {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new(current_dir)
    }
}
