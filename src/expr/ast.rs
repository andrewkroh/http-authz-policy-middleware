// Abstract Syntax Tree (AST) for the expression language

use std::fmt;

/// Expression AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Boolean literal (true/false)
    BoolLiteral(bool),

    /// String literal
    StringLiteral(String),

    /// Built-in identifier (method, path, host)
    Ident(Ident),

    /// Function call
    FuncCall { name: String, args: Vec<Expr> },

    /// Binary operation (comparison operators)
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// NOT expression
    Not(Box<Expr>),

    /// AND expression
    And(Box<Expr>, Box<Expr>),

    /// OR expression
    Or(Box<Expr>, Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::BoolLiteral(b) => write!(f, "{}", b),
            Expr::StringLiteral(s) => write!(f, "\"{}\"", s),
            Expr::Ident(id) => write!(f, "{}", id),
            Expr::FuncCall { name, args } => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::BinaryOp { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expr::Not(expr) => write!(f, "(NOT {})", expr),
            Expr::And(left, right) => write!(f, "({} AND {})", left, right),
            Expr::Or(left, right) => write!(f, "({} OR {})", left, right),
        }
    }
}

/// Built-in identifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ident {
    Method,
    Path,
    Host,
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ident::Method => write!(f, "method"),
            Ident::Path => write!(f, "path"),
            Ident::Host => write!(f, "host"),
        }
    }
}

/// Binary operators (comparison operators)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    /// String equality (==)
    Eq,

    /// String inequality (!=)
    Neq,

    /// String prefix match (startsWith)
    StartsWith,

    /// String suffix match (endsWith)
    EndsWith,

    /// Substring match (contains - infix)
    Contains,

    /// Regex match (matches)
    Matches,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Eq => write!(f, "=="),
            BinOp::Neq => write!(f, "!="),
            BinOp::StartsWith => write!(f, "startsWith"),
            BinOp::EndsWith => write!(f, "endsWith"),
            BinOp::Contains => write!(f, "contains"),
            BinOp::Matches => write!(f, "matches"),
        }
    }
}
