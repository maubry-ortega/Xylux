//! # Alux Syntax Highlighting
//!
//! Syntax highlighting implementation for the Alux scripting language.

use std::collections::HashMap;

use async_trait::async_trait;
use tracing::debug;

use crate::core::Result;
use crate::syntax::{HighlightToken, SyntaxHighlighter, TokenType};

/// Alux syntax highlighter.
pub struct AluxSyntax {
    /// Keywords in the Alux language.
    keywords: HashMap<String, TokenType>,
}

impl AluxSyntax {
    /// Create a new Alux syntax highlighter.
    pub fn new() -> Self {
        let mut keywords = HashMap::new();

        // Control flow keywords
        keywords.insert("if".to_string(), TokenType::Keyword);
        keywords.insert("else".to_string(), TokenType::Keyword);
        keywords.insert("elif".to_string(), TokenType::Keyword);
        keywords.insert("while".to_string(), TokenType::Keyword);
        keywords.insert("for".to_string(), TokenType::Keyword);
        keywords.insert("loop".to_string(), TokenType::Keyword);
        keywords.insert("break".to_string(), TokenType::Keyword);
        keywords.insert("continue".to_string(), TokenType::Keyword);
        keywords.insert("return".to_string(), TokenType::Keyword);

        // Declaration keywords
        keywords.insert("fn".to_string(), TokenType::Keyword);
        keywords.insert("let".to_string(), TokenType::Keyword);
        keywords.insert("const".to_string(), TokenType::Keyword);
        keywords.insert("mut".to_string(), TokenType::Keyword);
        keywords.insert("struct".to_string(), TokenType::Keyword);
        keywords.insert("enum".to_string(), TokenType::Keyword);
        keywords.insert("trait".to_string(), TokenType::Keyword);
        keywords.insert("impl".to_string(), TokenType::Keyword);
        keywords.insert("mod".to_string(), TokenType::Keyword);
        keywords.insert("use".to_string(), TokenType::Keyword);
        keywords.insert("import".to_string(), TokenType::Keyword);
        keywords.insert("export".to_string(), TokenType::Keyword);

        // Xylux-specific keywords
        keywords.insert("task".to_string(), TokenType::Keyword);
        keywords.insert("component".to_string(), TokenType::Keyword);
        keywords.insert("system".to_string(), TokenType::Keyword);
        keywords.insert("resource".to_string(), TokenType::Keyword);
        keywords.insert("entity".to_string(), TokenType::Keyword);
        keywords.insert("event".to_string(), TokenType::Keyword);
        keywords.insert("scene".to_string(), TokenType::Keyword);
        keywords.insert("node".to_string(), TokenType::Keyword);
        keywords.insert("shader".to_string(), TokenType::Keyword);
        keywords.insert("material".to_string(), TokenType::Keyword);
        keywords.insert("texture".to_string(), TokenType::Keyword);
        keywords.insert("mesh".to_string(), TokenType::Keyword);
        keywords.insert("animation".to_string(), TokenType::Keyword);
        keywords.insert("sound".to_string(), TokenType::Keyword);

        // Async keywords
        keywords.insert("async".to_string(), TokenType::Keyword);
        keywords.insert("await".to_string(), TokenType::Keyword);
        keywords.insert("spawn".to_string(), TokenType::Keyword);

        // Memory management
        keywords.insert("new".to_string(), TokenType::Keyword);
        keywords.insert("delete".to_string(), TokenType::Keyword);
        keywords.insert("ref".to_string(), TokenType::Keyword);
        keywords.insert("deref".to_string(), TokenType::Keyword);

        // Boolean literals
        keywords.insert("true".to_string(), TokenType::Keyword);
        keywords.insert("false".to_string(), TokenType::Keyword);
        keywords.insert("null".to_string(), TokenType::Keyword);

        // Visibility
        keywords.insert("pub".to_string(), TokenType::Keyword);
        keywords.insert("priv".to_string(), TokenType::Keyword);

        // Types
        keywords.insert("i8".to_string(), TokenType::Type);
        keywords.insert("i16".to_string(), TokenType::Type);
        keywords.insert("i32".to_string(), TokenType::Type);
        keywords.insert("i64".to_string(), TokenType::Type);
        keywords.insert("u8".to_string(), TokenType::Type);
        keywords.insert("u16".to_string(), TokenType::Type);
        keywords.insert("u32".to_string(), TokenType::Type);
        keywords.insert("u64".to_string(), TokenType::Type);
        keywords.insert("f32".to_string(), TokenType::Type);
        keywords.insert("f64".to_string(), TokenType::Type);
        keywords.insert("bool".to_string(), TokenType::Type);
        keywords.insert("char".to_string(), TokenType::Type);
        keywords.insert("string".to_string(), TokenType::Type);
        keywords.insert("vec".to_string(), TokenType::Type);
        keywords.insert("map".to_string(), TokenType::Type);
        keywords.insert("set".to_string(), TokenType::Type);

        // Xylux types
        keywords.insert("Vec2".to_string(), TokenType::Type);
        keywords.insert("Vec3".to_string(), TokenType::Type);
        keywords.insert("Vec4".to_string(), TokenType::Type);
        keywords.insert("Mat3".to_string(), TokenType::Type);
        keywords.insert("Mat4".to_string(), TokenType::Type);
        keywords.insert("Quat".to_string(), TokenType::Type);
        keywords.insert("Color".to_string(), TokenType::Type);
        keywords.insert("Transform".to_string(), TokenType::Type);

        Self { keywords }
    }

