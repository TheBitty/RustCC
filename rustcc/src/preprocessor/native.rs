use crate::preprocessor::Preprocessor;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// A native Rust implementation of the C preprocessor
pub struct NativePreprocessor {
    /// Include directories to search
    include_dirs: Vec<PathBuf>,
    /// Predefined macros
    defines: HashMap<String, String>,
    /// Whether to keep preprocessor directives as comments
    keep_comments: bool,
}

impl NativePreprocessor {
    /// Create a new native preprocessor
    pub fn new() -> Self {
        NativePreprocessor {
            include_dirs: vec![PathBuf::from("/usr/include"), PathBuf::from("/usr/local/include")],
            defines: HashMap::new(),
            keep_comments: false,
        }
    }

    /// Add an include directory
    pub fn add_include_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.include_dirs.push(dir.as_ref().to_path_buf());
        self
    }

    /// Add multiple include directories
    pub fn add_include_dirs<P: AsRef<Path>>(mut self, dirs: Vec<P>) -> Self {
        for dir in dirs {
            self.include_dirs.push(dir.as_ref().to_path_buf());
        }
        self
    }

    /// Add a define
    pub fn add_define(mut self, name: &str, value: &str) -> Self {
        self.defines.insert(name.to_string(), value.to_string());
        self
    }

    /// Set whether to keep comments
    pub fn keep_comments(mut self, keep: bool) -> Self {
        self.keep_comments = keep;
        self
    }

    /// Process a #include directive
    fn process_include(&self, line: &str, current_file_dir: &Path) -> Result<String, String> {
        // Extract the file path from the #include directive
        let file_path = if line.contains('<') && line.contains('>') {
            // System include path: #include <file>
            let start = line.find('<').unwrap() + 1;
            let end = line.find('>').unwrap();
            &line[start..end]
        } else if line.contains('"') {
            // Local include path: #include "file"
            let start = line.find('"').unwrap() + 1;
            let end = line.rfind('"').unwrap();
            &line[start..end]
        } else {
            return Err(format!("Invalid #include directive: {}", line));
        };

        // Try to find the file in the include directories or relative to the current file
        let mut file_content = String::new();
        let mut found = false;

        // First try relative to current file (for #include "file")
        if line.contains('"') {
            let local_path = current_file_dir.join(file_path);
            if local_path.exists() {
                file_content = fs::read_to_string(&local_path)
                    .map_err(|e| format!("Failed to read include file {}: {}", local_path.display(), e))?;
                found = true;
            }
        }

        // If not found, try include directories
        if !found {
            for dir in &self.include_dirs {
                let include_path = dir.join(file_path);
                if include_path.exists() {
                    file_content = fs::read_to_string(&include_path)
                        .map_err(|e| format!("Failed to read include file {}: {}", include_path.display(), e))?;
                    found = true;
                    break;
                }
            }
        }

        if !found {
            return Err(format!("Include file not found: {}", file_path));
        }

        // Process the included file
        Ok(self.preprocess_content(&file_content, current_file_dir)?)
    }

    /// Process a #define directive
    fn process_define(&mut self, line: &str) -> Result<(), String> {
        // Remove the #define part
        let define_part = line.trim_start_matches("#define").trim();
        
        // Split by first space to get name and value
        if let Some(space_pos) = define_part.find(char::is_whitespace) {
            let name = define_part[..space_pos].trim().to_string();
            let value = define_part[space_pos..].trim().to_string();
            self.defines.insert(name, value);
        } else {
            // Simple define without value
            self.defines.insert(define_part.to_string(), "1".to_string());
        }
        
        Ok(())
    }

    /// Process an #ifdef or #ifndef directive
    fn evaluate_conditional(&self, line: &str, is_ifndef: bool) -> bool {
        let directive = if is_ifndef { "#ifndef" } else { "#ifdef" };
        let macro_name = line.trim_start_matches(directive).trim();
        
        // Return true if the macro is defined (or not defined for ifndef)
        self.defines.contains_key(macro_name) != is_ifndef
    }

    /// Preprocess the content of a file
    fn preprocess_content(&self, content: &str, current_dir: &Path) -> Result<String, String> {
        let mut result = String::new();
        let mut in_comment = false;
        let mut skip_until_endif = false;
        let mut skip_until_else = false;
        let mut conditional_stack: Vec<bool> = Vec::new();

        // Create a mutable copy of self to handle #define directives
        let mut preprocessor = NativePreprocessor {
            include_dirs: self.include_dirs.clone(),
            defines: self.defines.clone(),
            keep_comments: self.keep_comments,
        };
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                result.push('\n');
                continue;
            }
            
            // Handle multi-line comments
            if in_comment {
                if trimmed.contains("*/") {
                    in_comment = false;
                    if self.keep_comments {
                        result.push_str(line);
                        result.push('\n');
                    }
                } else if self.keep_comments {
                    result.push_str(line);
                    result.push('\n');
                }
                continue;
            }
            
            // Check for comment start
            if trimmed.contains("/*") && !trimmed.contains("*/") {
                in_comment = true;
                if self.keep_comments {
                    result.push_str(line);
                    result.push('\n');
                }
                continue;
            }
            
            // Skip single-line comments
            if trimmed.starts_with("//") {
                if self.keep_comments {
                    result.push_str(line);
                    result.push('\n');
                }
                continue;
            }
            
            // Process preprocessor directives
            if trimmed.starts_with('#') {
                // Handle conditional compilation
                if trimmed.starts_with("#ifdef") || trimmed.starts_with("#ifndef") {
                    let is_ifndef = trimmed.starts_with("#ifndef");
                    let condition_met = preprocessor.evaluate_conditional(trimmed, is_ifndef);
                    
                    conditional_stack.push(condition_met);
                    
                    if !condition_met {
                        skip_until_endif = true;
                        skip_until_else = true;
                    }
                    
                    continue;
                } else if trimmed.starts_with("#else") {
                    if let Some(&last) = conditional_stack.last() {
                        if last {
                            skip_until_endif = true;
                        } else if skip_until_else {
                            skip_until_endif = false;
                            skip_until_else = false;
                        }
                    }
                    continue;
                } else if trimmed.starts_with("#endif") {
                    if !conditional_stack.is_empty() {
                        conditional_stack.pop();
                        
                        // If we're out of all nested conditionals, stop skipping
                        if conditional_stack.is_empty() || !conditional_stack.contains(&false) {
                            skip_until_endif = false;
                            skip_until_else = false;
                        }
                    }
                    continue;
                }
                
                // Skip processing if we're in a failed conditional
                if skip_until_endif {
                    continue;
                }
                
                // Process include directive
                if trimmed.starts_with("#include") {
                    match self.process_include(trimmed, current_dir) {
                        Ok(included_content) => {
                            result.push_str(&included_content);
                            result.push('\n');
                        }
                        Err(e) => return Err(e),
                    }
                    continue;
                }
                
                // Process define directive
                if trimmed.starts_with("#define") {
                    preprocessor.process_define(trimmed)?;
                    continue;
                }
                
                // Process undef directive
                if trimmed.starts_with("#undef") {
                    let macro_name = trimmed.trim_start_matches("#undef").trim();
                    preprocessor.defines.remove(macro_name);
                    continue;
                }
                
                // Skip other preprocessor directives for now
                continue;
            }
            
            // Skip lines in failed conditionals
            if skip_until_endif {
                continue;
            }
            
            // Process regular code line
            // TODO: Implement macro expansion
            
            result.push_str(line);
            result.push('\n');
        }
        
        Ok(result)
    }
}

