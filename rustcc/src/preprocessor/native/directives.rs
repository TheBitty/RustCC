use super::NativePreprocessor;

impl NativePreprocessor {
    /// Process a preprocessor directive
    pub(crate) fn process_directive(&mut self, line: &str, current_file: &str) -> Result<String, String> {
        let trimmed = line.trim();
        
        if trimmed.starts_with("#include") {
            // Handle include directive
            self.process_include(trimmed, current_file)
        } else if trimmed.starts_with("#define") {
            // Handle define directive
            self.process_define(trimmed)?;
            Ok("".to_string())
        } else if trimmed.starts_with("#undef") {
            // Handle undef directive
            self.process_undef(trimmed)?;
            Ok("".to_string())
        } else if trimmed.starts_with("#ifdef") || trimmed.starts_with("#ifndef") || trimmed.starts_with("#if") {
            // Handle conditional compilation start
            Ok("".to_string())
        } else if trimmed.starts_with("#else") || trimmed.starts_with("#elif") {
            // Handle conditional compilation else/elif
            Ok("".to_string())
        } else if trimmed.starts_with("#endif") {
            // Handle conditional compilation end
            Ok("".to_string())
        } else if trimmed.starts_with("#pragma") {
            // Handle pragma directive (currently ignored)
            Ok("".to_string())
        } else if trimmed.starts_with("#error") {
            // Handle error directive
            self.process_error(trimmed)
        } else if trimmed.starts_with("#warning") {
            // Handle warning directive
            self.process_warning(trimmed);
            Ok("".to_string())
        } else if trimmed == "#" {
            // Empty directive, ignore
            Ok("".to_string())
        } else {
            // Unknown directive
            Err(format!("Unknown preprocessor directive: {}", trimmed))
        }
    }

    /// Process a #undef directive
    pub(crate) fn process_undef(&mut self, line: &str) -> Result<(), String> {
        // Remove the #undef part
        let undef_part = line.trim_start_matches("#undef").trim();
        
        // Remove the macro from defines
        self.defines.remove(undef_part);
        
        Ok(())
    }

    /// Process an #error directive
    pub(crate) fn process_error(&self, line: &str) -> Result<String, String> {
        // Remove the #error part
        let error_part = line.trim_start_matches("#error").trim();
        
        // Return an error with the message
        Err(format!("Error directive: {}", error_part))
    }

    /// Process a #warning directive
    pub(crate) fn process_warning(&self, line: &str) {
        // Remove the #warning part
        let warning_part = line.trim_start_matches("#warning").trim();
        
        // Print a warning message
        eprintln!("Warning: {}", warning_part);
    }

    /// Evaluate a conditional directive
    pub(crate) fn evaluate_conditional(&self, line: &str) -> Result<bool, String> {
        let trimmed = line.trim();
        
        if trimmed.starts_with("#ifdef") {
            // Check if macro is defined
            let macro_name = trimmed.trim_start_matches("#ifdef").trim();
            Ok(self.defines.contains_key(macro_name))
        } else if trimmed.starts_with("#ifndef") {
            // Check if macro is not defined
            let macro_name = trimmed.trim_start_matches("#ifndef").trim();
            Ok(!self.defines.contains_key(macro_name))
        } else if trimmed.starts_with("#if") {
            // Evaluate complex expression
            let expr = trimmed.trim_start_matches("#if").trim();
            self.evaluate_if_expression(expr)
        } else if trimmed.starts_with("#elif") {
            // Evaluate complex expression for elif
            let expr = trimmed.trim_start_matches("#elif").trim();
            self.evaluate_if_expression(expr)
        } else {
            Err(format!("Not a conditional directive: {}", trimmed))
        }
    }

    /// Evaluate a complex expression in #if directive
    pub(crate) fn evaluate_if_expression(&self, expr: &str) -> Result<bool, String> {
        // Tokenize the expression
        let tokens = self.tokenize_expression(expr)?;
        
        // Parse and evaluate the expression
        self.parse_expression(&tokens, 0).map(|(result, _)| result)
    }

    /// Tokenize an expression into tokens
    fn tokenize_expression(&self, expr: &str) -> Result<Vec<String>, String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut chars = expr.chars().peekable();
        
