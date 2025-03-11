#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_initialization() {
        let source = String::from("int main() {}");
        let lexer = Lexer::new(source.clone());
        
        assert_eq!(lexer.source, source);
        assert_eq!(lexer.tokens.len(), 0);
        assert_eq!(lexer.line, 1);
        assert_eq!(lexer.column, 1);
    }

    #[test]
    fn test_basic_token_scanning() {
        let source = String::from("int");
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens();

        // Should contain 'int' token and EOF token
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Int);
        assert_eq!(tokens[1].token_type, TokenType::EOF);
    }

    #[test]
    fn test_multiple_tokens() {
        let source = String::from("int main() {");
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens();

        let expected_types = vec![
            TokenType::Int,
            TokenType::Identifier,
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::EOF,
        ];

        assert_eq!(tokens.len(), expected_types.len());
        for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
            assert_eq!(token.token_type, *expected_type);
        }
    }
}