    /// Check if a character can be part of an identifier.
    fn is_identifier_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Check if a character can start an identifier.
    fn is_identifier_start(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    /// Check if a character is a digit.
    fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    /// Check if a character is whitespace.
    fn is_whitespace(c: char) -> bool {
        c.is_whitespace()
    }

    /// Parse a string literal.
    fn parse_string(&self, content: &str, start: usize) -> Option<usize> {
        let chars: Vec<char> = content.chars().collect();
        if start >= chars.len() {
            return None;
        }

        let quote_char = chars[start];
        if quote_char != '"' && quote_char != '\'' {
            return None;
        }

        let mut i = start + 1;
        let mut escaped = false;

        while i < chars.len() {
            let c = chars[i];

            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote_char {
                return Some(i + 1);
            }

            i += 1;
        }

        // Unclosed string, return end of content
        Some(chars.len())
    }

    /// Parse a number literal.
    fn parse_number(&self, content: &str, start: usize) -> Option<usize> {
        let chars: Vec<char> = content.chars().collect();
        if start >= chars.len() {
            return None;
        }

        let mut i = start;
        let mut has_dot = false;

        // Handle hex numbers
        if i + 1 < chars.len() && chars[i] == '0' && (chars[i + 1] == 'x' || chars[i + 1] == 'X') {
            i += 2;
            while i < chars.len() && chars[i].is_ascii_hexdigit() {
                i += 1;
            }
            return if i > start + 2 { Some(i) } else { None };
        }

        // Handle binary numbers
        if i + 1 < chars.len() && chars[i] == '0' && (chars[i + 1] == 'b' || chars[i + 1] == 'B') {
            i += 2;
            while i < chars.len() && (chars[i] == '0' || chars[i] == '1') {
                i += 1;
            }
            return if i > start + 2 { Some(i) } else { None };
        }

        // Handle decimal numbers
        while i < chars.len() {
            let c = chars[i];
            if Self::is_digit(c) {
                i += 1;
            } else if c == '.' && !has_dot {
                has_dot = true;
                i += 1;
            } else {
                break;
            }
        }

        // Handle scientific notation
        if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
            i += 1;
            if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                i += 1;
            }
            while i < chars.len() && Self::is_digit(chars[i]) {
                i += 1;
            }
        }

        // Handle type suffixes
        if i < chars.len() {
            let remaining: String = chars[i..].iter().collect();
            if remaining.starts_with("f32") || remaining.starts_with("f64") {
                i += 3;
            } else if remaining.starts_with("i8") || remaining.starts_with("u8") {
                i += 2;
            } else if remaining.starts_with("i16")
                || remaining.starts_with("u16")
                || remaining.starts_with("i32")
                || remaining.starts_with("u32")
                || remaining.starts_with("i64")
                || remaining.starts_with("u64")
            {
                i += 3;
            }
        }

        if i > start { Some(i) } else { None }
    }

    /// Parse an identifier.
    fn parse_identifier(&self, content: &str, start: usize) -> Option<usize> {
        let chars: Vec<char> = content.chars().collect();
        if start >= chars.len() {
            return None;
        }

        if !Self::is_identifier_start(chars[start]) {
            return None;
        }

        let mut i = start + 1;
        while i < chars.len() && Self::is_identifier_char(chars[i]) {
            i += 1;
        }

        Some(i)
    }

    /// Parse a comment.
    fn parse_comment(&self, content: &str, start: usize) -> Option<usize> {
        let chars: Vec<char> = content.chars().collect();
        if start + 1 >= chars.len() {
            return None;
        }

        // Single-line comment
        if chars[start] == '/' && chars[start + 1] == '/' {
            let mut i = start + 2;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            return Some(i);
        }

        // Multi-line comment
        if chars[start] == '/' && chars[start + 1] == '*' {
            let mut i = start + 2;
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    return Some(i + 2);
                }
                i += 1;
            }
            // Unclosed comment, return end of content
            return Some(chars.len());
        }

        None
    }
}

