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
    // Flag to indicate if we're at the start of a line (for preprocessor directives)
    at_line_start: bool,
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
        keywords.insert("switch".to_string(), TokenType::Switch);
        keywords.insert("case".to_string(), TokenType::Case);
        keywords.insert("default".to_string(), TokenType::Default);
        keywords.insert("do".to_string(), TokenType::Do);
        keywords.insert("sizeof".to_string(), TokenType::Sizeof);
        keywords.insert("const".to_string(), TokenType::Const);

        // Preprocessor keywords
        keywords.insert("include".to_string(), TokenType::PPInclude);
        keywords.insert("define".to_string(), TokenType::PPDefine);
        keywords.insert("undef".to_string(), TokenType::PPUndef);
        keywords.insert("ifdef".to_string(), TokenType::PPIfDef);
        keywords.insert("ifndef".to_string(), TokenType::PPIfNDef);
        keywords.insert("if".to_string(), TokenType::PPIf);
        keywords.insert("else".to_string(), TokenType::PPElse);
        keywords.insert("elif".to_string(), TokenType::PPElif);
        keywords.insert("endif".to_string(), TokenType::PPEndif);
        keywords.insert("pragma".to_string(), TokenType::PPPragma);
        keywords.insert("error".to_string(), TokenType::PPErrorDir);
        keywords.insert("warning".to_string(), TokenType::PPWarning);

        Lexer {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            keywords,
            at_line_start: true,
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

        // Check for preprocessor directive at the start of a line
        if c == '#' && self.at_line_start {
            self.preprocessor_directive();
            return;
        }

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
            ':' => self.add_token(TokenType::Colon),
            '+' => {
                if self.match_char('=') {
                    self.add_token(TokenType::PlusEqual)
                } else if self.match_char('+') {
                    self.add_token(TokenType::Increment)
                } else {
                    self.add_token(TokenType::Plus)
                }
            }
            '-' => {
                if self.match_char('=') {
                    self.add_token(TokenType::MinusEqual)
                } else if self.match_char('-') {
                    self.add_token(TokenType::Decrement)
                } else if self.match_char('>') {
                    self.add_token(TokenType::Arrow)
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
                            self.at_line_start = true;
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
                    if self.match_char('=') {
                        self.add_token(TokenType::ShiftLeftEqual)
                    } else {
                        self.add_token(TokenType::ShiftLeft)
                    }
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else if self.match_char('>') {
                    if self.match_char('=') {
                        self.add_token(TokenType::ShiftRightEqual)
                    } else {
                        self.add_token(TokenType::ShiftRight)
                    }
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '&' => {
                if self.match_char('&') {
                    self.add_token(TokenType::And)
                } else if self.match_char('=') {
                    self.add_token(TokenType::AmpersandEqual)
                } else {
                    self.add_token(TokenType::Ampersand)
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.add_token(TokenType::Or)
                } else if self.match_char('=') {
                    self.add_token(TokenType::PipeEqual)
                } else {
                    self.add_token(TokenType::Pipe)
                }
            }
            '^' => {
                if self.match_char('=') {
                    self.add_token(TokenType::CaretEqual)
                } else {
                    self.add_token(TokenType::Caret)
                }
            }
            '~' => self.add_token(TokenType::Tilde),
            ' ' | '\r' | '\t' => {
                // Whitespace doesn't affect line start status
            }
            '\n' => {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
            }
            '"' => self.string(),
            '\'' => self.char_literal(),
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

        // After processing a non-whitespace character, we're no longer at the start of a line
        if c != ' ' && c != '\r' && c != '\t' && c != '\n' {
            self.at_line_start = false;
        }
    }

    fn preprocessor_directive(&mut self) {
        // Add the # token
        self.add_token(TokenType::Hash);

        // Skip whitespace
        while self.peek() == ' ' || self.peek() == '\t' {
            self.advance();
        }

        // Mark the start of the directive name
        self.start = self.current;

        // Read the directive name
        while self.is_alpha(self.peek()) {
            self.advance();
        }

        // Get the directive name
        let directive = self.source[self.start..self.current].to_string();

        // Check if it's a known preprocessor directive
        match directive.as_str() {
            "include" => self.add_token(TokenType::PPInclude),
            "define" => self.add_token(TokenType::PPDefine),
            "undef" => self.add_token(TokenType::PPUndef),
            "ifdef" => self.add_token(TokenType::PPIfDef),
            "ifndef" => self.add_token(TokenType::PPIfNDef),
            "if" => self.add_token(TokenType::PPIf),
            "else" => self.add_token(TokenType::PPElse),
            "elif" => self.add_token(TokenType::PPElif),
            "endif" => self.add_token(TokenType::PPEndif),
            "pragma" => self.add_token(TokenType::PPPragma),
            "error" => self.add_token(TokenType::PPErrorDir),
            "warning" => self.add_token(TokenType::PPWarning),
            _ => self.add_token(TokenType::Error),
        }

        // For include directives, handle the path
        if directive == "include" {
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
        // For define directives, handle the macro name and value
        else if directive == "define" {
            // Skip whitespace
            while self.peek() == ' ' || self.peek() == '\t' {
                self.advance();
            }

            // Read the macro name
            self.start = self.current;
            while self.is_alphanumeric(self.peek()) {
                self.advance();
            }

            if self.start < self.current {
                let name = self.source[self.start..self.current].to_string();
                self.add_token_with_literal(TokenType::Identifier, name);

                // Skip whitespace
                while self.peek() == ' ' || self.peek() == '\t' {
                    self.advance();
                }

                // Read the macro value (rest of the line)
                self.start = self.current;
                while !self.is_at_end() && self.peek() != '\n' {
                    self.advance();
                }

                if self.start < self.current {
                    let value = self.source[self.start..self.current].to_string();
                    self.add_token_with_literal(TokenType::StringLiteral, value);
                }
            } else {
                self.add_token(TokenType::Error);
            }
        }
        // For other directives, just consume the rest of the line
        else {
            while !self.is_at_end() && self.peek() != '\n' {
                self.advance();
            }
        }
    }

    fn string(&mut self) {
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
                            if self.is_hex_digit(self.peek()) {
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
                            if self.is_octal_digit(self.peek()) {
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

    fn char_literal(&mut self) {
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

    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == '_'
    }

    fn is_alphanumeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn is_hex_digit(&self, c: char) -> bool {
        c.is_ascii_digit() || c.is_ascii_hexdigit()
    }

    fn is_octal_digit(&self, c: char) -> bool {
        c.is_ascii_digit() && c <= '7'
    }

    fn number(&mut self) {
        // Check for hex, octal, or binary literals
        if self.peek() == 'x' || self.peek() == 'X' {
            // Hex literal (0x...)
            self.advance(); // Consume 'x'

            // Read hex digits
            while self.is_hex_digit(self.peek()) {
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
            && self.is_octal_digit(self.peek())
        {
            // Octal literal (0...)
            // Read octal digits
            while self.is_octal_digit(self.peek()) {
                self.advance();
            }

            // Extract the value
            let value = self.source[self.start..self.current].to_string();
            self.add_token_with_literal(TokenType::IntegerLiteral, value);
            return;
        }

        // Decimal literal
        while self.is_digit(self.peek()) {
            self.advance();
        }

        // Look for a decimal point
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance(); // Consume the '.'

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        // Look for an exponent
        if self.peek() == 'e' || self.peek() == 'E' {
            self.advance(); // Consume 'e'

            // Optional sign
            if self.peek() == '+' || self.peek() == '-' {
                self.advance();
            }

            // Exponent digits
            if self.is_digit(self.peek()) {
                while self.is_digit(self.peek()) {
                    self.advance();
                }
            } else {
                // Invalid exponent
                self.add_token(TokenType::Error);
                return;
            }
        }

        // Look for a type suffix (U, L, UL, LL, etc.)
        if self.peek() == 'u' || self.peek() == 'U' {
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
        self.add_token_with_literal(TokenType::IntegerLiteral, value);
    }

    fn identifier(&mut self) {
        while self.is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = self.source[self.start..self.current].to_string();

        // Check if it's a keyword
        if let Some(token_type) = self.keywords.get(&text) {
            self.add_token(token_type.clone());
        } else {
            self.add_token_with_literal(TokenType::Identifier, text);
        }
    }
}
