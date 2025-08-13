//! # Theme Module
//!
//! Theme management for Xylux IDE.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::{Result, XyluxError};

/// Theme configuration for the IDE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name.
    pub name: String,
    /// Theme description.
    pub description: String,
    /// Theme author.
    pub author: String,
    /// Theme version.
    pub version: String,

    // Basic colors
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    pub selection: String,

    // UI colors
    pub ui: UiColors,

    // Syntax highlighting colors
    pub syntax: SyntaxColors,

    // Editor colors
    pub editor: EditorColors,

    // Terminal colors
    pub terminal: TerminalColors,
}

/// UI-specific colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    pub border: String,
    pub border_focused: String,
    pub title: String,
    pub title_focused: String,
    pub menu: String,
    pub menu_selected: String,
    pub button: String,
    pub button_hover: String,
    pub button_pressed: String,
    pub scrollbar: String,
    pub scrollbar_thumb: String,
    pub status_bar: String,
    pub status_bar_text: String,
    pub tab: String,
    pub tab_active: String,
    pub tab_text: String,
    pub tab_text_active: String,
}

/// Syntax highlighting colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    pub keyword: String,
    pub string: String,
    pub number: String,
    pub comment: String,
    pub function: String,
    pub variable: String,
    pub r#type: String,
    pub operator: String,
    pub punctuation: String,
    pub constant: String,
    pub attribute: String,
    pub macro_name: String,
    pub label: String,
    pub namespace: String,
    pub property: String,
    pub struct_name: String,
    pub enum_name: String,
    pub interface: String,
    pub parameter: String,
    pub lifetime: String,
    pub builtin: String,
    pub error: String,
    pub warning: String,
    pub info: String,
    pub hint: String,
}

/// Editor-specific colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorColors {
    pub line_number: String,
    pub line_number_active: String,
    pub gutter: String,
    pub gutter_active: String,
    pub indent_guide: String,
    pub whitespace: String,
    pub matching_bracket: String,
    pub current_line: String,
    pub find_match: String,
    pub find_match_active: String,
    pub error_underline: String,
    pub warning_underline: String,
    pub info_underline: String,
    pub hint_underline: String,
}

/// Terminal colors (ANSI colors).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalColors {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
}

impl Theme {
    /// Load a theme by name.
    pub fn load(theme_name: &str) -> Result<Self> {
        match theme_name {
            "dark" | "default" => Ok(Self::dark_theme()),
            "light" => Ok(Self::light_theme()),
            "solarized_dark" => Ok(Self::solarized_dark_theme()),
            "solarized_light" => Ok(Self::solarized_light_theme()),
            "monokai" => Ok(Self::monokai_theme()),
            "github" => Ok(Self::github_theme()),
            "dracula" => Ok(Self::dracula_theme()),
            "one_dark" => Ok(Self::one_dark_theme()),
            _ => Err(XyluxError::config(format!("Unknown theme: {}", theme_name))),
        }
    }

