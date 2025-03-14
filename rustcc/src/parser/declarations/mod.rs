use crate::parser::ast::{Expression, Function, FunctionParameter, Statement, StructField, Type};
use crate::parser::Parser;
use crate::parser::token::TokenType;

impl Parser {
    pub fn parse_function_with_name(&mut self, return_type: Type, name: String) -> Result<Function, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;

        // Parse parameters list
        let mut parameters = Vec::new();
        let mut is_variadic = false;
        
        if !self.check(TokenType::RightParen) {
            loop {
                // Check for variadic functions with ...
                if self.match_token(TokenType::Dot) {
                    if self.match_token(TokenType::Dot) && self.match_token(TokenType::Dot) {
                        is_variadic = true;
                        break;
                    } else {
                        return Err("Expected '...' for variadic function".to_string());
                    }
                }

                let param_type = self.parse_type()?;
                let param_name = self
                    .consume(TokenType::Identifier, "Expected parameter name")?
                    .lexeme
                    .clone();

                parameters.push(FunctionParameter {
                    name: param_name,
                    data_type: param_type,
                });

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;

        // Handle function declarations without bodies
        if self.match_token(TokenType::Semicolon) {
            // Function declaration without body
            return Ok(Function {
                name,
                return_type,
                parameters,
                body: Vec::new(),
                is_variadic,
                is_external: true,
            });
        }

        self.consume(TokenType::LeftBrace, "Expected '{' before function body")?;

        let mut body = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after function body")?;

        Ok(Function {
            name,
            return_type,
            parameters,
            body,
            is_variadic,
            is_external: false,
        })
    }

    pub fn parse_global_variable_with_name(&mut self, data_type: Type, name: String) -> Result<Statement, String> {
        let mut initializer = Expression::IntegerLiteral(0); // Default initializer
        
        if self.match_token(TokenType::Equal) {
            initializer = self.parse_expression()?;
        }
        
        self.consume(TokenType::Semicolon, "Expected ';' after variable declaration")?;
        
        Ok(Statement::VariableDeclaration {
            name,
            data_type: Some(data_type),
            initializer,
            is_global: true,
        })
    }

    pub fn parse_variable_declaration(&mut self) -> Result<Statement, String> {
        let data_type = self.parse_type()?;

        // Handle array declarations
        let name_token = self.consume(TokenType::Identifier, "Expected variable name")?;
        let name = name_token.lexeme.clone();

        // Check if it's an array declaration
        if self.match_token(TokenType::LeftBracket) {
            // Array declaration
            let size_expr = if !self.check(TokenType::RightBracket) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.consume(TokenType::RightBracket, "Expected ']' after array size")?;

            // Handle array initialization
            let initializer = if self.match_token(TokenType::Equal) {
                if self.match_token(TokenType::LeftBrace) {
                    // Array initializer list
                    let mut elements = Vec::new();

                    if !self.check(TokenType::RightBrace) {
                        loop {
                            elements.push(self.parse_expression()?);

                            if !self.match_token(TokenType::Comma) {
                                break;
                            }
                        }
                    }

                    self.consume(
                        TokenType::RightBrace,
                        "Expected '}' after array initializer",
                    )?;

                    Expression::ArrayLiteral(elements)
                } else {
                    self.parse_expression()?
                }
            } else {
                // Default initialization
                Expression::ArrayLiteral(Vec::new())
            };

            self.consume(
                TokenType::Semicolon,
                "Expected ';' after variable declaration",
            )?;

            return Ok(Statement::ArrayDeclaration {
                name,
                data_type: Some(data_type),
                size: size_expr,
                initializer,
                is_global: false,
            });
        }

        // Regular variable declaration
        let initializer = if self.match_token(TokenType::Equal) {
            self.parse_expression()?
        } else {
            // Default initialization
            match data_type {
                Type::Int => Expression::IntegerLiteral(0),
                Type::Char => Expression::CharLiteral('\0'),
                _ => Expression::IntegerLiteral(0),
            }
        };

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        Ok(Statement::VariableDeclaration {
            name,
            data_type: Some(data_type),
            initializer,
            is_global: false,
        })
    }

    pub fn parse_type(&mut self) -> Result<Type, String> {
        // Handle const qualifier
        let is_const = self.match_token(TokenType::Const);
        
        let base_type = if self.match_token(TokenType::Int) {
            Type::Int
        } else if self.match_token(TokenType::Void) {
            Type::Void
        } else if self.match_token(TokenType::Char) {
            Type::Char
        } else if self.match_token(TokenType::Struct) {
            // Handle struct types
            let name = self
                .consume(TokenType::Identifier, "Expected struct name")?
                .lexeme
                .clone();
            Type::Struct(name)
        } else {
            return Err("Expected type".to_string());
        };
        
        // Apply const qualifier if present
        let mut result_type = if is_const {
            Type::Const(Box::new(base_type))
        } else {
            base_type
        };
        
        // Handle pointer types (possibly multiple levels)
        while self.match_token(TokenType::Star) {
            result_type = Type::Pointer(Box::new(result_type));
        }
        
        Ok(result_type)
    }

    pub fn parse_struct(&mut self) -> Result<crate::parser::ast::Struct, String> {
        // Consume struct name
        let name_token = self.consume(TokenType::Identifier, "Expected struct name")?;
        let name = name_token.lexeme.clone();

        // Handle forward declarations
        if self.match_token(TokenType::Semicolon) {
            return Ok(crate::parser::ast::Struct {
                name,
                fields: Vec::new(),
            });
        }

        self.consume(TokenType::LeftBrace, "Expected '{' after struct name")?;

        let mut fields = Vec::new();

        // Parse struct fields
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let mut field_type = self.parse_type()?;
            let field_name = self
                .consume(TokenType::Identifier, "Expected field name")?
                .lexeme
                .clone();

            // Check for array declaration and convert to array type
            if self.match_token(TokenType::LeftBracket) {
                if !self.check(TokenType::RightBracket) {
                    // Parse size expression, but ignore it for now
                    self.parse_expression()?;
                }
                self.consume(TokenType::RightBracket, "Expected ']' after array size")?;

                // Convert to array type
                field_type = Type::Array(Box::new(field_type), None);
            }

            self.consume(TokenType::Semicolon, "Expected ';' after field declaration")?;

            fields.push(StructField {
                name: field_name,
                data_type: field_type,
            });
        }

        self.consume(TokenType::RightBrace, "Expected '}' after struct fields")?;
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after struct declaration",
        )?;

        Ok(crate::parser::ast::Struct { name, fields })
    }
} 