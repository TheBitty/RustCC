use crate::parser::ast::Type;
use crate::parser::error::{Error, ErrorKind, Result};
use crate::parser::token::TokenType;
use crate::parser::Parser;

impl Parser {
    /// Parse a C type specification
    pub fn parse_type(&mut self) -> Result<Type> {
        // Check for type qualifiers
        let mut is_const = false;
        if self.match_token(TokenType::Const) {
            is_const = true;
        }

        // Parse the base type
        let mut base_type = if self.match_token(TokenType::Int) {
            Type::Int
        } else if self.match_token(TokenType::Char) {
            Type::Char
        } else if self.match_token(TokenType::Void) {
            Type::Void
        } else if self.match_token(TokenType::Struct) {
            // Parse struct type
            let name = self.consume(TokenType::Identifier, "Expected struct name")?
                .lexeme.clone();
            Type::Struct(name)
        } else {
            return Err(Error::from_token(
                ErrorKind::InvalidType("Expected type specifier".to_string()),
                &self.peek(),
                "Expected a valid type specifier".to_string(),
            ));
        };

        // Handle pointers
        while self.match_token(TokenType::Star) {
            base_type = Type::Pointer(Box::new(base_type));
        }

        // Apply const qualifier if present
        if is_const {
            base_type = Type::Const(Box::new(base_type));
        }

        Ok(base_type)
    }

    /// Parse a type name (used in casts and sizeof)
    pub fn parse_type_name(&mut self) -> Result<Type> {
        self.parse_type()
    }

    /// Parse an array type
    pub fn parse_array_type(&mut self, element_type: Type) -> Result<Type> {
        // Consume the opening bracket
        self.consume(TokenType::LeftBracket, "Expected '[' for array type")?;

        // Check if size is specified
        let size = if !self.check(TokenType::RightBracket) {
            // Parse the size expression
            let size_expr = self.parse_expression()?;
            
            // For now, we only support constant integer sizes
            // In a full implementation, we would evaluate constant expressions
            if let crate::parser::ast::Expression::IntegerLiteral(size) = size_expr {
                Some(size as usize)
            } else {
                None
            }
        } else {
            None
        };

        // Consume the closing bracket
        self.consume(TokenType::RightBracket, "Expected ']' after array size")?;

        Ok(Type::Array(Box::new(element_type), size))
    }

    /// Check if the current token is a type specifier
    pub fn is_type_specifier(&self) -> bool {
        self.check(TokenType::Int) || 
        self.check(TokenType::Char) || 
        self.check(TokenType::Void) || 
        self.check(TokenType::Const) ||
        self.check(TokenType::Struct)
    }
} 