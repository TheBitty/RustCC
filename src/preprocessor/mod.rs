use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a preprocessor directive
#[derive(Debug, Clone)]
pub enum Directive {
    Include(String),
    Define(String, Option<String>),
    Undef(String),
    IfDef(String),
    IfNDef(String),
    If(String),
    Else,
    Elif(String),
    Endif,
    Pragma(String),
    Error(String),
    Warning(String),
}

/// Represents the preprocessor state
pub struct Preprocessor {
    /// Map of defined macros and their values
    macros: HashMap<String, Option<String>>,
    /// Stack of conditional compilation states
    condition_stack: Vec<bool>,
    /// Current condition state (whether we're in a true branch of #if/#ifdef/#ifndef)
    current_condition: bool,
    /// Include paths to search for header files
    include_paths: Vec<PathBuf>,
    /// Already included files to prevent circular includes
    included_files: Vec<PathBuf>,
}

impl Preprocessor {
    /// Create a new preprocessor with default settings
    pub fn new() -> Self {
        let mut preprocessor = Preprocessor {
            macros: HashMap::new(),
            condition_stack: Vec::new(),
            current_condition: true,
            include_paths: Vec::new(),
            included_files: Vec::new(),
        };

        // Add standard include paths
        preprocessor.add_include_path("/usr/include");
        preprocessor.add_include_path("/usr/local/include");

        // Add some standard predefined macros
        preprocessor.define("__STDC__", Some("1"));
        preprocessor.define("__STDC_VERSION__", Some("201710L"));

        preprocessor
    }

    /// Add an include path to search for header files
    pub fn add_include_path<P: AsRef<Path>>(&mut self, path: P) {
        self.include_paths.push(PathBuf::from(path.as_ref()));
    }

    /// Define a macro
    pub fn define(&mut self, name: &str, value: Option<&str>) {
        self.macros.insert(
            name.to_string(),
            value.map(|v| v.to_string()),
        );
    }

    /// Undefine a macro
    pub fn undefine(&mut self, name: &str) {
        self.macros.remove(name);
    }

    /// Check if a macro is defined
    pub fn is_defined(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Get the value of a macro
    pub fn get_macro_value(&self, name: &str) -> Option<&Option<String>> {
        self.macros.get(name)
    }

    /// Process a source file with preprocessor directives
    pub fn process_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<String, String> {
        let file_path = file_path.as_ref();
        
        // Check if file has already been included to prevent circular includes
        if self.included_files.contains(&file_path.to_path_buf()) {
            return Ok(String::new()); // Skip already included files
        }
        
        // Read the file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
            
        self.included_files.push(file_path.to_path_buf());
        
        // Process the content
        self.process_content(&content, file_path)
    }

    /// Process content with preprocessor directives
    pub fn process_content<P: AsRef<Path>>(&mut self, content: &str, file_path: P) -> Result<String, String> {
        let file_path = file_path.as_ref();
        let mut result = String::new();
        
        // Split the content into lines
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Check if the line is a preprocessor directive
            if line.starts_with('#') {
                let directive = self.parse_directive(&line[1..].trim())?;
                
                match directive {
                    Directive::Include(path) => {
                        if self.current_condition {
                            let included_content = self.handle_include(&path, file_path)?;
                            result.push_str(&included_content);
                        }
                    },
                    Directive::Define(name, value) => {
                        if self.current_condition {
                            self.define(&name, value.as_deref());
                        }
                    },
                    Directive::Undef(name) => {
                        if self.current_condition {
                            self.undefine(&name);
                        }
                    },
                    Directive::IfDef(name) => {
                        self.condition_stack.push(self.current_condition);
                        if !self.current_condition {
                            // If we're already in a false branch, this is also false
                            self.current_condition = false;
                        } else {
                            // Otherwise, check if the macro is defined
                            self.current_condition = self.is_defined(&name);
                        }
                    },
                    Directive::IfNDef(name) => {
                        self.condition_stack.push(self.current_condition);
                        if !self.current_condition {
                            // If we're already in a false branch, this is also false
                            self.current_condition = false;
                        } else {
                            // Otherwise, check if the macro is not defined
                            self.current_condition = !self.is_defined(&name);
                        }
                    },
                    Directive::If(expr) => {
                        self.condition_stack.push(self.current_condition);
                        if !self.current_condition {
                            // If we're already in a false branch, this is also false
                            self.current_condition = false;
                        } else {
                            // Otherwise, evaluate the expression
                            self.current_condition = self.evaluate_condition(&expr)?;
                        }
                    },
                    Directive::Else => {
                        if let Some(&parent_condition) = self.condition_stack.last() {
                            if parent_condition {
                                // Only flip the condition if the parent condition is true
                                self.current_condition = !self.current_condition;
                            }
                        } else {
                            return Err("Unexpected #else directive".to_string());
                        }
                    },
                    Directive::Elif(expr) => {
                        if let Some(&parent_condition) = self.condition_stack.last() {
                            if parent_condition && !self.current_condition {
                                // Only evaluate if parent is true and we haven't taken a branch yet
                                self.current_condition = self.evaluate_condition(&expr)?;
                            } else if parent_condition {
                                // We've already taken a branch, so this is false
                                self.current_condition = false;
                            }
                        } else {
                            return Err("Unexpected #elif directive".to_string());
                        }
                    },
                    Directive::Endif => {
                        if let Some(parent_condition) = self.condition_stack.pop() {
                            self.current_condition = parent_condition;
                        } else {
                            return Err("Unexpected #endif directive".to_string());
                        }
                    },
                    Directive::Pragma(pragma) => {
                        // Pragmas are ignored for now
                        // In a real implementation, we would handle specific pragmas
                    },
                    Directive::Error(msg) => {
                        if self.current_condition {
                            return Err(format!("#error: {}", msg));
                        }
                    },
                    Directive::Warning(msg) => {
                        if self.current_condition {
                            eprintln!("Warning: {}", msg);
                        }
                    },
                }
            } else if self.current_condition {
                // If we're in a true branch, add the line to the result
                // Replace any macros in the line
                let processed_line = self.expand_macros(line);
                result.push_str(&processed_line);
                result.push('\n');
            }
            
            i += 1;
        }
        
        Ok(result)
    }

