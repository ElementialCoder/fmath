use crate::bytecode::{Bytecode, Program};
use std::collections::HashMap;
use crate::ast::Expr;
// use std::io::Write; // Commented out for clarity
// Evaluate an AST expression in the interpreter context (for user function bodies)
fn eval_expr(
    expr: &Expr,
    vars: &mut HashMap<String, f64>,
    user_functions: &HashMap<String, (String, Expr)>,
    rng: &mut impl rand::RngCore,
) -> Result<f64, &'static str> {
    use crate::lexer::SpecialFunction;
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Ident(name) => {
            match vars.get(name).copied() {
                Some(val) => Ok(val),
                None => {
                    Err("Variable not found in function body")
                }
            }
        },
        Expr::Assign { name, expr } => {
            let val = eval_expr(expr, vars, user_functions, rng)?;
            vars.insert(name.clone(), val);
            Ok(val)
        }
        Expr::BinaryOp { left, op, right } => {
            let l = eval_expr(left, vars, user_functions, rng)?;
            let r = eval_expr(right, vars, user_functions, rng)?;
            Ok(match op {
                crate::lexer::BinaryOperator::Plus => l + r,
                crate::lexer::BinaryOperator::Minus => l - r,
                crate::lexer::BinaryOperator::Star => l * r,
                crate::lexer::BinaryOperator::Slash => l / r,
                crate::lexer::BinaryOperator::Pow => l.powf(r),
            })
        }
        Expr::Function { func, arg } => {
            let val = eval_expr(arg, vars, user_functions, rng)?;
            Ok(match func {
                SpecialFunction::Sin => val.sin(),
                SpecialFunction::Cos => val.cos(),
                SpecialFunction::Tan => val.tan(),
                SpecialFunction::Cot => 1.0 / val.tan(),
                SpecialFunction::Sec => 1.0 / val.cos(),
                SpecialFunction::Csc => 1.0 / val.sin(),
                SpecialFunction::Sinh => val.sinh(),
                SpecialFunction::Cosh => val.cosh(),
                SpecialFunction::Tanh => val.tanh(),
                SpecialFunction::Asinh => val.asinh(),
                SpecialFunction::Acosh => val.acosh(),
                SpecialFunction::Atanh => val.atanh(),
                SpecialFunction::Exp => val.exp(),
                SpecialFunction::Log => val.ln(),
                SpecialFunction::Log10 => val.log10(),
                SpecialFunction::Log2 => val.log2(),
                SpecialFunction::Sqrt => val.sqrt(),
                SpecialFunction::Abs => val.abs(),
                SpecialFunction::Asin => val.asin(),
                SpecialFunction::Acos => val.acos(),
                SpecialFunction::Atan => val.atan(),
                SpecialFunction::Acot => (1.0 / val).atan(),
                SpecialFunction::Asec => (1.0 / val).acos(),
                SpecialFunction::Acsc => (1.0 / val).asin(),
                SpecialFunction::Pow => val, // Not used here
                SpecialFunction::Fact => factorial(val),
                SpecialFunction::LogBase => return Err("log base not supported in user function body"),
                SpecialFunction::Floor => val.floor(),
                SpecialFunction::Rand => rand::Rng::random(rng),
                SpecialFunction::RandInt => return Err("randint not supported in user function body"),
            })
        }
        Expr::FunctionCall { name, arg } => {
            let arg_val = eval_expr(arg, vars, user_functions, rng)?;
            let (param, body) = user_functions.get(name).ok_or("User-defined function not found in body")?;
            let old = vars.insert(param.clone(), arg_val);
            let result = eval_expr(body, vars, user_functions, rng)?;
            if let Some(v) = old {
                vars.insert(param.clone(), v);
            } else {
                vars.remove(param);
            }
            Ok(result)
        }
        Expr::Sequence(exprs) => {
            let mut last = 0.0;
            for e in exprs {
                last = eval_expr(e, vars, user_functions, rng)?;
            }
            Ok(last)
        }
        Expr::FunctionDef { .. } => Err("Nested function definitions not supported in body"),
        Expr::Sum { from, to, param, body } => {
            let from_val = eval_expr(from, vars, user_functions, rng)?;
            let to_val = eval_expr(to, vars, user_functions, rng)?;
            let from_i = from_val.ceil() as i64;
            let to_i = to_val.floor() as i64;
            let mut acc = 0.0;
            for i in from_i..=to_i {
                let old = vars.insert(param.clone(), i as f64);
                acc += eval_expr(body, vars, user_functions, rng)?;
                if let Some(v) = old { vars.insert(param.clone(), v); } else { vars.remove(param); }
            }
            Ok(acc)
        }
        Expr::Product { from, to, param, body } => {
            let from_val = eval_expr(from, vars, user_functions, rng)?;
            let to_val = eval_expr(to, vars, user_functions, rng)?;
            let from_i = from_val.ceil() as i64;
            let to_i = to_val.floor() as i64;
            let mut acc = 1.0;
            for i in from_i..=to_i {
                let old = vars.insert(param.clone(), i as f64);
                acc *= eval_expr(body, vars, user_functions, rng)?;
                if let Some(v) = old { vars.insert(param.clone(), v); } else { vars.remove(param); }
            }
            Ok(acc)
        }
    }
}
/// Executes a bytecode program and returns the result or an error message.
#[inline]
pub fn run_bytecode_with_functions(
    program: &Program,
    user_functions: &HashMap<String, (String, Expr)>,
) -> Result<f64, &'static str> {
    let mut stack: Vec<f64> = Vec::with_capacity(16);
    let mut vars: HashMap<String, f64> = HashMap::new();
    let mut rng = rand::rng();
    for instr in program {
        match instr {
            Bytecode::CallUserFunction(name) => {
                // Look up the function definition (single-argument only)
                let (arg_name, body) = user_functions.get(name)
                    .ok_or("User-defined function not found")?;
                let arg_val = stack.pop().ok_or("Stack underflow on user function call")?;
                // Save old value if shadowing
                let old = vars.insert(arg_name.clone(), arg_val);
                // Evaluate the function body recursively
                let result = eval_expr(body, &mut vars, user_functions, &mut rng)?;
                // Restore old value
                if let Some(v) = old {
                    vars.insert(arg_name.clone(), v);
                } else {
                    vars.remove(arg_name);
                }
                stack.push(result);
            }

            Bytecode::Rand => {
                stack.push(rand::Rng::random(&mut rng));
            }
            Bytecode::RandInt => {
                let b = stack.pop().ok_or("Stack underflow on RandInt (b)")?;
                let a = stack.pop().ok_or("Stack underflow on RandInt (a)")?;
                let (amin, amax) = if a <= b { (a, b) } else { (b, a) };
                let amin = amin.ceil() as i64;
                let amax = amax.floor() as i64;
                if amin > amax {
                    return Err("Invalid range for randint: min > max");
                }
                let val = rand::Rng::random_range(&mut rng, amin..=amax);
                stack.push(val as f64);
            }
            Bytecode::LogBase => {
                let b = stack.pop().ok_or("Stack underflow on LogBase (b)")?;
                let a = stack.pop().ok_or("Stack underflow on LogBase (a)")?;
                stack.push(b.log(a));
            }
            // Bytecode::Fact is not used in interpreter mode
            Bytecode::PushNumber(n) => stack.push(*n),
            Bytecode::Add => {
                let b = stack.pop().ok_or("Stack underflow on Add")?;
                let a = stack.pop().ok_or("Stack underflow on Add")?;
                stack.push(a + b);
            }
            Bytecode::Mul => {
                let b = stack.pop().ok_or("Stack underflow on Mul")?;
                let a = stack.pop().ok_or("Stack underflow on Mul")?;
                stack.push(a * b);
            }
            Bytecode::Div => {
                let b = stack.pop().ok_or("Stack underflow on Div")?;
                let a = stack.pop().ok_or("Stack underflow on Div")?;
                stack.push(a / b);
            }
            Bytecode::Sin => {
                let a = stack.pop().ok_or("Stack underflow on Sin")?;
                stack.push(a.sin());
            }
            Bytecode::Cos => {
                let a = stack.pop().ok_or("Stack underflow on Cos")?;
                stack.push(a.cos());
            }
            Bytecode::Tan => {
                let a = stack.pop().ok_or("Stack underflow on Tan")?;
                stack.push(a.tan());
            }
            Bytecode::Cot => {
                let a = stack.pop().ok_or("Stack underflow on Cot")?;
                stack.push(1.0 / a.tan());
            }
            Bytecode::Sec => {
                let a = stack.pop().ok_or("Stack underflow on Sec")?;
                stack.push(1.0 / a.cos());
            }
            Bytecode::Csc => {
                let a = stack.pop().ok_or("Stack underflow on Csc")?;
                stack.push(1.0 / a.sin());
            }
            Bytecode::Sinh => {
                let a = stack.pop().ok_or("Stack underflow on Sinh")?;
                stack.push(a.sinh());
            }
            Bytecode::Cosh => {
                let a = stack.pop().ok_or("Stack underflow on Cosh")?;
                stack.push(a.cosh());
            }
            Bytecode::Tanh => {
                let a = stack.pop().ok_or("Stack underflow on Tanh")?;
                stack.push(a.tanh());
            }
            Bytecode::Asinh => {
                let a = stack.pop().ok_or("Stack underflow on Asinh")?;
                stack.push(a.asinh());
            }
            Bytecode::Acosh => {
                let a = stack.pop().ok_or("Stack underflow on Acosh")?;
                stack.push(a.acosh());
            }
            Bytecode::Atanh => {
                let a = stack.pop().ok_or("Stack underflow on Atanh")?;
                stack.push(a.atanh());
            }
            Bytecode::Exp => {
                let a = stack.pop().ok_or("Stack underflow on Exp")?;
                stack.push(a.exp());
            }
            Bytecode::Log10 => {
                let a = stack.pop().ok_or("Stack underflow on Log10")?;
                stack.push(a.log10());
            }
            Bytecode::Log2 => {
                let a = stack.pop().ok_or("Stack underflow on Log2")?;
                stack.push(a.log2());
            }
            Bytecode::Fact => {
                let a = stack.pop().ok_or("Stack underflow on Fact")?;
                stack.push(factorial(a));
            }
            Bytecode::Floor => {
                let a = stack.pop().ok_or("Stack underflow on Floor")?;
                stack.push(a.floor());
            }
            Bytecode::Sub => {
                let b = stack.pop().ok_or("Stack underflow on Sub")?;
                let a = stack.pop().ok_or("Stack underflow on Sub")?;
                stack.push(a - b);
            }
            Bytecode::Log => {
                let a = stack.pop().ok_or("Stack underflow on Log")?;
                stack.push(a.ln());
            }
            Bytecode::Sqrt => {
                let a = stack.pop().ok_or("Stack underflow on Sqrt")?;
                stack.push(a.sqrt());
            }
            Bytecode::Abs => {
                let a = stack.pop().ok_or("Stack underflow on Abs")?;
                stack.push(a.abs());
            }
            Bytecode::Asin => {
                let a = stack.pop().ok_or("Stack underflow on Asin")?;
                stack.push(a.asin());
            }
            Bytecode::Acos => {
                let a = stack.pop().ok_or("Stack underflow on Acos")?;
                stack.push(a.acos());
            }
            Bytecode::Atan => {
                let a = stack.pop().ok_or("Stack underflow on Atan")?;
                stack.push(a.atan());
            }
            Bytecode::Acot => {
                let a = stack.pop().ok_or("Stack underflow on Acot")?;
                stack.push((1.0 / a).atan());
            }
            Bytecode::Asec => {
                let a = stack.pop().ok_or("Stack underflow on Asec")?;
                stack.push((1.0 / a).acos());
            }
            Bytecode::Acsc => {
                let a = stack.pop().ok_or("Stack underflow on Acsc")?;
                stack.push((1.0 / a).asin());
            }
            Bytecode::Pow => {
                let b = stack.pop().ok_or("Stack underflow on Pow")?;
                let a = stack.pop().ok_or("Stack underflow on Pow")?;
                stack.push(a.powf(b));
            }
            Bytecode::StoreVar(name) => {
                let val = stack.pop().ok_or("Stack underflow on StoreVar")?;
                vars.insert(name.clone(), val);
            }
            Bytecode::LoadVar(name) => {
                if !vars.contains_key(name) {
                    eprintln!("[DEBUG] Variable map: {:?}", vars);
                }
                let val = vars.get(name).ok_or("Variable not found")?;
                stack.push(*val);
            }
            Bytecode::SumLoop { from, to, param, body } => {
                let mut from_stack = Vec::new();
                run_bytecode_with_functions_inner(from, user_functions, &mut vars, &mut from_stack)?;
                let from_val = from_stack.pop().ok_or("No result on stack (from)")?;
                let mut to_stack = Vec::new();
                run_bytecode_with_functions_inner(to, user_functions, &mut vars, &mut to_stack)?;
                let to_val = to_stack.pop().ok_or("No result on stack (to)")?;
                let from_i = from_val.ceil() as i64;
                let to_i = to_val.floor() as i64;
                let mut acc = 0.0;
                for i in from_i..=to_i {
                    vars.insert(param.clone(), i as f64);
                    let mut body_stack = Vec::new();
                    run_bytecode_with_functions_inner(body, user_functions, &mut vars, &mut body_stack)?;
                    let result = body_stack.pop().ok_or("No result on stack (body)")?;
                    acc += result;
                }
                vars.remove(param);
                stack.push(acc);
            }
            Bytecode::ProductLoop { from, to, param, body } => {
                let mut from_stack = Vec::new();
                run_bytecode_with_functions_inner(from, user_functions, &mut vars, &mut from_stack)?;
                let from_val = from_stack.pop().ok_or("No result on stack (from)")?;
                let mut to_stack = Vec::new();
                run_bytecode_with_functions_inner(to, user_functions, &mut vars, &mut to_stack)?;
                let to_val = to_stack.pop().ok_or("No result on stack (to)")?;
                let from_i = from_val.ceil() as i64;
                let to_i = to_val.floor() as i64;
                let mut acc = 1.0;
                for i in from_i..=to_i {
                    vars.insert(param.clone(), i as f64);
                    let mut body_stack = Vec::new();
                    run_bytecode_with_functions_inner(body, user_functions, &mut vars, &mut body_stack)?;
                    let result = body_stack.pop().ok_or("No result on stack (body)")?;
                    acc *= result;
                }
                vars.remove(param);
                stack.push(acc);
            }
        }
    }
    stack.pop().ok_or("No result on stack")
}

