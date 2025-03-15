use crate::analyzer::SemanticAnalyzer;
use crate::codegen::CodeGenerator;
use crate::config::Config;
use crate::parser::lexer::Lexer;
use crate::parser::Parser;
use crate::preprocessor::{NativePreprocessor, Preprocessor};
use crate::transforms::obfuscation::{
    ControlFlowObfuscator, DeadCodeInserter, StringEncryptor, VariableObfuscator,
};
use crate::transforms::Transform;
use std::fs;
use std::path::{Path, PathBuf};

/// Main compiler struct that orchestrates the compilation process
pub struct Compiler {
    source_file: String,
    output_file: String,
    optimization_level: OptimizationLevel,
    obfuscation_level: ObfuscationLevel,
    language_dialect: LanguageDialect,
    config: Option<Config>,
    verbose: bool,
    /// Include paths for the preprocessor
    include_paths: Vec<PathBuf>,
    /// Macro definitions for the preprocessor
    defines: std::collections::HashMap<String, String>,
    /// Whether to only preprocess the source file
    preprocess_only: bool,
}

/// Optimization levels for the compiler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// No optimizations
    None,
    /// Basic optimizations like constant folding
    Basic,
    /// Full optimizations including dead code elimination and function inlining
    Full,
}

/// Obfuscation levels for the compiler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObfuscationLevel {
    /// No obfuscation
    None,
    /// Basic obfuscation like variable renaming and string encryption
    Basic,
    /// Aggressive obfuscation including control flow flattening and dead code insertion
    Aggressive,
}

/// Language dialect options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LanguageDialect {
    /// C89 standard
    C89,
    /// C99 standard
    C99,
    /// C11 standard
    C11,
    /// C17 standard
    C17,
    /// C++ standard
    CPlusPlus,
}

impl Compiler {
    /// Create a new compiler instance with default settings
    pub fn new(source_file: String, output_file: String) -> Self {
        Compiler {
            source_file,
            output_file,
            optimization_level: OptimizationLevel::None,
            obfuscation_level: ObfuscationLevel::None,
            language_dialect: LanguageDialect::C11,
            config: None,
            verbose: false,
            include_paths: Vec::new(),
            defines: std::collections::HashMap::new(),
            preprocess_only: false,
        }
    }

    /// Set the optimization level
    pub fn with_optimization(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }

    /// Set the obfuscation level
    pub fn with_obfuscation(mut self, level: ObfuscationLevel) -> Self {
        self.obfuscation_level = level;
        self
    }

    /// Set the language dialect
    #[allow(dead_code)]
    pub fn with_language_dialect(mut self, dialect: LanguageDialect) -> Self {
        self.language_dialect = dialect;
        self
    }

    /// Set the configuration
    #[allow(dead_code)]
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Add an include path for the preprocessor
    #[allow(dead_code)]
    pub fn add_include_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.include_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Add a macro definition for the preprocessor
    #[allow(dead_code)]
    pub fn add_define(mut self, name: &str, value: &str) -> Self {
        self.defines.insert(name.to_string(), value.to_string());
        self
    }

    /// Enable or disable verbose output
    #[allow(dead_code)]
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Load configuration from a file
    #[allow(dead_code)]
    pub fn with_config_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self, String> {
        let config = Config::from_file(path)?;
        self.config = Some(config);
        Ok(self)
    }

    /// Set whether to only preprocess the source file
    pub fn preprocess_only(mut self, preprocess_only: bool) -> Self {
        self.preprocess_only = preprocess_only;
        self
    }

    /// Compiles the source file to the output file
    pub fn compile(&self) -> Result<(), String> {
        if self.verbose {
            println!("Compiling {} to {}", self.source_file, self.output_file);
        }
        
        // Sanitize and validate file paths
        let source_path = self.sanitize_path(&self.source_file)?;
        let output_path = self.sanitize_path(&self.output_file)?;
        
        // Create preprocessor
        let mut preprocessor = NativePreprocessor::new();
        
        // Add include paths
        for path in &self.include_paths {
            preprocessor.add_include_dir(path.to_str().unwrap_or(""));
        }
        
        // Add defines from compiler config
        if let Some(config) = &self.config {
            for (name, value) in &config.preprocessor.defines {
                if let Some(val) = value {
                    preprocessor.add_define(name, val);
                }
            }
            
            // Add include paths from compiler config
            for path in &config.preprocessor.include_paths {
                preprocessor.add_include_dir(path);
            }
            
            // Set keep comments option
            preprocessor.keep_comments(config.preprocessor.keep_comments);
        }
        
        // Add defines from compiler instance
        for (name, value) in &self.defines {
            preprocessor.add_define(name, value);
        }
        
        // Preprocess the source file
        if self.verbose {
            println!("Preprocessing source file...");
        }
        
        let preprocessed_content = preprocessor.preprocess_file(source_path.to_str().unwrap_or(""))?;
        
        // Create a temporary file for the preprocessed content
        let preprocessed_path = PathBuf::from(format!("{}.i", self.source_file));
        fs::write(&preprocessed_path, &preprocessed_content)
            .map_err(|e| format!("Failed to write preprocessed file: {}", e))?;
        
        // If preprocess_only is true, just copy the preprocessed file to the output path and return
        if self.preprocess_only {
            if self.verbose {
                println!("Preprocessing only, copying to output file...");
            }
            
            // Copy the preprocessed file to the output path
            fs::copy(&preprocessed_path, &output_path)
                .map_err(|e| format!("Failed to copy preprocessed file to output: {}", e))?;
            return Ok(());
        }
        
        // Read the preprocessed file
        let mut source = fs::read_to_string(&preprocessed_path)
            .map_err(|e| format!("Failed to read preprocessed file: {}", e))?;
        // Ensure the source ends with a newline
        if !source.ends_with("\n") {
            source.push('\n');
        }
            
        if self.verbose {
            println!("Preprocessing completed. Processed source:\n{}", source);
        }

        // Lexical analysis
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens();

        if self.verbose {
            println!("Lexical analysis completed: {} tokens", tokens.len());
        }

        // Parsing
        let mut parser = Parser::new(tokens.clone());
        let mut ast = match parser.parse() {
            Ok(ast) => ast,
            Err(err) => return Err(format!("Parsing error: {}", err)),
        };

        if self.verbose {
            println!("Parsing completed");
        }

        // Semantic analysis
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast)?;

