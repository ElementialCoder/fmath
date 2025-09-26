// sum(from: a, to: b, para: para_name, expr)
fn parse_sum_product(tokens: &[Token], pos: usize) -> Option<(Expr, usize)> {
    let (is_sum, start) = match tokens.get(pos) {
        Some(Token::Sum) => (true, pos + 1),
        Some(Token::Product) => (false, pos + 1),
        _ => return None,
    };
    if let Some(Token::LParen) = tokens.get(start) {
        // sum(product)(from: a, to: b, para: para_name, expr)
        let mut idx = start + 1;
        // from: expr
        if let Some(Token::Ident(from_kw)) = tokens.get(idx) {
            if from_kw == "from" {
                idx += 1;
                let (from_expr, next_idx) = parse_expr(tokens, idx);
                idx = next_idx;
                if let Some(Token::Comma) = tokens.get(idx) {
                    idx += 1;
                    // to: expr
                    if let Some(Token::Ident(to_kw)) = tokens.get(idx) {
                        if to_kw == "to" {
                            idx += 1;
                            let (to_expr, next_idx) = parse_expr(tokens, idx);
                            idx = next_idx;
                            if let Some(Token::Comma) = tokens.get(idx) {
                                idx += 1;
                                // para: para_name
                                if let Some(Token::Ident(para_kw)) = tokens.get(idx) {
                                    if para_kw == "para" {
                                        idx += 1;
                                        if let Some(Token::Ident(param_name)) = tokens.get(idx) {
                                            idx += 1;
                                            if let Some(Token::Comma) = tokens.get(idx) {
                                                idx += 1;
                                                // expr
                                                let (body_expr, next_idx) = parse_expr(tokens, idx);
                                                idx = next_idx;
                                                if let Some(Token::RParen) = tokens.get(idx) {
                                                    let expr = if is_sum {
                                                        Expr::Sum {
                                                            from: Box::new(from_expr),
                                                            to: Box::new(to_expr),
                                                            param: param_name.clone(),
                                                            body: Box::new(body_expr),
                                                        }
                                                    } else {
                                                        Expr::Product {
                                                            from: Box::new(from_expr),
                                                            to: Box::new(to_expr),
                                                            param: param_name.clone(),
                                                            body: Box::new(body_expr),
                                                        }
                                                    };
                                                    return Some((expr, idx + 1));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
// ...existing code...

// Parser for math interpreter
use crate::lexer::{Token, BinaryOperator};
use crate::ast::Expr;

// Recursive descent parser for fast evaluation
use std::collections::HashMap;
/// Parses lines of tokens into (main expression, user function map)
pub fn parse(lines: Vec<Vec<Token>>) -> (Expr, HashMap<String, (String, Expr)>) {
    use crate::ast::Expr;
    let mut exprs = Vec::new();
    let mut user_functions = HashMap::new();
    for tokens in lines {
        if tokens.is_empty() { continue; }
        // Filter out function definition lines from main exprs
        let is_func_def = matches!(tokens.get(0), Some(Token::Def));
        if is_func_def {
            if let Some(Token::Ident(name)) = tokens.get(1) {
                if let Some(Token::LParen) = tokens.get(2) {
                    if let Some(Token::Ident(arg_name)) = tokens.get(3) {
                        if let Some(Token::RParen) = tokens.get(4) {
                            if let Some(Token::Assign) = tokens.get(5) {
                                let (body, _) = parse_expr(&tokens, 6);
                                user_functions.insert(name.clone(), (arg_name.clone(), body));
                                continue;
                            }
                        }
                    }
                }
            }
        }
        // Only push non-function-def lines to exprs
        if !is_func_def {
            let (expr, next_pos) = parse_statement(&tokens, 0);
            if next_pos < tokens.len() {
                panic!("Unexpected token: {:?}", tokens[next_pos]);
            }
            exprs.push(expr);
        }
    }
    let main_expr = if exprs.len() == 1 {
        exprs.pop().unwrap()
    } else {
        Expr::Sequence(exprs)
    };
    (main_expr, user_functions)
}
// Parse a statement: assignment or expression
fn parse_statement(tokens: &[Token], pos: usize) -> (Expr, usize) {
    // function definition: def name(arg) = expr
    if let Some(Token::Def) = tokens.get(pos) {
        if let Some(Token::Ident(name)) = tokens.get(pos + 1) {
            if let Some(Token::LParen) = tokens.get(pos + 2) {
                if let Some(Token::Ident(arg_name)) = tokens.get(pos + 3) {
                    if let Some(Token::RParen) = tokens.get(pos + 4) {
                        if let Some(Token::Assign) = tokens.get(pos + 5) {
                            let (body, next_pos) = parse_expr(tokens, pos + 6);
                            return (Expr::FunctionDef {
                                name: name.clone(),
                                arg: arg_name.clone(),
                                body: Box::new(body),
                            }, next_pos);
                        }
                    }
                }
            }
        }
    }
    // variable declaration/assignment: var Ident = expr
    if let Some(Token::Var) = tokens.get(pos) {
        if let Some(Token::Ident(name)) = tokens.get(pos + 1) {
            if let Some(Token::Assign) = tokens.get(pos + 2) {
                let (expr, next_pos) = parse_expr(tokens, pos + 3);
                return (Expr::Assign { name: name.clone(), expr: Box::new(expr) }, next_pos);
            }
        }
    }
    // Fallback: parse any expression (including sum/product) as a statement
    parse_expr(tokens, pos)
}

fn parse_sequence(tokens: &[Token], pos: usize) -> (Expr, usize) {
    let mut exprs = Vec::new();
    let (first, mut pos) = parse_expr(tokens, pos);
    exprs.push(first);
    while pos < tokens.len() {
        if let Token::Comma = tokens[pos] {
            let (next, next_pos) = parse_expr(tokens, pos + 1);
            exprs.push(next);
            pos = next_pos;
        } else {
            break;
        }
    }
    if exprs.len() == 1 {
        (exprs.pop().unwrap(), pos)
    } else {
        (Expr::Sequence(exprs), pos)
    }
}

fn parse_expr(tokens: &[Token], pos: usize) -> (Expr, usize) {
    let (mut left, mut pos) = parse_term(tokens, pos);
    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Operator(BinaryOperator::Plus) => {
                let (right, next_pos) = parse_term(tokens, pos + 1);
                left = Expr::BinaryOp { left: Box::new(left), op: BinaryOperator::Plus, right: Box::new(right) };
                pos = next_pos;
            }
            Token::Operator(BinaryOperator::Minus) => {
                let (right, next_pos) = parse_term(tokens, pos + 1);
                left = Expr::BinaryOp { left: Box::new(left), op: BinaryOperator::Minus, right: Box::new(right) };
                pos = next_pos;
            }
            _ => break,
        }
    }
    (left, pos)
}

fn parse_term(tokens: &[Token], pos: usize) -> (Expr, usize) {
    let (mut left, mut pos) = parse_power(tokens, pos);
    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Operator(BinaryOperator::Star) => {
                let (right, next_pos) = parse_power(tokens, pos + 1);
                left = Expr::BinaryOp { left: Box::new(left), op: BinaryOperator::Star, right: Box::new(right) };
                pos = next_pos;
            }
            Token::Operator(BinaryOperator::Slash) => {
                let (right, next_pos) = parse_power(tokens, pos + 1);
                left = Expr::BinaryOp { left: Box::new(left), op: BinaryOperator::Slash, right: Box::new(right) };
                pos = next_pos;
            }
            _ => break,
        }
    }
    (left, pos)
}

// Parse power operator (right-associative)
fn parse_power(tokens: &[Token], pos: usize) -> (Expr, usize) {
    let (mut left, mut pos) = parse_factor(tokens, pos);
    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Operator(BinaryOperator::Pow) => {
                let (right, next_pos) = parse_power(tokens, pos + 1);
                left = Expr::BinaryOp { left: Box::new(left), op: BinaryOperator::Pow, right: Box::new(right) };
                pos = next_pos;
            }
            _ => break,
        }
    }
    (left, pos)
}

fn parse_factor(tokens: &[Token], pos: usize) -> (Expr, usize) {
    // sum/product
    if let Some((sumprod, next_pos)) = parse_sum_product(tokens, pos) {
        return (sumprod, next_pos);
    }
    // sum/product not supported in compiled mode
    let (mut expr, mut pos) = match &tokens[pos] {
        Token::Operator(BinaryOperator::Minus) => {
            // Unary minus: -factor
            let (expr, next_pos) = parse_factor(tokens, pos + 1);
            (Expr::BinaryOp {
                left: Box::new(Expr::Number(0.0)),
                op: BinaryOperator::Minus,
                right: Box::new(expr),
            }, next_pos)
        }
        Token::Pipe => {
            // Absolute value: |expr|
            let (inner, next_pos) = parse_expr(tokens, pos + 1);
            if let Some(Token::Pipe) = tokens.get(next_pos) {
                (Expr::Function { func: crate::lexer::SpecialFunction::Abs, arg: Box::new(inner) }, next_pos + 1)
            } else {
                panic!("Expected closing | for absolute value")
            }
        }
        Token::Number(n) => (Expr::Number(*n), pos + 1),
        // Function call: name(expr)
        Token::Ident(name) => {
            if let Some(Token::LParen) = tokens.get(pos + 1) {
                let (arg, mut next_pos) = parse_expr(tokens, pos + 2);
                let mut args = vec![arg];
                while let Some(Token::Comma) = tokens.get(next_pos) {
                    let (next_arg, np) = parse_expr(tokens, next_pos + 1);
                    args.push(next_arg);
                    next_pos = np;
                }
                if let Some(Token::RParen) = tokens.get(next_pos) {
                    if args.len() == 1 {
                        (Expr::FunctionCall { name: name.clone(), arg: Box::new(args.remove(0)) }, next_pos + 1)
                    } else {
                        (Expr::FunctionCall { name: name.clone(), arg: Box::new(Expr::Sequence(args)) }, next_pos + 1)
                    }
                } else {
                    panic!("Expected closing parenthesis after function call arguments")
                }
            } else {
                (Expr::Ident(name.clone()), pos + 1)
            }
        }
        Token::Function(func) => {
            if let Some(Token::LParen) = tokens.get(pos + 1) {
                // Support zero or more arguments (comma-separated)
                if let Some(Token::RParen) = tokens.get(pos + 2) {
                    // No arguments: f()
                    (Expr::Function { func: *func, arg: Box::new(Expr::Sequence(vec![])) }, pos + 3)
                } else {
                    // One or more arguments: f(arg1, arg2, ...)
                    let (arg, mut next_pos) = parse_expr(tokens, pos + 2);
                    let mut args = vec![arg];
                    while let Some(Token::Comma) = tokens.get(next_pos) {
                        let (next_arg, np) = parse_expr(tokens, next_pos + 1);
                        args.push(next_arg);
                        next_pos = np;
                    }
                    if let Some(Token::RParen) = tokens.get(next_pos) {
                        (Expr::Function { func: *func, arg: Box::new(Expr::Sequence(args)) }, next_pos + 1)
                    } else {
                        panic!("Expected closing parenthesis after function arguments")
                    }
                }
            } else {
                panic!("Expected opening parenthesis after function name")
            }
        }
        Token::LParen => {
            let (expr, next_pos) = parse_expr(tokens, pos + 1);
            if let Some(Token::RParen) = tokens.get(next_pos) {
                (expr, next_pos + 1)
            } else {
                panic!("Expected closing parenthesis")
            }
        }
        Token::Operator(op) => {
            panic!("Operator token {:?} in invalid position. Likely missing operand before or after operator.", op)
        }
    _ => panic!("Unexpected token: {:?}. Only compiled mode is supported; sum/product are not allowed.", tokens[pos]),
    };
    // Postfix factorial: expr!
    while pos < tokens.len() {
        if let Token::Function(crate::lexer::SpecialFunction::Fact) = &tokens[pos] {
            expr = Expr::Function { func: crate::lexer::SpecialFunction::Fact, arg: Box::new(expr) };
            pos += 1;
        } else {
            break;
        }
    }
    (expr, pos)
}
