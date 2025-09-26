// AST for math interpreter
/// The abstract syntax tree for math expressions.
#[derive(Debug, Clone)]
pub enum Expr {
    /// A numeric literal.
    Number(f64),
    /// A variable reference.
    Ident(String),
    /// An assignment: variable = value
    Assign {
        name: String,
        expr: Box<Expr>,
    },
    /// A binary operation (e.g., +, -, *, /).
    BinaryOp {
        left: Box<Expr>,
        op: crate::lexer::BinaryOperator,
        right: Box<Expr>,
    },
    /// A special function call (e.g., sin, cos, exp).
    Function {
        func: crate::lexer::SpecialFunction,
        arg: Box<Expr>,
    },
    /// A user-defined function definition: def name(arg) = body
    FunctionDef {
        name: String,
        arg: String,
        body: Box<Expr>,
    },
    /// A user-defined function call: name(expr)
    FunctionCall {
        name: String,
        arg: Box<Expr>,
    },
    /// A sequence of expressions (comma-separated)
    Sequence(Vec<Expr>),
    /// Sum(from, to, param, expr)
    Sum {
        from: Box<Expr>,
        to: Box<Expr>,
        param: String,
        body: Box<Expr>,
    },
    /// Product(from, to, param, expr)
    Product {
        from: Box<Expr>,
        to: Box<Expr>,
        param: String,
        body: Box<Expr>,
    },
}
