// Bytecode instructions for the math compiler/interpreter
use bincode::{Encode, Decode};
#[derive(Debug, Clone, Encode, Decode)]
pub enum Bytecode {
    PushNumber(f64),
    Add,
    Sub,
    Mul,
    Div,
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
    StoreVar(String),
    LoadVar(String),
    CallUserFunction(String),
    SumLoop {
        from: Box<Program>,
        to: Box<Program>,
        param: String,
        body: Box<Program>,
    },
    ProductLoop {
        from: Box<Program>,
        to: Box<Program>,
        param: String,
        body: Box<Program>,
    },
    // Add more as needed
}

// A bytecode program is just a sequence of instructions
pub type Program = Vec<Bytecode>;
