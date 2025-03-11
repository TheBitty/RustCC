use crate::parser::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::analyzer::SemanticAnalyzer;
use crate::codegen::CodeGenerator;
use std::fs;
use std::path::Path;

pub struct Compiler {
    source_file: String,
    output_file: String,
}

impl Compiler {
    pub fn new(source_file: String, output_file: String) -> Self {
        Compiler {
            source_file,
            output_file,
        }
    }
    
    pub fn compile(&self) -> Result<(), String> {
        println!("Compiling {} to {}", self.source_file, self.output_file);
        
        // Read the source file
        let source = match fs::read_to_string(&self.source_file) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to read source file: {}", e)),
        };
        
        println!("Source code:\n{}", source);
        
        // Step 1: Lexical Analysis
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens();
        
        println!("Tokens: {:?}", tokens);
        
        // Step 2: Parsing
        let mut parser = Parser::new(tokens.clone());
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                println!("Parsing error: {}", e);
                return Err(e);
            }
        };
        
        println!("AST: {:?}", ast);
        
        // Step 3: Semantic Analysis
        let mut analyzer = SemanticAnalyzer::new();
        if let Err(e) = analyzer.analyze(&ast) {
            println!("Semantic error: {}", e);
            return Err(e);
        }
        
        // Step 4: Code Generation
        let mut generator = CodeGenerator::new();
        let assembly = generator.generate(&ast);
        
        println!("Generated assembly:\n{}", assembly);
        
        // Write the output file
        let output_path = Path::new(&self.output_file);
        match fs::write(output_path, assembly) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to write output file: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_compiler_pipeline() {
        // Create a temporary C file with a simple main function
        let mut source_file = NamedTempFile::new().unwrap();
        write!(source_file, "int main() {{ return 42; }}").unwrap();
        
        // Create an output path
        let output_file = NamedTempFile::new().unwrap();
        
        // Create and run the compiler
        let compiler = Compiler::new(
            source_file.path().to_string_lossy().to_string(),
            output_file.path().to_string_lossy().to_string()
        );
        
        let result = compiler.compile();
        assert!(result.is_ok(), "Compilation failed: {:?}", result);
        
        // Check that the output file was created and contains assembly
        let output_contents = fs::read_to_string(output_file.path()).unwrap();
        assert!(output_contents.contains("_main:"), "Output should contain main function");
        assert!(output_contents.contains("mov $42, %rax"), "Output should contain return value");
    }
    
    #[test]
    fn test_compiler_with_variables() {
        // Test a program with variables and arithmetic
        let mut source_file = NamedTempFile::new().unwrap();
        write!(source_file, "
            int main() {{
                int x = 10;
                int y = 20;
                return x + y;
            }}
        ").unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let compiler = Compiler::new(
            source_file.path().to_string_lossy().to_string(),
            output_file.path().to_string_lossy().to_string()
        );
        
        let result = compiler.compile();
        assert!(result.is_ok(), "Compilation failed: {:?}", result);
        
        // Check that the output file contains variable operations
        let output_contents = fs::read_to_string(output_file.path()).unwrap();
        assert!(output_contents.contains("_main:"), "Output should contain main function");
        assert!(output_contents.contains("mov"), "Output should contain move instructions");
        assert!(output_contents.contains("add"), "Output should contain add instruction");
    }
}
