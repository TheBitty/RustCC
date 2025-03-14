// identifiers.rs
// Handling of identifiers and keywords

use crate::parser::token::TokenType;
use crate::parser::lexer::Lexer;
use crate::parser::lexer::utils::is_alphanumeric;

impl Lexer {
    /// Handles an identifier (or keyword)
    pub(crate) fn handle_identifier(&mut self) {
        while is_alphanumeric(self.peek()) {
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