impl Preprocessor for NativePreprocessor {
    fn is_available(&self) -> bool {
        // Native preprocessor is always available
        true
    }
    
    fn preprocess_file(&self, input_path: &Path) -> Result<PathBuf, String> {
        // Read the input file
        let content = fs::read_to_string(input_path)
            .map_err(|e| format!("Failed to read input file: {}", e))?;
        
        // Get the directory of the input file
        let current_dir = input_path.parent().unwrap_or_else(|| Path::new("."));
        
        // Preprocess the content
        let processed = self.preprocess_content(&content, current_dir)?;
        
        // Write to temporary file
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| format!("Failed to create temporary file: {}", e))?;
        
        temp_file.write_all(processed.as_bytes())
            .map_err(|e| format!("Failed to write to temporary file: {}", e))?;
        
        // Fix: Handle the .keep() error explicitly instead of using ?
        temp_file.into_temp_path()
            .keep()
            .map_err(|e| format!("Failed to preserve temporary file: {}", e))
    }
    
    fn preprocess_string(&self, source: &str) -> Result<String, String> {
        // Preprocess the content
        self.preprocess_content(source, Path::new("."))
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
    use tempfile::tempdir;
    
    #[test]
    fn test_process_include() {
        let temp_dir = tempdir().unwrap();
        let header_path = temp_dir.path().join("test.h");
        
        fs::write(&header_path, "int test_function(void);").unwrap();
        
        let preprocessor = NativePreprocessor::new().add_include_dir(temp_dir.path());
        
        let source = format!("#include \"{}\"", header_path.file_name().unwrap().to_string_lossy());
        let result = preprocessor.process_include(&source, temp_dir.path()).unwrap();
        
        assert!(result.contains("int test_function(void);"));
    }
    
    #[test]
    fn test_simple_define() {
        let mut preprocessor = NativePreprocessor::new();
        preprocessor.process_define("#define TEST 42").unwrap();
        
        assert_eq!(preprocessor.defines.get("TEST"), Some(&"42".to_string()));
    }
    
    #[test]
    fn test_conditional_compilation() {
        let mut preprocessor = NativePreprocessor::new();
        preprocessor.defines.insert("DEBUG".to_string(), "1".to_string());
        
        let source = r#"
            #ifdef DEBUG
            void debug_function(void);
            #else
            void release_function(void);
            #endif
        "#;
        
        let result = preprocessor.preprocess_string(source).unwrap();
        assert!(result.contains("void debug_function(void);"));
        assert!(!result.contains("void release_function(void);"));
    }
} 