fn factorial(x: f64) -> f64 {
    if x < 0.0 { return f64::NAN; }
    if x == 0.0 { return 1.0; }
    let mut acc = 1.0;
    let mut n = x.floor() as u64;
    while n > 1 {
        acc *= n as f64;
        n -= 1;
    }
    acc
}

fn run_bytecode_with_functions_inner(
    program: &Program,
    user_functions: &HashMap<String, (String, Expr)>,
    vars: &mut HashMap<String, f64>,
    stack: &mut Vec<f64>,
) -> Result<(), &'static str> {
    let mut rng = rand::rng();
    for instr in program {
        match instr {
            Bytecode::CallUserFunction(name) => {
                let (arg_name, body) = user_functions.get(name)
                    .ok_or("User-defined function not found")?;
                let arg_val = stack.pop().ok_or("Stack underflow on user function call")?;
                let old = vars.insert(arg_name.clone(), arg_val);
                let result = eval_expr(body, vars, user_functions, &mut rng)?;
                if let Some(v) = old {
                    vars.insert(arg_name.clone(), v);
                } else {
                    vars.remove(arg_name);
                }
                stack.push(result);
            }
            Bytecode::Rand => {
                stack.push(rand::Rng::random(&mut rng));
            }
            Bytecode::RandInt => {
                let b = stack.pop().ok_or("Stack underflow on RandInt (b)")?;
                let a = stack.pop().ok_or("Stack underflow on RandInt (a)")?;
                let (amin, amax) = if a <= b { (a, b) } else { (b, a) };
                let amin = amin.ceil() as i64;
                let amax = amax.floor() as i64;
                if amin > amax {
                    return Err("Invalid range for randint: min > max");
                }
                let val = rand::Rng::random_range(&mut rng, amin..=amax);
                stack.push(val as f64);
            }
            Bytecode::LogBase => {
                let b = stack.pop().ok_or("Stack underflow on LogBase (b)")?;
                let a = stack.pop().ok_or("Stack underflow on LogBase (a)")?;
                stack.push(b.log(a));
            }
            Bytecode::PushNumber(n) => stack.push(*n),
            Bytecode::Add => {
                let b = stack.pop().ok_or("Stack underflow on Add")?;
                let a = stack.pop().ok_or("Stack underflow on Add")?;
                stack.push(a + b);
            }
            Bytecode::Mul => {
                let b = stack.pop().ok_or("Stack underflow on Mul")?;
                let a = stack.pop().ok_or("Stack underflow on Mul")?;
                stack.push(a * b);
            }
            Bytecode::Div => {
                let b = stack.pop().ok_or("Stack underflow on Div")?;
                let a = stack.pop().ok_or("Stack underflow on Div")?;
                stack.push(a / b);
            }
            Bytecode::Sin => {
                let a = stack.pop().ok_or("Stack underflow on Sin")?;
                stack.push(a.sin());
            }
            Bytecode::Cos => {
                let a = stack.pop().ok_or("Stack underflow on Cos")?;
                stack.push(a.cos());
            }
            Bytecode::Tan => {
                let a = stack.pop().ok_or("Stack underflow on Tan")?;
                stack.push(a.tan());
            }
            Bytecode::Cot => {
                let a = stack.pop().ok_or("Stack underflow on Cot")?;
                stack.push(1.0 / a.tan());
            }
            Bytecode::Sec => {
                let a = stack.pop().ok_or("Stack underflow on Sec")?;
                stack.push(1.0 / a.cos());
            }
            Bytecode::Csc => {
                let a = stack.pop().ok_or("Stack underflow on Csc")?;
                stack.push(1.0 / a.sin());
            }
            Bytecode::Sinh => {
                let a = stack.pop().ok_or("Stack underflow on Sinh")?;
                stack.push(a.sinh());
            }
            Bytecode::Cosh => {
                let a = stack.pop().ok_or("Stack underflow on Cosh")?;
                stack.push(a.cosh());
            }
            Bytecode::Tanh => {
                let a = stack.pop().ok_or("Stack underflow on Tanh")?;
                stack.push(a.tanh());
            }
            Bytecode::Asinh => {
                let a = stack.pop().ok_or("Stack underflow on Asinh")?;
                stack.push(a.asinh());
            }
            Bytecode::Acosh => {
                let a = stack.pop().ok_or("Stack underflow on Acosh")?;
                stack.push(a.acosh());
            }
            Bytecode::Atanh => {
                let a = stack.pop().ok_or("Stack underflow on Atanh")?;
                stack.push(a.atanh());
            }
            Bytecode::Exp => {
                let a = stack.pop().ok_or("Stack underflow on Exp")?;
                stack.push(a.exp());
            }
            Bytecode::Log10 => {
                let a = stack.pop().ok_or("Stack underflow on Log10")?;
                stack.push(a.log10());
            }
            Bytecode::Log2 => {
                let a = stack.pop().ok_or("Stack underflow on Log2")?;
                stack.push(a.log2());
            }
            Bytecode::Fact => {
                let a = stack.pop().ok_or("Stack underflow on Fact")?;
                stack.push(factorial(a));
            }
            Bytecode::Floor => {
                let a = stack.pop().ok_or("Stack underflow on Floor")?;
                stack.push(a.floor());
            }
            Bytecode::Sub => {
                let b = stack.pop().ok_or("Stack underflow on Sub")?;
                let a = stack.pop().ok_or("Stack underflow on Sub")?;
                stack.push(a - b);
            }
            Bytecode::Log => {
                let a = stack.pop().ok_or("Stack underflow on Log")?;
                stack.push(a.ln());
            }
            Bytecode::Sqrt => {
                let a = stack.pop().ok_or("Stack underflow on Sqrt")?;
                stack.push(a.sqrt());
            }
            Bytecode::Abs => {
                let a = stack.pop().ok_or("Stack underflow on Abs")?;
                stack.push(a.abs());
            }
            Bytecode::Asin => {
                let a = stack.pop().ok_or("Stack underflow on Asin")?;
                stack.push(a.asin());
            }
            Bytecode::Acos => {
                let a = stack.pop().ok_or("Stack underflow on Acos")?;
                stack.push(a.acos());
            }
            Bytecode::Atan => {
                let a = stack.pop().ok_or("Stack underflow on Atan")?;
                stack.push(a.atan());
            }
            Bytecode::Acot => {
                let a = stack.pop().ok_or("Stack underflow on Acot")?;
                stack.push((1.0 / a).atan());
            }
            Bytecode::Asec => {
                let a = stack.pop().ok_or("Stack underflow on Asec")?;
                stack.push((1.0 / a).acos());
            }
            Bytecode::Acsc => {
                let a = stack.pop().ok_or("Stack underflow on Acsc")?;
                stack.push((1.0 / a).asin());
            }
            Bytecode::Pow => {
                let b = stack.pop().ok_or("Stack underflow on Pow")?;
                let a = stack.pop().ok_or("Stack underflow on Pow")?;
                stack.push(a.powf(b));
            }
            Bytecode::StoreVar(name) => {
                let val = stack.pop().ok_or("Stack underflow on StoreVar")?;
                vars.insert(name.clone(), val);
            }
            Bytecode::LoadVar(name) => {
                let val = vars.get(name).ok_or("Variable not found")?;
                stack.push(*val);
            }
            Bytecode::SumLoop { from, to, param, body } => {
                let mut from_stack = Vec::new();
                run_bytecode_with_functions_inner(from, user_functions, vars, &mut from_stack)?;
                let from_val = from_stack.pop().ok_or("No result on stack (from)")?;
                let mut to_stack = Vec::new();
                run_bytecode_with_functions_inner(to, user_functions, vars, &mut to_stack)?;
                let to_val = to_stack.pop().ok_or("No result on stack (to)")?;
                let from_i = from_val.ceil() as i64;
                let to_i = to_val.floor() as i64;
                let mut acc = 0.0;
                for i in from_i..=to_i {
                    let old = vars.insert(param.clone(), i as f64);
                    let mut body_stack = Vec::new();
                    run_bytecode_with_functions_inner(body, user_functions, vars, &mut body_stack)?;
                    let result = body_stack.pop().ok_or("No result on stack (body)")?;
                    acc += result;
                    if let Some(v) = old { vars.insert(param.clone(), v); } else { vars.remove(param); }
                }
                stack.push(acc);
            }
            Bytecode::ProductLoop { from, to, param, body } => {
                let mut from_stack = Vec::new();
                run_bytecode_with_functions_inner(from, user_functions, vars, &mut from_stack)?;
                let from_val = from_stack.pop().ok_or("No result on stack (from)")?;
                let mut to_stack = Vec::new();
                run_bytecode_with_functions_inner(to, user_functions, vars, &mut to_stack)?;
                let to_val = to_stack.pop().ok_or("No result on stack (to)")?;
                let from_i = from_val.ceil() as i64;
                let to_i = to_val.floor() as i64;
                let mut acc = 1.0;
                for i in from_i..=to_i {
                    let old = vars.insert(param.clone(), i as f64);
                    let mut body_stack = Vec::new();
                    run_bytecode_with_functions_inner(body, user_functions, vars, &mut body_stack)?;
                    let result = body_stack.pop().ok_or("No result on stack (body)")?;
                    acc *= result;
                    if let Some(v) = old { vars.insert(param.clone(), v); } else { vars.remove(param); }
                }
                stack.push(acc);
            }
        }
    }
    Ok(())
}
