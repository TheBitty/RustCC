#[cfg(test)]
mod tests {
    use crate::parser::lexer::Lexer;
    use crate::parser::token::TokenType;

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
    
    #[test]
    fn test_whitespace_handling() {
        let source = String::from("int  \t\n   main");
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens();
        
        assert_eq!(tokens.len(), 3); // int, main, EOF
        assert_eq!(tokens[0].token_type, TokenType::Int);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].line, 2); // Should be on the second line
    }
    
    #[test]
    fn test_operators() {
        let source = String::from("+ - * / % = == != < <= > >= && || !");
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens();
        
        let expected_types = vec![
            TokenType::Plus, TokenType::Minus, TokenType::Star, TokenType::Slash, 
            TokenType::Percent, TokenType::Equal, TokenType::EqualEqual, TokenType::BangEqual,
            TokenType::Less, TokenType::LessEqual, TokenType::Greater, TokenType::GreaterEqual,
            TokenType::And, TokenType::Or, TokenType::Bang, TokenType::EOF,
        ];
        
        assert_eq!(tokens.len(), expected_types.len());
        for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
            assert_eq!(token.token_type, *expected_type);
        }
    }
    
    #[test]
    fn test_compound_assignment_operators() {
        let source = String::from("+= -= *= /= %=");
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens();
        
        let expected_types = vec![
            TokenType::PlusEqual, TokenType::MinusEqual, TokenType::StarEqual, 
            TokenType::SlashEqual, TokenType::PercentEqual, TokenType::EOF,
        ];
        
        assert_eq!(tokens.len(), expected_types.len());
        for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
            assert_eq!(token.token_type, *expected_type);
        }
    }
    
    #[test]
    fn test_keywords() {
        let source = String::from("int char void if else while for return break continue struct");
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens();
        
        let expected_types = vec![
            TokenType::Int, TokenType::Char, TokenType::Void, TokenType::If, 
            TokenType::Else, TokenType::While, TokenType::For, TokenType::Return,
            TokenType::Break, TokenType::Continue, TokenType::Struct, TokenType::EOF,
        ];
        
        assert_eq!(tokens.len(), expected_types.len());
        for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
            assert_eq!(token.token_type, *expected_type);
        }
    }
    
    #[test]
    fn test_integer_literals() {
        let source = String::from("0 42 123456789");
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens();
        
        assert_eq!(tokens.len(), 4); // 3 numbers + EOF
        assert_eq!(tokens[0].token_type, TokenType::IntegerLiteral);
        assert_eq!(tokens[0].literal, Some("0".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::IntegerLiteral);
        assert_eq!(tokens[1].literal, Some("42".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::IntegerLiteral);
        assert_eq!(tokens[2].literal, Some("123456789".to_string()));
    }
}