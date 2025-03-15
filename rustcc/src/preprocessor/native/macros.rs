use std::collections::HashMap;
use regex;
use chrono;

use super::NativePreprocessor;

impl NativePreprocessor {
    /// Process a #define directive
    #[allow(dead_code)]
    pub(crate) fn process_define(&mut self, line: &str) -> Result<(), String> {
        // Remove the #define part
        let define_part = line.trim_start_matches("#define").trim();
        
        // Check if it's empty
        if define_part.is_empty() {
            return Err("Empty #define directive".to_string());
        }
        
        // Check if it's a function-like macro or an object-like macro
        if let Some(paren_pos) = define_part.find('(') {
            // Function-like macro
            let name_end = paren_pos;
            let name = define_part[..name_end].trim().to_string();
            
            // Find the closing parenthesis
            if let Some(close_paren_pos) = define_part[paren_pos..].find(')') {
                let params_str = &define_part[paren_pos + 1..paren_pos + close_paren_pos];
                let body_start = paren_pos + close_paren_pos + 1;
                
                // Get the body (everything after the closing parenthesis)
                let body = if body_start < define_part.len() {
                    define_part[body_start..].trim().to_string()
                } else {
                    // Empty body
                    String::new()
                };
                
                // Parse the parameters
                let params = params_str.split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>();
                
                // Store the function-like macro
                // For simplicity, we'll prefix with FUNCTION_MACRO: to distinguish
                let macro_value = format!("FUNCTION_MACRO:{}:{}", 
                    params.join(","), 
                    body);
                
                self.defines.insert(name, macro_value);
            } else {
                return Err("Missing closing parenthesis in function-like macro".to_string());
            }
        } else {
            // Object-like macro
            // Find the first space to separate name and value
            if let Some(space_pos) = define_part.find(char::is_whitespace) {
                let name = define_part[..space_pos].trim().to_string();
                let value = define_part[space_pos..].trim().to_string();
                
                self.defines.insert(name, value);
            } else {
                // No value, just a name (like #define DEBUG)
                let name = define_part.trim().to_string();
                self.defines.insert(name, "1".to_string());
            }
        }
        
        Ok(())
    }

    /// Expand macros in a line of text
    #[allow(dead_code)]
    pub(crate) fn expand_macros(&self, line: &str) -> String {
        let mut result = line.to_string();
        
        // Expand predefined macros first
        result = self.expand_predefined_macros(&result);
        
        // Expand user-defined macros
        // We need to loop to handle nested macro expansions
        let mut changed = true;
        let mut iteration = 0;
        let max_iterations = 10; // Prevent infinite recursion
        
        while changed && iteration < max_iterations {
            changed = false;
            iteration += 1;
            
            for (name, value) in &self.defines {
                // Skip function-like macros for now
                if value.starts_with("FUNCTION_MACRO:") {
                    // Handle function-like macros separately
                    let mut temp_result = result.clone();
                    if self.expand_function_macro(&mut temp_result, name, value) {
                        changed = true;
                        result = temp_result;
                    }
                    continue;
                }
                
                // For object-like macros, use regex to ensure we only replace whole words
                let pattern = format!(r"\b{}\b", regex::escape(name));
                if let Ok(re) = regex::Regex::new(&pattern) {
                    let new_result = re.replace_all(&result, value.as_str()).to_string();
                    if new_result != result {
                        changed = true;
                        result = new_result;
                    }
                }
            }
        }
        
        if iteration >= max_iterations {
            // Add a warning comment for potential circular macro expansion
            result.push_str("\n/* WARNING: Possible circular macro expansion */\n");
        }
        
        result
    }
    
