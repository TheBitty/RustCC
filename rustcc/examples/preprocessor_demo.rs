use rustcc::preprocessor::{GccPreprocessor, PreprocessorConfig, Preprocessor};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Example complex C code with various preprocessor directives
static COMPLEX_C_CODE: &str = r#"
#define BUFFER_SIZE 1024
#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define STRING_LITERAL(x) #x
#define CONCAT(a, b) a##b

// Conditional compilation example
#ifdef DEBUG
    #define LOG(msg) printf("DEBUG: %s\n", msg)
#else
    #define LOG(msg) /* nothing */
#endif

#include <stdio.h>

// Function-like macro with variable arguments
#define debug_print(fmt, ...) printf(fmt, __VA_ARGS__)

// Complex token-pasting example
#define DECLARE_GETTER(type, name) type get_##name() { return name; }

int main() {
    // Using the MAX macro
    int max_value = MAX(10, 20);
    
    // Using buffer size constant
    char buffer[BUFFER_SIZE];
    
    // String stringification - quote the argument to avoid parsing issues
    const char* str = STRING_LITERAL("Hello, World!");
    
    // Token pasting
    int CONCAT(value, 1) = 42;
    
    // Using the debug_print macro
    debug_print("Value: %d\n", max_value);
    
    // Using the LOG macro (controlled by DEBUG define)
    LOG("This is a debug message");
    
    return 0;
}
"#;

fn main() -> std::io::Result<()> {
    println!("RustCC Preprocessor Demonstration");
    println!("=================================\n");

    // Create a temporary file with our complex C code
    let temp_dir = tempfile::tempdir()?;
    let input_file = temp_dir.path().join("complex.c");
    fs::write(&input_file, COMPLEX_C_CODE)?;

    println!("Original C code:");
    println!("{}", COMPLEX_C_CODE);
    println!("\n=================================\n");

    // Create a preprocessor with default settings
    let default_preprocessor = GccPreprocessor::new();
    if !default_preprocessor.is_available() {
        println!("GCC preprocessor is not available on this system");
        return Ok(());
    }

    // Preprocess with default settings
    println!("Preprocessed with default settings (no DEBUG defined):");
    match default_preprocessor.preprocess_file(&input_file) {
        Ok(output_path) => {
            let preprocessed = fs::read_to_string(output_path)?;
            println!("{}", preprocessed);
        }
        Err(e) => {
            println!("Error preprocessing file: {}", e);
            return Ok(());
        }
    }

    println!("\n=================================\n");

    // Create a custom preprocessor configuration
    let mut custom_config = PreprocessorConfig::default();
    
    // Define DEBUG macro
    let mut defines = HashMap::new();
    defines.insert("DEBUG".to_string(), None);
    custom_config.defines = defines;
    
    // Keep comments in the output
    custom_config.keep_comments = true;
    
    // Create a preprocessor with our custom configuration
    let custom_preprocessor = GccPreprocessor::with_config(custom_config);

    // Preprocess with custom settings
    println!("Preprocessed with DEBUG defined and comments preserved:");
    match custom_preprocessor.preprocess_file(&input_file) {
        Ok(output_path) => {
            let preprocessed = fs::read_to_string(output_path)?;
            println!("{}", preprocessed);
        }
        Err(e) => {
            println!("Error preprocessing file: {}", e);
            return Ok(());
        }
    }

    println!("\n=================================\n");

    // Demonstrate string-based preprocessing
    println!("Direct string preprocessing example:");
    let simple_code = r#"
    #define VALUE 42
    int main() {
        return VALUE;
    }
    "#;
    
    match default_preprocessor.preprocess_string(simple_code) {
        Ok(preprocessed) => {
            println!("Original:\n{}", simple_code);
            println!("\nPreprocessed:\n{}", preprocessed);
        }
        Err(e) => {
            println!("Error preprocessing string: {}", e);
        }
    }

    println!("\n=================================\n");
    println!("Demonstration complete!");

    Ok(())
} 