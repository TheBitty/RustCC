use crate::parser::token::{Token, TokenType};
use std::collections::HashMap;

pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    keywords: HashMap<String, TokenType>,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("int".to_string(), TokenType::Int);
        keywords.insert("char".to_string(), TokenType::Char);
        keywords.insert("void".to_string(), TokenType::Void);
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("else".to_string(), TokenType::Else);
        keywords.insert("while".to_string(), TokenType::While);
        keywords.insert("for".to_string(), TokenType::For);
        keywords.insert("return".to_string(), TokenType::Return);
        keywords.insert("break".to_string(), TokenType::Break);
        keywords.insert("continue".to_string(), TokenType::Continue);
        keywords.insert("struct".to_string(), TokenType::Struct);

        Lexer {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            keywords,
        }
    }

    // This will be the main method to scan all tokens from the source
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
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ';' => self.add_token(TokenType::Semicolon),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '+' => {
                if self.match_char('=') {
                    self.add_token(TokenType::PlusEqual)
                } else {
                    self.add_token(TokenType::Plus)
                }
            }
            '-' => {
                if self.match_char('=') {
                    self.add_token(TokenType::MinusEqual)
                } else {
                    self.add_token(TokenType::Minus)
                }
            }
            '*' => {
                if self.match_char('=') {
                    self.add_token(TokenType::StarEqual)
                } else {
                    self.add_token(TokenType::Star)
                }
            }
            '/' => {
                if self.match_char('/') {
                    // A comment goes until the end of the line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    // A block comment goes until */
                    while !(self.is_at_end() || self.peek() == '*' && self.peek_next() == '/') {
                        if self.peek() == '\n' {
                            self.line += 1;
                            self.column = 0;
                        }
                        self.advance();
                    }

                    // Consume the */
                    if !self.is_at_end() {
                        self.advance(); // *
                        self.advance(); // /
                    }
                } else if self.match_char('=') {
                    self.add_token(TokenType::SlashEqual)
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            '%' => {
                if self.match_char('=') {
                    self.add_token(TokenType::PercentEqual)
                } else {
                    self.add_token(TokenType::Percent)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual)
                } else if self.match_char('<') {
                    self.add_token(TokenType::ShiftLeft)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else if self.match_char('>') {
                    self.add_token(TokenType::ShiftRight)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '&' => {
                if self.match_char('&') {
                    self.add_token(TokenType::And)
                } else {
                    self.add_token(TokenType::Ampersand)
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.add_token(TokenType::Or)
                } else {
                    self.add_token(TokenType::Pipe)
                }
            }
            '^' => self.add_token(TokenType::Caret),
            '~' => self.add_token(TokenType::Tilde),
            ' ' | '\r' | '\t' => (), // Ignore whitespace
            '\n' => {
                self.line += 1;
                self.column = 1;
            }
            _ => {
                if self.is_digit(c) {
                    self.number();
                } else if self.is_alpha(c) {
                    self.identifier();
                } else {
                    self.add_token(TokenType::Error);
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

    fn match_char(&mut self, expected: char) -> bool {
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

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
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

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: String) {
        let lexeme = self.source[self.start..self.current].to_string();
        self.tokens.push(Token {
            token_type,
            lexeme,
            line: self.line,
            column: self.column - (self.current - self.start),
            literal: Some(literal),
        });
    }

    // Helper methods for identifying characters
    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alphanumeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    // Methods for handling specific token types
    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        let literal = self.source[self.start..self.current].to_string();
        self.add_token_with_literal(TokenType::IntegerLiteral, literal);
    }

    fn identifier(&mut self) {
        while !self.is_at_end() && self.is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = self.source[self.start..self.current].to_string();
        let token_type = self
            .keywords
            .get(&text)
            .cloned()
            .unwrap_or(TokenType::Identifier);

        if token_type == TokenType::Identifier {
            self.add_token_with_literal(token_type, text.clone());
        } else {
            self.add_token(token_type);
        }
    }
}
