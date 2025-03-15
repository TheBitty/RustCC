use crate::preprocessor::Preprocessor;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use regex;

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
        let mut include_dirs = Vec::new();
        
        // Add platform-specific standard include paths
        #[cfg(target_os = "linux")]
        {
            include_dirs.push(PathBuf::from("/usr/include"));
            include_dirs.push(PathBuf::from("/usr/local/include"));
            
            // GCC includes - detect version
            if let Ok(entries) = fs::read_dir("/usr/lib/gcc") {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Ok(versions) = fs::read_dir(entry.path()) {
                            for ver in versions.flatten() {
                                if ver.path().is_dir() {
                                    let include_path = ver.path().join("include");
                                    if include_path.exists() {
                                        include_dirs.push(include_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            include_dirs.push(PathBuf::from("/usr/include"));
            include_dirs.push(PathBuf::from("/usr/local/include"));
            
            // Check for Xcode Command Line Tools or SDK includes
            let xcode_paths = vec![
                "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include",
                "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/usr/include",
            ];
            
            for path in xcode_paths {
                let path_buf = PathBuf::from(path);
                if path_buf.exists() {
                    include_dirs.push(path_buf);
                }
            }
            
            // Check brew includes
            let brew_path = PathBuf::from("/opt/homebrew/include");
            if brew_path.exists() {
                include_dirs.push(brew_path);
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // MSVC includes
            if let Ok(program_files) = std::env::var("ProgramFiles(x86)") {
                let msvc_path = PathBuf::from(program_files).join("Microsoft Visual Studio");
                if msvc_path.exists() {
                    if let Ok(versions) = fs::read_dir(&msvc_path) {
                        for ver in versions.flatten() {
                            let include_path = ver.path().join("VC").join("include");
                            if include_path.exists() {
                                include_dirs.push(include_path);
                            }
                        }
                    }
                }
            }
            
            // MinGW includes
            if let Ok(program_files) = std::env::var("ProgramFiles") {
                let mingw_path = PathBuf::from(program_files).join("MinGW").join("include");
                if mingw_path.exists() {
                    include_dirs.push(mingw_path);
                }
            }
        }
        
        // Add current directory
        include_dirs.push(PathBuf::from("."));
        
        NativePreprocessor {
            include_dirs,
            defines: HashMap::new(),
            keep_comments: false,
        }
    }

    /// Add an include directory
    pub fn add_include_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        let path = dir.as_ref().to_path_buf();
        // Add the directory only if it exists
        if path.exists() && path.is_dir() {
            self.include_dirs.push(path);
        }
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
    
    /// Expand macros in a line of code
    fn expand_macros(&self, line: &str) -> String {
        let mut result = line.to_string();
        
        // Simple macro expansion - replace each defined macro with its value
        for (name, value) in &self.defines {
            // Use word boundaries to avoid partial replacement
            let pattern = format!(r"\b{}\b", regex::escape(name));
            if let Ok(regex) = regex::Regex::new(&pattern) {
                result = regex.replace_all(&result, value).to_string();
            }
        }
        
        result
    }

    /// Preprocess the content of a file
    fn preprocess_content(&self, content: &str, current_dir: &Path) -> Result<String, String> {
        let mut result = String::new();
        
        // Create a mutable copy of self to handle #define directives
        let mut preprocessor = NativePreprocessor {
            include_dirs: self.include_dirs.clone(),
            defines: self.defines.clone(),
            keep_comments: self.keep_comments,
        };
        
        // Track conditional compilation state
        let mut conditional_stack: Vec<bool> = Vec::new();
        let mut skip_until_endif = false;
        
        // Process each line
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Handle preprocessor directives
            if trimmed.starts_with('#') {
                // Handle conditional compilation
                if trimmed.starts_with("#ifdef") {
                    let is_defined = preprocessor.evaluate_conditional(trimmed, false);
                    conditional_stack.push(is_defined);
                    skip_until_endif = !is_defined || skip_until_endif;
                    continue;
                }
                
                if trimmed.starts_with("#ifndef") {
                    let is_defined = preprocessor.evaluate_conditional(trimmed, true);
                    conditional_stack.push(is_defined);
                    skip_until_endif = !is_defined || skip_until_endif;
                    continue;
                }
                
                if trimmed.starts_with("#else") {
                    if let Some(last) = conditional_stack.last_mut() {
                        *last = !*last;
                        skip_until_endif = !*last || (conditional_stack.len() > 1 && !conditional_stack[conditional_stack.len() - 2]);
                    }
                    continue;
                }
                
                if trimmed.starts_with("#endif") {
                    if !conditional_stack.is_empty() {
                        conditional_stack.pop();
                        skip_until_endif = conditional_stack.iter().any(|&x| !x);
                    }
                    continue;
                }
                
                // Skip other directives if in a failed conditional
                if skip_until_endif {
                    continue;
                }
                
                // Process include directive
                if trimmed.starts_with("#include") {
                    // Extract the included file path
                    let start = trimmed.find('"').or_else(|| trimmed.find('<'));
                    let end = trimmed.rfind('"').or_else(|| trimmed.rfind('>'));
                    
                    if start.is_none() || end.is_none() || start.unwrap() >= end.unwrap() {
                        return Err(format!("Invalid include directive: {}", trimmed));
                    }
                    
                    let start_idx = start.unwrap() + 1;
                    let end_idx = end.unwrap();
                    let include_path = &trimmed[start_idx..end_idx];
                    
                    // Check if this is a system include or a local include
                    let is_system = trimmed.contains('<') && trimmed.contains('>');
                    
                    // Try to find the include file
                    let mut include_file_path = None;
                    
                    // For local includes, first try relative to the current file
                    if !is_system {
                        let local_path = current_dir.join(include_path);
                        if local_path.exists() {
                            include_file_path = Some(local_path);
                        }
                    }
                    
                    // Try each include directory
                    if include_file_path.is_none() {
                        for dir in &preprocessor.include_dirs {
                            let full_path = dir.join(include_path);
                            if full_path.exists() {
                                include_file_path = Some(full_path);
                                break;
                            }
                        }
                    }
                    
                    // Try the current directory as a fallback
                    if include_file_path.is_none() {
                        let current_dir_path = PathBuf::from(".").join(include_path);
                        if current_dir_path.exists() {
                            include_file_path = Some(current_dir_path);
                        }
                    }
                    
                    // Process the include file if found
                    if let Some(path) = include_file_path {
                        // Read the file content
                        let include_content = fs::read_to_string(&path)
                            .map_err(|_| format!("Cannot read include file: {}", path.display()))?;
                            
                        // Get the directory of the include file for resolving nested includes
                        let parent_dir = path.parent().unwrap_or(Path::new("."));
                        
                        // Create a new preprocessor with the current defines
                        let include_preprocessor = NativePreprocessor {
                            include_dirs: preprocessor.include_dirs.clone(),
                            defines: preprocessor.defines.clone(),
                            keep_comments: preprocessor.keep_comments,
                        };
                        
                        // Process the included content recursively
                        let processed_include = include_preprocessor.preprocess_content(&include_content, parent_dir)?;
                        
                        // Add the processed include content to the result
                        result.push_str(&processed_include);
                        
                        // Update our defines with any new defines from the included file
                        if let Ok(include_defines) = include_preprocessor.extract_defines(&include_content) {
                            for (name, value) in include_defines {
                                preprocessor.defines.insert(name, value);
                            }
                        }
                        
                        continue;
                    } else {
                        return Err(format!("Include file not found: {}", include_path));
                    }
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
            // Expand macros in the line
            let expanded_line = preprocessor.expand_macros(line);
            result.push_str(&expanded_line);
            result.push('\n');
        }
        
        Ok(result)
    }
    
    /// Extract defines from content without processing includes
    fn extract_defines(&self, content: &str) -> Result<HashMap<String, String>, String> {
        let mut defines = HashMap::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("#define") {
                // Remove the #define part
                let define_part = trimmed.trim_start_matches("#define").trim();
                
                // Split by first space to get name and value
                if let Some(space_pos) = define_part.find(char::is_whitespace) {
                    let name = define_part[..space_pos].trim().to_string();
                    let value = define_part[space_pos..].trim().to_string();
                    defines.insert(name, value);
                } else {
                    // Simple define without value
                    defines.insert(define_part.to_string(), "1".to_string());
                }
            }
        }
        
        Ok(defines)
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