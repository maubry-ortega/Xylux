//! # Syntax Highlighter Trait
//!
//! Trait definitions and interfaces for syntax highlighting functionality.

use crate::core::Result;
use crate::syntax::{HighlightToken, TokenType};

/// Trait for syntax highlighters.
#[async_trait::async_trait]
pub trait SyntaxHighlighter {
    /// Highlight the given content and return tokens.
    async fn highlight(&self, content: &str) -> Result<Vec<HighlightToken>>;

    /// Get the language name this highlighter supports.
    fn language(&self) -> &str {
        "unknown"
    }

    /// Get file extensions this highlighter supports.
    fn file_extensions(&self) -> Vec<&str> {
        vec![]
    }

    /// Check if this highlighter supports the given file extension.
    fn supports_extension(&self, extension: &str) -> bool {
        self.file_extensions().contains(&extension)
    }

    /// Get theme-based colors for token types.
    fn get_token_color(&self, token_type: &TokenType) -> Option<&str> {
        match token_type {
            TokenType::Text => None,
            TokenType::Keyword => Some("#569cd6"),     // Blue
            TokenType::String => Some("#ce9178"),      // Orange
            TokenType::Number => Some("#b5cea8"),      // Light green
            TokenType::Comment => Some("#6a9955"),     // Green
            TokenType::Function => Some("#dcdcaa"),    // Yellow
            TokenType::Variable => Some("#9cdcfe"),    // Light blue
            TokenType::Type => Some("#4ec9b0"),        // Cyan
            TokenType::Operator => Some("#d4d4d4"),    // Light gray
            TokenType::Punctuation => Some("#d4d4d4"), // Light gray
        }
    }
}

/// Configuration for syntax highlighting.
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    /// Whether to enable syntax highlighting.
    pub enabled: bool,
    /// Whether to highlight matching brackets.
    pub highlight_matching_brackets: bool,
    /// Whether to highlight current line.
    pub highlight_current_line: bool,
    /// Maximum file size to highlight (in bytes).
    pub max_file_size: usize,
    /// Whether to use semantic highlighting when available.
    pub use_semantic_highlighting: bool,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            highlight_matching_brackets: true,
            highlight_current_line: true,
            max_file_size: 1024 * 1024, // 1 MB
            use_semantic_highlighting: true,
        }
    }
}

/// Represents a syntax theme.
#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    /// Theme name.
    pub name: String,
    /// Background color.
    pub background: String,
    /// Foreground color.
    pub foreground: String,
    /// Colors for different token types.
    pub token_colors: std::collections::HashMap<TokenType, String>,
}

impl SyntaxTheme {
    /// Create a new dark theme.
    pub fn dark() -> Self {
        let mut token_colors = std::collections::HashMap::new();
        token_colors.insert(TokenType::Keyword, "#569cd6".to_string());
        token_colors.insert(TokenType::String, "#ce9178".to_string());
        token_colors.insert(TokenType::Number, "#b5cea8".to_string());
        token_colors.insert(TokenType::Comment, "#6a9955".to_string());
        token_colors.insert(TokenType::Function, "#dcdcaa".to_string());
        token_colors.insert(TokenType::Variable, "#9cdcfe".to_string());
        token_colors.insert(TokenType::Type, "#4ec9b0".to_string());
        token_colors.insert(TokenType::Operator, "#d4d4d4".to_string());
        token_colors.insert(TokenType::Punctuation, "#d4d4d4".to_string());

        Self {
            name: "Dark".to_string(),
            background: "#1e1e1e".to_string(),
            foreground: "#d4d4d4".to_string(),
            token_colors,
        }
    }

    /// Create a new light theme.
    pub fn light() -> Self {
        let mut token_colors = std::collections::HashMap::new();
        token_colors.insert(TokenType::Keyword, "#0000ff".to_string());
        token_colors.insert(TokenType::String, "#a31515".to_string());
        token_colors.insert(TokenType::Number, "#098658".to_string());
        token_colors.insert(TokenType::Comment, "#008000".to_string());
        token_colors.insert(TokenType::Function, "#795e26".to_string());
        token_colors.insert(TokenType::Variable, "#001080".to_string());
        token_colors.insert(TokenType::Type, "#267f99".to_string());
        token_colors.insert(TokenType::Operator, "#000000".to_string());
        token_colors.insert(TokenType::Punctuation, "#000000".to_string());

        Self {
            name: "Light".to_string(),
            background: "#ffffff".to_string(),
            foreground: "#000000".to_string(),
            token_colors,
        }
    }

    /// Get color for a token type.
    pub fn get_color(&self, token_type: &TokenType) -> Option<&String> {
        self.token_colors.get(token_type)
    }
}

/// Utility functions for syntax highlighting.
pub mod utils {
    use super::*;

    /// Check if a character is a word character.
    pub fn is_word_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    /// Check if a character is whitespace.
    pub fn is_whitespace(ch: char) -> bool {
        ch.is_whitespace()
    }

    /// Check if a character starts an identifier.
    pub fn is_identifier_start(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_'
    }

