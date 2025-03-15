mod analyzer;
mod codegen;
mod compiler;
mod config;
mod parser;
mod preprocessor;
mod transforms;

use crate::compiler::{Compiler, ObfuscationLevel, OptimizationLevel};
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err("Usage: rustcc <source_file> [options]\nOptions:\n  -o <file>: Output file\n  -O0, -O1, -O2: Optimization level\n  -obf0, -obf1, -obf2: Obfuscation level\n  -I<dir>: Add directory to include search path\n  -E: Preprocess only".to_string());
    }

    let mut source_file = String::new();
    let mut output_file = String::new();
    let mut opt_level = OptimizationLevel::None;
    let mut obf_level = ObfuscationLevel::None;
    let mut include_paths = Vec::new();
    let mut preprocess_only = false;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        
        if arg.starts_with("-") {
            // Handle options
            if arg.starts_with("-I") {
                // Handle include path
                let path = if arg.len() > 2 {
                    // Path is part of the argument (e.g., -I/usr/include)
                    arg[2..].to_string()
                } else if i + 1 < args.len() {
                    // Path is the next argument (e.g., -I /usr/include)
                    i += 1;
                    args[i].clone()
                } else {
                    return Err("Missing path after -I option".to_string());
                };
                
                include_paths.push(PathBuf::from(path));
            } else if arg == "-o" {
                // Handle output file
                if i + 1 < args.len() {
                    i += 1;
                    output_file = args[i].clone();
                } else {
                    return Err("Missing file after -o option".to_string());
                }
            } else {
                // Handle other options
                match arg.as_str() {
                    "-O0" => opt_level = OptimizationLevel::None,
                    "-O1" => opt_level = OptimizationLevel::Basic,
                    "-O2" => opt_level = OptimizationLevel::Full,
                    "-obf0" => obf_level = ObfuscationLevel::None,
                    "-obf1" => obf_level = ObfuscationLevel::Basic,
                    "-obf2" => obf_level = ObfuscationLevel::Aggressive,
                    "-E" => preprocess_only = true,
                    _ => return Err(format!("Unknown option: {}", arg)),
                }
            }
        } else {
            // This is the source file
            if source_file.is_empty() {
                source_file = arg.clone();
            } else {
                return Err(format!("Unexpected argument: {}", arg));
            }
        }
        
        i += 1;
    }
    
    // Check if source file was provided
    if source_file.is_empty() {
        return Err("No source file provided".to_string());
    }
    
    // If no output file was specified, use the source file name with .o extension
    if output_file.is_empty() {
        let source_path = PathBuf::from(&source_file);
        let file_stem = source_path.file_stem().unwrap_or_default().to_string_lossy();
        
        if preprocess_only {
            // For preprocessing only, use .i extension
            output_file = format!("{}.i", file_stem);
        } else {
            // For compilation, use .o extension
            output_file = format!("{}.o", file_stem);
        }
    }

    // Create and configure the compiler
    let mut compiler = Compiler::new(source_file.clone(), output_file.clone())
        .with_optimization(opt_level)
        .with_obfuscation(obf_level);
    
    // Add include paths
    for path in include_paths {
        compiler = compiler.add_include_path(path);
    }
    
    // Set preprocess only mode if requested
    if preprocess_only {
        compiler = compiler.preprocess_only(true);
    }

    // Run the compiler
    match compiler.compile() {
        Ok(_) => {
            println!("Compilation successful!");
            println!(
                "Options: Optimization={:?}, Obfuscation={:?}",
                opt_level, obf_level
            );
            Ok(())
        }
        Err(e) => Err(e),
    }
}
