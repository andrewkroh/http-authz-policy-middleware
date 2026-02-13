// Copyright (c) 2025 Andrew Kroh
// SPDX-License-Identifier: MIT

// Type checker and compiler for the expression language

use super::ast::{BinOp, CompiledRegex, Expr, Ident};
use super::parser;
use std::fmt;

/// Type in the expression language
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// String type
    Str,
    /// String list type
    StrList,
    /// Boolean type
    Bool,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Str => write!(f, "string"),
            Type::StrList => write!(f, "[]string"),
            Type::Bool => write!(f, "bool"),
        }
    }
}

/// Compile-time type checking error
#[derive(Debug, Clone, PartialEq)]
pub struct CompileError {
    pub message: String,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Compile error: {}", self.message)
    }
}

impl std::error::Error for CompileError {}

impl From<parser::ParseError> for CompileError {
    fn from(err: parser::ParseError) -> Self {
        CompileError {
            message: format!("Parse error: {}", err.message),
        }
    }
}

/// Compiled program ready for evaluation
#[derive(Debug, Clone)]
pub struct Program {
    pub(crate) root: Expr,
}

impl Program {
    /// Compile an expression from a string
    pub fn compile(input: &str) -> Result<Self, CompileError> {
        // Parse the expression
        let parsed = parser::parse(input)?;

        // Type check and transform the expression (e.g., pre-compile regex patterns)
        let (expr_type, root) = type_check(&parsed)?;

        // Ensure top-level expression is boolean
        if expr_type != Type::Bool {
            return Err(CompileError {
                message: format!("Top-level expression must be boolean, got {}", expr_type),
            });
        }

        Ok(Program { root })
    }
}

/// Type check an expression recursively, returning the type and a
/// potentially-transformed expression (e.g., `matches` is replaced with
/// `RegexMatch` containing a pre-compiled regex).
fn type_check(expr: &Expr) -> Result<(Type, Expr), CompileError> {
    match expr {
        Expr::BoolLiteral(b) => Ok((Type::Bool, Expr::BoolLiteral(*b))),

        Expr::StringLiteral(s) => Ok((Type::Str, Expr::StringLiteral(s.clone()))),

        Expr::Ident(ident) => match ident {
            Ident::Method | Ident::Path | Ident::Host => {
                Ok((Type::Str, Expr::Ident(ident.clone())))
            }
        },

        Expr::BinaryOp { op, left, right } => {
            let (left_type, left_compiled) = type_check(left)?;
            let (right_type, right_compiled) = type_check(right)?;

            match op {
                BinOp::Eq | BinOp::Neq | BinOp::StartsWith | BinOp::EndsWith => {
                    if left_type != Type::Str {
                        return Err(CompileError {
                            message: format!(
                                "Operator {} requires string operands, got {} on left",
                                op, left_type
                            ),
                        });
                    }
                    if right_type != Type::Str {
                        return Err(CompileError {
                            message: format!(
                                "Operator {} requires string operands, got {} on right",
                                op, right_type
                            ),
                        });
                    }
                    Ok((
                        Type::Bool,
                        Expr::BinaryOp {
                            op: op.clone(),
                            left: Box::new(left_compiled),
                            right: Box::new(right_compiled),
                        },
                    ))
                }

                BinOp::Matches => {
                    // The matches operator requires string on the left
                    if left_type != Type::Str {
                        return Err(CompileError {
                            message: format!(
                                "Operator matches requires string operands, got {} on left",
                                left_type
                            ),
                        });
                    }

                    // Security: The pattern (right operand) MUST be a string literal
                    // to prevent regex injection from dynamic sources like headers.
                    let pattern = match right.as_ref() {
                        Expr::StringLiteral(s) => s,
                        _ => {
                            return Err(CompileError {
                                message: "Operator matches requires a string literal as the pattern; dynamic patterns are not allowed".to_string(),
                            });
                        }
                    };

                    // Pre-compile the regex at compile time
                    let compiled = CompiledRegex::new(pattern).map_err(|e| CompileError {
                        message: format!("Invalid regex pattern '{}': {}", pattern, e),
                    })?;

                    Ok((
                        Type::Bool,
                        Expr::RegexMatch {
                            expr: Box::new(left_compiled),
                            regex: compiled,
                        },
                    ))
                }

                BinOp::Contains => {
                    // contains operator: []string contains string -> bool
                    if left_type != Type::StrList {
                        return Err(CompileError {
                            message: format!(
                                "Operator contains requires []string as first operand, got {}",
                                left_type
                            ),
                        });
                    }
                    if right_type != Type::Str {
                        return Err(CompileError {
                            message: format!(
                                "Operator contains requires string as second operand, got {}",
                                right_type
                            ),
                        });
                    }
                    Ok((
                        Type::Bool,
                        Expr::BinaryOp {
                            op: BinOp::Contains,
                            left: Box::new(left_compiled),
                            right: Box::new(right_compiled),
                        },
                    ))
                }
            }
        }

        Expr::RegexMatch { .. } => {
            // RegexMatch nodes are only produced by the compiler, never by the parser.
            // If we encounter one here, just pass it through.
            Ok((Type::Bool, expr.clone()))
        }

        Expr::And(left, right) => {
            let (left_type, left_compiled) = type_check(left)?;
            let (right_type, right_compiled) = type_check(right)?;

            if left_type != Type::Bool {
                return Err(CompileError {
                    message: format!(
                        "Boolean operator requires bool operands, got {} on left",
                        left_type
                    ),
                });
            }
            if right_type != Type::Bool {
                return Err(CompileError {
                    message: format!(
                        "Boolean operator requires bool operands, got {} on right",
                        right_type
                    ),
                });
            }

            Ok((
                Type::Bool,
                Expr::And(Box::new(left_compiled), Box::new(right_compiled)),
            ))
        }

        Expr::Or(left, right) => {
            let (left_type, left_compiled) = type_check(left)?;
            let (right_type, right_compiled) = type_check(right)?;

            if left_type != Type::Bool {
                return Err(CompileError {
                    message: format!(
                        "Boolean operator requires bool operands, got {} on left",
                        left_type
                    ),
                });
            }
            if right_type != Type::Bool {
                return Err(CompileError {
                    message: format!(
                        "Boolean operator requires bool operands, got {} on right",
                        right_type
                    ),
                });
            }

            Ok((
                Type::Bool,
                Expr::Or(Box::new(left_compiled), Box::new(right_compiled)),
            ))
        }

        Expr::Not(inner) => {
            let (inner_type, inner_compiled) = type_check(inner)?;
            if inner_type != Type::Bool {
                return Err(CompileError {
                    message: format!("NOT operator requires bool operand, got {}", inner_type),
                });
            }
            Ok((Type::Bool, Expr::Not(Box::new(inner_compiled))))
        }

        Expr::FuncCall { name, args } => type_check_function(name, args),
    }
}

