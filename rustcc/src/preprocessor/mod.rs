//! Preprocessor module for RustCC
//!
//! This module provides functionality to preprocess C source files before parsing.
//! It integrates with GCC's preprocessor to handle all standard C preprocessor directives.

mod gcc;
#[cfg(test)]
mod tests;

pub use gcc::{GccPreprocessor, PreprocessorConfig};

/// Trait defining the interface for preprocessors
pub trait Preprocessor {
    /// Check if the preprocessor is available on the system
    fn is_available(&self) -> bool;
    
    /// Preprocess a C source file
    /// 
    /// # Arguments
    ///
    /// * `input_path` - Path to the C source file to preprocess
    ///
    /// # Returns
    ///
    /// Result with the path to the preprocessed file, or an error if preprocessing fails
    fn preprocess_file(&self, input_path: &std::path::Path) -> Result<std::path::PathBuf, String>;
    
    /// Preprocess C source code from a string
    ///
    /// # Arguments
    ///
    /// * `source` - C source code to preprocess
    ///
    /// # Returns
    ///
    /// Result with the preprocessed code as a string, or an error if preprocessing fails
    fn preprocess_string(&self, source: &str) -> Result<String, String>;
} 