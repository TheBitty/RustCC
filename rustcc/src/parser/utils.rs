use crate::parser::Parser;
use crate::parser::token::{Token, TokenType};

impl Parser {
    // Helper method to synchronize after error
    #[allow(dead_code)]
    pub fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Int
                | TokenType::Void
                | TokenType::Char
                | TokenType::If
                | TokenType::While
                | TokenType::For
                | TokenType::Return
                | TokenType::Struct => return,
                _ => {}
            }

            self.advance();
        }
    }

    // Helper methods
    pub fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn match_any(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t.clone()) {
                return true;
            }
        }
        false
    }

    pub fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    pub fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    pub fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    pub fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    pub fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(format!("{} at line {}", message, self.peek().line))
        }
    }

    // Check if the current token is any of the given types
    pub fn check_any(&self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t.clone()) {
                return true;
            }
        }
        false
    }

    // Check if the current token is at a new line (based on line number)
    pub fn is_at_new_line(&self) -> bool {
        if self.current > 0 && self.current < self.tokens.len() {
            let current_line = self.tokens[self.current].line;
            let prev_line = self.tokens[self.current - 1].line;
            return current_line > prev_line;
        }
        false
    }
} 