        if self.verbose {
            println!("Semantic analysis completed");
        }

        // Apply obfuscations based on the obfuscation level
        let obf_level = if let Some(config) = &self.config {
            config.get_obfuscation_level()
        } else {
            self.obfuscation_level
        };

        match obf_level {
            ObfuscationLevel::None => {
                if self.verbose {
                    println!("No obfuscations applied");
                }
            }
            ObfuscationLevel::Basic => {
                if self.verbose {
                    println!("Applying basic obfuscations");
                }
                // Apply variable renaming
                let variable_obfuscator = VariableObfuscator;
                variable_obfuscator.apply(&mut ast)?;

                // Apply string encryption
                let string_encryptor = StringEncryptor;
                string_encryptor.apply(&mut ast)?;
            }
            ObfuscationLevel::Aggressive => {
                if self.verbose {
                    println!("Applying aggressive obfuscations");
                }
                // Apply variable renaming
                let variable_obfuscator = VariableObfuscator;
                variable_obfuscator.apply(&mut ast)?;

                // Apply string encryption
                let string_encryptor = StringEncryptor;
                string_encryptor.apply(&mut ast)?;

                // Apply control flow flattening
                let control_flow_obfuscator = ControlFlowObfuscator;
                control_flow_obfuscator.apply(&mut ast)?;

                // Apply dead code insertion
                let _dead_code_ratio = if let Some(config) = &self.config {
                    config.obfuscation.dead_code_insertion_ratio
                } else {
                    0.2 // Default ratio
                };
                let dead_code_inserter = DeadCodeInserter;
                dead_code_inserter.apply(&mut ast)?;
            }
        }

        if self.verbose {
            println!("Code generation started");
        }

        // Code generation
        let mut generator = CodeGenerator::new();
        let output = generator.generate(&ast);

        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(&output_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }

        // Write the output to the file
        fs::write(&output_path, output)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        if self.verbose {
            println!("Compilation completed successfully");
        }

        Ok(())
    }

    /// Sanitize and validate a file path
    fn sanitize_path(&self, path: &str) -> Result<PathBuf, String> {
        // Convert to PathBuf to handle platform-specific path separators
        let path_buf = PathBuf::from(path);
        
        // Check if the path contains invalid characters
        #[cfg(windows)]
        {
            // Windows has more restricted path characters
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            let path_str = path_buf.to_string_lossy();
            
            for c in invalid_chars {
                if path_str.contains(c) {
                    return Err(format!("Path contains invalid character '{}': {}", c, path));
                }
            }
        }
        
        // For source files, check if they exist
        if path == &self.source_file && !path_buf.exists() {
            return Err(format!("Source file does not exist: {}", path));
        }
        
        // For output files, check if the parent directory exists or can be created
        if path == &self.output_file {
            if let Some(parent) = path_buf.parent() {
                if !parent.exists() {
                    // We'll create the directory later, just check if it's possible
                    if parent.to_string_lossy().len() > 255 {
                        return Err(format!("Output directory path is too long: {}", parent.display()));
                    }
                }
            }
        }
        
        // Check for path length limits
        if path_buf.to_string_lossy().len() > 255 {
            return Err(format!("Path is too long: {}", path));
        }
        
        Ok(path_buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

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
            output_file.path().to_string_lossy().to_string(),
        );

        let result = compiler.compile();
        assert!(result.is_ok(), "Compilation failed: {:?}", result);

        // Check that the output file was created and contains assembly
        let output_contents = fs::read_to_string(output_file.path()).unwrap();
        assert!(
            output_contents.contains("_main:"),
            "Output should contain main function"
        );
        assert!(
            output_contents.contains("mov $42, %rax"),
            "Output should contain return value"
        );
    }

    #[test]
    fn test_compiler_with_variables() {
        // Test a program with variables and arithmetic
        let mut source_file = NamedTempFile::new().unwrap();
        write!(
            source_file,
            "
            int main() {{
                int x = 10;
                int y = 20;
                return x + y;
            }}
        "
        )
        .unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let compiler = Compiler::new(
            source_file.path().to_string_lossy().to_string(),
            output_file.path().to_string_lossy().to_string(),
        );

        let result = compiler.compile();
        assert!(result.is_ok(), "Compilation failed: {:?}", result);

        // Check that the output file contains variable operations
        let output_contents = fs::read_to_string(output_file.path()).unwrap();
        assert!(
            output_contents.contains("_main:"),
            "Output should contain main function"
        );
        assert!(
            output_contents.contains("mov"),
            "Output should contain move instructions"
        );
        assert!(
            output_contents.contains("add"),
            "Output should contain add instruction"
        );
    }
}
