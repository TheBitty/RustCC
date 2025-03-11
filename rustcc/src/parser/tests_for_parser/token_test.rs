use rustcc::parser::token::{Token, TokenType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token {
            token_type: TokenType::Int,
            lexeme: "int".to_string(),
            line: 1,
            column: 5,
            literal: None,
        };

        assert_eq!(token.token_type, TokenType::Int);
        assert_eq!(token.lexeme, "int");
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 5);
        assert_eq!(token.literal, None);
    }

    #[test]
    fn test_token_with_literal() {
        let token = Token {
            token_type: TokenType::IntegerLiteral,
            lexeme: "42".to_string(),
            line: 2,
            column: 10,
            literal: Some("42".to_string()),
        };
        
        assert_eq!(token.token_type, TokenType::IntegerLiteral);
        assert_eq!(token.lexeme, "42");
        assert_eq!(token.line, 2);
        assert_eq!(token.column, 10);
        assert_eq!(token.literal, Some("42".to_string()));
    }

    #[test]
    fn test_token_debug_representation() {
        let token = Token {
            token_type: TokenType::Identifier,
            lexeme: "variable".to_string(),
            line: 3,
            column: 15,
            literal: None,
        };
        
        // Test that Debug implementation works
        let debug_str = format!("{:?}", token);
        assert!(debug_str.contains("Identifier"));
        assert!(debug_str.contains("variable"));
        assert!(debug_str.contains("3"));
        assert!(debug_str.contains("15"));
    }

    #[test]
    fn test_token_equality() {
        let token1 = Token {
            token_type: TokenType::Plus,
            lexeme: "+".to_string(),
            line: 1,
            column: 5,
            literal: None,
        };
        
        let token2 = Token {
            token_type: TokenType::Plus,
            lexeme: "+".to_string(),
            line: 1,
            column: 5,
            literal: None,
        };
        
        let token3 = Token {
            token_type: TokenType::Minus,
            lexeme: "-".to_string(),
            line: 1,
            column: 6,
            literal: None,
        };
        
        assert_eq!(token1, token2); // Should be equal
        assert_ne!(token1, token3); // Should not be equal
    }
}