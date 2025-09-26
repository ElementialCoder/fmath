// Lexer for math interpreter
/// Supported binary operators for math expressions.
use bincode::{Encode, Decode};
#[derive(Debug, Clone, Copy, PartialEq, Encode, Decode)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Star,
    Slash,
    Pow, // ^ operator
    // Add more operators here
}

/// Tokens produced by the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Encode, Decode)]
pub enum SpecialFunction {
    Sin,
    Cos,
    Tan,
    Cot,
    Sec,
    Csc,
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    Exp,
    Log,
    Log10,
    Log2,
    Sqrt,
    Abs,
    Asin,
    Acos,
    Atan,
    Acot,
    Asec,
    Acsc,
    Pow,
    Fact,
    LogBase,
    Floor,
    Rand,
    RandInt,
    // Add more as needed
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum Token {
    Number(f64),
    Operator(BinaryOperator),
    Function(SpecialFunction),
    Ident(String),
    Assign,
    LParen,
    RParen,
    Comma,
    Def,
    EndDef,
    Arrow,
    Var, // Added for variable declaration
    Pipe, // For |expr| absolute value
    Sum,
    Product,
}

/// Tokenizes a string input into a vector of tokens.
pub fn tokenize(input: &str) -> Vec<Vec<Token>> {
    input
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .map(|line| {
            let mut tokens = Vec::with_capacity(line.len() / 2);
            let mut chars = line.chars().peekable();
            while let Some(&c) = chars.peek() {
                match c {
                    '0'..='9' | '.' => {
                        let mut num = String::new();
                        while let Some(&d) = chars.peek() {
                            if d.is_ascii_digit() || d == '.' {
                                num.push(d);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if let Ok(n) = num.parse() {
                            tokens.push(Token::Number(n));
                        }
                    }
                    '+' => { tokens.push(Token::Operator(BinaryOperator::Plus)); chars.next(); }
                    '-' => { tokens.push(Token::Operator(BinaryOperator::Minus)); chars.next(); }
                    '*' => { tokens.push(Token::Operator(BinaryOperator::Star)); chars.next(); }
                    '/' => { tokens.push(Token::Operator(BinaryOperator::Slash)); chars.next(); }
                    '^' => { tokens.push(Token::Operator(BinaryOperator::Pow)); chars.next(); }
                    '!' => { tokens.push(Token::Function(SpecialFunction::Fact)); chars.next(); }
                    '(' => { tokens.push(Token::LParen); chars.next(); }
                    ')' => { tokens.push(Token::RParen); chars.next(); }
                    '|' => { tokens.push(Token::Pipe); chars.next(); }
                    ',' => { tokens.push(Token::Comma); chars.next(); }
                    '=' => {
                        // Support '=>' as Arrow, otherwise Assign
                        chars.next();
                        if let Some('>') = chars.peek() {
                            chars.next();
                            tokens.push(Token::Arrow);
                        } else {
                            tokens.push(Token::Assign);
                        }
                    }
                    c if c.is_alphabetic() => {
                        let mut ident = String::new();
                        while let Some(&d) = chars.peek() {
                            if d.is_alphanumeric() || d == '_' {
                                ident.push(d);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        match ident.to_ascii_lowercase().as_str() {
                            "sum" => tokens.push(Token::Sum),
                            "product" => tokens.push(Token::Product),
                            "def" => tokens.push(Token::Def),
                            "end" => tokens.push(Token::EndDef),
                            "var" => tokens.push(Token::Var),
                            "sin" => tokens.push(Token::Function(SpecialFunction::Sin)),
                            "cos" => tokens.push(Token::Function(SpecialFunction::Cos)),
                            "tan" => tokens.push(Token::Function(SpecialFunction::Tan)),
                            "cot" => tokens.push(Token::Function(SpecialFunction::Cot)),
                            "sec" => tokens.push(Token::Function(SpecialFunction::Sec)),
                            "csc" => tokens.push(Token::Function(SpecialFunction::Csc)),
                            "sinh" => tokens.push(Token::Function(SpecialFunction::Sinh)),
                            "cosh" => tokens.push(Token::Function(SpecialFunction::Cosh)),
                            "tanh" => tokens.push(Token::Function(SpecialFunction::Tanh)),
                            "asinh" => tokens.push(Token::Function(SpecialFunction::Asinh)),
                            "acosh" => tokens.push(Token::Function(SpecialFunction::Acosh)),
                            "atanh" => tokens.push(Token::Function(SpecialFunction::Atanh)),
                            "exp" => tokens.push(Token::Function(SpecialFunction::Exp)),
                            "log" => tokens.push(Token::Function(SpecialFunction::Log)),
                            "log10" => tokens.push(Token::Function(SpecialFunction::Log10)),
                            "log2" => tokens.push(Token::Function(SpecialFunction::Log2)),
                            "sqrt" => tokens.push(Token::Function(SpecialFunction::Sqrt)),
                            "abs" => tokens.push(Token::Function(SpecialFunction::Abs)),
                            "acos" => tokens.push(Token::Function(SpecialFunction::Acos)),
                            "atan" => tokens.push(Token::Function(SpecialFunction::Atan)),
                            "acot" => tokens.push(Token::Function(SpecialFunction::Acot)),
                            "asec" => tokens.push(Token::Function(SpecialFunction::Asec)),
                            "acsc" => tokens.push(Token::Function(SpecialFunction::Acsc)),
                            "pow" => tokens.push(Token::Function(SpecialFunction::Pow)),
                            "floor" => tokens.push(Token::Function(SpecialFunction::Floor)),
                            "rand" => tokens.push(Token::Function(SpecialFunction::Rand)),
                            "randint" => tokens.push(Token::Function(SpecialFunction::RandInt)),
                            _ => tokens.push(Token::Ident(ident)),
                        }
                    }
                    c if c.is_whitespace() => { chars.next(); }
                    _ => { chars.next(); }
                }
            }
            tokens
        })
        .filter(|tokens| !tokens.is_empty())
        .collect()
}
