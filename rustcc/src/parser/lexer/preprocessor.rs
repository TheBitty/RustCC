// preprocessor.rs
// Handling of preprocessor directives

use crate::parser::token::TokenType;
use crate::parser::lexer::Lexer;
use crate::parser::lexer::utils::{is_alpha, is_digit, is_alphanumeric, is_whitespace};

impl Lexer {
    /// Handles a preprocessor directive
    pub(crate) fn preprocessor_directive(&mut self) {
        // Add the # token
        self.add_token(TokenType::Hash);

        // Skip whitespace
        while self.peek() == ' ' || self.peek() == '\t' {
            self.advance();
        }

        // Mark the start of the directive name
        self.start = self.current;

        // Read the directive name
        while is_alpha(self.peek()) {
            self.advance();
        }

        // Get the directive name
        let directive = self.source[self.start..self.current].to_string();

        // Check if it's a known preprocessor directive
        match directive.as_str() {
            "include" => {
                self.add_token(TokenType::PPInclude);
                self.handle_include_directive();
            },
            "define" => {
                self.add_token(TokenType::PPDefine);
                self.handle_define_directive();
            },
            "undef" => {
                self.add_token(TokenType::PPUndef);
                self.handle_undef_directive();
            },
            "ifdef" => self.add_token(TokenType::PPIfDef),
            "ifndef" => self.add_token(TokenType::PPIfNDef),
            "if" => self.add_token(TokenType::PPIf),
            "else" => self.add_token(TokenType::PPElse),
            "elif" => self.add_token(TokenType::PPElif),
            "endif" => self.add_token(TokenType::PPEndif),
            "pragma" => {
                self.add_token(TokenType::PPPragma);
                self.handle_pragma_directive();
            },
            "error" => {
                self.add_token(TokenType::PPErrorDir);
                self.handle_error_directive();
            },
            "warning" => {
                self.add_token(TokenType::PPWarning);
                self.handle_warning_directive();
            },
            "line" => {
                self.add_token(TokenType::PPLine);
                self.handle_line_directive();
            },
            _ => self.add_token(TokenType::Error),
        }
    }

    /// Handles an #include directive
    pub(crate) fn handle_include_directive(&mut self) {
        // Skip whitespace
        while self.peek() == ' ' || self.peek() == '\t' {
            self.advance();
        }

        // Check if it's a system include or local include
        if self.peek() == '<' {
            self.advance(); // Consume <
            self.start = self.current;

            // Read until >
            while !self.is_at_end() && self.peek() != '>' && self.peek() != '\n' {
                self.advance();
            }

            if self.peek() == '>' {
                let path = self.source[self.start..self.current].to_string();
                self.add_token_with_literal(TokenType::StringLiteral, path);
                self.advance(); // Consume >
            } else {
                self.add_token(TokenType::Error);
            }
        } else if self.peek() == '"' {
            self.advance(); // Consume "
            self.start = self.current;

            // Read until "
            while !self.is_at_end() && self.peek() != '"' && self.peek() != '\n' {
                self.advance();
            }

            if self.peek() == '"' {
                let path = self.source[self.start..self.current].to_string();
                self.add_token_with_literal(TokenType::StringLiteral, path);
                self.advance(); // Consume "
            } else {
                self.add_token(TokenType::Error);
            }
        } else {
            self.add_token(TokenType::Error);
        }
    }

    /// Handles a #define directive
    pub(crate) fn handle_define_directive(&mut self) {
        // Skip whitespace
        while self.peek() == ' ' || self.peek() == '\t' {
            self.advance();
        }

        // Read the macro name
        self.start = self.current;
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        if self.start < self.current {
            let name = self.source[self.start..self.current].to_string();
            self.add_token_with_literal(TokenType::Identifier, name);

            // Check for macro parameters
            if self.peek() == '(' && !is_whitespace(self.peek_next()) {
                self.advance(); // Consume '('
                self.add_token(TokenType::LeftParen);

                // Parse parameter list
                let mut param_count = 0;
                while !self.is_at_end() && self.peek() != ')' {
                    // Skip whitespace
                    while is_whitespace(self.peek()) {
                        self.advance();
                    }

                    // Read parameter name
                    self.start = self.current;
                    while is_alphanumeric(self.peek()) {
                        self.advance();
                    }

                    if self.start < self.current {
                        let param = self.source[self.start..self.current].to_string();
                        self.add_token_with_literal(TokenType::Identifier, param);
                        param_count += 1;
                    }

                    // Skip whitespace
                    while is_whitespace(self.peek()) {
                        self.advance();
                    }

                    // Check for comma or closing paren
                    if self.peek() == ',' {
                        self.advance();
                        self.add_token(TokenType::Comma);
                    } else if self.peek() != ')' {
                        // Error: expected comma or closing paren
                        self.add_token(TokenType::Error);
                        break;
                    }
                }

                // Check for variadic macro with ... parameter
                if param_count > 0 && self.peek() == '.' {
                    if self.peek_next() == '.' {
                        self.advance(); // Consume first '.'
                        self.advance(); // Consume second '.'
                        if self.peek() == '.' {
                            self.advance(); // Consume third '.'
                            self.add_token(TokenType::Ellipsis);
                        } else {
                            self.add_token(TokenType::Error);
                        }
                    }
                }

                // Consume closing paren
                if self.peek() == ')' {
                    self.advance();
                    self.add_token(TokenType::RightParen);
                } else {
                    self.add_token(TokenType::Error);
                }
            }

            // Skip whitespace
            while self.peek() == ' ' || self.peek() == '\t' {
                self.advance();
            }

            // Read the macro value (rest of the line)
            self.process_macro_replacement();
        } else {
            self.add_token(TokenType::Error);
        }
    }

