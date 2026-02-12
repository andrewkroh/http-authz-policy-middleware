// Lexer (tokenizer) for the expression language

use std::fmt;

/// Token types in the expression language
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    String(String),
    Ident(String),

    // Punctuation
    LParen,    // (
    RParen,    // )
    Comma,     // ,

    // Comparison operators
    OpEq,          // ==
    OpNeq,         // !=
    OpStartsWith,  // startsWith
    OpEndsWith,    // endsWith
    OpContains,    // contains
    OpMatches,     // matches

    // Boolean operators (keywords)
    KwAnd,  // AND
    KwOr,   // OR
    KwNot,  // NOT

    // End of input
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Ident(s) => write!(f, "{}", s),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::OpEq => write!(f, "=="),
            Token::OpNeq => write!(f, "!="),
            Token::OpStartsWith => write!(f, "startsWith"),
            Token::OpEndsWith => write!(f, "endsWith"),
            Token::OpContains => write!(f, "contains"),
            Token::OpMatches => write!(f, "matches"),
            Token::KwAnd => write!(f, "AND"),
            Token::KwOr => write!(f, "OR"),
            Token::KwNot => write!(f, "NOT"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Lexer error with position information
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub pos: usize,
    pub message: String,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lexer error at position {}: {}", self.pos, self.message)
    }
}

impl std::error::Error for LexError {}

/// Lexer for tokenizing expression strings
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    current_char: Option<char>,
}

impl Lexer {
    /// Create a new lexer from input string
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.first().copied();

        Lexer {
            input: chars,
            pos: 0,
            current_char,
        }
    }

    /// Advance to the next character
    fn advance(&mut self) {
        self.pos += 1;
        self.current_char = if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        };
    }

    /// Peek at the next character without advancing
    fn peek(&self) -> Option<char> {
        if self.pos + 1 < self.input.len() {
            Some(self.input[self.pos + 1])
        } else {
            None
        }
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Read a string literal (enclosed in double quotes)
    fn read_string(&mut self) -> Result<String, LexError> {
        let start_pos = self.pos;
        let mut result = String::new();

        // Skip opening quote
        self.advance();

        while let Some(ch) = self.current_char {
            match ch {
                '"' => {
                    // End of string
                    self.advance();
                    return Ok(result);
                }
                '\\' => {
                    // Escape sequence
                    self.advance();
                    match self.current_char {
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('r') => result.push('\r'),
                        Some('\\') => result.push('\\'),
                        Some('"') => result.push('"'),
                        Some(ch) => result.push(ch), // Keep other escaped chars as-is
                        None => {
                            return Err(LexError {
                                pos: self.pos,
                                message: "Unterminated escape sequence".to_string(),
                            });
                        }
                    }
                    self.advance();
                }
                _ => {
                    result.push(ch);
                    self.advance();
                }
            }
        }

        Err(LexError {
            pos: start_pos,
            message: "Unterminated string literal".to_string(),
        })
    }

    /// Read an identifier or keyword
    fn read_ident_or_keyword(&mut self) -> String {
        let mut result = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        result
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        match self.current_char {
            None => Ok(Token::Eof),

            Some('(') => {
                self.advance();
                Ok(Token::LParen)
            }

            Some(')') => {
                self.advance();
                Ok(Token::RParen)
            }

            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }

            Some('"') => {
                let s = self.read_string()?;
                Ok(Token::String(s))
            }

            Some('=') => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Token::OpEq)
                } else {
                    Err(LexError {
                        pos: self.pos,
                        message: "Expected '==' but found single '='".to_string(),
                    })
                }
            }

            Some('!') => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Token::OpNeq)
                } else {
                    Err(LexError {
                        pos: self.pos,
                        message: "Expected '!=' but found single '!'".to_string(),
                    })
                }
            }

            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_ident_or_keyword();

                // Check if it's a keyword/operator
                match ident.as_str() {
                    "AND" => Ok(Token::KwAnd),
                    "OR" => Ok(Token::KwOr),
                    "NOT" => Ok(Token::KwNot),
                    "startsWith" => Ok(Token::OpStartsWith),
                    "endsWith" => Ok(Token::OpEndsWith),
                    "contains" => Ok(Token::OpContains),
                    "matches" => Ok(Token::OpMatches),
                    _ => Ok(Token::Ident(ident)),
                }
            }

            Some(ch) => Err(LexError {
                pos: self.pos,
                message: format!("Unexpected character: '{}'", ch),
            }),
        }
    }

    /// Tokenize the entire input into a vector of tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let mut lexer = Lexer::new(r#"method == "GET""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4); // ident, ==, string, EOF
        assert_eq!(tokens[0], Token::Ident("method".to_string()));
        assert_eq!(tokens[1], Token::OpEq);
        assert_eq!(tokens[2], Token::String("GET".to_string()));
        assert_eq!(tokens[3], Token::Eof);
    }

    #[test]
    fn test_string_with_escapes() {
        let mut lexer = Lexer::new(r#""hello \"world\" \n""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2); // string, EOF
        assert_eq!(tokens[0], Token::String("hello \"world\" \n".to_string()));
    }

    #[test]
    fn test_all_operators() {
        let input = r#"== != startsWith endsWith contains matches"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::OpEq);
        assert_eq!(tokens[1], Token::OpNeq);
        assert_eq!(tokens[2], Token::OpStartsWith);
        assert_eq!(tokens[3], Token::OpEndsWith);
        assert_eq!(tokens[4], Token::OpContains);
        assert_eq!(tokens[5], Token::OpMatches);
    }

    #[test]
    fn test_all_keywords() {
        let mut lexer = Lexer::new("AND OR NOT");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::KwAnd);
        assert_eq!(tokens[1], Token::KwOr);
        assert_eq!(tokens[2], Token::KwNot);
    }

    #[test]
    fn test_function_call() {
        let mut lexer = Lexer::new(r#"header("X-Test")"#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Ident("header".to_string()));
        assert_eq!(tokens[1], Token::LParen);
        assert_eq!(tokens[2], Token::String("X-Test".to_string()));
        assert_eq!(tokens[3], Token::RParen);
    }

    #[test]
    fn test_complex_expression() {
        let input = r#"contains(headerList("X-Auth-User-Teams"), "platform-eng")"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::OpContains); // contains is an operator!
        assert_eq!(tokens[1], Token::LParen);
        assert_eq!(tokens[2], Token::Ident("headerList".to_string()));
        assert_eq!(tokens[3], Token::LParen);
        assert_eq!(tokens[4], Token::String("X-Auth-User-Teams".to_string()));
        assert_eq!(tokens[5], Token::RParen);
        assert_eq!(tokens[6], Token::Comma);
        assert_eq!(tokens[7], Token::String("platform-eng".to_string()));
        assert_eq!(tokens[8], Token::RParen);
    }

    #[test]
    fn test_error_unterminated_string() {
        let mut lexer = Lexer::new(r#""unterminated"#);
        let result = lexer.tokenize();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unterminated string"));
    }

    #[test]
    fn test_error_unexpected_char() {
        let mut lexer = Lexer::new("@invalid");
        let result = lexer.tokenize();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unexpected character"));
    }

    #[test]
    fn test_error_single_equals() {
        let mut lexer = Lexer::new("method = value");
        let result = lexer.tokenize();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Expected '=='"));
    }
}
