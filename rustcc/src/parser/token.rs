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
    // Keywords - Basic types
    Int,
    Char,
    Short,
    Long,
    Float,
    Double,
    Void,
    Bool,      // _Bool in C99+
    Complex,   // _Complex in C99+
    Imaginary, // _Imaginary in C99+

    // Type qualifiers
    Const,
    Volatile,
    Restrict, // C99+
    Atomic,   // C11+

    // Storage class specifiers
    Auto,
    Register,
    Static,
    Extern,
    Typedef,
    ThreadLocal, // _Thread_local in C11+

    // Control flow
    If,
    Else,
    While,
    For,
    Return,
    Break,
    Continue,
    Switch,
    Case,
    Default,
    Do,
    Goto,

    // Other keywords
    Sizeof,
    Alignas,      // C11+
    Alignof,      // C11+
    Generic,      // _Generic in C11+
    Noreturn,     // _Noreturn in C11+
    StaticAssert, // _Static_assert in C11+

    // Struct/Union
    Struct,
    Union,
    Enum,

    // Function specifiers
    Inline, // C99+

    // Unsigned type specifier
    Unsigned,
    Signed,

    // Identifiers and literals
    Identifier,
    IntegerLiteral,
    FloatLiteral,
    CharLiteral,
    StringLiteral,

    // Special string literals
    UCharLiteral,     // u'x' - C11+
    UStringLiteral,   // u"..." - C11+
    U8StringLiteral,  // u8"..." - C11+
    U16StringLiteral, // U"..." - C11+
    U32StringLiteral, // L"..." - wide string
    WideLiteral,      // L'x' - wide char

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
    Question,   // Ternary operator ?

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

    // Preprocessor token operators
    PPHashHash, // ## token concatenation
    PPHash,     // # stringification

    // Delimiters
    LeftParen,
    RightParen, // ( )
    LeftBrace,
    RightBrace, // { }
    LeftBracket,
    RightBracket, // [ ]
    Semicolon,
    Comma,
    Dot,      // ; , .
    Colon,    // :
    Ellipsis, // ... for variadic functions

    // Preprocessor directives
    Hash, // # symbol at start of line
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
    PPLine, // #line directive
    
    // C23 preprocessor directives
    PPEmbed,    // #embed directive for embedding binary files
    PPElifDef,  // #elifdef directive
    PPElifNDef, // #elifndef directive

    // Special tokens
    Eof,
    Error,

    // Aliases for logical operators
    LogicalOr,  // Alias for Or
    LogicalAnd, // Alias for And

    // Aliases for bitwise operators
    BitwiseOr,  // Alias for Pipe
    BitwiseAnd, // Alias for Ampersand
    BitwiseXor, // Alias for Caret

    // Aliases for shift operators
    LeftShift,  // Alias for ShiftLeft
    RightShift, // Alias for ShiftRight

    // Aliases for increment/decrement operators
    PlusPlus,   // Alias for Increment
    MinusMinus, // Alias for Decrement
}

// Implement aliases for token types to maintain compatibility
impl TokenType {
    pub fn is_logical_or(&self) -> bool {
        matches!(self, TokenType::Or | TokenType::LogicalOr)
    }

    pub fn is_logical_and(&self) -> bool {
        matches!(self, TokenType::And | TokenType::LogicalAnd)
    }

    pub fn is_bitwise_or(&self) -> bool {
        matches!(self, TokenType::Pipe | TokenType::BitwiseOr)
    }

    pub fn is_bitwise_and(&self) -> bool {
        matches!(self, TokenType::Ampersand | TokenType::BitwiseAnd)
    }

    pub fn is_bitwise_xor(&self) -> bool {
        matches!(self, TokenType::Caret | TokenType::BitwiseXor)
    }

    pub fn is_left_shift(&self) -> bool {
        matches!(self, TokenType::ShiftLeft | TokenType::LeftShift)
    }

    pub fn is_right_shift(&self) -> bool {
        matches!(self, TokenType::ShiftRight | TokenType::RightShift)
    }

    pub fn is_increment(&self) -> bool {
        matches!(self, TokenType::Increment | TokenType::PlusPlus)
    }

    pub fn is_decrement(&self) -> bool {
        matches!(self, TokenType::Decrement | TokenType::MinusMinus)
    }
}
