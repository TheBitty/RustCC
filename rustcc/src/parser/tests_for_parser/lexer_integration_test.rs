use rustcc::parser::lexer::Lexer;
use rustcc::parser::token::TokenType;

#[test]
fn test_complete_program_lexing() {
    let source = r#"
        int main() {
            return 42;
        }
    "#.to_string();

    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();

    // Verify the token sequence is correct
    let expected_types = vec![
        TokenType::Int,
        TokenType::Identifier, // "main"
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::IntegerLiteral, // "42"
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::EOF,
    ];

    assert_eq!(tokens.len(), expected_types.len());
    for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
        assert_eq!(token.token_type, *expected_type);
    }
}