    /// Processes a macro replacement value
    pub(crate) fn process_macro_replacement(&mut self) {
        self.start = self.current;
        let mut nesting_level = 0;
        let mut in_string = false;
        let mut in_char_literal = false;
        
        // Process until end of line, handling any escape sequences or string literals
        while !self.is_at_end() && self.peek() != '\n' {
            let c = self.peek();
            
            // Handle string literals
            if c == '"' && !in_char_literal {
                in_string = !in_string;
            }
            
            // Handle character literals
            if c == '\'' && !in_string {
                in_char_literal = !in_char_literal;
            }
            
            // Handle escape sequences in strings or character literals
            if (in_string || in_char_literal) && c == '\\' {
                self.advance(); // Consume the backslash
                if !self.is_at_end() && self.peek() != '\n' {
                    self.advance(); // Consume the escaped character
                    continue;
                }
            }
            
            // Handle preprocessor stringification (#) and token concatenation (##) operators
            if !in_string && !in_char_literal {
                if c == '#' {
                    // Save the current text as a token if any
                    if self.start < self.current {
                        let text = self.source[self.start..self.current].to_string();
                        self.add_token_with_literal(TokenType::Identifier, text);
                    }
                    
                    self.advance(); // Consume '#'
                    
                    // Check for ## (token concatenation)
                    if self.peek() == '#' {
                        self.advance(); // Consume second '#'
                        self.add_token(TokenType::PPHashHash);
                    } else {
                        // Single # (stringification)
                        self.add_token(TokenType::PPHash);
                    }
                    
                    self.start = self.current;
                    continue;
                }
                
                // Keep track of nesting parentheses for complex expressions
                if c == '(' {
                    nesting_level += 1;
                } else if c == ')' {
                    nesting_level -= 1;
                }
            }
            
            self.advance();
        }
        
        // Add any remaining text as a token
        if self.start < self.current {
            let value = self.source[self.start..self.current].to_string();
            self.add_token_with_literal(TokenType::StringLiteral, value);
        }
    }

    /// Handles a #undef directive
    pub(crate) fn handle_undef_directive(&mut self) {
        // Skip whitespace
        while self.peek() == ' ' || self.peek() == '\t' {
            self.advance();
        }

        // Read the macro name
        self.start = self.current;
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        if self.start < self.current {
            let name = self.source[self.start..self.current].to_string();
            self.add_token_with_literal(TokenType::Identifier, name);
        } else {
            self.add_token(TokenType::Error);
        }
    }

    /// Handles a #pragma directive
    pub(crate) fn handle_pragma_directive(&mut self) {
        // Skip whitespace
        while self.peek() == ' ' || self.peek() == '\t' {
            self.advance();
        }

        // Read the pragma directive
        self.start = self.current;
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }

        if self.start < self.current {
            let pragma = self.source[self.start..self.current].to_string();
            self.add_token_with_literal(TokenType::StringLiteral, pragma);
        }
    }

    /// Handles a #error directive
    pub(crate) fn handle_error_directive(&mut self) {
        // Skip whitespace
        while self.peek() == ' ' || self.peek() == '\t' {
            self.advance();
        }

        // Read the error message
        self.start = self.current;
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }

        if self.start < self.current {
            let message = self.source[self.start..self.current].to_string();
            self.add_token_with_literal(TokenType::StringLiteral, message);
        }
    }

    /// Handles a #warning directive
    pub(crate) fn handle_warning_directive(&mut self) {
        // Skip whitespace
        while self.peek() == ' ' || self.peek() == '\t' {
            self.advance();
        }

        // Read the warning message
        self.start = self.current;
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }

        if self.start < self.current {
            let message = self.source[self.start..self.current].to_string();
            self.add_token_with_literal(TokenType::StringLiteral, message);
        }
    }

    /// Handles a #line directive
    pub(crate) fn handle_line_directive(&mut self) {
        // Skip whitespace
        while self.peek() == ' ' || self.peek() == '\t' {
            self.advance();
        }

        // Read the line number
        self.start = self.current;
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.start < self.current {
            let line_num = self.source[self.start..self.current].to_string();
            self.add_token_with_literal(TokenType::IntegerLiteral, line_num);

            // Skip whitespace
            while self.peek() == ' ' || self.peek() == '\t' {
                self.advance();
            }

            // Optionally read the filename
            if self.peek() == '"' {
                self.advance(); // Consume opening quote
                self.start = self.current;

                // Read until closing quote
                while !self.is_at_end() && self.peek() != '"' && self.peek() != '\n' {
                    self.advance();
                }

                if self.peek() == '"' {
                    let filename = self.source[self.start..self.current].to_string();
                    self.add_token_with_literal(TokenType::StringLiteral, filename);
                    self.advance(); // Consume closing quote
                } else {
                    self.add_token(TokenType::Error);
                }
            }
        } else {
            self.add_token(TokenType::Error);
        }
    }
} 