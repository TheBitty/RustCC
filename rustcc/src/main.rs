mod parser {
    pub mod lexer;
    pub mod token;
    pub mod ast;
    pub mod parser;
}
mod analyzer;
mod codegen;
mod compiler;

use std::env;
use crate::compiler::Compiler;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        return Err("Usage: rustcc <source_file> <output_file>".to_string());
    }

    let compiler = Compiler::new(args[1].clone(), args[2].clone());
    compiler.compile()?;
    
    println!("Compilation successful!");
    Ok(())
}
