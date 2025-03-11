use crate::parser::lexer::Lexer;
use crate::parser::token::TokenType;
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
        // Read the source file
        let source = match fs::read_to_string(&self.source_file) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to read source file: {}", e)),
        };
        
        // Step 1: Lexical Analysis
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens().clone();
        
        // Basic validation: check if the file at least has a main function
        let main_fn_exists = tokens.iter().enumerate().any(|(i, token)| {
            if i + 2 < tokens.len() {
                token.token_type == TokenType::Int &&
                tokens[i + 1].token_type == TokenType::Identifier && 
                tokens[i + 1].lexeme == "main" &&
                tokens[i + 2].token_type == TokenType::LeftParen
            } else {
                false
            }
        });
        
        if !main_fn_exists {
            return Err("No main function found in the source file".to_string());
        }
        
        // Placeholder for further compilation steps
        // TODO: Implement parsing
        // TODO: Implement semantic analysis
        // TODO: Implement code generation
        
        // For now, just write a placeholder output file
        let output_path = Path::new(&self.output_file);
        match fs::write(output_path, "// Generated code will go here") {
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
        write!(source_file, "int main() {{ return 0; }}").unwrap();
        
        // Create an output path
        let output_file = NamedTempFile::new().unwrap();
        
        // Create and run the compiler
        let compiler = Compiler::new(
            source_file.path().to_string_lossy().to_string(),
            output_file.path().to_string_lossy().to_string()
        );
        
        // Read the source file content for debugging
        let source_content = fs::read_to_string(source_file.path()).unwrap();
        println!("Source file content: {:?}", source_content);
        
        let result = compiler.compile();
        
        // If compilation fails, print the tokens for debugging
        if result.is_err() {
            let mut lexer = Lexer::new(source_content);
            let tokens = lexer.scan_tokens();
            println!("Tokens: {:?}", tokens);
        }
        
        assert!(result.is_ok(), "Compilation failed: {:?}", result);
        
        // Check that the output file was created
        let output_contents = fs::read_to_string(output_file.path()).unwrap();
        assert!(!output_contents.is_empty(), "Output file is empty");
    }
    
    #[test]
    fn test_compiler_with_invalid_input() {
        // Create a temporary C file without a main function
        let mut source_file = NamedTempFile::new().unwrap();
        write!(source_file, "int add(int a, int b) {{ return a + b; }}").unwrap();
        
        // Create an output path
        let output_file = NamedTempFile::new().unwrap();
        
        // Create and run the compiler
        let compiler = Compiler::new(
            source_file.path().to_string_lossy().to_string(),
            output_file.path().to_string_lossy().to_string()
        );
        
        let result = compiler.compile();
        assert!(result.is_err(), "Compilation should have failed but succeeded");
        assert!(result.unwrap_err().contains("No main function found"), 
                "Error message should mention the missing main function");
    }
}
