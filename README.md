# fmath

A modular, extensible math interpreter and compiler written in Rust. Supports advanced math expressions, user-defined functions, variables, and compiled sum/product constructs.

## Features
- Arithmetic expressions with variables
- User-defined functions
- Advanced math functions (trig, log, sqrt, etc.)
- Explicit variable declaration
- Sum and product constructs (in compiled mode)
- Bytecode compiler and interpreter

## Example Usage

**Example: Ramanujan's Pi Approximation**

```
def rama(n) = (4*n)!*(1103+26390*n)/((((n*4)!)^4)*396^(4*n))
var pi_conjugate = sum(from: 0,to: 20,para: x,rama(x))*2*2^(1/2)/9801
1/pi_conjugate
```

## Getting Started

1. **Build the project:**
   ```sh
   cargo build
   ```
2. **Run an example:**
   ```sh
   cargo run examples/function_example.mth
   # or compiled mode
   cargo run examples/function_example.mthc
   ```
3. **Compile a .mth file to .mthc:**
   ```sh
   cargo run -- examples/col.mth --compile-only
   ```

## Project Structure
- `src/` — Source code (lexer, parser, ast, compiler, bytecode, interpreter, main)
- `examples/` — Example math scripts

## License
MIT
