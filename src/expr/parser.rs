// Recursive descent parser for the expression language

use super::ast::{BinOp, Expr, Ident};
use super::lexer::{LexError, Lexer, Token};
use std::fmt;

/// Parser error with position information
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub pos: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.pos, self.message)
    }
}

impl std::error::Error for ParseError {}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError {
            pos: err.pos,
            message: err.message,
        }
    }
}

/// Recursive descent parser
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
    pos: usize,
}

impl Parser {
    /// Create a new parser from input string
    pub fn new(input: &str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token()?;
        let peek_token = lexer.next_token()?;

        Ok(Parser {
            lexer,
            current_token,
            peek_token,
            pos: 0,
        })
    }

    /// Advance to the next token
    fn advance(&mut self) -> Result<(), ParseError> {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token()?;
        self.pos += 1;
        Ok(())
    }

    /// Check if current token matches expected, and advance if so
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current_token != expected {
            return Err(ParseError {
                pos: self.pos,
                message: format!("Expected {:?}, got {:?}", expected, self.current_token),
            });
        }
        self.advance()
    }

    /// Parse an expression (entry point)
    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_or_expr()?;

        // Ensure we've consumed all input
        if self.current_token != Token::Eof {
            return Err(ParseError {
                pos: self.pos,
                message: format!("Unexpected token after expression: {:?}", self.current_token),
            });
        }

        Ok(expr)
    }

    /// Parse OR expression (lowest precedence)
    /// or_expr ::= and_expr ("OR" and_expr)*
    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;

        while self.current_token == Token::KwOr {
            self.advance()?;
            let right = self.parse_and_expr()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// Parse AND expression
    /// and_expr ::= not_expr ("AND" not_expr)*
    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not_expr()?;

        while self.current_token == Token::KwAnd {
            self.advance()?;
            let right = self.parse_not_expr()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// Parse NOT expression
    /// not_expr ::= "NOT" not_expr | comparison
    fn parse_not_expr(&mut self) -> Result<Expr, ParseError> {
        if self.current_token == Token::KwNot {
            self.advance()?;
            let expr = self.parse_not_expr()?;
            Ok(Expr::Not(Box::new(expr)))
        } else {
            self.parse_comparison()
        }
    }

    /// Parse comparison expression
    /// comparison ::= value (comp_op value)? | comp_op "(" value "," value ")"
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        // Check for operator in function-style syntax: op(left, right)
        let op = match &self.current_token {
            Token::OpEq => Some(BinOp::Eq),
            Token::OpNeq => Some(BinOp::Neq),
            Token::OpStartsWith => Some(BinOp::StartsWith),
            Token::OpEndsWith => Some(BinOp::EndsWith),
            Token::OpContains => Some(BinOp::Contains),
            Token::OpMatches => Some(BinOp::Matches),
            _ => None,
        };

        if let Some(op) = op {
            if self.peek_token == Token::LParen {
                // Function-style operator: op(left, right)
                self.advance()?; // consume operator
                self.expect(Token::LParen)?;
                let left = self.parse_or_expr()?;
                self.expect(Token::Comma)?;
                let right = self.parse_or_expr()?;
                self.expect(Token::RParen)?;
                return Ok(Expr::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                });
            }
        }

        // Normal infix syntax: left op right
        let left = self.parse_value()?;

        // Check for infix binary operator
        let op = match &self.current_token {
            Token::OpEq => Some(BinOp::Eq),
            Token::OpNeq => Some(BinOp::Neq),
            Token::OpStartsWith => Some(BinOp::StartsWith),
            Token::OpEndsWith => Some(BinOp::EndsWith),
            Token::OpContains => Some(BinOp::Contains),
            Token::OpMatches => Some(BinOp::Matches),
            _ => None,
        };

        if let Some(op) = op {
            self.advance()?;
            let right = self.parse_value()?;
            Ok(Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            // No operator, just return the value
            Ok(left)
        }
    }

    /// Parse value expression
    /// value ::= string | func_call | ident | "(" expr ")"
    fn parse_value(&mut self) -> Result<Expr, ParseError> {
        match &self.current_token {
            Token::String(s) => {
                let expr = Expr::StringLiteral(s.clone());
                self.advance()?;
                Ok(expr)
            }

            Token::Ident(name) => {
                let name = name.clone();

                // Check if it's a function call or just an identifier
                if self.peek_token == Token::LParen {
                    // Function call
                    self.advance()?; // consume ident
                    self.parse_func_call(name)
                } else {
                    // Check if it's a built-in identifier
                    let ident = match name.as_str() {
                        "method" => Ident::Method,
                        "path" => Ident::Path,
                        "host" => Ident::Host,
                        _ => {
                            // Unknown identifier - could be a function name used incorrectly
                            return Err(ParseError {
                                pos: self.pos,
                                message: format!(
                                    "Unknown identifier '{}'. Expected: method, path, host, or function call",
                                    name
                                ),
                            });
                        }
                    };
                    self.advance()?;
                    Ok(Expr::Ident(ident))
                }
            }

            Token::LParen => {
                self.advance()?; // consume (
                let expr = self.parse_or_expr()?; // parse inner expression
                self.expect(Token::RParen)?; // consume )
                Ok(expr)
            }

            _ => Err(ParseError {
                pos: self.pos,
                message: format!("Expected value, got {:?}", self.current_token),
            }),
        }
    }

    /// Parse function call (after consuming function name)
    /// func_call ::= ident "(" arg_list? ")"
    /// arg_list ::= expr ("," expr)*
    fn parse_func_call(&mut self, name: String) -> Result<Expr, ParseError> {
        self.expect(Token::LParen)?;

        let mut args = Vec::new();

        // Check for empty argument list
        if self.current_token == Token::RParen {
            self.advance()?;
            return Ok(Expr::FuncCall { name, args });
        }

        // Parse arguments
        loop {
            let arg = self.parse_or_expr()?;
            args.push(arg);

            if self.current_token == Token::Comma {
                self.advance()?;
                continue;
            } else if self.current_token == Token::RParen {
                self.advance()?;
                break;
            } else {
                return Err(ParseError {
                    pos: self.pos,
                    message: format!(
                        "Expected ',' or ')' in function call, got {:?}",
                        self.current_token
                    ),
                });
            }
        }

        Ok(Expr::FuncCall { name, args })
    }
}

