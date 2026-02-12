// Expression evaluator - runtime evaluation against RequestContext

use super::ast::{BinOp, Expr, Ident};
use super::compiler::Program;
use crate::context::RequestContext;
use regex::Regex;
use std::fmt;

/// Value types during evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    StrList(Vec<String>),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(s) => write!(f, "\"{}\"", s),
            Value::StrList(list) => {
                write!(f, "[")?;
                for (i, s) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\"", s)?;
                }
                write!(f, "]")
            }
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

/// Runtime evaluation error
#[derive(Debug, Clone, PartialEq)]
pub struct EvalError {
    pub message: String,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Evaluation error: {}", self.message)
    }
}

impl std::error::Error for EvalError {}

impl Program {
    /// Evaluate the program against a request context
    pub fn eval(&self, ctx: &RequestContext) -> Result<bool, EvalError> {
        match eval_expr(&self.root, ctx)? {
            Value::Bool(b) => Ok(b),
            _ => Err(EvalError {
                message: "Expression did not evaluate to boolean".to_string(),
            }),
        }
    }
}

/// Evaluate an expression recursively
fn eval_expr(expr: &Expr, ctx: &RequestContext) -> Result<Value, EvalError> {
    match expr {
        Expr::BoolLiteral(b) => Ok(Value::Bool(*b)),

        Expr::StringLiteral(s) => Ok(Value::Str(s.clone())),

        Expr::Ident(ident) => match ident {
            Ident::Method => Ok(Value::Str(ctx.method.clone())),
            Ident::Path => Ok(Value::Str(ctx.path.clone())),
            Ident::Host => Ok(Value::Str(ctx.host.clone())),
        },

        Expr::BinaryOp { op, left, right } => {
            let left_val = eval_expr(left, ctx)?;
            let right_val = eval_expr(right, ctx)?;
            eval_binop(op, left_val, right_val)
        }

        Expr::And(left, right) => {
            let left_val = eval_expr(left, ctx)?;
            match left_val {
                Value::Bool(false) => Ok(Value::Bool(false)), // Short-circuit
                Value::Bool(true) => {
                    let right_val = eval_expr(right, ctx)?;
                    match right_val {
                        Value::Bool(b) => Ok(Value::Bool(b)),
                        _ => Err(EvalError {
                            message: "AND operator requires boolean operands".to_string(),
                        }),
                    }
                }
                _ => Err(EvalError {
                    message: "AND operator requires boolean operands".to_string(),
                }),
            }
        }

        Expr::Or(left, right) => {
            let left_val = eval_expr(left, ctx)?;
            match left_val {
                Value::Bool(true) => Ok(Value::Bool(true)), // Short-circuit
                Value::Bool(false) => {
                    let right_val = eval_expr(right, ctx)?;
                    match right_val {
                        Value::Bool(b) => Ok(Value::Bool(b)),
                        _ => Err(EvalError {
                            message: "OR operator requires boolean operands".to_string(),
                        }),
                    }
                }
                _ => Err(EvalError {
                    message: "OR operator requires boolean operands".to_string(),
                }),
            }
        }

        Expr::Not(inner) => {
            let val = eval_expr(inner, ctx)?;
            match val {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(EvalError {
                    message: "NOT operator requires boolean operand".to_string(),
                }),
            }
        }

        Expr::FuncCall { name, args } => eval_function(name, args, ctx),
    }
}

/// Evaluate a binary operator
fn eval_binop(op: &BinOp, left: Value, right: Value) -> Result<Value, EvalError> {
    match (op, left, right) {
        (BinOp::Eq, Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l == r)),
        (BinOp::Neq, Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l != r)),
        (BinOp::StartsWith, Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l.starts_with(&r))),
        (BinOp::EndsWith, Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l.ends_with(&r))),

        (BinOp::Contains, Value::StrList(list), Value::Str(item)) => {
            Ok(Value::Bool(list.contains(&item)))
        }

        (BinOp::Matches, Value::Str(text), Value::Str(pattern)) => {
            // Compile regex and match
            let regex = Regex::new(&pattern).map_err(|e| EvalError {
                message: format!("Invalid regex pattern '{}': {}", pattern, e),
            })?;
            Ok(Value::Bool(regex.is_match(&text)))
        }

        _ => Err(EvalError {
            message: format!("Type mismatch in binary operator {}", op),
        }),
    }
}