/// Type check a function call, returning the type and the reconstructed expression
fn type_check_function(name: &str, args: &[Expr]) -> Result<(Type, Expr), CompileError> {
    // Helper to build the reconstructed FuncCall expression
    let build_func =
        |name: &str, compiled_args: Vec<Expr>, typ: Type| -> Result<(Type, Expr), CompileError> {
            Ok((
                typ,
                Expr::FuncCall {
                    name: name.to_string(),
                    args: compiled_args,
                },
            ))
        };

    match name {
        // header(name: string) -> string
        "header" => {
            if args.len() != 1 {
                return Err(CompileError {
                    message: format!("Function 'header' expects 1 argument, got {}", args.len()),
                });
            }
            let (arg_type, arg_compiled) = type_check(&args[0])?;
            if arg_type != Type::Str {
                return Err(CompileError {
                    message: format!(
                        "Function 'header' expects string argument, got {}",
                        arg_type
                    ),
                });
            }
            build_func(name, vec![arg_compiled], Type::Str)
        }

        // headerValues(name: string) -> []string
        "headerValues" => {
            if args.len() != 1 {
                return Err(CompileError {
                    message: format!(
                        "Function 'headerValues' expects 1 argument, got {}",
                        args.len()
                    ),
                });
            }
            let (arg_type, arg_compiled) = type_check(&args[0])?;
            if arg_type != Type::Str {
                return Err(CompileError {
                    message: format!(
                        "Function 'headerValues' expects string argument, got {}",
                        arg_type
                    ),
                });
            }
            build_func(name, vec![arg_compiled], Type::StrList)
        }

        // headerList(name: string) -> []string
        "headerList" => {
            if args.len() != 1 {
                return Err(CompileError {
                    message: format!(
                        "Function 'headerList' expects 1 argument, got {}",
                        args.len()
                    ),
                });
            }
            let (arg_type, arg_compiled) = type_check(&args[0])?;
            if arg_type != Type::Str {
                return Err(CompileError {
                    message: format!(
                        "Function 'headerList' expects string argument, got {}",
                        arg_type
                    ),
                });
            }
            build_func(name, vec![arg_compiled], Type::StrList)
        }

        // contains(list: []string, item: string) -> bool
        // Note: This is handled by BinaryOp in the parser when used as contains(...)
        "contains" => {
            if args.len() != 2 {
                return Err(CompileError {
                    message: format!(
                        "Function 'contains' expects 2 arguments, got {}",
                        args.len()
                    ),
                });
            }
            let (list_type, list_compiled) = type_check(&args[0])?;
            let (item_type, item_compiled) = type_check(&args[1])?;

            if list_type != Type::StrList {
                return Err(CompileError {
                    message: format!(
                        "Function 'contains' expects []string as first argument, got {}",
                        list_type
                    ),
                });
            }
            if item_type != Type::Str {
                return Err(CompileError {
                    message: format!(
                        "Function 'contains' expects string as second argument, got {}",
                        item_type
                    ),
                });
            }
            build_func(name, vec![list_compiled, item_compiled], Type::Bool)
        }

        // anyOf(list: []string, items: ...string) -> bool
        "anyOf" => {
            if args.len() < 2 {
                return Err(CompileError {
                    message: format!(
                        "Function 'anyOf' expects at least 2 arguments, got {}",
                        args.len()
                    ),
                });
            }

            let mut compiled_args = Vec::with_capacity(args.len());

            // First argument must be []string
            let (list_type, list_compiled) = type_check(&args[0])?;
            if list_type != Type::StrList {
                return Err(CompileError {
                    message: format!(
                        "Function 'anyOf' expects []string as first argument, got {}",
                        list_type
                    ),
                });
            }
            compiled_args.push(list_compiled);

            // Remaining arguments must be strings
            for (i, arg) in args.iter().skip(1).enumerate() {
                let (arg_type, arg_compiled) = type_check(arg)?;
                if arg_type != Type::Str {
                    return Err(CompileError {
                        message: format!(
                            "Function 'anyOf' expects string arguments, got {} at position {}",
                            arg_type,
                            i + 2
                        ),
                    });
                }
                compiled_args.push(arg_compiled);
            }

            build_func(name, compiled_args, Type::Bool)
        }

        // allOf(list: []string, items: ...string) -> bool
        "allOf" => {
            if args.len() < 2 {
                return Err(CompileError {
                    message: format!(
                        "Function 'allOf' expects at least 2 arguments, got {}",
                        args.len()
                    ),
                });
            }

            let mut compiled_args = Vec::with_capacity(args.len());

            // First argument must be []string
            let (list_type, list_compiled) = type_check(&args[0])?;
            if list_type != Type::StrList {
                return Err(CompileError {
                    message: format!(
                        "Function 'allOf' expects []string as first argument, got {}",
                        list_type
                    ),
                });
            }
            compiled_args.push(list_compiled);

            // Remaining arguments must be strings
            for (i, arg) in args.iter().skip(1).enumerate() {
                let (arg_type, arg_compiled) = type_check(arg)?;
                if arg_type != Type::Str {
                    return Err(CompileError {
                        message: format!(
                            "Function 'allOf' expects string arguments, got {} at position {}",
                            arg_type,
                            i + 2
                        ),
                    });
                }
                compiled_args.push(arg_compiled);
            }

            build_func(name, compiled_args, Type::Bool)
        }

        _ => Err(CompileError {
            message: format!("Unknown function '{}'", name),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_expression() {
        let program = Program::compile(r#"method == "GET""#).unwrap();
        assert!(matches!(program.root, Expr::BinaryOp { .. }));
    }

    #[test]
    fn test_compile_complex_expression() {
        let program =
            Program::compile(r#"contains(headerList("X-Auth-User-Teams"), "platform-eng")"#)
                .unwrap();
        assert!(matches!(program.root, Expr::BinaryOp { .. }));
    }

    #[test]
    fn test_error_top_level_not_bool() {
        let result = Program::compile(r#"method"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("must be boolean"));
    }

    #[test]
    fn test_error_type_mismatch_and() {
        let result = Program::compile(r#"method AND path"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("bool operands"));
    }

    #[test]
    fn test_error_contains_wrong_type() {
        let result = Program::compile(r#"contains("foo", "bar")"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("[]string"));
    }

    #[test]
    fn test_error_function_arity() {
        let result = Program::compile(r#"header("X-Test", "extra")"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("expects 1 argument"));
    }

    #[test]
    fn test_error_anyof_arity() {
        let result = Program::compile(r#"anyOf(headerList("X-Test"))"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("at least 2 arguments"));
    }

    #[test]
    fn test_valid_anyof() {
        let program =
            Program::compile(r#"anyOf(headerList("X-Teams"), "platform-eng", "devops")"#).unwrap();
        assert!(matches!(program.root, Expr::FuncCall { .. }));
    }

    #[test]
    fn test_valid_allof() {
        let program =
            Program::compile(r#"allOf(headerList("X-Teams"), "platform-eng", "devops")"#).unwrap();
        assert!(matches!(program.root, Expr::FuncCall { .. }));
    }

    #[test]
    fn test_error_unknown_function() {
        let result = Program::compile(r#"unknownFunc("test")"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unknown function"));
    }

    #[test]
    fn test_matches_requires_literal_pattern() {
        // Dynamic patterns (e.g., from headers) must be rejected to prevent regex injection
        let result = Program::compile(r#"matches(path, header("X-Pattern"))"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("string literal"),
            "Expected 'string literal' error, got: {}",
            err.message
        );
    }

    #[test]
    fn test_matches_invalid_regex_caught_at_compile() {
        let result = Program::compile(r#"matches(path, "[invalid")"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("Invalid regex"),
            "Expected 'Invalid regex' error, got: {}",
            err.message
        );
    }

    #[test]
    fn test_matches_valid_regex_compiles() {
        let program = Program::compile(r#"matches(path, "^/api/v[0-9]+/.*")"#).unwrap();
        assert!(
            matches!(program.root, Expr::RegexMatch { .. }),
            "Expected RegexMatch, got: {:?}",
            program.root
        );
    }

    #[test]
    fn test_matches_infix_dynamic_rejected() {
        // Infix syntax with dynamic pattern must also be rejected
        let result = Program::compile(r#"path matches header("X")"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("string literal"),
            "Expected 'string literal' error, got: {}",
            err.message
        );
    }
}
