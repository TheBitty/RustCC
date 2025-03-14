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
        TokenType::Eof,
    ];

    assert_eq!(tokens.len(), expected_types.len());
    for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
        assert_eq!(token.token_type, *expected_type);
    }
}

#[test]
fn test_invalid_character_handling() {
    let source = String::from("@");
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();

    assert_eq!(tokens[0].token_type, TokenType::Error);
}

#[test]
fn test_function_with_parameters() {
    let source = r#"int add(int a, int b) {
        return a + b;
    }"#.to_string();
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();
    
    let expected_types = vec![
        TokenType::Int,
        TokenType::Identifier, // "add"
        TokenType::LeftParen,
        TokenType::Int,
        TokenType::Identifier, // "a"
        TokenType::Comma,
        TokenType::Int,
        TokenType::Identifier, // "b"
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::Identifier, // "a"
        TokenType::Plus,
        TokenType::Identifier, // "b"
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ];
    
    assert_eq!(tokens.len(), expected_types.len());
    for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
        assert_eq!(token.token_type, *expected_type);
    }
}

#[test]
fn test_if_else_statement() {
    let source = r#"if (x > 0) {
        return x;
    } else {
        return -x;
    }"#.to_string();
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();
    
    let expected_types = vec![
        TokenType::If,
        TokenType::LeftParen,
        TokenType::Identifier, // "x"
        TokenType::Greater,
        TokenType::IntegerLiteral, // "0"
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::Identifier, // "x"
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Else,
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::Minus,
        TokenType::Identifier, // "x"
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ];
    
    assert_eq!(tokens.len(), expected_types.len());
    for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
        assert_eq!(token.token_type, *expected_type);
    }
}

#[test]
fn test_while_loop() {
    let source = r#"while (i < 10) {
        sum = sum + i;
        i = i + 1;
    }"#.to_string();
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();
    
    let expected_types = vec![
        TokenType::While,
        TokenType::LeftParen,
        TokenType::Identifier, // "i"
        TokenType::Less,
        TokenType::IntegerLiteral, // "10"
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Identifier, // "sum"
        TokenType::Equal,
        TokenType::Identifier, // "sum"
        TokenType::Plus,
        TokenType::Identifier, // "i"
        TokenType::Semicolon,
        TokenType::Identifier, // "i"
        TokenType::Equal,
        TokenType::Identifier, // "i"
        TokenType::Plus,
        TokenType::IntegerLiteral, // "1"
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ];
    
    assert_eq!(tokens.len(), expected_types.len());
    for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
        assert_eq!(token.token_type, *expected_type);
    }
}

#[test]
fn test_complex_expression() {
    let source = "x = a * b + c / (d - e);".to_string();
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();
    
    let expected_types = vec![
        TokenType::Identifier, // "x"
        TokenType::Equal,
        TokenType::Identifier, // "a"
        TokenType::Star,
        TokenType::Identifier, // "b"
        TokenType::Plus,
        TokenType::Identifier, // "c"
        TokenType::Slash,
        TokenType::LeftParen,
        TokenType::Identifier, // "d"
        TokenType::Minus,
        TokenType::Identifier, // "e"
        TokenType::RightParen,
        TokenType::Semicolon,
        TokenType::Eof,
    ];
    
    assert_eq!(tokens.len(), expected_types.len());
    for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
        assert_eq!(token.token_type, *expected_type);
    }
}