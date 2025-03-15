// literals.rs
// Handling of string, character, and numeric literals

use crate::parser::lexer::utils::{is_digit, is_hex_digit, is_octal_digit};
use crate::parser::lexer::Lexer;
use crate::parser::token::TokenType;

impl Lexer {
    /// Handles string literals
    pub(crate) fn handle_string_literal(&mut self) {
        // Check for prefixed string literals: L"string", u"string", U"string", u8"string"
        if self.current > 1 {
            let prev_char = self.source.chars().nth(self.current - 2).unwrap_or('\0');
            if prev_char == 'L' {
                self.string_with_type(TokenType::U32StringLiteral);
                return;
            } else if prev_char == 'u' {
                if self.current > 2
                    && self.source.chars().nth(self.current - 3).unwrap_or('\0') == 'u'
                    && self.source.chars().nth(self.current - 2).unwrap_or('\0') == '8'
                {
                    // Handle u8"string"
                    self.string_with_type(TokenType::U8StringLiteral);
                    return;
                }
                // Handle u"string"
                self.string_with_type(TokenType::UStringLiteral);
                return;
            } else if prev_char == 'U' {
                // Handle U"string"
                self.string_with_type(TokenType::U16StringLiteral);
                return;
            }
        }
        // Regular string
        self.string();
    }

    /// Handles character literals
    pub(crate) fn handle_char_literal(&mut self) {
        // Check for prefixed char literals: L'x', u'x', U'x'
        if self.current > 1 {
            let prev_char = self.source.chars().nth(self.current - 2).unwrap_or('\0');
            if prev_char == 'L' {
                self.char_literal_with_type(TokenType::WideLiteral);
                return;
            } else if prev_char == 'u' {
                self.char_literal_with_type(TokenType::UCharLiteral);
                return;
            } else if prev_char == 'U' {
                // Handle U'x' - C11 Unicode character literal
                self.char_literal_with_type(TokenType::U16StringLiteral);
                return;
            }
        }
        // Regular character
        self.char_literal();
    }

    /// Process a standard string literal
    pub(crate) fn string(&mut self) {
        // Process characters until we reach the closing quote or end of file
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.advance();
        }

        // Unterminated string
        if self.is_at_end() {
            // Report error: Unterminated string
            self.add_token(TokenType::Error);
            return;
        }

        // Consume the closing "
        self.advance();

        // Extract the string value (without the quotes)
        let value = self.source[self.start + 1..self.current - 1].to_string();
        
        // Process escape sequences
        let processed_value = self.process_escape_sequences(value);
        
