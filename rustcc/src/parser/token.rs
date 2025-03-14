#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,   // The kind of token (from your enum)
    pub lexeme: String,          // The actual text from the source code
    pub line: usize,             // Line where the token appears
    pub column: usize,           // Column where the token appears
    pub literal: Option<String>, // Optional literal value for constants/strings
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
    Switch,
    Case,
    Default,
    Do,
    Sizeof,
    Const,  // Added for const qualifier

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

    // Compound assignment operators
    ShiftLeftEqual,
    ShiftRightEqual,
    AmpersandEqual,
    PipeEqual,
    CaretEqual,

    // Increment/decrement operators
    Increment, // ++
    Decrement, // --

    // Structure access
    Arrow, // ->

    // Delimiters
    LeftParen,
    RightParen, // ( )
    LeftBrace,
    RightBrace, // { }
    LeftBracket,
    RightBracket, // [ ]
    Semicolon,
    Comma,
    Dot,   // ; , .
    Colon, // :

    // Preprocessor directives
    Hash, // # symbol
    PPInclude,
    PPDefine,
    PPUndef,
    PPIfDef,
    PPIfNDef,
    PPIf,
    PPElse,
    PPElif,
    PPEndif,
    PPPragma,
    PPErrorDir,
    PPWarning,

    // Special tokens
    Eof,
    Error,
}
