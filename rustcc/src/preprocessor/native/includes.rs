use std::path::{Path, PathBuf};
use std::fs;

use super::NativePreprocessor;
use crate::preprocessor::Preprocessor;

impl NativePreprocessor {
    /// Process an #include directive
    #[allow(dead_code)]
    pub(crate) fn process_include(&mut self, line: &str, current_file: &str) -> Result<String, String> {
        // Remove the #include part
        let include_part = line.trim_start_matches("#include").trim();
        
        // Check if it's a system include (<...>) or a local include ("...")
        let (is_system, path) = if include_part.starts_with('<') && include_part.ends_with('>') {
            // System include
            (true, include_part[1..include_part.len() - 1].trim().to_string())
        } else if include_part.starts_with('"') && include_part.ends_with('"') {
            // Local include
            (false, include_part[1..include_part.len() - 1].trim().to_string())
        } else {
            return Err(format!("Invalid include format: {}", include_part));
        };
        
        // Expand macros in the path
        let expanded_path = self.expand_macros(&path);
        
        // Find the include file
        let include_file = self.find_include_file(&expanded_path, is_system, current_file)?;
        
        // Increment include depth to prevent infinite recursion
        self.include_depth += 1;
        if self.include_depth > self.max_include_depth {
            self.include_depth -= 1;
            return Err(format!("Maximum include depth exceeded ({})", self.max_include_depth));
        }
        
        // Read the file content
        let content = match fs::read_to_string(&include_file) {
            Ok(content) => content,
            Err(e) => {
                self.include_depth -= 1;
                return Err(format!("Failed to read include file {}: {}", include_file.display(), e));
            }
        };
        
        // Process the included content
        let file_name = include_file.to_string_lossy();
        let result = match self.preprocess_string(&content, &file_name) {
            Ok(result) => result,
            Err(e) => {
                self.include_depth -= 1;
                return Err(e);
            }
        };
        
        // Decrement include depth
        self.include_depth -= 1;
        
        Ok(result)
    }

    /// Find an include file
    #[allow(dead_code)]
    pub(crate) fn find_include_file(&self, path: &str, is_system: bool, current_file: &str) -> Result<PathBuf, String> {
        // For local includes, first check relative to the current file
        if !is_system {
            let current_dir = Path::new(current_file).parent().unwrap_or_else(|| Path::new(""));
            let local_path = current_dir.join(path);
            
            if local_path.exists() {
                return Ok(local_path);
            }
        }
        
        // Check all include directories
        for dir in &self.include_dirs {
            let include_path = dir.join(path);
            if include_path.exists() {
                return Ok(include_path);
            }
        }
        
        // If it's a system header, allow it even if not found
        // This is to handle standard headers that might not be present
        // but shouldn't cause compilation to fail
        if is_system && self.is_standard_header(path) {
            // Return a placeholder path that won't be read
            return Ok(PathBuf::from(path));
        }
        
        Err(format!("Include file not found: {}", path))
    }

    /// Check if a path is a standard header
    #[allow(dead_code)]
    pub(crate) fn is_standard_header(&self, path: &str) -> bool {
        // List of common standard headers
        const STANDARD_HEADERS: &[&str] = &[
            "stdio.h", "stdlib.h", "string.h", "math.h", "assert.h",
            "ctype.h", "errno.h", "float.h", "limits.h", "locale.h",
            "setjmp.h", "signal.h", "stdarg.h", "stddef.h", "time.h"
        ];
        
        STANDARD_HEADERS.contains(&path)
    }

    // Other methods will be added here
} 