    /// Expand predefined macros like __FILE__, __LINE__, etc.
    fn expand_predefined_macros(&self, line: &str) -> String {
        let mut result = line.to_string();
        
        // Replace __FILE__ with the current file
        let file_pattern = r"\b__FILE__\b";
        if let Ok(re) = regex::Regex::new(file_pattern) {
            result = re.replace_all(&result, "\"unknown\"").to_string();
        }
        
        // Replace __LINE__ with the current line number (placeholder)
        let line_pattern = r"\b__LINE__\b";
        if let Ok(re) = regex::Regex::new(line_pattern) {
            result = re.replace_all(&result, "0").to_string();
        }
        
        // Replace __DATE__ with the current date
        let date_pattern = r"\b__DATE__\b";
        if let Ok(re) = regex::Regex::new(date_pattern) {
            let date = chrono::Local::now().format("\"%b %d %Y\"").to_string();
            result = re.replace_all(&result, date.as_str()).to_string();
        }
        
        // Replace __TIME__ with the current time
        let time_pattern = r"\b__TIME__\b";
        if let Ok(re) = regex::Regex::new(time_pattern) {
            let time = chrono::Local::now().format("\"%H:%M:%S\"").to_string();
            result = re.replace_all(&result, time.as_str()).to_string();
        }
        
        result
    }
    
    /// Expand a function-like macro
    #[allow(dead_code)]
    pub(crate) fn expand_function_macro(&self, input: &mut String, name: &str, value: &str) -> bool {
        // Check if the value has the function macro format
        let parts: Vec<&str> = value.splitn(3, ':').collect();
        if parts.len() != 3 || parts[0] != "FUNCTION_MACRO" {
            return false;
        }
        
        let params_str = parts[1];
        let replacement = parts[2];
        
        // Parse parameters
        let params: Vec<&str> = if params_str.is_empty() {
            Vec::new()
        } else {
            params_str.split(',').collect()
        };
        
        // Create a regex to find macro invocations
        // This pattern looks for macro_name(args)
        let pattern = format!(r"\b{}\s*\((.*?)\)", regex::escape(name));
        let regex = match regex::Regex::new(&pattern) {
            Ok(r) => r,
            Err(_) => return false,
        };
        
        let mut modified = false;
        
        // Find and replace all macro invocations
        while let Some(captures) = regex.captures(input) {
            modified = true;
            
            // Extract arguments
            let full_match = captures.get(0).unwrap().as_str();
            let args_str = captures.get(1).unwrap().as_str();
            let args = self.parse_macro_args(args_str);
            
            // Check if argument count matches parameter count
            if args.len() != params.len() {
                // Replace with an error message if argument count doesn't match
                let error = format!("/* ERROR: Macro {} expected {} arguments, got {} */", 
                               name, params.len(), args.len());
                *input = input.replacen(full_match, &error, 1);
                continue;
            }
            
            // Perform the replacement
            let mut result = replacement.to_string();
            
            // Replace each parameter with its argument
            for (i, param) in params.iter().enumerate() {
                let arg = &args[i];
                let param_pattern = format!(r"\b{}\b", regex::escape(param));
                if let Ok(re) = regex::Regex::new(&param_pattern) {
                    result = re.replace_all(&result, arg).to_string();
                }
            }
            
            // Replace the macro invocation with the expanded text
            *input = input.replacen(full_match, &result, 1);
        }
        
        modified
    }
    
    /// Parse macro arguments, handling nested parentheses and commas in strings
    #[allow(dead_code)]
    pub(crate) fn parse_macro_args(&self, args_str: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut paren_depth = 0;
        let mut in_string = false;
        let mut in_char = false;
        let mut escaped = false;
        
        for c in args_str.chars() {
            match c {
                '(' if !in_string && !in_char => {
                    paren_depth += 1;
                    current_arg.push(c);
                },
                ')' if !in_string && !in_char => {
                    paren_depth -= 1;
                    current_arg.push(c);
                },
                ',' if paren_depth == 0 && !in_string && !in_char => {
                    // Argument separator at top level
                    args.push(current_arg.trim().to_string());
                    current_arg = String::new();
                },
                '"' if !escaped => {
                    in_string = !in_string;
                    current_arg.push(c);
                },
                '\'' if !escaped => {
                    in_char = !in_char;
                    current_arg.push(c);
                },
                '\\' => {
                    escaped = !escaped;
                    current_arg.push(c);
                },
                _ => {
                    escaped = false;
                    current_arg.push(c);
                }
            }
        }
        
        // Add the last argument if not empty
        if !current_arg.trim().is_empty() {
            args.push(current_arg.trim().to_string());
        }
        
        args
    }

