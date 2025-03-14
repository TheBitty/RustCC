// scanner.rs
// Core scanning functionality for the lexer

use crate::parser::token::{Token, TokenType};
use crate::parser::lexer::utils::{is_alpha, is_digit, is_alphanumeric, is_whitespace};
use crate::parser::lexer::Lexer;

impl Lexer {
    /// Returns whether the scanner has reached the end of the source
    pub(crate) fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    /// Advances the current position and returns the current character
    pub(crate) fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        self.column += 1;
        c
    }

    /// Consumes the next character if it matches the expected character
    pub(crate) fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        self.column += 1;
        true
    }

    /// Peeks at the current character without advancing the position
    pub(crate) fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    /// Peeks at the next character without advancing the position
    pub(crate) fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
    }

    /// Adds a token with the given type
    pub(crate) fn add_token(&mut self, token_type: TokenType) {
        let lexeme = self.source[self.start..self.current].to_string();
        self.tokens.push(Token {
            token_type,
            lexeme,
            line: self.line,
            column: self.column - (self.current - self.start),
            literal: None,
        });
    }

    /// Adds a token with the given type and literal value
    pub(crate) fn add_token_with_literal(&mut self, token_type: TokenType, literal: String) {
        let lexeme = self.source[self.start..self.current].to_string();
        self.tokens.push(Token {
            token_type,
            lexeme,
            line: self.line,
            column: self.column - (self.current - self.start),
            literal: Some(literal),
        });
    }

    /// Scans all tokens from the source and returns them as a vector
    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            line: self.line,
            column: self.column,
            literal: None,
        });

        &self.tokens
    }

    /// Scans a single token from the source
    fn scan_token(&mut self) {
        let c = self.advance();

        // Check for preprocessor directive at the start of a line
        if c == '#' && self.at_line_start {
            self.preprocessor_directive();
            return;
        }

        match c {
            // Single-character tokens
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ';' => self.add_token(TokenType::Semicolon),
            ',' => self.add_token(TokenType::Comma),
            
            // Potentially multi-character tokens
            '.' => self.handle_dot(),
            ':' => self.add_token(TokenType::Colon),
            '?' => self.add_token(TokenType::Question),
            '+' => self.handle_plus(),
            '-' => self.handle_minus(),
            '*' => self.handle_star(),
            '/' => self.handle_slash(),
            '%' => self.handle_percent(),
            '=' => self.handle_equal(),
            '!' => self.handle_bang(),
            '<' => self.handle_less(),
            '>' => self.handle_greater(),
            '&' => self.handle_ampersand(),
            '|' => self.handle_pipe(),
            '^' => self.handle_caret(),
            '~' => self.add_token(TokenType::Tilde),
            '#' => self.handle_hash(),
            
            // Whitespace
            ' ' | '\r' | '\t' => {
                // Whitespace doesn't affect line start status
            },
            '\n' => {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
            },
            
            // Literals and identifiers
            '"' => self.handle_string_literal(),
            '\'' => self.handle_char_literal(),
            _ => {
                if is_digit(c) {
                    self.handle_number();
                } else if is_alpha(c) {
                    self.handle_identifier();
                } else {
                    self.add_token(TokenType::Error);
                }
            }
        }

        // After processing a non-whitespace character, we're no longer at the start of a line
        if c != ' ' && c != '\r' && c != '\t' && c != '\n' {
            self.at_line_start = false;
        }
    }
} 