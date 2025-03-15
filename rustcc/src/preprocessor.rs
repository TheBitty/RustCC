//! Preprocessor module for RustCC
//! 
//! This module provides the preprocessor functionality for the RustCC compiler.
//! It handles preprocessor directives, macro expansion, and conditional compilation.

pub mod native;
pub mod tests;

/// Trait for preprocessors
pub trait Preprocessor {
    /// Check if the preprocessor is available
    fn is_available(&self) -> bool;
    
    /// Preprocess a file
    fn preprocess_file(&mut self, file_path: &str) -> Result<String, String>;
    
    /// Preprocess a string
    fn preprocess_string(&mut self, content: &str, file_name: &str) -> Result<String, String>;
}

// Re-export NativePreprocessor for convenience
pub use native::NativePreprocessor;

/// Create a new preprocessor
pub fn create_preprocessor() -> Box<dyn Preprocessor> {
    Box::new(native::NativePreprocessor::new())
} 