/// Evaluate a function call
fn eval_function(name: &str, args: &[Expr], ctx: &RequestContext) -> Result<Value, EvalError> {
    match name {
        "header" => {
            // header(name: string) -> string
            let name_val = eval_expr(&args[0], ctx)?;
            match name_val {
                Value::Str(name) => {
                    let value = ctx.header(&name);
                    Ok(Value::Str(value.to_string()))
                }
                _ => Err(EvalError {
                    message: "header() expects string argument".to_string(),
                }),
            }
        }

        "headerValues" => {
            // headerValues(name: string) -> []string
            let name_val = eval_expr(&args[0], ctx)?;
            match name_val {
                Value::Str(name) => {
                    let values = ctx.header_values(&name);
                    Ok(Value::StrList(values.to_vec()))
                }
                _ => Err(EvalError {
                    message: "headerValues() expects string argument".to_string(),
                }),
            }
        }

        "headerList" => {
            // headerList(name: string) -> []string
            let name_val = eval_expr(&args[0], ctx)?;
            match name_val {
                Value::Str(name) => {
                    let list = ctx.header_list(&name);
                    Ok(Value::StrList(list))
                }
                _ => Err(EvalError {
                    message: "headerList() expects string argument".to_string(),
                }),
            }
        }

        "contains" => {
            // contains(list: []string, item: string) -> bool
            let list_val = eval_expr(&args[0], ctx)?;
            let item_val = eval_expr(&args[1], ctx)?;

            match (list_val, item_val) {
                (Value::StrList(list), Value::Str(item)) => {
                    Ok(Value::Bool(list.contains(&item)))
                }
                _ => Err(EvalError {
                    message: "contains() expects ([]string, string)".to_string(),
                }),
            }
        }

        "anyOf" => {
            // anyOf(list: []string, items: ...string) -> bool
            let list_val = eval_expr(&args[0], ctx)?;
            let list = match list_val {
                Value::StrList(l) => l,
                _ => {
                    return Err(EvalError {
                        message: "anyOf() expects []string as first argument".to_string(),
                    })
                }
            };

            // Check if any of the items are in the list
            for arg in args.iter().skip(1) {
                let item_val = eval_expr(arg, ctx)?;
                match item_val {
                    Value::Str(item) => {
                        if list.contains(&item) {
                            return Ok(Value::Bool(true));
                        }
                    }
                    _ => {
                        return Err(EvalError {
                            message: "anyOf() expects string arguments".to_string(),
                        })
                    }
                }
            }

            Ok(Value::Bool(false))
        }

        "allOf" => {
            // allOf(list: []string, items: ...string) -> bool
            let list_val = eval_expr(&args[0], ctx)?;
            let list = match list_val {
                Value::StrList(l) => l,
                _ => {
                    return Err(EvalError {
                        message: "allOf() expects []string as first argument".to_string(),
                    })
                }
            };

            // Check if all of the items are in the list
            for arg in args.iter().skip(1) {
                let item_val = eval_expr(arg, ctx)?;
                match item_val {
                    Value::Str(item) => {
                        if !list.contains(&item) {
                            return Ok(Value::Bool(false));
                        }
                    }
                    _ => {
                        return Err(EvalError {
                            message: "allOf() expects string arguments".to_string(),
                        })
                    }
                }
            }

            Ok(Value::Bool(true))
        }

        _ => Err(EvalError {
            message: format!("Unknown function '{}'", name),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TestRequest;
    use std::collections::HashMap;

    fn make_context(method: &str, path: &str, host: &str) -> RequestContext {
        let req = TestRequest {
            method: method.to_string(),
            path: path.to_string(),
            host: host.to_string(),
            headers: HashMap::new(),
        };
        RequestContext::from_test(&req)
    }

    fn make_context_with_headers(
        method: &str,
        path: &str,
        host: &str,
        headers: HashMap<String, String>,
    ) -> RequestContext {
        let req = TestRequest {
            method: method.to_string(),
            path: path.to_string(),
            host: host.to_string(),
            headers,
        };
        RequestContext::from_test(&req)
    }

    #[test]
    fn test_eval_simple_comparison() {
        let program = Program::compile(r#"method == "GET""#).unwrap();
        let ctx = make_context("GET", "/api", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), true);

        let ctx = make_context("POST", "/api", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), false);
    }

    #[test]
    fn test_eval_starts_with() {
        let program = Program::compile(r#"path startsWith "/api""#).unwrap();
        let ctx = make_context("GET", "/api/users", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), true);

        let ctx = make_context("GET", "/public", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), false);
    }

    #[test]
    fn test_eval_and_expression() {
        let program = Program::compile(r#"method == "GET" AND path startsWith "/api""#).unwrap();

        let ctx = make_context("GET", "/api/users", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), true);

        let ctx = make_context("POST", "/api/users", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), false);

        let ctx = make_context("GET", "/public", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), false);
    }

    #[test]
    fn test_eval_or_expression() {
        let program = Program::compile(r#"method == "GET" OR method == "HEAD""#).unwrap();

        let ctx = make_context("GET", "/", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), true);

        let ctx = make_context("HEAD", "/", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), true);

        let ctx = make_context("POST", "/", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), false);
    }

    #[test]
    fn test_eval_not_expression() {
        let program = Program::compile(r#"NOT method == "DELETE""#).unwrap();

        let ctx = make_context("GET", "/", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), true);

        let ctx = make_context("DELETE", "/", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), false);
    }

    #[test]
    fn test_eval_header_function() {
        let mut headers = HashMap::new();
        headers.insert("X-Test".to_string(), "value123".to_string());

        let program = Program::compile(r#"header("X-Test") == "value123""#).unwrap();
        let ctx = make_context_with_headers("GET", "/", "example.com", headers);
        assert_eq!(program.eval(&ctx).unwrap(), true);
    }

    #[test]
    fn test_eval_header_list_contains() {
        let mut headers = HashMap::new();
        headers.insert("X-Teams".to_string(), "platform-eng,devops,sre".to_string());

        let program =
            Program::compile(r#"contains(headerList("X-Teams"), "platform-eng")"#).unwrap();
        let ctx = make_context_with_headers("GET", "/", "example.com", headers);
        assert_eq!(program.eval(&ctx).unwrap(), true);
    }

    #[test]
    fn test_eval_anyof() {
        let mut headers = HashMap::new();
        headers.insert("X-Teams".to_string(), "platform-eng,devops".to_string());

        let program =
            Program::compile(r#"anyOf(headerList("X-Teams"), "platform-eng", "sre")"#).unwrap();
        let ctx = make_context_with_headers("GET", "/", "example.com", headers);
        assert_eq!(program.eval(&ctx).unwrap(), true);
    }

    #[test]
    fn test_eval_allof() {
        let mut headers = HashMap::new();
        headers.insert("X-Teams".to_string(), "platform-eng,devops,sre".to_string());

        let program =
            Program::compile(r#"allOf(headerList("X-Teams"), "platform-eng", "devops")"#).unwrap();
        let ctx = make_context_with_headers("GET", "/", "example.com", headers.clone());
        assert_eq!(program.eval(&ctx).unwrap(), true);

        // Missing one team
        headers.insert("X-Teams".to_string(), "platform-eng".to_string());
        let ctx = make_context_with_headers("GET", "/", "example.com", headers);
        assert_eq!(program.eval(&ctx).unwrap(), false);
    }

    #[test]
    fn test_eval_regex_matches() {
        let program = Program::compile(r#"matches(path, "^/api/v[0-9]+/.*")"#).unwrap();

        let ctx = make_context("GET", "/api/v1/users", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), true);

        let ctx = make_context("GET", "/api/users", "example.com");
        assert_eq!(program.eval(&ctx).unwrap(), false);
    }

    #[test]
    fn test_eval_regex_error() {
        let program = Program::compile(r#"matches(path, "[invalid")"#).unwrap();
        let ctx = make_context("GET", "/", "example.com");
        let result = program.eval(&ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("regex"));
    }

    #[test]
    fn test_eval_complex_expression() {
        let mut headers = HashMap::new();
        headers.insert("X-Teams".to_string(), "platform-eng,devops".to_string());

        let program = Program::compile(
            r#"(method == "GET" OR method == "HEAD") AND contains(headerList("X-Teams"), "platform-eng")"#,
        )
        .unwrap();

        let ctx = make_context_with_headers("GET", "/api", "example.com", headers.clone());
        assert_eq!(program.eval(&ctx).unwrap(), true);

        let ctx = make_context_with_headers("POST", "/api", "example.com", headers);
        assert_eq!(program.eval(&ctx).unwrap(), false);
    }
}