impl Default for AluxSyntax {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SyntaxHighlighter for AluxSyntax {
    async fn highlight(&self, content: &str) -> Result<Vec<HighlightToken>> {
        debug!("Highlighting Alux content ({} chars)", content.len());

        let mut tokens = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            // Skip whitespace
            if Self::is_whitespace(c) {
                i += 1;
                continue;
            }

            // Comments
            if c == '/' && i + 1 < chars.len() {
                if let Some(end) = self.parse_comment(content, i) {
                    tokens.push(HighlightToken::new(i, end, TokenType::Comment));
                    i = end;
                    continue;
                }
            }

            // String literals
            if c == '"' || c == '\'' {
                if let Some(end) = self.parse_string(content, i) {
                    tokens.push(HighlightToken::new(i, end, TokenType::String));
                    i = end;
                    continue;
                }
            }

            // Number literals
            if Self::is_digit(c)
                || (c == '.' && i + 1 < chars.len() && Self::is_digit(chars[i + 1]))
            {
                if let Some(end) = self.parse_number(content, i) {
                    tokens.push(HighlightToken::new(i, end, TokenType::Number));
                    i = end;
                    continue;
                }
            }

            // Identifiers and keywords
            if Self::is_identifier_start(c) {
                if let Some(end) = self.parse_identifier(content, i) {
                    let text: String = chars[i..end].iter().collect();
                    let token_type = self.keywords.get(&text).cloned().unwrap_or_else(|| {
                        // Check if it looks like a function call
                        if end < chars.len() && chars[end] == '(' {
                            TokenType::Function
                        } else if text.chars().next().unwrap().is_uppercase() {
                            // Likely a type or constant
                            TokenType::Type
                        } else {
                            TokenType::Variable
                        }
                    });

                    tokens.push(HighlightToken::new(i, end, token_type));
                    i = end;
                    continue;
                }
            }

            // Operators
            match c {
                '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~' => {
                    let mut end = i + 1;

                    // Handle multi-character operators
                    if end < chars.len() {
                        match (c, chars[end]) {
                            ('=', '=')
                            | ('!', '=')
                            | ('<', '=')
                            | ('>', '=')
                            | ('+', '+')
                            | ('-', '-')
                            | ('+', '=')
                            | ('-', '=')
                            | ('*', '=')
                            | ('/', '=')
                            | ('%', '=')
                            | ('&', '&')
                            | ('|', '|')
                            | ('<', '<')
                            | ('>', '>')
                            | ('^', '=')
                            | ('&', '=')
                            | ('|', '=') => {
                                end += 1;
                            }
                            _ => {}
                        }
                    }

                    tokens.push(HighlightToken::new(i, end, TokenType::Operator));
                    i = end;
                    continue;
                }
                _ => {}
            }

            // Punctuation
            match c {
                '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '.' | ':' | '?' => {
                    tokens.push(HighlightToken::new(i, i + 1, TokenType::Punctuation));
                    i += 1;
                    continue;
                }
                _ => {}
            }

            // Skip unknown characters
            i += 1;
        }