        while let Some(c) = chars.next() {
            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    // Whitespace separates tokens
                    if !current_token.is_empty() {
                        tokens.push(current_token);
                        current_token = String::new();
                    }
                },
                '(' | ')' | '!' | '&' | '|' | '=' | '<' | '>' => {
                    // Special characters are separate tokens
                    if !current_token.is_empty() {
                        tokens.push(current_token);
                        current_token = String::new();
                    }
                    
                    // Handle multi-character operators
                    match c {
                        '&' if chars.peek() == Some(&'&') => {
                            chars.next(); // Consume the second '&'
                            tokens.push("&&".to_string());
                        },
                        '|' if chars.peek() == Some(&'|') => {
                            chars.next(); // Consume the second '|'
                            tokens.push("||".to_string());
                        },
                        '=' if chars.peek() == Some(&'=') => {
                            chars.next(); // Consume the second '='
                            tokens.push("==".to_string());
                        },
                        '!' if chars.peek() == Some(&'=') => {
                            chars.next(); // Consume the '='
                            tokens.push("!=".to_string());
                        },
                        '<' if chars.peek() == Some(&'=') => {
                            chars.next(); // Consume the '='
                            tokens.push("<=".to_string());
                        },
                        '>' if chars.peek() == Some(&'=') => {
                            chars.next(); // Consume the '='
                            tokens.push(">=".to_string());
                        },
                        _ => {
                            tokens.push(c.to_string());
                        }
                    }
                },
                _ => {
                    // Part of a token
                    current_token.push(c);
                }
            }
        }
        
        // Add the last token if not empty
        if !current_token.is_empty() {
            tokens.push(current_token);
        }
        
        Ok(tokens)
    }

    /// Parse and evaluate an expression
    /// Returns the result and the index of the next token to process
    fn parse_expression(&self, tokens: &[String], start_idx: usize) -> Result<(bool, usize), String> {
        if start_idx >= tokens.len() {
            return Err("Unexpected end of expression".to_string());
        }
        
        // Parse the first term
        let (mut result, mut idx) = self.parse_term(tokens, start_idx)?;
        
        // Process operators
        while idx < tokens.len() {
            match tokens[idx].as_str() {
                "&&" => {
                    // Logical AND
                    let (right, next_idx) = self.parse_term(tokens, idx + 1)?;
                    result = result && right;
                    idx = next_idx;
                },
                "||" => {
                    // Logical OR
                    let (right, next_idx) = self.parse_term(tokens, idx + 1)?;
                    result = result || right;
                    idx = next_idx;
                },
                ")" => {
                    // End of a parenthesized expression
                    break;
                },
                _ => {
                    // Unknown operator or end of expression
                    break;
                }
            }
        }
        
        Ok((result, idx))
    }

    /// Parse and evaluate a term
    /// Returns the result and the index of the next token to process
    fn parse_term(&self, tokens: &[String], start_idx: usize) -> Result<(bool, usize), String> {
        if start_idx >= tokens.len() {
            return Err("Unexpected end of term".to_string());
        }
        
        match tokens[start_idx].as_str() {
            "(" => {
                // Parenthesized expression
                let (result, idx) = self.parse_expression(tokens, start_idx + 1)?;
                
                // Ensure we have a closing parenthesis
                if idx >= tokens.len() || tokens[idx] != ")" {
                    return Err("Missing closing parenthesis".to_string());
                }
                
                Ok((result, idx + 1))
            },
            "!" => {
                // Logical NOT
                let (result, idx) = self.parse_term(tokens, start_idx + 1)?;
                Ok((!result, idx))
            },
            "defined" => {
                // Check if a macro is defined
                if start_idx + 1 >= tokens.len() {
                    return Err("Missing argument for 'defined'".to_string());
                }
                
                let mut idx = start_idx + 1;
                let mut macro_name = String::new();
                
                // Handle both forms: defined(MACRO) and defined MACRO
                if tokens[idx] == "(" {
                    idx += 1;
                    if idx >= tokens.len() {
                        return Err("Unexpected end after 'defined('".to_string());
                    }
                    
                    macro_name = tokens[idx].clone();
                    idx += 1;
                    
                    if idx >= tokens.len() || tokens[idx] != ")" {
                        return Err("Missing closing parenthesis for 'defined'".to_string());
                    }
                    
                    idx += 1;
                } else {
                    macro_name = tokens[idx].clone();
                    idx += 1;
                }
                
                let result = self.defines.contains_key(&macro_name);
                Ok((result, idx))
            },
            _ => {
                // Value or comparison
                let (value, mut idx) = self.parse_value(tokens, start_idx)?;
                
                // Check for comparison operators
                if idx < tokens.len() {
                    match tokens[idx].as_str() {
                        "==" => {
                            let (right, next_idx) = self.parse_value(tokens, idx + 1)?;
                            Ok((value == right, next_idx))
                        },
                        "!=" => {
                            let (right, next_idx) = self.parse_value(tokens, idx + 1)?;
                            Ok((value != right, next_idx))
                        },
                        "<" => {
                            let (right, next_idx) = self.parse_value(tokens, idx + 1)?;
                            Ok((value < right, next_idx))
                        },
                        "<=" => {
                            let (right, next_idx) = self.parse_value(tokens, idx + 1)?;
                            Ok((value <= right, next_idx))
                        },
                        ">" => {
                            let (right, next_idx) = self.parse_value(tokens, idx + 1)?;
                            Ok((value > right, next_idx))
                        },
                        ">=" => {
                            let (right, next_idx) = self.parse_value(tokens, idx + 1)?;
                            Ok((value >= right, next_idx))
                        },
                        _ => {
                            // No comparison, just return the value as a boolean
                            Ok((value != 0, idx))
                        }
                    }
                } else {
                    // End of expression, return the value as a boolean
                    Ok((value != 0, idx))
                }
            }
        }
    }

    /// Parse and evaluate a value
    /// Returns the numeric value and the index of the next token to process
    fn parse_value(&self, tokens: &[String], start_idx: usize) -> Result<(i64, usize), String> {
        if start_idx >= tokens.len() {
            return Err("Unexpected end of value".to_string());
        }
        
        let token = &tokens[start_idx];
        
        // Try to parse as a number
        if let Ok(value) = token.parse::<i64>() {
            return Ok((value, start_idx + 1));
        }
        
        // Check if it's a defined macro
        if let Some(value) = self.defines.get(token) {
            // Try to parse the macro value as a number
            if let Ok(num) = value.parse::<i64>() {
                return Ok((num, start_idx + 1));
            }
            
            // Special case for string literals
            if value.starts_with('"') && value.ends_with('"') {
                // String literals are treated as non-zero in conditional expressions
                return Ok((1, start_idx + 1));
            }
            
            // Default to 1 for defined macros without a numeric value
            return Ok((1, start_idx + 1));
        }
        
        // Undefined macro, treat as 0
        Ok((0, start_idx + 1))
    }
} 