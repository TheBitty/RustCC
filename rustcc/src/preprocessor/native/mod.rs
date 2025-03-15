//! Native preprocessor implementation for RustCC
//!
//! This module provides a native implementation of the preprocessor for the RustCC compiler.
//! It handles preprocessor directives, macro expansion, and conditional compilation.

use std::collections::HashMap;
use std::fs;
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
    #[allow(dead_code)]
    pub(crate) include_depth: usize,
    /// Maximum include depth
    #[allow(dead_code)]
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

    /// Process a conditional block (#if, #ifdef, #ifndef)
    fn process_conditional_block(&mut self, lines: &[&str], start_idx: usize, file_name: &str) 
        -> Result<(String, usize), String> {
        let start_line = lines[start_idx].trim();
        let mut result = String::new();
        let mut i = start_idx;
        let mut depth = 1;
        let mut include_output = self.evaluate_conditional(start_line)?;
        let mut processed_elif = false;
        
        // Skip the initial directive line
        i += 1;
        
        // Process lines until we find the matching #endif
        while i < lines.len() && depth > 0 {
            let line = lines[i];
            let trimmed = line.trim();
            
            if trimmed.starts_with("#if") {
                // Nested conditional
                depth += 1;
                if include_output {
                    // If we're including the parent block, process this nested block
                    let (nested_result, new_i) = self.process_conditional_block(lines, i, file_name)?;
                    result.push_str(&nested_result);
                    i = new_i;
                    continue;
                }
            } else if trimmed.starts_with("#endif") {
                depth -= 1;
                if depth == 0 {
                    // This is the end of our conditional block
                    break;
                }
            } else if depth == 1 && (trimmed.starts_with("#else") || trimmed.starts_with("#elif")) {
                if trimmed.starts_with("#else") {
                    // Toggle inclusion for #else if we haven't processed an #elif that was true
                    if !processed_elif {
                        include_output = !include_output;
                    } else {
                        include_output = false;
                    }
                } else if trimmed.starts_with("#elif") {
                    // For #elif, check the condition if nothing before was true
                    if !processed_elif && !include_output {
                        include_output = self.evaluate_conditional(trimmed.replace("#elif", "#if").as_str())?;
                        if include_output {
                            processed_elif = true;
                        }
                    } else {
                        // If something before was true, don't include this block
                        include_output = false;
                    }
                }
                i += 1;
                continue;
            }
            
            // Include the line if the condition is met
            if include_output {
                if trimmed.starts_with('#') {
                    // Process directives inside the conditional
                    match self.process_directive(line, file_name) {
                        Ok(processed) => result.push_str(&processed),
                        Err(e) => return Err(e),
                    }
                } else {
                    // Expand macros in regular lines
                    let expanded = self.expand_macros(line);
                    result.push_str(&expanded);
                    result.push('\n');
                }
            }
            
            i += 1;
        }
        
        Ok((result, i))
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
        
        // Add the directory of the file to include paths temporarily for this file
        if let Some(dir) = path.parent() {
            if dir.exists() {
                if let Some(dir_str) = dir.to_str() {
                    self.add_include_dir(dir_str);
                }
            }
        }
        
        // Process the content using the file name
        let file_name = path.to_str().unwrap_or(file_path);
        self.preprocess_string(&content, file_name)
    }
    
    /// Preprocess a string
    fn preprocess_string(&mut self, content: &str, file_name: &str) -> Result<String, String> {
        // Split content into lines for line-by-line processing
        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::new();
        let mut i = 0;
        
        // Process each line
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            
            // Check if it's a preprocessor directive
            if trimmed.starts_with('#') {
                if trimmed.starts_with("#if") || trimmed.starts_with("#ifdef") || trimmed.starts_with("#ifndef") {
                    // This is the start of a conditional block
                    // Find the end of the block and process it
                    let (processed, new_i) = self.process_conditional_block(&lines, i, file_name)?;
                    result.push_str(&processed);
                    i = new_i;
                    continue;
                } else {
                    // Other directives (#include, #define, etc.)
                    match self.process_directive(line, file_name) {
                        Ok(processed) => result.push_str(&processed),
                        Err(e) => return Err(e),
                    }
                }
            } else {
                // Regular code line - expand macros
                let expanded = self.expand_macros(line);
                result.push_str(&expanded);
                result.push('\n');
            }
            
            i += 1;
        }
        
        Ok(result)
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