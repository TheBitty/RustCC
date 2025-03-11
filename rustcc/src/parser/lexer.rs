use crate::parser::token::{Token, TokenType};

pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Lexer {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
        }
    }
    
    // This will be the main method to scan all tokens from the source
    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: String::new(),
            line: self.line,
            column: self.column,
            literal: None,
        });
        
        &self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            // ... add more cases for other tokens
            ' ' | '\r' | '\t' => (), // Ignore whitespace
            '\n' => {
                self.line += 1;
                self.column = 1;
            },
            _ => {
                if self.is_digit(c) {
                    self.number();
                } else if self.is_alpha(c) {
                    self.identifier();
                } else {
                    // Handle error or unexpected character
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        self.column += 1;
        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        let lexeme = self.source[self.start..self.current].to_string();
        self.tokens.push(Token {
            token_type,
            lexeme,
            line: self.line,
            column: self.column - (self.current - self.start),
            literal: None,
        });
    }

    // Helper methods for identifying characters
    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    // Methods for handling specific token types
    fn number(&mut self) {
        // TODO: Implement number scanning
    }

    fn identifier(&mut self) {
        // TODO: Implement identifier scanning
    }
}

