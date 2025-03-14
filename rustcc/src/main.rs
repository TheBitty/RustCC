mod parser {
    pub mod lexer;
    pub mod token;
    pub mod ast;
    pub mod parser;
}
mod analyzer;
mod codegen;
mod transforms;
mod compiler;

use std::env;
use crate::compiler::{Compiler, OptimizationLevel, ObfuscationLevel};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        return Err("Usage: rustcc <source_file> <output_file> [options]\nOptions:\n  -O0, -O1, -O2: Optimization level\n  -obf0, -obf1, -obf2: Obfuscation level".to_string());
    }

    let source_file = &args[1];
    let output_file = &args[2];

    // Parse command line options
    let mut opt_level = OptimizationLevel::None;
    let mut obf_level = ObfuscationLevel::None;

    for arg in args.iter().skip(3) {
        match arg.as_str() {
            "-O0" => opt_level = OptimizationLevel::None,
            "-O1" => opt_level = OptimizationLevel::Basic,
            "-O2" => opt_level = OptimizationLevel::Full,
            "-obf0" => obf_level = ObfuscationLevel::None,
            "-obf1" => obf_level = ObfuscationLevel::Basic,
            "-obf2" => obf_level = ObfuscationLevel::Aggressive,
            _ => return Err(format!("Unknown option: {}", arg)),
        }
    }

    // Create and configure the compiler
    let compiler = Compiler::new(source_file.clone(), output_file.clone())
        .with_optimization(opt_level)
        .with_obfuscation(obf_level);

    // Run the compiler
    match compiler.compile() {
        Ok(_) => {
            println!("Compilation successful!");
            println!("Options: Optimization={:?}, Obfuscation={:?}", opt_level, obf_level);
            Ok(())
        },
        Err(e) => Err(e),
    }
}
