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
        // TODO: Implement the scanning logic
        // This will be implemented in the next step
        
        // Add EOF token at the end
        self.tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: String::new(),
            line: self.line,
            column: self.column,
            literal: None,
        });
        
        &self.tokens
    }
}

