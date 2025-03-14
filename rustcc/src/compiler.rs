use crate::analyzer::SemanticAnalyzer;
use crate::codegen::CodeGenerator;
use crate::config::Config;
use crate::parser::lexer::Lexer;
use crate::parser::Parser;
use crate::preprocessor::{GccPreprocessor, Preprocessor, PreprocessorConfig};
use crate::transforms::obfuscation::{
    ControlFlowObfuscator, DeadCodeInserter, StringEncryptor, VariableObfuscator,
};
use crate::transforms::optimization::{ConstantFolder, DeadCodeEliminator, FunctionInliner};
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
    /// Configuration for the preprocessor
    preprocessor_config: Option<PreprocessorConfig>,
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
            preprocessor_config: None,
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
    pub fn with_language_dialect(mut self, dialect: LanguageDialect) -> Self {
        self.language_dialect = dialect;
        self
    }

    /// Set the configuration
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the preprocessor configuration
    pub fn with_preprocessor_config(mut self, config: PreprocessorConfig) -> Self {
        self.preprocessor_config = Some(config);
        self
    }

    /// Enable or disable verbose output
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

    /// Create a preprocessor configuration based on compiler settings
    fn create_preprocessor_config(&self) -> PreprocessorConfig {
        let mut config = match &self.preprocessor_config {
            Some(config) => config.clone(),
            None => PreprocessorConfig::default(),
        };
        
        // If we have a compiler config, extract preprocessor settings
        if let Some(compiler_config) = &self.config {
            // Add include paths from compiler config if they exist
            for path in &compiler_config.preprocessor.include_paths {
                config.include_paths.push(PathBuf::from(path));
            }
            
            // Add defines from compiler config
            for (name, value) in &compiler_config.preprocessor.defines {
                config.defines.insert(name.clone(), value.clone());
            }
            
            // Additional flags
            for flag in &compiler_config.preprocessor.additional_flags {
                config.gcc_flags.push(flag.clone());
            }
            
            // Keep comments setting
            config.keep_comments = compiler_config.preprocessor.keep_comments;
            
            // GCC path
            if let Some(path) = &compiler_config.preprocessor.gcc_path {
                config.gcc_path = Some(PathBuf::from(path));
            }
        }
        
        // Set dialect-specific flags
        match self.language_dialect {
            LanguageDialect::C89 => config.gcc_flags.push("-std=c89".to_string()),
            LanguageDialect::C99 => config.gcc_flags.push("-std=c99".to_string()),
            LanguageDialect::C11 => config.gcc_flags.push("-std=c11".to_string()),
            LanguageDialect::C17 => config.gcc_flags.push("-std=c17".to_string()),
            LanguageDialect::CPlusPlus => config.gcc_flags.push("-std=c++11".to_string()),
        }
        
        config
    }

    /// Compiles the source file to the output file
    pub fn compile(&self) -> Result<(), String> {
        if self.verbose {
            println!("Compiling {} to {}", self.source_file, self.output_file);
        }
        
        // Create preprocessor
        let preprocessor_config = self.create_preprocessor_config();
        let preprocessor = GccPreprocessor::with_config(preprocessor_config);
        
        // Check if GCC is available
        if !preprocessor.is_available() {
            return Err("GCC preprocessor is not available. Please install GCC or specify a valid path.".to_string());
        }
        
        // Preprocess the source file
        let source_path = Path::new(&self.source_file);
        
        if self.verbose {
            println!("Preprocessing source file...");
        }
        
        let preprocessed_path = preprocessor.preprocess_file(source_path)?;
        
        // Read the preprocessed file
        let source = fs::read_to_string(&preprocessed_path)
            .map_err(|e| format!("Failed to read preprocessed file: {}", e))?;
            
        if self.verbose {
            println!("Preprocessing completed");
        }

        // Lexical analysis
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens();

        if self.verbose {
            println!("Lexical analysis completed: {} tokens", tokens.len());
        }

        // Parsing
        let mut parser = Parser::new(tokens.clone());
        let mut ast = parser.parse()?;

        if self.verbose {
            println!("Parsing completed");
        }

        // Semantic analysis
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast)?;

        if self.verbose {
            println!("Semantic analysis completed");
        }

        // Apply optimizations based on the optimization level
        let opt_level = if let Some(config) = &self.config {
            config.get_optimization_level()
        } else {
            self.optimization_level
        };

        match opt_level {
            OptimizationLevel::None => {
                if self.verbose {
                    println!("No optimizations applied");
                }
            }
            OptimizationLevel::Basic => {
                if self.verbose {
                    println!("Applying basic optimizations");
                }
                // Apply constant folding
                let constant_folder = ConstantFolder;
                constant_folder.apply(&mut ast)?;
            }
            OptimizationLevel::Full => {
                if self.verbose {
                    println!("Applying full optimizations");
                }
                // Apply constant folding
                let constant_folder = ConstantFolder;
                constant_folder.apply(&mut ast)?;

                // Apply function inlining
                let inline_threshold = if let Some(config) = &self.config {
                    config.optimization.inline_threshold
                } else {
                    10 // Default threshold
                };
                let function_inliner = FunctionInliner::new(inline_threshold, false);
                function_inliner.apply(&mut ast)?;

                // Apply dead code elimination
                let dead_code_eliminator = DeadCodeEliminator;
                dead_code_eliminator.apply(&mut ast)?;
            }
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

        // Write the output to the file
        fs::write(&self.output_file, output)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        if self.verbose {
            println!("Compilation completed successfully");
        }

        Ok(())
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
