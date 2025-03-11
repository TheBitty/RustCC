pub struct Token {
    pub token_type: TokenType,   // The kind of token (from your enum)
    pub lexeme: String,          // The actual text from the source code
    pub line: usize,             // Line where the token appears
    pub column: usize,           // Column where the token appears
    pub literal: Option<String>, // Optional literal value for constants/strings
}