    /// Add standard predefined macros
    pub(crate) fn add_standard_defines(&mut self) {
        // C standard version
        self.defines.insert("__STDC__".to_string(), "1".to_string());
        self.defines.insert("__STDC_VERSION__".to_string(), "201710L".to_string()); // C17
        self.defines.insert("__STDC_HOSTED__".to_string(), "1".to_string());
        
        // Platform-specific macros
        #[cfg(target_os = "linux")]
        {
            self.defines.insert("__linux__".to_string(), "1".to_string());
            self.defines.insert("__unix__".to_string(), "1".to_string());
            self.defines.insert("__GNUC__".to_string(), "4".to_string());
            self.defines.insert("__GNUC_MINOR__".to_string(), "2".to_string());
        }
        
        #[cfg(target_os = "macos")]
        {
            self.defines.insert("__APPLE__".to_string(), "1".to_string());
            self.defines.insert("__MACH__".to_string(), "1".to_string());
            self.defines.insert("__unix__".to_string(), "1".to_string());
            
            // Check if we're on Apple Silicon
            #[cfg(target_arch = "aarch64")]
            {
                self.defines.insert("__aarch64__".to_string(), "1".to_string());
                self.defines.insert("__arm64__".to_string(), "1".to_string());
                self.defines.insert("__ARM_ARCH".to_string(), "8".to_string());
            }
            
            // Check if we're on Intel
            #[cfg(target_arch = "x86_64")]
            {
                self.defines.insert("__x86_64__".to_string(), "1".to_string());
                self.defines.insert("__amd64__".to_string(), "1".to_string());
            }
            
            // Add clang-specific macros
            self.defines.insert("__clang__".to_string(), "1".to_string());
            self.defines.insert("__clang_major__".to_string(), "13".to_string());
            self.defines.insert("__clang_minor__".to_string(), "0".to_string());
        }
        
        #[cfg(target_os = "windows")]
        {
            self.defines.insert("_WIN32".to_string(), "1".to_string());
            
            #[cfg(target_arch = "x86_64")]
            {
                self.defines.insert("_WIN64".to_string(), "1".to_string());
                self.defines.insert("__x86_64__".to_string(), "1".to_string());
            }
            
            // Add MSVC-specific macros
            self.defines.insert("_MSC_VER".to_string(), "1929".to_string());
        }
        
        // Architecture-specific macros
        #[cfg(target_arch = "x86_64")]
        {
            self.defines.insert("__x86_64__".to_string(), "1".to_string());
            self.defines.insert("__LP64__".to_string(), "1".to_string());
            self.defines.insert("__SIZEOF_POINTER__".to_string(), "8".to_string());
        }
        
        #[cfg(target_arch = "x86")]
        {
            self.defines.insert("__i386__".to_string(), "1".to_string());
            self.defines.insert("__SIZEOF_POINTER__".to_string(), "4".to_string());
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            self.defines.insert("__aarch64__".to_string(), "1".to_string());
            self.defines.insert("__LP64__".to_string(), "1".to_string());
            self.defines.insert("__SIZEOF_POINTER__".to_string(), "8".to_string());
        }
        
        // Add date and time macros
        let now = std::time::SystemTime::now();
        
        // Convert to a date/time
        let datetime = chrono::DateTime::<chrono::Utc>::from(now);
        
        // Format as required by C standard
        let date = datetime.format("%b %d %Y").to_string();
        let time = datetime.format("%H:%M:%S").to_string();
        
        self.defines.insert("__DATE__".to_string(), format!("\"{}\"", date));
        self.defines.insert("__TIME__".to_string(), format!("\"{}\"", time));
        
        // Add other standard macros
        self.defines.insert("__FUNCTION__".to_string(), "\"\"".to_string());
        self.defines.insert("__func__".to_string(), "\"\"".to_string());
        self.defines.insert("__FILE__".to_string(), "\"\"".to_string());
        self.defines.insert("__LINE__".to_string(), "0".to_string());
    }
    
    /// Extract defines from content without processing includes
    #[allow(dead_code)]
    pub(crate) fn extract_defines(&self, content: &str) -> Result<HashMap<String, String>, String> {
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