    /// Check if a character can be part of an identifier.
    pub fn is_identifier_part(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    /// Check if a character is a digit.
    pub fn is_digit(ch: char) -> bool {
        ch.is_ascii_digit()
    }

    /// Check if a character can start a number.
    pub fn is_number_start(ch: char) -> bool {
        ch.is_ascii_digit() || ch == '.'
    }

    /// Extract keywords from content using simple pattern matching.
    pub fn extract_keywords(content: &str, keywords: &[&str]) -> Vec<HighlightToken> {
        let mut tokens = Vec::new();

        for keyword in keywords {
            let mut start = 0;
            while let Some(pos) = content[start..].find(keyword) {
                let absolute_pos = start + pos;

                // Check if it's a whole word (not part of another identifier)
                let before_ok = absolute_pos == 0
                    || !is_identifier_part(content.chars().nth(absolute_pos - 1).unwrap_or(' '));
                let after_pos = absolute_pos + keyword.len();
                let after_ok = after_pos >= content.len()
                    || !is_identifier_part(content.chars().nth(after_pos).unwrap_or(' '));

                if before_ok && after_ok {
                    tokens.push(HighlightToken::new(
                        absolute_pos,
                        absolute_pos + keyword.len(),
                        TokenType::Keyword,
                    ));
                }

                start = absolute_pos + 1;
            }
        }

        tokens.sort_by_key(|t| t.start);
        tokens
    }

    /// Extract string literals from content.
    pub fn extract_strings(content: &str, quote_chars: &[char]) -> Vec<HighlightToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            if quote_chars.contains(&ch) {
                let start = i;
                i += 1;

                // Find closing quote
                while i < chars.len() {
                    if chars[i] == ch && (i == 0 || chars[i - 1] != '\\') {
                        i += 1;
                        break;
                    }
                    i += 1;
                }

                tokens.push(HighlightToken::new(start, i, TokenType::String));
            } else {
                i += 1;
            }
        }

        tokens
    }

    /// Extract line comments from content.
    pub fn extract_line_comments(content: &str, comment_start: &str) -> Vec<HighlightToken> {
        let mut tokens = Vec::new();

        for (line_start, line) in content.lines().enumerate() {
            if let Some(comment_pos) = line.find(comment_start) {
                let absolute_start = content
                    [..content.lines().take(line_start).map(|l| l.len() + 1).sum::<usize>()]
                    .len()
                    + comment_pos;
                let absolute_end = absolute_start + line.len() - comment_pos;

                tokens.push(HighlightToken::new(absolute_start, absolute_end, TokenType::Comment));
            }
        }

        tokens
    }

    /// Extract numbers from content.
    pub fn extract_numbers(content: &str) -> Vec<HighlightToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            if is_number_start(ch) {
                let start = i;

                // Consume digits before decimal point
                while i < chars.len() && is_digit(chars[i]) {
                    i += 1;
                }

                // Check for decimal point
                if i < chars.len() && chars[i] == '.' {
                    i += 1;
                    // Consume digits after decimal point
                    while i < chars.len() && is_digit(chars[i]) {
                        i += 1;
                    }
                }

                // Check for scientific notation
                if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    i += 1;
                    if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                        i += 1;
                    }
                    while i < chars.len() && is_digit(chars[i]) {
                        i += 1;
                    }
                }

                if i > start {
                    tokens.push(HighlightToken::new(start, i, TokenType::Number));
                }
            } else {
                i += 1;
            }
        }

        tokens
    }

    /// Merge overlapping tokens, giving priority to longer matches.
    pub fn merge_tokens(mut tokens: Vec<HighlightToken>) -> Vec<HighlightToken> {
        if tokens.is_empty() {
            return tokens;
        }

        tokens.sort_by_key(|t| (t.start, std::cmp::Reverse(t.end - t.start)));

        let mut merged = Vec::new();
        let mut last_end = 0;

        for token in tokens {
            if token.start >= last_end {
                last_end = token.end;
                merged.push(token);
            }
        }

        merged.sort_by_key(|t| t.start);
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::utils::*;
    use super::*;

    #[test]
    fn test_keyword_extraction() {
        let content = "fn main() { let x = 42; }";
        let keywords = &["fn", "let"];
        let tokens = extract_keywords(content, keywords);

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 2);
        assert_eq!(tokens[1].start, 12);
        assert_eq!(tokens[1].end, 15);
    }

    #[test]
    fn test_string_extraction() {
        let content = r#"let s = "hello world";"#;
        let tokens = extract_strings(content, &['"']);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].start, 8);
        assert_eq!(tokens[0].end, 21);
    }

    #[test]
    fn test_number_extraction() {
        let content = "let x = 42; let y = 3.14;";
        let tokens = extract_numbers(content);

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].start, 8);
        assert_eq!(tokens[0].end, 10);
        assert_eq!(tokens[1].start, 20);
        assert_eq!(tokens[1].end, 24);
    }

    #[test]
    fn test_comment_extraction() {
        let content = "fn main() { // this is a comment\n    let x = 42; // another comment\n}";
        let tokens = extract_line_comments(content, "//");

        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_token_merging() {
        let tokens = vec![
            HighlightToken::new(0, 5, TokenType::Keyword),
            HighlightToken::new(3, 8, TokenType::String), // Overlapping
            HighlightToken::new(10, 15, TokenType::Number),
        ];

        let merged = merge_tokens(tokens);
        assert_eq!(merged.len(), 2); // Overlapping token should be removed
    }

    #[test]
    fn test_syntax_theme() {
        let theme = SyntaxTheme::dark();
        assert_eq!(theme.name, "Dark");
        assert!(theme.get_color(&TokenType::Keyword).is_some());

        let light_theme = SyntaxTheme::light();
        assert_eq!(light_theme.name, "Light");
        assert_ne!(
            theme.get_color(&TokenType::Keyword),
            light_theme.get_color(&TokenType::Keyword)
        );
    }

    #[test]
    fn test_character_classification() {
        assert!(is_word_char('a'));
        assert!(is_word_char('_'));
        assert!(is_word_char('5'));
        assert!(!is_word_char(' '));
        assert!(!is_word_char('!'));

        assert!(is_identifier_start('a'));
        assert!(is_identifier_start('_'));
        assert!(!is_identifier_start('5'));

        assert!(is_number_start('5'));
        assert!(is_number_start('.'));
        assert!(!is_number_start('a'));
    }
}
