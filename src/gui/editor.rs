//! # GUI Editor Module
//!
//! Text editor component for the GUI interface

use super::FileBuffer;
use crate::syntax::{HighlightToken, TokenType};

/// Text editor widget for the GUI
pub struct EditorWidget {
    /// Current file buffer
    buffer: FileBuffer,
    /// Highlight tokens for current content
    highlight_tokens: Vec<HighlightToken>,
}

impl EditorWidget {
    /// Create a new editor widget
    pub fn new() -> Self {
        Self { buffer: FileBuffer::new(), highlight_tokens: Vec::new() }
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

    /// Update highlight tokens to be used on next render
    pub fn set_highlight_tokens(&mut self, tokens: Vec<HighlightToken>) {
        self.highlight_tokens = tokens;
    }

    fn color_for_token(token_type: &TokenType) -> egui::Color32 {
        match token_type {
            TokenType::Keyword => egui::Color32::from_rgb(86, 156, 214),
            TokenType::String => egui::Color32::from_rgb(206, 145, 120),
            TokenType::Number => egui::Color32::from_rgb(181, 206, 168),
            TokenType::Comment => egui::Color32::from_rgb(106, 153, 85),
            TokenType::Function => egui::Color32::from_rgb(220, 220, 170),
            TokenType::Variable => egui::Color32::from_rgb(156, 220, 254),
            TokenType::Type => egui::Color32::from_rgb(78, 201, 176),
            TokenType::Operator | TokenType::Punctuation | TokenType::Text => {
                egui::Color32::from_rgb(212, 212, 212)
            }
        }
    }

    /// Draw the editor widget
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let available_rect = ui.available_rect_before_wrap();

        egui::ScrollArea::both()
            .id_source("editor_scroll")
            .show(ui, |ui| {
                // Text editor
                let mut text_edit = egui::TextEdit::multiline(&mut self.buffer.content)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(30)
                    .lock_focus(true);

                // Syntax highlighting layouter
                let tokens = self.highlight_tokens.clone();
                let mut binding = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                    use egui::text::{LayoutJob, TextFormat};
                    let mut job = LayoutJob::default();
                    job.wrap.max_width = wrap_width;

                    // Fallback color
                    let default = TextFormat {
                        font_id: egui::FontId::monospace(14.0),
                        color: egui::Color32::from_rgb(212, 212, 212),
                        ..Default::default()
                    };

                    let mut cursor = 0usize;
                    for t in tokens.iter() {
                        let start = t.start.min(text.len());
                        let end = t.end.min(text.len());
                        if cursor < start {
                            job.append(&text[cursor..start], 0.0, default.clone());
                        }
                        let format = TextFormat {
                            font_id: egui::FontId::monospace(14.0),
                            color: Self::color_for_token(&t.token_type),
                            ..Default::default()
                        };
                        if start < end {
                            job.append(&text[start..end], 0.0, format);
                        }
                        cursor = end;
                    }
                    if cursor < text.len() {
                        job.append(&text[cursor..], 0.0, default);
                    }

                    ui.fonts(|f| f.layout_job(job))
                };
                text_edit = text_edit.layouter(&mut binding);

                let response = ui.add_sized(available_rect.size(), text_edit);

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
