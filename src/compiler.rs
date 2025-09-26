use crate::ast::Expr;
use crate::bytecode::{Bytecode, Program};
use crate::lexer::BinaryOperator;

/// Compile an AST expression into bytecode instructions.
pub fn compile(expr: &Expr, program: &mut Program) {
    match expr {
        Expr::Sum { from, to, param, body } => {
            // Compile from, to, and body as sub-programs
            let mut from_prog = Vec::new();
            let mut to_prog = Vec::new();
            let mut body_prog = Vec::new();
            compile(from, &mut from_prog);
            compile(to, &mut to_prog);
            compile(body, &mut body_prog);
            program.push(Bytecode::SumLoop {
                from: Box::new(from_prog),
                to: Box::new(to_prog),
                param: param.clone(),
                body: Box::new(body_prog),
            });
        }
        Expr::Product { from, to, param, body } => {
            let mut from_prog = Vec::new();
            let mut to_prog = Vec::new();
            let mut body_prog = Vec::new();
            compile(from, &mut from_prog);
            compile(to, &mut to_prog);
            compile(body, &mut body_prog);
            program.push(Bytecode::ProductLoop {
                from: Box::new(from_prog),
                to: Box::new(to_prog),
                param: param.clone(),
                body: Box::new(body_prog),
            });
        }
            Expr::Number(n) => {
                program.push(Bytecode::PushNumber(*n));
            }
            Expr::Ident(name) => {
                program.push(Bytecode::LoadVar(name.clone()));
            }
            Expr::Assign { name, expr } => {
                compile(expr, program);
                program.push(Bytecode::StoreVar(name.clone()));
            }
            Expr::BinaryOp { left, op, right } => {
                compile(left, program);
                compile(right, program);
                match op {
                    BinaryOperator::Plus => program.push(Bytecode::Add),
                    BinaryOperator::Minus => program.push(Bytecode::Sub),
                    BinaryOperator::Star => program.push(Bytecode::Mul),
                    BinaryOperator::Slash => program.push(Bytecode::Div),
                    BinaryOperator::Pow => program.push(Bytecode::Pow),
                }
            }
            Expr::Function { func, arg } => {
                use crate::lexer::SpecialFunction;
                match func {
                    SpecialFunction::Rand => {
                        program.push(Bytecode::Rand);
                    }
                    SpecialFunction::RandInt => {
                        // randint(a, b): arg is a Sequence of two expressions
                        if let Expr::Sequence(seq) = &**arg {
                            if seq.len() == 2 {
                                compile(&seq[0], program);
                                compile(&seq[1], program);
                                program.push(Bytecode::RandInt);
                            } else {
                                panic!("randint expects 2 arguments");
                            }
                        } else {
                            panic!("randint expects 2 arguments");
                        }
                    }
                    SpecialFunction::LogBase => {
                        // log(a, b) is handled as a binary function, so arg is a Sequence
                        if let Expr::Sequence(seq) = &**arg {
                            if seq.len() == 2 {
                                compile(&seq[0], program);
                                compile(&seq[1], program);
                                program.push(Bytecode::LogBase);
                            } else {
                                panic!("log base function expects 2 arguments");
                            }
                        } else {
                            panic!("log base function expects 2 arguments");
                        }
                    }
                    SpecialFunction::Fact => {
                        compile(arg, program);
                        program.push(Bytecode::Fact);
                    }
                    SpecialFunction::Sin => { compile(arg, program); program.push(Bytecode::Sin); }
                    SpecialFunction::Cos => { compile(arg, program); program.push(Bytecode::Cos); }
                    SpecialFunction::Tan => { compile(arg, program); program.push(Bytecode::Tan); }
                    SpecialFunction::Cot => { compile(arg, program); program.push(Bytecode::Cot); }
                    SpecialFunction::Sec => { compile(arg, program); program.push(Bytecode::Sec); }
                    SpecialFunction::Csc => { compile(arg, program); program.push(Bytecode::Csc); }
                    SpecialFunction::Sinh => { compile(arg, program); program.push(Bytecode::Sinh); }
                    SpecialFunction::Cosh => { compile(arg, program); program.push(Bytecode::Cosh); }
                    SpecialFunction::Tanh => { compile(arg, program); program.push(Bytecode::Tanh); }
                    SpecialFunction::Asinh => { compile(arg, program); program.push(Bytecode::Asinh); }
                    SpecialFunction::Acosh => { compile(arg, program); program.push(Bytecode::Acosh); }
                    SpecialFunction::Atanh => { compile(arg, program); program.push(Bytecode::Atanh); }
                    SpecialFunction::Exp => { compile(arg, program); program.push(Bytecode::Exp); }
                    SpecialFunction::Log => { compile(arg, program); program.push(Bytecode::Log); }
                    SpecialFunction::Log10 => { compile(arg, program); program.push(Bytecode::Log10); }
                    SpecialFunction::Log2 => { compile(arg, program); program.push(Bytecode::Log2); }
                    SpecialFunction::Sqrt => { compile(arg, program); program.push(Bytecode::Sqrt); }
                    SpecialFunction::Abs => { compile(arg, program); program.push(Bytecode::Abs); }
                    SpecialFunction::Asin => { compile(arg, program); program.push(Bytecode::Asin); }
                    SpecialFunction::Acos => { compile(arg, program); program.push(Bytecode::Acos); }
                    SpecialFunction::Atan => { compile(arg, program); program.push(Bytecode::Atan); }
                    SpecialFunction::Acot => { compile(arg, program); program.push(Bytecode::Acot); }
                    SpecialFunction::Asec => { compile(arg, program); program.push(Bytecode::Asec); }
                    SpecialFunction::Acsc => { compile(arg, program); program.push(Bytecode::Acsc); }
                    SpecialFunction::Pow => { compile(arg, program); program.push(Bytecode::Pow); }
                    SpecialFunction::Floor => { compile(arg, program); program.push(Bytecode::Floor); }
                }
            }
            Expr::FunctionDef { .. } => {
                // Do not emit code for function definitions here; handled at runtime
            }
            Expr::FunctionCall { name, arg } => {
                compile(arg, program);
                program.push(Bytecode::CallUserFunction(name.clone()));
            }
            Expr::Sequence(exprs) => {
                if exprs.is_empty() { return; }
                for (i, e) in exprs.iter().enumerate() {
                    compile(e, program);
                    // Only emit dummy pop for non-assignment expressions
                    if i + 1 != exprs.len() {
                        if !matches!(e, Expr::Assign { .. }) {
                            program.push(Bytecode::StoreVar("_tmp".to_string()));
                        }
                    }
                }
            }
    }
}
