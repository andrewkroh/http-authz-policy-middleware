// Type checker and compiler for the expression language

use super::ast::{BinOp, Expr, Ident};
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
        let root = parser::parse(input)?;

        // Type check the expression
        let expr_type = type_check(&root)?;

        // Ensure top-level expression is boolean
        if expr_type != Type::Bool {
            return Err(CompileError {
                message: format!("Top-level expression must be boolean, got {}", expr_type),
            });
        }

        Ok(Program { root })
    }
}

/// Type check an expression recursively
fn type_check(expr: &Expr) -> Result<Type, CompileError> {
    match expr {
        Expr::BoolLiteral(_) => Ok(Type::Bool),

        Expr::StringLiteral(_) => Ok(Type::Str),

        Expr::Ident(ident) => match ident {
            Ident::Method | Ident::Path | Ident::Host => Ok(Type::Str),
        },

        Expr::BinaryOp { op, left, right } => {
            let left_type = type_check(left)?;
            let right_type = type_check(right)?;

            match op {
                BinOp::Eq | BinOp::Neq | BinOp::StartsWith | BinOp::EndsWith | BinOp::Matches => {
                    // All comparison operators require (string, string) -> bool
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
                    Ok(Type::Bool)
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
                    Ok(Type::Bool)
                }
            }
        }

        Expr::And(left, right) | Expr::Or(left, right) => {
            let left_type = type_check(left)?;
            let right_type = type_check(right)?;

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

            Ok(Type::Bool)
        }

        Expr::Not(inner) => {
            let inner_type = type_check(inner)?;
            if inner_type != Type::Bool {
                return Err(CompileError {
                    message: format!("NOT operator requires bool operand, got {}", inner_type),
                });
            }
            Ok(Type::Bool)
        }

        Expr::FuncCall { name, args } => type_check_function(name, args),
    }
}

/// Type check a function call
fn type_check_function(name: &str, args: &[Expr]) -> Result<Type, CompileError> {
    match name {
        // header(name: string) -> string
        "header" => {
            if args.len() != 1 {
                return Err(CompileError {
                    message: format!("Function 'header' expects 1 argument, got {}", args.len()),
                });
            }
            let arg_type = type_check(&args[0])?;
            if arg_type != Type::Str {
                return Err(CompileError {
                    message: format!(
                        "Function 'header' expects string argument, got {}",
                        arg_type
                    ),
                });
            }
            Ok(Type::Str)
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
            let arg_type = type_check(&args[0])?;
            if arg_type != Type::Str {
                return Err(CompileError {
                    message: format!(
                        "Function 'headerValues' expects string argument, got {}",
                        arg_type
                    ),
                });
            }
            Ok(Type::StrList)
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
            let arg_type = type_check(&args[0])?;
            if arg_type != Type::Str {
                return Err(CompileError {
                    message: format!(
                        "Function 'headerList' expects string argument, got {}",
                        arg_type
                    ),
                });
            }
            Ok(Type::StrList)
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
            let list_type = type_check(&args[0])?;
            let item_type = type_check(&args[1])?;

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
            Ok(Type::Bool)
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

            // First argument must be []string
            let list_type = type_check(&args[0])?;
            if list_type != Type::StrList {
                return Err(CompileError {
                    message: format!(
                        "Function 'anyOf' expects []string as first argument, got {}",
                        list_type
                    ),
                });
            }

            // Remaining arguments must be strings
            for (i, arg) in args.iter().skip(1).enumerate() {
                let arg_type = type_check(arg)?;
                if arg_type != Type::Str {
                    return Err(CompileError {
                        message: format!(
                            "Function 'anyOf' expects string arguments, got {} at position {}",
                            arg_type,
                            i + 2
                        ),
                    });
                }
            }

            Ok(Type::Bool)
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

            // First argument must be []string
            let list_type = type_check(&args[0])?;
            if list_type != Type::StrList {
                return Err(CompileError {
                    message: format!(
                        "Function 'allOf' expects []string as first argument, got {}",
                        list_type
                    ),
                });
            }

            // Remaining arguments must be strings
            for (i, arg) in args.iter().skip(1).enumerate() {
                let arg_type = type_check(arg)?;
                if arg_type != Type::Str {
                    return Err(CompileError {
                        message: format!(
                            "Function 'allOf' expects string arguments, got {} at position {}",
                            arg_type,
                            i + 2
                        ),
                    });
                }
            }

            Ok(Type::Bool)
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
}
