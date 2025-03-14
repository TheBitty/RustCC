pub mod parser {
    pub mod lexer;
    pub mod token;
    pub mod ast;
    pub mod parser;
}
pub mod analyzer;
pub mod codegen;
pub mod transforms;
pub mod compiler;

// Re-export key components
pub use compiler::{Compiler, OptimizationLevel, ObfuscationLevel};
pub use codegen::{CodeGenerator, Backend};
pub use transforms::Transform;

/// Compiles a C file to assembly with the given options
/// 
/// # Arguments
/// 
/// * `source_file` - Path to the C source file
/// * `output_file` - Path where the assembly output should be written
/// * `opt_level` - Optimization level
/// * `obf_level` - Obfuscation level
/// 
/// # Returns
/// 
/// Result indicating success or an error message
pub fn compile(
    source_file: &str,
    output_file: &str,
    opt_level: OptimizationLevel,
    obf_level: ObfuscationLevel,
) -> Result<(), String> {
    Compiler::new(source_file.to_string(), output_file.to_string())
        .with_optimization(opt_level)
        .with_obfuscation(obf_level)
        .compile()
} 