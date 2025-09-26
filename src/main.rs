use std::fs::File;
use std::io::Write;
// Add bincode for compact binary serialization
use bincode::encode_to_vec;
use std::path::Path;
use bincode::decode_from_slice;

mod lexer;
mod parser;
mod ast;
mod interpreter;
mod bytecode;
mod compiler;

/// Entry point for the math interpreter CLI.
/// This main function is minimal and delegates all logic to modules, making it easy to reuse the core for GUI or graphing.
use std::fs;
use std::env;

fn main() -> Result<(), i32> {
	let args: Vec<String> = env::args().collect();
	let mut base_path = String::from("examples/math_example");
	for arg in &args[1..] {
		if arg != "--compile-only" {
			base_path = arg.clone();
		}
	}

	let (mthc_path, mth_src_path, run_mthc_direct) = if base_path.ends_with(".mthc") {
		(base_path.clone(), base_path.trim_end_matches(".mthc").to_string() + ".mth", true)
	} else if base_path.ends_with(".mth") {
		(base_path.trim_end_matches(".mth").to_string() + ".mthc", base_path.clone(), false)
	} else {
		(format!("{}.mthc", base_path), format!("{}.mth", base_path), false)
	};

	if run_mthc_direct && Path::new(&mthc_path).exists() {
		// Always run .mthc file if specified
		let bytes = fs::read(&mthc_path).expect("Failed to read .mthc file");
		let program = decode_from_slice::<Vec<bytecode::Bytecode>, _>(&bytes, bincode::config::standard())
			.expect("Failed to decode bytecode").0;
		// Load function definitions from .mth file if available
		let mut user_functions = std::collections::HashMap::new();
		if Path::new(&mth_src_path).exists() {
			let input = fs::read_to_string(&mth_src_path).expect("Failed to read .mth file");
			let lines = lexer::tokenize(&input);
			let (_, uf) = parser::parse(lines);
			user_functions = uf;
		}
	// [DEBUG] Compiled bytecode output removed
		return match interpreter::run_bytecode_with_functions(&program, &user_functions) {
			   Ok(result) => {
				   println!("Result: {}", result);
				   Ok(())
			   },
			   Err(e) => {
				   eprintln!("Error: {}", e);
				   Err(1)
			   }
		   };
	}

	if Path::new(&mth_src_path).exists() {
		// Only compile .mth to .mthc, do not run .mth source
		let input = fs::read_to_string(&mth_src_path).expect("Failed to read .mth file");
		let lines = lexer::tokenize(&input);
	let (ast, _user_functions) = parser::parse(lines);
		let mut program = Vec::new();
		compiler::compile(&ast, &mut program);
		// Serialize bytecode to compact binary file
		let encoded = encode_to_vec(&program, bincode::config::standard()).expect("Failed to serialize bytecode");
		let mut file = File::create(&mthc_path).expect("Failed to create file");
		file.write_all(&encoded).expect("Failed to write file");
		println!("File saved to {}", mthc_path);
		return Ok(());
	}

	if Path::new(&mthc_path).exists() {
		// Load and decode bytecode from .mthc file and run it
		let bytes = fs::read(&mthc_path).expect("Failed to read .mthc file");
		let program = decode_from_slice::<Vec<bytecode::Bytecode>, _>(&bytes, bincode::config::standard())
			.expect("Failed to decode bytecode").0;
		// Load function definitions from .mth file if available
		let mut user_functions = std::collections::HashMap::new();
		if Path::new(&mth_src_path).exists() {
			let input = fs::read_to_string(&mth_src_path).expect("Failed to read .mth file");
			let lines = lexer::tokenize(&input);
			let (_, uf) = parser::parse(lines);
			user_functions = uf;
		}
	// [DEBUG] Compiled bytecode output removed
		return match interpreter::run_bytecode_with_functions(&program, &user_functions) {
			Ok(result) => {
				println!("Result: {}", result);
				Ok(())
			},
			Err(e) => {
				eprintln!("Error: {}", e);
				Err(1)
			}
		};
	} else {
		panic!("Neither {} nor {} found", mthc_path, mth_src_path);
	}
}

// Recursively collect user-defined functions from the AST