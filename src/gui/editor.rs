//! # GUI Editor Module
//!
//! Text editor component for the GUI interface

use super::FileBuffer;

/// Text editor widget for the GUI
pub struct EditorWidget {
    /// Current file buffer
    buffer: FileBuffer,
}

impl EditorWidget {
    /// Create a new editor widget
    pub fn new() -> Self {
        Self { buffer: FileBuffer::new() }
    }

    /// Set the current buffer
    pub fn set_buffer(&mut self, buffer: FileBuffer) {
        self.buffer = buffer;
    }

    /// Get the current buffer
    pub fn get_buffer(&self) -> &FileBuffer {
        &self.buffer
    }

    /// Get the current buffer mutably
    pub fn get_buffer_mut(&mut self) -> &mut FileBuffer {
        &mut self.buffer
    }

    /// Draw the editor widget
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let available_rect = ui.available_rect_before_wrap();

        egui::ScrollArea::both()
            .id_source("editor_scroll")
            .show(ui, |ui| {
                // Text editor
                let response = ui.add_sized(
                    available_rect.size(),
                    egui::TextEdit::multiline(&mut self.buffer.content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(30)
                        .lock_focus(true),
                );

                if response.changed() {
                    self.buffer.modified = true;
                }

                response
            })
            .inner
    }
}

impl Default for EditorWidget {
    fn default() -> Self {
        Self::new()
    }
}
