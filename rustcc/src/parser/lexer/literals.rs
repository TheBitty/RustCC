// literals.rs
// Handling of string, character, and numeric literals

use crate::parser::token::TokenType;
use crate::parser::lexer::Lexer;
use crate::parser::lexer::utils::{is_digit, is_hex_digit, is_octal_digit, is_whitespace};

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
                if self.current > 2 && self.source.chars().nth(self.current - 3).unwrap_or('\0') == 'u' 
                   && self.source.chars().nth(self.current - 2).unwrap_or('\0') == '8' {
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
        // Check for prefixed char literals: L'x', u'x'
        if self.current > 1 {
            let prev_char = self.source.chars().nth(self.current - 2).unwrap_or('\0');
            if prev_char == 'L' {
                self.char_literal_with_type(TokenType::WideLiteral);
                return;
            } else if prev_char == 'u' {
                self.char_literal_with_type(TokenType::UCharLiteral);
                return;
            }
        }
        // Regular char literal
        self.char_literal();
    }

    /// Handles a standard string literal
    pub(crate) fn string(&mut self) {
        // Read until closing quote or end of file
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
            }

            // Handle escape sequences
            if self.peek() == '\\' && !self.is_at_end() {
                self.advance(); // Consume the backslash

                // Handle common escape sequences
                match self.peek() {
                    'n' | 'r' | 't' | '\\' | '"' | '\'' => {
                        self.advance();
                    }
                    'x' => {
                        // Hex escape sequence \xHH
                        self.advance(); // Consume 'x'
                        // Read up to 2 hex digits
                        for _ in 0..2 {
                            if is_hex_digit(self.peek()) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    '0'..='7' => {
                        // Octal escape sequence \OOO
                        // Read up to 3 octal digits
                        for _ in 0..3 {
                            if is_octal_digit(self.peek()) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    _ => {
                        // Invalid escape sequence, but we'll just consume it
                        self.advance();
                    }
                }
            } else {
                self.advance();
            }
        }

        if self.is_at_end() {
            // Unterminated string
            self.add_token(TokenType::Error);
            return;
        }

        // Consume the closing quote
        self.advance();

        // Extract the string value (without the quotes)
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_literal(TokenType::StringLiteral, value);
    }

    /// Handles a standard character literal
    pub(crate) fn char_literal(&mut self) {
        // Read until closing quote or end of file
        while !self.is_at_end() && self.peek() != '\'' {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
            }

            // Handle escape sequences
            if self.peek() == '\\' && !self.is_at_end() {
                self.advance(); // Consume the backslash
                self.advance(); // Consume the escaped character
            } else {
                self.advance();
            }
        }

        if self.is_at_end() {
            // Unterminated character literal
            self.add_token(TokenType::Error);
            return;
        }

        // Consume the closing quote
        self.advance();

        // Extract the character value (without the quotes)
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_literal(TokenType::CharLiteral, value);
    }

    /// Handles a string literal with a specific type (prefixed string literal)
    pub(crate) fn string_with_type(&mut self, token_type: TokenType) {
        // This method handles prefixed string literals like L"string", u"string", etc.
        // The prefix and opening quote have already been consumed
        
        // Read until closing quote or end of file
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
            }

            // Handle escape sequences
            if self.peek() == '\\' && !self.is_at_end() {
                self.advance(); // Consume the backslash

                // Handle common escape sequences
                match self.peek() {
                    'n' | 'r' | 't' | '\\' | '"' | '\'' => {
                        self.advance();
                    }
                    'x' => {
                        // Hex escape sequence \xHH
                        self.advance(); // Consume 'x'
                        // Read up to 2 hex digits
                        for _ in 0..2 {
                            if is_hex_digit(self.peek()) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    'u' => {
                        // Unicode escape sequence \u{HHHHHH}
                        self.advance(); // Consume 'u'
                        if self.peek() == '{' {
                            self.advance(); // Consume '{'
                            // Read up to 6 hex digits
                            let mut count = 0;
                            while is_hex_digit(self.peek()) && count < 6 {
                                self.advance();
                                count += 1;
                            }
                            // Consume closing '}'
                            if self.peek() == '}' {
                                self.advance();
                            }
                        } else {
                            // 4-digit form \uHHHH
                            for _ in 0..4 {
                                if is_hex_digit(self.peek()) {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    'U' => {
                        // 8-digit Unicode escape \UHHHHHHHH
                        self.advance(); // Consume 'U'
                        // Read 8 hex digits
                        for _ in 0..8 {
                            if is_hex_digit(self.peek()) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    '0'..='7' => {
                        // Octal escape sequence \OOO
                        // Read up to 3 octal digits
                        for _ in 0..3 {
                            if is_octal_digit(self.peek()) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    _ => {
                        // Invalid escape sequence, but we'll just consume it
                        self.advance();
                    }
                }
            } else {
                self.advance();
            }
        }

        if self.is_at_end() {
            // Unterminated string
            self.add_token(TokenType::Error);
            return;
        }

        // Consume the closing quote
        self.advance();

        // Extract the string value (without the quotes)
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_literal(token_type, value);
    }

    /// Handles a character literal with a specific type (prefixed character literal)
    pub(crate) fn char_literal_with_type(&mut self, token_type: TokenType) {
        // This method handles prefixed char literals like L'x', u'x', etc.
        // The prefix and opening quote have already been consumed
        
        // Read until closing quote or end of file
        while !self.is_at_end() && self.peek() != '\'' {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
            }

            // Handle escape sequences
            if self.peek() == '\\' && !self.is_at_end() {
                self.advance(); // Consume the backslash
                self.advance(); // Consume the escaped character
            } else {
                self.advance();
            }
        }

        if self.is_at_end() {
            // Unterminated character literal
            self.add_token(TokenType::Error);
            return;
        }

        // Consume the closing quote
        self.advance();

        // Extract the character value (without the quotes)
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_literal(token_type, value);
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
        } else if self.source.chars().nth(self.start).unwrap() == '0'
            && is_octal_digit(self.peek())
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