    /// Get list of available themes.
    pub fn available_themes() -> Vec<&'static str> {
        vec![
            "dark",
            "light",
            "solarized_dark",
            "solarized_light",
            "monokai",
            "github",
            "dracula",
            "one_dark",
        ]
    }

    /// Create the default dark theme.
    pub fn dark_theme() -> Self {
        Self {
            name: "Dark".to_string(),
            description: "Default dark theme".to_string(),
            author: "Xylux IDE".to_string(),
            version: "1.0.0".to_string(),

            foreground: "#FFFFFF".to_string(),
            background: "#1E1E1E".to_string(),
            cursor: "#FFFFFF".to_string(),
            selection: "#264F78".to_string(),

            ui: UiColors {
                border: "#3C3C3C".to_string(),
                border_focused: "#007ACC".to_string(),
                title: "#CCCCCC".to_string(),
                title_focused: "#FFFFFF".to_string(),
                menu: "#2D2D30".to_string(),
                menu_selected: "#094771".to_string(),
                button: "#0E639C".to_string(),
                button_hover: "#1177BB".to_string(),
                button_pressed: "#0A4F85".to_string(),
                scrollbar: "#424242".to_string(),
                scrollbar_thumb: "#686868".to_string(),
                status_bar: "#007ACC".to_string(),
                status_bar_text: "#FFFFFF".to_string(),
                tab: "#2D2D30".to_string(),
                tab_active: "#1E1E1E".to_string(),
                tab_text: "#969696".to_string(),
                tab_text_active: "#FFFFFF".to_string(),
            },

            syntax: SyntaxColors {
                keyword: "#569CD6".to_string(),
                string: "#CE9178".to_string(),
                number: "#B5CEA8".to_string(),
                comment: "#6A9955".to_string(),
                function: "#DCDCAA".to_string(),
                variable: "#9CDCFE".to_string(),
                r#type: "#4EC9B0".to_string(),
                operator: "#D4D4D4".to_string(),
                punctuation: "#D4D4D4".to_string(),
                constant: "#4FC1FF".to_string(),
                attribute: "#92C5F7".to_string(),
                macro_name: "#569CD6".to_string(),
                label: "#C8C8C8".to_string(),
                namespace: "#4EC9B0".to_string(),
                property: "#92C5F7".to_string(),
                struct_name: "#4EC9B0".to_string(),
                enum_name: "#4EC9B0".to_string(),
                interface: "#B8D7A3".to_string(),
                parameter: "#9CDCFE".to_string(),
                lifetime: "#C586C0".to_string(),
                builtin: "#569CD6".to_string(),
                error: "#F44747".to_string(),
                warning: "#FF8C00".to_string(),
                info: "#007ACC".to_string(),
                hint: "#808080".to_string(),
            },

            editor: EditorColors {
                line_number: "#858585".to_string(),
                line_number_active: "#FFFFFF".to_string(),
                gutter: "#1E1E1E".to_string(),
                gutter_active: "#1E1E1E".to_string(),
                indent_guide: "#404040".to_string(),
                whitespace: "#404040".to_string(),
                matching_bracket: "#0064FF".to_string(),
                current_line: "#2A2D2E".to_string(),
                find_match: "#515C6A".to_string(),
                find_match_active: "#613214".to_string(),
                error_underline: "#F44747".to_string(),
                warning_underline: "#FF8C00".to_string(),
                info_underline: "#007ACC".to_string(),
                hint_underline: "#808080".to_string(),
            },

            terminal: TerminalColors {
                black: "#000000".to_string(),
                red: "#CD3131".to_string(),
                green: "#0DBC79".to_string(),
                yellow: "#E5E510".to_string(),
                blue: "#2472C8".to_string(),
                magenta: "#BC3FBC".to_string(),
                cyan: "#11A8CD".to_string(),
                white: "#E5E5E5".to_string(),
                bright_black: "#666666".to_string(),
                bright_red: "#F14C4C".to_string(),
                bright_green: "#23D18B".to_string(),
                bright_yellow: "#F5F543".to_string(),
                bright_blue: "#3B8EEA".to_string(),
                bright_magenta: "#D670D6".to_string(),
                bright_cyan: "#29B8DB".to_string(),
                bright_white: "#E5E5E5".to_string(),
            },
        }
    }

    /// Create a light theme.
    pub fn light_theme() -> Self {
        Self {
            name: "Light".to_string(),
            description: "Light theme".to_string(),
            author: "Xylux IDE".to_string(),
            version: "1.0.0".to_string(),

            foreground: "#000000".to_string(),
            background: "#FFFFFF".to_string(),
            cursor: "#000000".to_string(),
            selection: "#ADD6FF".to_string(),

            ui: UiColors {
                border: "#C8C8C8".to_string(),
                border_focused: "#007ACC".to_string(),
                title: "#333333".to_string(),
                title_focused: "#000000".to_string(),
                menu: "#F3F3F3".to_string(),
                menu_selected: "#E4E6F1".to_string(),
                button: "#007ACC".to_string(),
                button_hover: "#005A9E".to_string(),
                button_pressed: "#004578".to_string(),
                scrollbar: "#C1C1C1".to_string(),
                scrollbar_thumb: "#969696".to_string(),
                status_bar: "#007ACC".to_string(),
                status_bar_text: "#FFFFFF".to_string(),
                tab: "#F3F3F3".to_string(),
                tab_active: "#FFFFFF".to_string(),
                tab_text: "#6F6F6F".to_string(),
                tab_text_active: "#000000".to_string(),
            },

            syntax: SyntaxColors {
                keyword: "#0000FF".to_string(),
                string: "#A31515".to_string(),
                number: "#098658".to_string(),
                comment: "#008000".to_string(),
                function: "#795E26".to_string(),
                variable: "#001080".to_string(),
                r#type: "#267F99".to_string(),
                operator: "#000000".to_string(),
                punctuation: "#000000".to_string(),
                constant: "#0451A5".to_string(),
                attribute: "#267F99".to_string(),
                macro_name: "#0000FF".to_string(),
                label: "#000000".to_string(),
                namespace: "#267F99".to_string(),
                property: "#001080".to_string(),
                struct_name: "#267F99".to_string(),
                enum_name: "#267F99".to_string(),
                interface: "#267F99".to_string(),
                parameter: "#001080".to_string(),
                lifetime: "#AF00DB".to_string(),
                builtin: "#0000FF".to_string(),
                error: "#E51400".to_string(),
                warning: "#BF8803".to_string(),
                info: "#1BA1E2".to_string(),
                hint: "#6C6C6C".to_string(),
            },

            editor: EditorColors {
                line_number: "#237893".to_string(),
                line_number_active: "#0B216F".to_string(),
                gutter: "#F5F5F5".to_string(),
                gutter_active: "#F5F5F5".to_string(),
                indent_guide: "#D3D3D3".to_string(),
                whitespace: "#D3D3D3".to_string(),
                matching_bracket: "#0064FF".to_string(),
                current_line: "#F2F8FC".to_string(),
                find_match: "#A8AC94".to_string(),
                find_match_active: "#F6B94D".to_string(),
                error_underline: "#E51400".to_string(),
                warning_underline: "#BF8803".to_string(),
                info_underline: "#1BA1E2".to_string(),
                hint_underline: "#6C6C6C".to_string(),
            },

            terminal: TerminalColors {
                black: "#000000".to_string(),
                red: "#CD3131".to_string(),
                green: "#00BC00".to_string(),
                yellow: "#949800".to_string(),
                blue: "#0451A5".to_string(),
                magenta: "#BC05BC".to_string(),
                cyan: "#0598BC".to_string(),
                white: "#555555".to_string(),
                bright_black: "#666666".to_string(),
                bright_red: "#CD3131".to_string(),
                bright_green: "#14CE14".to_string(),
                bright_yellow: "#B5BA00".to_string(),
                bright_blue: "#0451A5".to_string(),
                bright_magenta: "#BC05BC".to_string(),
                bright_cyan: "#0598BC".to_string(),
                bright_white: "#A5A5A5".to_string(),
            },
        }
    }

    /// Create Solarized Dark theme.
    pub fn solarized_dark_theme() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            description: "Solarized dark color scheme".to_string(),
            author: "Ethan Schoonover".to_string(),
            version: "1.0.0".to_string(),

            foreground: "#839496".to_string(),
            background: "#002B36".to_string(),
            cursor: "#839496".to_string(),
            selection: "#073642".to_string(),

            ui: UiColors {
                border: "#586E75".to_string(),
                border_focused: "#268BD2".to_string(),
                title: "#93A1A1".to_string(),
                title_focused: "#FDF6E3".to_string(),
                menu: "#073642".to_string(),
                menu_selected: "#586E75".to_string(),
                button: "#268BD2".to_string(),
                button_hover: "#2AA198".to_string(),
                button_pressed: "#859900".to_string(),
                scrollbar: "#586E75".to_string(),
                scrollbar_thumb: "#657B83".to_string(),
                status_bar: "#268BD2".to_string(),
                status_bar_text: "#FDF6E3".to_string(),
                tab: "#073642".to_string(),
                tab_active: "#002B36".to_string(),
                tab_text: "#657B83".to_string(),
                tab_text_active: "#839496".to_string(),
            },

            syntax: SyntaxColors {
                keyword: "#859900".to_string(),
                string: "#2AA198".to_string(),
                number: "#D33682".to_string(),
                comment: "#586E75".to_string(),
                function: "#268BD2".to_string(),
                variable: "#839496".to_string(),
                r#type: "#B58900".to_string(),
                operator: "#839496".to_string(),
                punctuation: "#839496".to_string(),
                constant: "#CB4B16".to_string(),
                attribute: "#268BD2".to_string(),
                macro_name: "#859900".to_string(),
                label: "#839496".to_string(),
                namespace: "#B58900".to_string(),
                property: "#839496".to_string(),
                struct_name: "#B58900".to_string(),
                enum_name: "#B58900".to_string(),
                interface: "#B58900".to_string(),
                parameter: "#839496".to_string(),
                lifetime: "#D33682".to_string(),
                builtin: "#859900".to_string(),
                error: "#DC322F".to_string(),
                warning: "#B58900".to_string(),
                info: "#268BD2".to_string(),
                hint: "#586E75".to_string(),
            },

            editor: EditorColors {
                line_number: "#586E75".to_string(),
                line_number_active: "#839496".to_string(),
                gutter: "#002B36".to_string(),
                gutter_active: "#002B36".to_string(),
                indent_guide: "#073642".to_string(),
                whitespace: "#073642".to_string(),
                matching_bracket: "#268BD2".to_string(),
                current_line: "#073642".to_string(),
                find_match: "#B58900".to_string(),
                find_match_active: "#CB4B16".to_string(),
                error_underline: "#DC322F".to_string(),
                warning_underline: "#B58900".to_string(),
                info_underline: "#268BD2".to_string(),
                hint_underline: "#586E75".to_string(),
            },

            terminal: TerminalColors {
                black: "#073642".to_string(),
                red: "#DC322F".to_string(),
                green: "#859900".to_string(),
                yellow: "#B58900".to_string(),
                blue: "#268BD2".to_string(),
                magenta: "#D33682".to_string(),
                cyan: "#2AA198".to_string(),
                white: "#EEE8D5".to_string(),
                bright_black: "#002B36".to_string(),
                bright_red: "#CB4B16".to_string(),
                bright_green: "#586E75".to_string(),
                bright_yellow: "#657B83".to_string(),
                bright_blue: "#839496".to_string(),
                bright_magenta: "#6C71C4".to_string(),
                bright_cyan: "#93A1A1".to_string(),
                bright_white: "#FDF6E3".to_string(),
            },
        }
    }

    /// Create Solarized Light theme.
    pub fn solarized_light_theme() -> Self {
        let mut theme = Self::solarized_dark_theme();
        theme.name = "Solarized Light".to_string();
        theme.description = "Solarized light color scheme".to_string();
        theme.foreground = "#657B83".to_string();
        theme.background = "#FDF6E3".to_string();
        theme.cursor = "#657B83".to_string();
        theme.selection = "#EEE8D5".to_string();

        // Swap some colors for light theme
        theme.ui.border = "#93A1A1".to_string();
        theme.ui.title = "#586E75".to_string();
        theme.ui.menu = "#EEE8D5".to_string();
        theme.ui.tab = "#EEE8D5".to_string();
        theme.ui.tab_active = "#FDF6E3".to_string();
        theme.ui.tab_text = "#93A1A1".to_string();
        theme.ui.tab_text_active = "#657B83".to_string();

        theme.editor.gutter = "#FDF6E3".to_string();
        theme.editor.current_line = "#EEE8D5".to_string();
        theme.editor.indent_guide = "#EEE8D5".to_string();
        theme.editor.whitespace = "#EEE8D5".to_string();

        theme
    }

    /// Create Monokai theme.
    pub fn monokai_theme() -> Self {
        Self {
            name: "Monokai".to_string(),
            description: "Monokai color scheme".to_string(),
            author: "Wimer Hazenberg".to_string(),
            version: "1.0.0".to_string(),

            foreground: "#F8F8F2".to_string(),
            background: "#272822".to_string(),
            cursor: "#F8F8F0".to_string(),
            selection: "#49483E".to_string(),

            ui: UiColors {
                border: "#3E3D32".to_string(),
                border_focused: "#66D9EF".to_string(),
                title: "#F8F8F2".to_string(),
                title_focused: "#F8F8F2".to_string(),
                menu: "#3E3D32".to_string(),
                menu_selected: "#49483E".to_string(),
                button: "#66D9EF".to_string(),
                button_hover: "#A6E22E".to_string(),
                button_pressed: "#F92672".to_string(),
                scrollbar: "#49483E".to_string(),
                scrollbar_thumb: "#75715E".to_string(),
                status_bar: "#66D9EF".to_string(),
                status_bar_text: "#272822".to_string(),
                tab: "#3E3D32".to_string(),
                tab_active: "#272822".to_string(),
                tab_text: "#75715E".to_string(),
                tab_text_active: "#F8F8F2".to_string(),
            },

            syntax: SyntaxColors {
                keyword: "#F92672".to_string(),
                string: "#E6DB74".to_string(),
                number: "#AE81FF".to_string(),
                comment: "#75715E".to_string(),
                function: "#A6E22E".to_string(),
                variable: "#F8F8F2".to_string(),
                r#type: "#66D9EF".to_string(),
                operator: "#F92672".to_string(),
                punctuation: "#F8F8F2".to_string(),
                constant: "#AE81FF".to_string(),
                attribute: "#A6E22E".to_string(),
                macro_name: "#F92672".to_string(),
                label: "#F8F8F2".to_string(),
                namespace: "#66D9EF".to_string(),
                property: "#F8F8F2".to_string(),
                struct_name: "#66D9EF".to_string(),
                enum_name: "#66D9EF".to_string(),
                interface: "#66D9EF".to_string(),
                parameter: "#FD971F".to_string(),
                lifetime: "#F92672".to_string(),
                builtin: "#F92672".to_string(),
                error: "#F92672".to_string(),
                warning: "#FD971F".to_string(),
                info: "#66D9EF".to_string(),
                hint: "#75715E".to_string(),
            },

            editor: EditorColors {
                line_number: "#90908A".to_string(),
                line_number_active: "#F8F8F2".to_string(),
                gutter: "#272822".to_string(),
                gutter_active: "#272822".to_string(),
                indent_guide: "#3E3D32".to_string(),
                whitespace: "#3E3D32".to_string(),
                matching_bracket: "#66D9EF".to_string(),
                current_line: "#3E3D32".to_string(),
                find_match: "#FFE792".to_string(),
                find_match_active: "#F92672".to_string(),
                error_underline: "#F92672".to_string(),
                warning_underline: "#FD971F".to_string(),
                info_underline: "#66D9EF".to_string(),
                hint_underline: "#75715E".to_string(),
            },

            terminal: TerminalColors {
                black: "#272822".to_string(),
                red: "#F92672".to_string(),
                green: "#A6E22E".to_string(),
                yellow: "#F4BF75".to_string(),
                blue: "#66D9EF".to_string(),
                magenta: "#AE81FF".to_string(),
                cyan: "#A1EFE4".to_string(),
                white: "#F8F8F2".to_string(),
                bright_black: "#75715E".to_string(),
                bright_red: "#F92672".to_string(),
                bright_green: "#A6E22E".to_string(),
                bright_yellow: "#F4BF75".to_string(),
                bright_blue: "#66D9EF".to_string(),
                bright_magenta: "#AE81FF".to_string(),
                bright_cyan: "#A1EFE4".to_string(),
                bright_white: "#F9F8F5".to_string(),
            },
        }
    }

    /// Create GitHub theme.
    pub fn github_theme() -> Self {
        Self::light_theme() // GitHub theme is similar to light theme
    }

    /// Create Dracula theme.
    pub fn dracula_theme() -> Self {
        Self {
            name: "Dracula".to_string(),
            description: "Dracula color scheme".to_string(),
            author: "Dracula Theme".to_string(),
            version: "1.0.0".to_string(),

            foreground: "#F8F8F2".to_string(),
            background: "#282A36".to_string(),
            cursor: "#F8F8F2".to_string(),
            selection: "#44475A".to_string(),

            ui: UiColors {
                border: "#6272A4".to_string(),
                border_focused: "#BD93F9".to_string(),
                title: "#F8F8F2".to_string(),
                title_focused: "#F8F8F2".to_string(),
                menu: "#44475A".to_string(),
                menu_selected: "#6272A4".to_string(),
                button: "#BD93F9".to_string(),
                button_hover: "#FF79C6".to_string(),
                button_pressed: "#8BE9FD".to_string(),
                scrollbar: "#44475A".to_string(),
                scrollbar_thumb: "#6272A4".to_string(),
                status_bar: "#BD93F9".to_string(),
                status_bar_text: "#282A36".to_string(),
                tab: "#44475A".to_string(),
                tab_active: "#282A36".to_string(),
                tab_text: "#6272A4".to_string(),
                tab_text_active: "#F8F8F2".to_string(),
            },

            syntax: SyntaxColors {
                keyword: "#FF79C6".to_string(),
                string: "#F1FA8C".to_string(),
                number: "#BD93F9".to_string(),
                comment: "#6272A4".to_string(),
                function: "#50FA7B".to_string(),
                variable: "#F8F8F2".to_string(),
                r#type: "#8BE9FD".to_string(),
                operator: "#FF79C6".to_string(),
                punctuation: "#F8F8F2".to_string(),
                constant: "#BD93F9".to_string(),
                attribute: "#50FA7B".to_string(),
                macro_name: "#FF79C6".to_string(),
                label: "#F8F8F2".to_string(),
                namespace: "#8BE9FD".to_string(),
                property: "#F8F8F2".to_string(),
                struct_name: "#8BE9FD".to_string(),
                enum_name: "#8BE9FD".to_string(),
                interface: "#8BE9FD".to_string(),
                parameter: "#FFB86C".to_string(),
                lifetime: "#FF79C6".to_string(),
                builtin: "#FF79C6".to_string(),
                error: "#FF5555".to_string(),
                warning: "#FFB86C".to_string(),
                info: "#8BE9FD".to_string(),
                hint: "#6272A4".to_string(),
            },

            editor: EditorColors {
                line_number: "#6272A4".to_string(),
                line_number_active: "#F8F8F2".to_string(),
                gutter: "#282A36".to_string(),
                gutter_active: "#282A36".to_string(),
                indent_guide: "#44475A".to_string(),
                whitespace: "#44475A".to_string(),
                matching_bracket: "#BD93F9".to_string(),
                current_line: "#44475A".to_string(),
                find_match: "#FFB86C".to_string(),
                find_match_active: "#FF79C6".to_string(),
                error_underline: "#FF5555".to_string(),
                warning_underline: "#FFB86C".to_string(),
                info_underline: "#8BE9FD".to_string(),
                hint_underline: "#6272A4".to_string(),
            },

            terminal: TerminalColors {
                black: "#21222C".to_string(),
                red: "#FF5555".to_string(),
                green: "#50FA7B".to_string(),
                yellow: "#F1FA8C".to_string(),
                blue: "#BD93F9".to_string(),
                magenta: "#FF79C6".to_string(),
                cyan: "#8BE9FD".to_string(),
                white: "#F8F8F2".to_string(),
                bright_black: "#6272A4".to_string(),
                bright_red: "#FF6E6E".to_string(),
                bright_green: "#69FF94".to_string(),
                bright_yellow: "#FFFFA5".to_string(),
                bright_blue: "#D6ACFF".to_string(),
                bright_magenta: "#FF92DF".to_string(),
                bright_cyan: "#A4FFFF".to_string(),
                bright_white: "#FFFFFF".to_string(),
            },
        }
    }

    /// Create One Dark theme.
    pub fn one_dark_theme() -> Self {
        Self {
            name: "One Dark".to_string(),
            description: "One Dark color scheme".to_string(),
            author: "Atom".to_string(),
            version: "1.0.0".to_string(),

            foreground: "#ABB2BF".to_string(),
            background: "#282C34".to_string(),
            cursor: "#528BFF".to_string(),
            selection: "#3E4451".to_string(),

            ui: UiColors {
                border: "#3E4451".to_string(),
                border_focused: "#528BFF".to_string(),
                title: "#ABB2BF".to_string(),
                title_focused: "#FFFFFF".to_string(),
                menu: "#21252B".to_string(),
                menu_selected: "#2C313C".to_string(),
                button: "#528BFF".to_string(),
                button_hover: "#61AFEF".to_string(),
                button_pressed: "#2979FF".to_string(),
                scrollbar: "#3E4451".to_string(),
                scrollbar_thumb: "#5C6370".to_string(),
                status_bar: "#528BFF".to_string(),
                status_bar_text: "#FFFFFF".to_string(),
                tab: "#21252B".to_string(),
                tab_active: "#282C34".to_string(),
                tab_text: "#5C6370".to_string(),
                tab_text_active: "#ABB2BF".to_string(),
            },

            syntax: SyntaxColors {
                keyword: "#C678DD".to_string(),
                string: "#98C379".to_string(),
                number: "#D19A66".to_string(),
                comment: "#5C6370".to_string(),
                function: "#61AFEF".to_string(),
                variable: "#ABB2BF".to_string(),
                r#type: "#E06C75".to_string(),
                operator: "#56B6C2".to_string(),
                punctuation: "#ABB2BF".to_string(),
                constant: "#D19A66".to_string(),
                attribute: "#E5C07B".to_string(),
                macro_name: "#C678DD".to_string(),
                label: "#ABB2BF".to_string(),
                namespace: "#E06C75".to_string(),
                property: "#ABB2BF".to_string(),
                struct_name: "#E06C75".to_string(),
                enum_name: "#E06C75".to_string(),
                interface: "#E06C75".to_string(),
                parameter: "#ABB2BF".to_string(),
                lifetime: "#C678DD".to_string(),
                builtin: "#C678DD".to_string(),
                error: "#E06C75".to_string(),
                warning: "#E5C07B".to_string(),
                info: "#61AFEF".to_string(),
                hint: "#5C6370".to_string(),
            },

            editor: EditorColors {
                line_number: "#4B5263".to_string(),
                line_number_active: "#ABB2BF".to_string(),
                gutter: "#282C34".to_string(),
                gutter_active: "#282C34".to_string(),
                indent_guide: "#3E4451".to_string(),
                whitespace: "#3E4451".to_string(),
                matching_bracket: "#528BFF".to_string(),
                current_line: "#2C313C".to_string(),
                find_match: "#E5C07B".to_string(),
                find_match_active: "#D19A66".to_string(),
                error_underline: "#E06C75".to_string(),
                warning_underline: "#E5C07B".to_string(),
                info_underline: "#61AFEF".to_string(),
                hint_underline: "#5C6370".to_string(),
            },

            terminal: TerminalColors {
                black: "#282C34".to_string(),
                red: "#E06C75".to_string(),
                green: "#98C379".to_string(),
                yellow: "#E5C07B".to_string(),
                blue: "#61AFEF".to_string(),
                magenta: "#C678DD".to_string(),
                cyan: "#56B6C2".to_string(),
                white: "#ABB2BF".to_string(),
                bright_black: "#5C6370".to_string(),
                bright_red: "#E06C75".to_string(),
                bright_green: "#98C379".to_string(),
                bright_yellow: "#E5C07B".to_string(),
                bright_blue: "#61AFEF".to_string(),
                bright_magenta: "#C678DD".to_string(),
                bright_cyan: "#56B6C2".to_string(),
                bright_white: "#FFFFFF".to_string(),
            },
        }
    }

    /// Save theme to file.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let toml_content = toml::to_string_pretty(self)
            .map_err(|e| XyluxError::config(format!("Failed to serialize theme: {}", e)))?;

        std::fs::write(path, toml_content)
            .map_err(|e| XyluxError::io(e, "Failed to write theme file"))?;

        Ok(())
    }

    /// Load theme from file.
    pub fn load_from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| XyluxError::io(e, "Failed to read theme file"))?;

        let theme: Self = toml::from_str(&content)
            .map_err(|e| XyluxError::config(format!("Failed to parse theme file: {}", e)))?;

        Ok(theme)
    }

    /// Get color value by path (e.g., "syntax.keyword", "ui.border").
    pub fn get_color(&self, path: &str) -> Option<&str> {
        let parts: Vec<&str> = path.split('.').collect();
        match parts.as_slice() {
            ["foreground"] => Some(&self.foreground),
            ["background"] => Some(&self.background),
            ["cursor"] => Some(&self.cursor),
            ["selection"] => Some(&self.selection),
            ["ui", field] => match *field {
                "border" => Some(&self.ui.border),
                "border_focused" => Some(&self.ui.border_focused),
                "title" => Some(&self.ui.title),
                "title_focused" => Some(&self.ui.title_focused),
                "menu" => Some(&self.ui.menu),
                "menu_selected" => Some(&self.ui.menu_selected),
                "button" => Some(&self.ui.button),
                "button_hover" => Some(&self.ui.button_hover),
                "button_pressed" => Some(&self.ui.button_pressed),
                "scrollbar" => Some(&self.ui.scrollbar),
                "scrollbar_thumb" => Some(&self.ui.scrollbar_thumb),
                "status_bar" => Some(&self.ui.status_bar),
                "status_bar_text" => Some(&self.ui.status_bar_text),
                "tab" => Some(&self.ui.tab),
                "tab_active" => Some(&self.ui.tab_active),
                "tab_text" => Some(&self.ui.tab_text),
                "tab_text_active" => Some(&self.ui.tab_text_active),
                _ => None,
            },
            ["syntax", field] => match *field {
                "keyword" => Some(&self.syntax.keyword),
                "string" => Some(&self.syntax.string),
                "number" => Some(&self.syntax.number),
                "comment" => Some(&self.syntax.comment),
                "function" => Some(&self.syntax.function),
                "variable" => Some(&self.syntax.variable),
                "type" => Some(&self.syntax.r#type),
                "operator" => Some(&self.syntax.operator),
                "punctuation" => Some(&self.syntax.punctuation),
                "constant" => Some(&self.syntax.constant),
                "attribute" => Some(&self.syntax.attribute),
                "macro_name" => Some(&self.syntax.macro_name),
                "label" => Some(&self.syntax.label),
                "namespace" => Some(&self.syntax.namespace),
                "property" => Some(&self.syntax.property),
                "struct_name" => Some(&self.syntax.struct_name),
                "enum_name" => Some(&self.syntax.enum_name),
                "interface" => Some(&self.syntax.interface),
                "parameter" => Some(&self.syntax.parameter),
                "lifetime" => Some(&self.syntax.lifetime),
                "builtin" => Some(&self.syntax.builtin),
                "error" => Some(&self.syntax.error),
                "warning" => Some(&self.syntax.warning),
                "info" => Some(&self.syntax.info),
                "hint" => Some(&self.syntax.hint),
                _ => None,
            },
            ["editor", field] => match *field {
                "line_number" => Some(&self.editor.line_number),
                "line_number_active" => Some(&self.editor.line_number_active),
                "gutter" => Some(&self.editor.gutter),
                "gutter_active" => Some(&self.editor.gutter_active),
                "indent_guide" => Some(&self.editor.indent_guide),
                "whitespace" => Some(&self.editor.whitespace),
                "matching_bracket" => Some(&self.editor.matching_bracket),
                "current_line" => Some(&self.editor.current_line),
                "find_match" => Some(&self.editor.find_match),
                "find_match_active" => Some(&self.editor.find_match_active),
                "error_underline" => Some(&self.editor.error_underline),
                "warning_underline" => Some(&self.editor.warning_underline),
                "info_underline" => Some(&self.editor.info_underline),
                "hint_underline" => Some(&self.editor.hint_underline),
                _ => None,
            },
            _ => None,
        }
    }

    /// Create a custom theme with specified colors.
    pub fn custom(name: String, colors: HashMap<String, String>) -> Self {
        let mut theme = Self::dark_theme(); // Start with dark theme as base
        theme.name = name;
        theme.description = "Custom theme".to_string();
        theme.author = "User".to_string();

        // Apply custom colors
        for (path, color) in colors {
            // This is a simplified implementation
            // In a real implementation, you'd use reflection or a more sophisticated method
            match path.as_str() {
                "foreground" => theme.foreground = color,
                "background" => theme.background = color,
                "cursor" => theme.cursor = color,
                "selection" => theme.selection = color,
                _ => {} // Ignore unknown paths for now
            }
        }

        theme
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark_theme()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_theme_creation() {
        let theme = Theme::dark_theme();
        assert_eq!(theme.name, "Dark");
        assert!(!theme.foreground.is_empty());
        assert!(!theme.background.is_empty());
    }

    #[test]
    fn test_available_themes() {
        let themes = Theme::available_themes();
        assert!(themes.contains(&"dark"));
        assert!(themes.contains(&"light"));
        assert!(themes.contains(&"monokai"));
    }

    #[test]
    fn test_theme_loading() {
        assert!(Theme::load("dark").is_ok());
        assert!(Theme::load("light").is_ok());
        assert!(Theme::load("monokai").is_ok());
        assert!(Theme::load("unknown").is_err());
    }

    #[test]
    fn test_get_color() {
        let theme = Theme::dark_theme();

        assert_eq!(theme.get_color("foreground"), Some(theme.foreground.as_str()));
        assert_eq!(theme.get_color("ui.border"), Some(theme.ui.border.as_str()));
        assert_eq!(theme.get_color("syntax.keyword"), Some(theme.syntax.keyword.as_str()));
        assert_eq!(theme.get_color("editor.line_number"), Some(theme.editor.line_number.as_str()));
        assert_eq!(theme.get_color("invalid.path"), None);
    }

    #[test]
    fn test_theme_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let theme_path = temp_dir.path().join("test_theme.toml");

        let original_theme = Theme::dark_theme();
        original_theme.save_to_file(&theme_path).unwrap();

        let loaded_theme = Theme::load_from_file(&theme_path).unwrap();
        assert_eq!(original_theme.name, loaded_theme.name);
        assert_eq!(original_theme.foreground, loaded_theme.foreground);
        assert_eq!(original_theme.background, loaded_theme.background);
    }

    #[test]
    fn test_custom_theme() {
        let mut colors = HashMap::new();
        colors.insert("foreground".to_string(), "#123456".to_string());
        colors.insert("background".to_string(), "#654321".to_string());

        let theme = Theme::custom("My Theme".to_string(), colors);
        assert_eq!(theme.name, "My Theme");
        assert_eq!(theme.foreground, "#123456");
        assert_eq!(theme.background, "#654321");
    }

    #[test]
    fn test_solarized_themes() {
        let dark = Theme::solarized_dark_theme();
        let light = Theme::solarized_light_theme();

        assert_eq!(dark.name, "Solarized Dark");
        assert_eq!(light.name, "Solarized Light");
        assert_ne!(dark.background, light.background);
    }
}
