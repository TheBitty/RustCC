#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,   // The kind of token (from your enum)
    pub lexeme: String,          // The actual text from the source code
    pub line: usize,             // Line where the token appears
    pub column: usize,           // Column where the token appears
    pub literal: Option<String>, // Optional literal value for constants/strings
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum TokenType {
    // Keywords
    Int,
    Char,
    Void,
    If,
    Else,
    While,
    For,
    Return,
    Break,
    Continue,
    Struct,

    // Identifiers and literals
    Identifier,
    IntegerLiteral,
    CharLiteral,
    StringLiteral,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent, // Arithmetic operators
    Equal,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual, // Assignment operators
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual, // Comparison operators
    And,
    Or,
    Bang, // Logical operators
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    ShiftLeft,
    ShiftRight, // Bitwise operators

    // Delimiters
    LeftParen,
    RightParen, // ( )
    LeftBrace,
    RightBrace, // { }
    LeftBracket,
    RightBracket, // [ ]
    Semicolon,
    Comma,
    Dot, // ; , .

    // Special tokens
    EOF,
    Error,
}
