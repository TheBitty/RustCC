//! Native preprocessor implementation for RustCC
//!
//! This module provides a native implementation of the preprocessor for the RustCC compiler.
//! It handles preprocessor directives, macro expansion, and conditional compilation.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// Local modules
pub mod macros;
pub mod directives;
pub mod includes;
pub mod conditionals;

// Import functionality from submodules
use crate::preprocessor::Preprocessor;

/// Native preprocessor implementation
pub struct NativePreprocessor {
    /// Macro definitions
    pub(crate) defines: HashMap<String, String>,
    /// Include directories
    pub(crate) include_dirs: Vec<PathBuf>,
    /// Whether to keep comments in the preprocessed output
    pub(crate) keep_comments: bool,
    /// Current include depth (to prevent infinite recursion)
    pub(crate) include_depth: usize,
    /// Maximum include depth
    pub(crate) max_include_depth: usize,
}

impl NativePreprocessor {
    /// Create a new native preprocessor
    pub fn new() -> Self {
        let mut preprocessor = NativePreprocessor {
            defines: HashMap::new(),
            include_dirs: Vec::new(),
            keep_comments: false,
            include_depth: 0,
            max_include_depth: 64,
        };
        
        // Add standard predefined macros
        preprocessor.defines.insert("__STDC__".to_string(), "1".to_string());
        preprocessor.defines.insert("__STDC_VERSION__".to_string(), "201710L".to_string());
        
        // Set up platform-specific includes
        preprocessor.setup_platform_includes();
        
        preprocessor
    }

    /// Add a define to the preprocessor
    pub fn add_define(&mut self, name: &str, value: &str) {
        self.defines.insert(name.to_string(), value.to_string());
    }

    /// Add an include directory
    pub fn add_include_dir(&mut self, dir: &str) {
        let path = Path::new(dir).to_path_buf();
        if path.exists() && path.is_dir() {
            self.include_dirs.push(path);
        }
    }

    /// Set whether to keep comments in the preprocessed output
    pub fn keep_comments(&mut self, keep: bool) {
        self.keep_comments = keep;
    }

    /// Set the maximum include depth
    pub fn set_max_include_depth(&mut self, depth: usize) {
        self.max_include_depth = depth;
    }
    
    /// Set up platform-specific include directories
    pub(crate) fn setup_platform_includes(&mut self) {
        // Add standard include directories based on platform
        #[cfg(target_os = "linux")]
        {
            self.add_include_dir("/usr/include");
            self.add_include_dir("/usr/local/include");
        }
        
        #[cfg(target_os = "macos")]
        {
            self.add_include_dir("/usr/include");
            self.add_include_dir("/usr/local/include");
        }
        
        #[cfg(target_os = "windows")]
        {
            // Add Windows-specific include directories
        }
        
        // Add current directory
        self.add_include_dir(".");
    }
}

impl Preprocessor for NativePreprocessor {
    /// Check if the preprocessor is available
    fn is_available(&self) -> bool {
        true
    }
    
    /// Preprocess a file
    fn preprocess_file(&mut self, file_path: &str) -> Result<String, String> {
        // Read the file content
        let path = Path::new(file_path);
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;
        
        // Get the directory of the file
        let current_dir = path.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_str()
            .unwrap_or(".");
        
        // For now, just return the content as-is (placeholder)
        // In a real implementation, we would process the content
        Ok(content)
    }
    
    /// Preprocess a string
    fn preprocess_string(&mut self, content: &str, file_name: &str) -> Result<String, String> {
        // For now, just return the content as-is (placeholder)
        // In a real implementation, we would process the content
        Ok(content.to_string())
    }
}

impl Default for NativePreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_define() {
        let mut preprocessor = NativePreprocessor::new();
        preprocessor.defines.insert("VERSION".to_string(), "1.0".to_string());
        
        let source = "#ifdef VERSION\nconst char* version = VERSION;\n#endif";
        let result = preprocessor.preprocess_string(source, "test.c").unwrap();
        
        assert!(result.contains("const char* version = 1.0"));
    }

    #[test]
    fn test_conditional_compilation() {
        let mut preprocessor = NativePreprocessor::new();
        preprocessor.defines.insert("DEBUG".to_string(), "1".to_string());
        
        let source = "#ifdef DEBUG\nconst char* mode = \"debug\";\n#else\nconst char* mode = \"release\";\n#endif";
        let result = preprocessor.preprocess_string(source, "test.c").unwrap();
        
        assert!(result.contains("const char* mode = \"debug\""));
        assert!(!result.contains("const char* mode = \"release\""));
    }
} 