    /// Parse a preprocessor directive
    fn parse_directive(&self, directive: &str) -> Result<Directive, String> {
        let parts: Vec<&str> = directive.splitn(2, ' ').collect();
        let command = parts[0];
        let args = parts.get(1).map(|s| s.trim()).unwrap_or("");
        
        match command {
            "include" => {
                // Parse include path
                if args.starts_with('<') && args.ends_with('>') {
                    // System include
                    Ok(Directive::Include(args[1..args.len()-1].to_string()))
                } else if args.starts_with('"') && args.ends_with('"') {
                    // Local include
                    Ok(Directive::Include(args[1..args.len()-1].to_string()))
                } else {
                    Err(format!("Invalid include directive: {}", args))
                }
            },
            "define" => {
                // Parse define directive
                let define_parts: Vec<&str> = args.splitn(2, ' ').collect();
                let name = define_parts[0].to_string();
                let value = define_parts.get(1).map(|s| s.trim().to_string());
                
                Ok(Directive::Define(name, value))
            },
            "undef" => Ok(Directive::Undef(args.to_string())),
            "ifdef" => Ok(Directive::IfDef(args.to_string())),
            "ifndef" => Ok(Directive::IfNDef(args.to_string())),
            "if" => Ok(Directive::If(args.to_string())),
            "else" => Ok(Directive::Else),
            "elif" => Ok(Directive::Elif(args.to_string())),
            "endif" => Ok(Directive::Endif),
            "pragma" => Ok(Directive::Pragma(args.to_string())),
            "error" => Ok(Directive::Error(args.to_string())),
            "warning" => Ok(Directive::Warning(args.to_string())),
            _ => Err(format!("Unknown preprocessor directive: {}", command)),
        }
    }

    /// Handle an include directive
    fn handle_include<P: AsRef<Path>>(&mut self, path: &str, current_file: P) -> Result<String, String> {
        let current_dir = current_file.as_ref().parent().unwrap_or_else(|| Path::new("."));
        
        // First try to find the file relative to the current file
        let mut file_path = current_dir.join(path);
        
        if !file_path.exists() {
            // If not found, try the include paths
            let mut found = false;
            
            for include_path in &self.include_paths {
                file_path = include_path.join(path);
                
                if file_path.exists() {
                    found = true;
                    break;
                }
            }
            
            if !found {
                return Err(format!("Include file not found: {}", path));
            }
        }
        
        // Process the included file
        self.process_file(file_path)
    }

    /// Evaluate a preprocessor condition
    fn evaluate_condition(&self, expr: &str) -> Result<bool, String> {
        // This is a simplified implementation
        // In a real implementation, we would parse and evaluate the expression
        
        // For now, just check if the expression is a defined() check
        if expr.starts_with("defined(") && expr.ends_with(')') {
            let macro_name = &expr[8..expr.len()-1];
            return Ok(self.is_defined(macro_name));
        }
        
        // Otherwise, try to expand macros and evaluate as a boolean expression
        let expanded = self.expand_macros(expr);
        
        // Simple evaluation - treat non-zero as true, zero as false
        match expanded.trim() {
            "0" => Ok(false),
            "" => Ok(false),
            _ => Ok(true),
        }
    }

    /// Expand macros in a line of code
    fn expand_macros(&self, line: &str) -> String {
        let mut result = line.to_string();
        
        // Simple macro expansion - replace all occurrences of defined macros
        for (name, value) in &self.macros {
            if let Some(val) = value {
                // Only replace if the macro has a value
                result = result.replace(name, val);
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_define() {
        let mut preprocessor = Preprocessor::new();
        preprocessor.define("MAX_SIZE", Some("100"));
        
        let content = "#define MIN_SIZE 10\nint array[MAX_SIZE];\nint min = MIN_SIZE;";
        let result = preprocessor.process_content(content, "test.c").unwrap();
        
        assert!(result.contains("int array[100];"));
        assert!(result.contains("int min = 10;"));
    }

    #[test]
    fn test_conditional_compilation() {
        let mut preprocessor = Preprocessor::new();
        preprocessor.define("DEBUG", Some("1"));
        
        let content = "#ifdef DEBUG\nprintf(\"Debug mode\\n\");\n#else\nprintf(\"Release mode\\n\");\n#endif";
        let result = preprocessor.process_content(content, "test.c").unwrap();
        
        assert!(result.contains("printf(\"Debug mode\\n\");"));
        assert!(!result.contains("printf(\"Release mode\\n\");"));
    }
} 