        debug!("Generated {} highlight tokens", tokens.len());
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alux_syntax_highlighting() {
        let syntax = AluxSyntax::new();
        let content = r#"
fn main() {
    let x = 42;
    let name = "Hello, World!";
    println!(name);
}
"#;

        let tokens = syntax.highlight(content).await.unwrap();
        assert!(!tokens.is_empty());

        // Should have keywords
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Keyword));
        // Should have strings
        assert!(tokens.iter().any(|t| t.token_type == TokenType::String));
        // Should have numbers
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Number));
    }

    #[tokio::test]
    async fn test_comment_highlighting() {
        let syntax = AluxSyntax::new();
        let content = r#"
// This is a single line comment
fn test() {
    /* This is a
       multi-line comment */
    let x = 1;
}
"#;

        let tokens = syntax.highlight(content).await.unwrap();
        let comment_tokens: Vec<_> =
            tokens.iter().filter(|t| t.token_type == TokenType::Comment).collect();
        assert_eq!(comment_tokens.len(), 2);
    }

    #[tokio::test]
    async fn test_string_highlighting() {
        let syntax = AluxSyntax::new();
        let content = r#"
let single = 'single quotes';
let double = "double quotes";
let escaped = "with \"escaped\" quotes";
"#;

        let tokens = syntax.highlight(content).await.unwrap();
        let string_tokens: Vec<_> =
            tokens.iter().filter(|t| t.token_type == TokenType::String).collect();
        assert_eq!(string_tokens.len(), 3);
    }

    #[tokio::test]
    async fn test_number_highlighting() {
        let syntax = AluxSyntax::new();
        let content = r#"
let integer = 42;
let float = 3.14;
let hex = 0xFF;
let binary = 0b1010;
let scientific = 1.23e-4;
let typed = 42i32;
"#;

        let tokens = syntax.highlight(content).await.unwrap();
        let number_tokens: Vec<_> =
            tokens.iter().filter(|t| t.token_type == TokenType::Number).collect();
        assert_eq!(number_tokens.len(), 6);
    }

    #[tokio::test]
    async fn test_xylux_keywords() {
        let syntax = AluxSyntax::new();
        let content = r#"
component Position {
    x: f32,
    y: f32,
}

system movement_system() {
    // Game logic here
}

task async_task() {
    await some_operation();
}
"#;

        let tokens = syntax.highlight(content).await.unwrap();
        let keyword_tokens: Vec<_> =
            tokens.iter().filter(|t| t.token_type == TokenType::Keyword).collect();

        // Should highlight component, system, task, await
        assert!(keyword_tokens.len() >= 4);
    }

    #[tokio::test]
    async fn test_type_highlighting() {
        let syntax = AluxSyntax::new();
        let content = r#"
let position: Vec3 = Vec3::new(1.0, 2.0, 3.0);
let color: Color = Color::rgb(255, 128, 64);
let transform: Transform = Transform::identity();
"#;

        let tokens = syntax.highlight(content).await.unwrap();
        let type_tokens: Vec<_> =
            tokens.iter().filter(|t| t.token_type == TokenType::Type).collect();

        // Should highlight Vec3, Color, Transform
        assert!(type_tokens.len() >= 3);
    }

    #[tokio::test]
    async fn test_operator_highlighting() {
        let syntax = AluxSyntax::new();
        let content = "let result = a + b * c == d && e || f;";

        let tokens = syntax.highlight(content).await.unwrap();
        let operator_tokens: Vec<_> =
            tokens.iter().filter(|t| t.token_type == TokenType::Operator).collect();

        // Should highlight =, +, *, ==, &&, ||
        assert!(operator_tokens.len() >= 6);
    }

    #[test]
    fn test_keyword_detection() {
        let syntax = AluxSyntax::new();

        assert!(syntax.keywords.contains_key("fn"));
        assert!(syntax.keywords.contains_key("component"));
        assert!(syntax.keywords.contains_key("system"));
        assert!(syntax.keywords.contains_key("task"));
        assert!(syntax.keywords.contains_key("Vec3"));
        assert!(!syntax.keywords.contains_key("not_a_keyword"));
    }

    #[test]
    fn test_identifier_parsing() {
        let syntax = AluxSyntax::new();

        assert_eq!(syntax.parse_identifier("hello_world", 0), Some(11));
        assert_eq!(syntax.parse_identifier("_private", 0), Some(8));
        assert_eq!(syntax.parse_identifier("CamelCase", 0), Some(9));
        assert_eq!(syntax.parse_identifier("123invalid", 0), None);
    }

    #[test]
    fn test_number_parsing() {
        let syntax = AluxSyntax::new();

        assert_eq!(syntax.parse_number("42", 0), Some(2));
        assert_eq!(syntax.parse_number("3.14", 0), Some(4));
        assert_eq!(syntax.parse_number("0xFF", 0), Some(4));
        assert_eq!(syntax.parse_number("0b1010", 0), Some(6));
        assert_eq!(syntax.parse_number("1.23e-4", 0), Some(7));
        assert_eq!(syntax.parse_number("42i32", 0), Some(5));
    }

    #[test]
    fn test_string_parsing() {
        let syntax = AluxSyntax::new();

        assert_eq!(syntax.parse_string("\"hello\"", 0), Some(7));
        assert_eq!(syntax.parse_string("'world'", 0), Some(7));
        assert_eq!(syntax.parse_string("\"escaped\\\"quote\"", 0), Some(16));
        assert_eq!(syntax.parse_string("\"unclosed", 0), Some(9)); // End of content
    }

    #[test]
    fn test_comment_parsing() {
        let syntax = AluxSyntax::new();

        assert_eq!(syntax.parse_comment("// single line", 0), Some(14));
        assert_eq!(syntax.parse_comment("/* multi line */", 0), Some(16));
        assert_eq!(syntax.parse_comment("/* unclosed", 0), Some(11)); // End of content
        assert_eq!(syntax.parse_comment("not a comment", 0), None);
    }
}