        // Add the token with the processed value
        self.add_token_with_literal(TokenType::StringLiteral, processed_value);
    }

    /// Process a character literal
    pub(crate) fn char_literal(&mut self) {
        // Process characters until we reach the closing quote or end of file
        while self.peek() != '\'' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.advance();
        }

        // Unterminated character literal
        if self.is_at_end() {
            // Report error: Unterminated character literal
            self.add_token(TokenType::Error);
            return;
        }

        // Consume the closing '
        self.advance();

        // Extract the character value (without the quotes)
        let value = self.source[self.start + 1..self.current - 1].to_string();
        
        // Process escape sequences
        let processed_value = self.process_escape_sequences(value);
        
        // Validate character literal length
        if processed_value.chars().count() != 1 && !processed_value.is_empty() {
            // Multi-character literals are allowed in C but implementation-defined
            // We'll accept them but might want to warn
        }
        
        self.add_token_with_literal(TokenType::CharLiteral, processed_value);
    }

    /// Process a string literal with a specific type (L, u, U, u8)
    pub(crate) fn string_with_type(&mut self, token_type: TokenType) {
        // Adjust start to include the prefix
        let prefix_len = match token_type {
            TokenType::U8StringLiteral => 3, // u8"
            _ => 2,                          // L", u", U"
        };
        
        let original_start = self.start;
        self.start = self.start - prefix_len + 1; // +1 to account for the opening quote

        // Process characters until we reach the closing quote or end of file
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.advance();
        }

        // Unterminated string
        if self.is_at_end() {
            // Report error: Unterminated string
            self.add_token(TokenType::Error);
            self.start = original_start; // Restore original start position
            return;
        }

        // Consume the closing "
        self.advance();

        // Extract the string value (without the quotes and prefix)
        let value = self.source[self.start + prefix_len..self.current - 1].to_string();
        
        // Process escape sequences
        let processed_value = self.process_escape_sequences(value);
        
        // Add the token with the processed value
        self.add_token_with_literal(token_type, processed_value);
        
        // Restore original start position
        self.start = original_start;
    }

    /// Process a character literal with a specific type (L, u, U)
    pub(crate) fn char_literal_with_type(&mut self, token_type: TokenType) {
        // Adjust start to include the prefix
        let original_start = self.start;
        self.start = self.start - 1; // Adjust for the prefix (L, u, U)

        // Process characters until we reach the closing quote or end of file
        while self.peek() != '\'' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.advance();
        }

        // Unterminated character literal
        if self.is_at_end() {
            // Report error: Unterminated character literal
            self.add_token(TokenType::Error);
            self.start = original_start; // Restore original start position
            return;
        }

        // Consume the closing '
        self.advance();

        // Extract the character value (without the quotes and prefix)
        let value = self.source[self.start + 2..self.current - 1].to_string();
        
        // Process escape sequences
        let processed_value = self.process_escape_sequences(value);
        
        // Add the token with the processed value
        self.add_token_with_literal(token_type, processed_value);
        
        // Restore original start position
        self.start = original_start;
    }

    /// Process escape sequences in string and character literals
    /// Handles all standard C escape sequences including hex, octal, and Unicode
    fn process_escape_sequences(&self, value: String) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c == '\\' {
                // Handle escape sequence
                match chars.next() {
                    Some('a') => result.push('\x07'), // Bell (alert)
                    Some('b') => result.push('\x08'), // Backspace
                    Some('f') => result.push('\x0C'), // Form feed
                    Some('n') => result.push('\n'),   // Line feed
                    Some('r') => result.push('\r'),   // Carriage return
                    Some('t') => result.push('\t'),   // Horizontal tab
                    Some('v') => result.push('\x0B'), // Vertical tab
                    Some('\\') => result.push('\\'),  // Backslash
                    Some('\'') => result.push('\''),  // Single quote
                    Some('"') => result.push('"'),    // Double quote
                    Some('?') => result.push('?'),    // Question mark
                    
                    // Octal escape sequence \ooo
                    Some(c) if is_octal_digit(c) => {
                        let mut octal = String::new();
                        octal.push(c);
                        
                        // Read up to 2 more octal digits
                        for _ in 0..2 {
                            if let Some(&next) = chars.peek() {
                                if is_octal_digit(next) {
                                    octal.push(next);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        
                        // Convert octal to character
                        if let Ok(val) = u32::from_str_radix(&octal, 8) {
                            if let Some(c) = std::char::from_u32(val) {
                                result.push(c);
                            } else {
                                // Invalid Unicode code point
                                result.push('\u{FFFD}'); // Unicode replacement character
                            }
                        }
                    }
                    
                    // Hexadecimal escape sequence \xhh
                    Some('x') => {
                        let mut hex = String::new();
                        
                        // Read hex digits
                        while let Some(&next) = chars.peek() {
                            if is_hex_digit(next) {
                                hex.push(next);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        
                        if !hex.is_empty() {
                            // Convert hex to character
                            if let Ok(val) = u32::from_str_radix(&hex, 16) {
                                if let Some(c) = std::char::from_u32(val) {
                                    result.push(c);
                                } else {
                                    // Invalid Unicode code point
                                    result.push('\u{FFFD}'); // Unicode replacement character
                                }
                            }
                        } else {
                            // Invalid hex escape sequence
                            result.push('x');
                        }
                    }
                    
                    // Unicode escape sequences \uhhhh and \Uhhhhhhhh (C11)
                    Some('u') => {
                        // \uhhhh - 4 hex digits for 16-bit Unicode code point
                        let mut hex = String::new();
                        
                        // Read exactly 4 hex digits
                        for _ in 0..4 {
                            if let Some(&next) = chars.peek() {
                                if is_hex_digit(next) {
                                    hex.push(next);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        
                        if hex.len() == 4 {
                            // Convert hex to character
                            if let Ok(val) = u32::from_str_radix(&hex, 16) {
                                if let Some(c) = std::char::from_u32(val) {
                                    result.push(c);
                                } else {
                                    // Invalid Unicode code point
                                    result.push('\u{FFFD}'); // Unicode replacement character
                                }
                            }
                        } else {
                            // Invalid Unicode escape sequence
                            result.push('u');
                            for c in hex.chars() {
                                result.push(c);
                            }
                        }
                    }
                    
                    Some('U') => {
                        // \Uhhhhhhhh - 8 hex digits for 32-bit Unicode code point
                        let mut hex = String::new();
                        
                        // Read exactly 8 hex digits
                        for _ in 0..8 {
                            if let Some(&next) = chars.peek() {
                                if is_hex_digit(next) {
                                    hex.push(next);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        
                        if hex.len() == 8 {
                            // Convert hex to character
                            if let Ok(val) = u32::from_str_radix(&hex, 16) {
                                if let Some(c) = std::char::from_u32(val) {
                                    result.push(c);
                                } else {
                                    // Invalid Unicode code point
                                    result.push('\u{FFFD}'); // Unicode replacement character
                                }
                            }
                        } else {
                            // Invalid Unicode escape sequence
                            result.push('U');
                            for c in hex.chars() {
                                result.push(c);
                            }
                        }
                    }
                    
                    // Unknown escape sequence - C standard says to just output the character
                    Some(c) => result.push(c),
                    None => result.push('\\'), // Trailing backslash
                }
            } else {
                // Regular character
                result.push(c);
            }
        }
        
        result
    }

    /// Handles numeric literals (integer or floating-point)
    pub(crate) fn handle_number(&mut self) {
        // Check for hex, octal, or binary literals
        if self.peek() == 'x' || self.peek() == 'X' {
            // Hex literal (0x...)
            self.advance(); // Consume 'x'

            // Read hex digits
            while is_hex_digit(self.peek()) {
                self.advance();
            }

            // Extract the value
            let value = self.source[self.start..self.current].to_string();
            self.add_token_with_literal(TokenType::IntegerLiteral, value);
            return;
        } else if self.peek() == 'b' || self.peek() == 'B' {
            // Binary literal (0b...)
            self.advance(); // Consume 'b'

            // Read binary digits
            while self.peek() == '0' || self.peek() == '1' {
                self.advance();
            }

            // Extract the value
            let value = self.source[self.start..self.current].to_string();
            self.add_token_with_literal(TokenType::IntegerLiteral, value);
            return;
        } else if self.source.chars().nth(self.start).unwrap() == '0' && is_octal_digit(self.peek())
        {
            // Octal literal (0...)
            // Read octal digits
            while is_octal_digit(self.peek()) {
                self.advance();
            }

            // Extract the value
            let value = self.source[self.start..self.current].to_string();
            self.add_token_with_literal(TokenType::IntegerLiteral, value);
            return;
        }

        // Decimal literal
        while is_digit(self.peek()) {
            self.advance();
        }

        // Track if this is a floating-point number
        let mut is_float = false;

        // Look for a decimal point
        if self.peek() == '.' && is_digit(self.peek_next()) {
            is_float = true;
            self.advance(); // Consume the '.'

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        // Look for an exponent
        if self.peek() == 'e' || self.peek() == 'E' {
            is_float = true;
            self.advance(); // Consume 'e'

            // Optional sign
            if self.peek() == '+' || self.peek() == '-' {
                self.advance();
            }

            // Exponent digits
            if is_digit(self.peek()) {
                while is_digit(self.peek()) {
                    self.advance();
                }
            } else {
                // Invalid exponent
                self.add_token(TokenType::Error);
                return;
            }
        }

        // Look for a float type suffix (F, L)
        if self.peek() == 'f' || self.peek() == 'F' || self.peek() == 'l' || self.peek() == 'L' {
            is_float = true;
            self.advance();
        }
        // Look for an integer type suffix (U, L, UL, LL, etc.)
        else if self.peek() == 'u' || self.peek() == 'U' {
            self.advance();
            if self.peek() == 'l' || self.peek() == 'L' {
                self.advance();
                if self.peek() == 'l' || self.peek() == 'L' {
                    self.advance();
                }
            }
        } else if self.peek() == 'l' || self.peek() == 'L' {
            self.advance();
            if self.peek() == 'l' || self.peek() == 'L' {
                self.advance();
            }
            if self.peek() == 'u' || self.peek() == 'U' {
                self.advance();
            }
        }

        // Extract the value
        let value = self.source[self.start..self.current].to_string();

        // Add the token with the appropriate type
        if is_float {
            self.add_token_with_literal(TokenType::FloatLiteral, value);
        } else {
            self.add_token_with_literal(TokenType::IntegerLiteral, value);
        }
    }
}
