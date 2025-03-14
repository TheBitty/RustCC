use rustcc::compiler::{Compiler, OptimizationLevel, ObfuscationLevel};
use std::path::Path;

fn main() -> Result<(), String> {
    // Set up example test files
    let test_files = [
        ("tests/simple.c", "Simple variable addition"),
        ("tests/test.c", "Basic addition"),
        ("tests/control_flow.c", "Control flow example"),
    ];
    
    for (test_file, description) in &test_files {
        if !Path::new(test_file).exists() {
            println!("Test file {} not found, skipping", test_file);
            continue;
        }
        
        println!("\n=== Compiling {} ({}) ===\n", test_file, description);
        
        // 1. Compile with no optimization or obfuscation
        let output_file = format!("{}_plain.s", test_file.replace("/", "_"));
        let compiler = Compiler::new(test_file.to_string(), output_file.clone())
            .with_optimization(OptimizationLevel::None)
            .with_obfuscation(ObfuscationLevel::None);
        
        if let Err(e) = compiler.compile() {
            println!("❌ Plain compilation failed: {}", e);
            continue;
        }
        
        println!("✅ Plain compilation successful: {}", output_file);
        
        // 2. Compile with optimization
        let output_file = format!("{}_optimized.s", test_file.replace("/", "_"));
        let compiler = Compiler::new(test_file.to_string(), output_file.clone())
            .with_optimization(OptimizationLevel::Full)
            .with_obfuscation(ObfuscationLevel::None);
        
        if let Err(e) = compiler.compile() {
            println!("❌ Optimized compilation failed: {}", e);
            continue;
        }
        
        println!("✅ Optimized compilation successful: {}", output_file);
        
        // 3. Compile with obfuscation
        let output_file = format!("{}_obfuscated.s", test_file.replace("/", "_"));
        let compiler = Compiler::new(test_file.to_string(), output_file.clone())
            .with_optimization(OptimizationLevel::None)
            .with_obfuscation(ObfuscationLevel::Aggressive);
        
        if let Err(e) = compiler.compile() {
            println!("❌ Obfuscated compilation failed: {}", e);
            continue;
        }
        
        println!("✅ Obfuscated compilation successful: {}", output_file);
        
        // 4. Compile with both optimization and obfuscation
        let output_file = format!("{}_opt_obf.s", test_file.replace("/", "_"));
        let compiler = Compiler::new(test_file.to_string(), output_file.clone())
            .with_optimization(OptimizationLevel::Full)
            .with_obfuscation(ObfuscationLevel::Aggressive);
        
        if let Err(e) = compiler.compile() {
            println!("❌ Optimized+Obfuscated compilation failed: {}", e);
            continue;
        }
        
        println!("✅ Optimized+Obfuscated compilation successful: {}", output_file);
    }
    
    Ok(())
} 