/// Parse an expression from a string
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_comparison() {
        let expr = parse(r#"method == "GET""#).unwrap();

        match expr {
            Expr::BinaryOp { op, left, right } => {
                assert_eq!(op, BinOp::Eq);
                assert_eq!(*left, Expr::Ident(Ident::Method));
                assert_eq!(*right, Expr::StringLiteral("GET".to_string()));
            }
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let expr = parse(r#"contains(headerList("X-Auth-User-Teams"), "platform-eng")"#).unwrap();

        match expr {
            Expr::BinaryOp { op, left, right } => {
                assert_eq!(op, BinOp::Contains);

                // Left side: headerList("X-Auth-User-Teams")
                match &*left {
                    Expr::FuncCall { name, args } => {
                        assert_eq!(name, "headerList");
                        assert_eq!(args.len(), 1);
                        assert_eq!(args[0], Expr::StringLiteral("X-Auth-User-Teams".to_string()));
                    }
                    _ => panic!("Expected FuncCall on left"),
                }

                // Right side: "platform-eng"
                assert_eq!(*right, Expr::StringLiteral("platform-eng".to_string()));
            }
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_parse_and_expression() {
        let expr = parse(r#"path startsWith "/api" AND method == "GET""#).unwrap();

        match expr {
            Expr::And(left, right) => {
                // Left: path startsWith "/api"
                match &*left {
                    Expr::BinaryOp { op, .. } => assert_eq!(*op, BinOp::StartsWith),
                    _ => panic!("Expected BinaryOp on left"),
                }

                // Right: method == "GET"
                match &*right {
                    Expr::BinaryOp { op, .. } => assert_eq!(*op, BinOp::Eq),
                    _ => panic!("Expected BinaryOp on right"),
                }
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_parse_complex_nested() {
        let expr = parse(r#"(method == "GET" OR method == "HEAD") AND path startsWith "/public""#).unwrap();

        match expr {
            Expr::And(left, right) => {
                // Left: (method == "GET" OR method == "HEAD")
                match &*left {
                    Expr::Or(_, _) => {}
                    _ => panic!("Expected Or on left"),
                }

                // Right: path startsWith "/public"
                match &*right {
                    Expr::BinaryOp { op, .. } => assert_eq!(*op, BinOp::StartsWith),
                    _ => panic!("Expected BinaryOp on right"),
                }
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_parse_not_expression() {
        let expr = parse(r#"NOT method == "DELETE""#).unwrap();

        match expr {
            Expr::Not(inner) => match &*inner {
                Expr::BinaryOp { op, .. } => assert_eq!(*op, BinOp::Eq),
                _ => panic!("Expected BinaryOp inside Not"),
            },
            _ => panic!("Expected Not"),
        }
    }

    #[test]
    fn test_parse_parentheses() {
        let expr = parse(r#"(method == "GET")"#).unwrap();

        match expr {
            Expr::BinaryOp { op, .. } => assert_eq!(op, BinOp::Eq),
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_error_unclosed_paren() {
        let result = parse(r#"(method == "GET""#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Expected"));
    }

    #[test]
    fn test_error_unknown_identifier() {
        let result = parse(r#"unknown == "value""#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unknown identifier"));
    }

    #[test]
    fn test_error_unexpected_token() {
        let result = parse(r#"method == "GET" unexpected"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unexpected token"));
    }
}
