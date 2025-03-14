use crate::parser::ast::{Expression, Function, FunctionParameter, Statement, StructField, Type};
use crate::parser::error::{Error, ErrorKind, Result};
use crate::parser::token::TokenType;
use crate::parser::Parser;

impl Parser {
    /// Parse a function declaration or definition
    pub fn parse_function_with_name(
        &mut self,
        return_type: Type,
        name: String,
    ) -> Result<Function> {
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
                        return Err(Error::from_token(
                            ErrorKind::InvalidDeclaration(
                                "Expected '...' for variadic function".to_string(),
                            ),
                            &self.previous(),
                            "Expected '...' for variadic function".to_string(),
                        ));
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

    /// Parse a global variable declaration
    pub fn parse_global_variable_with_name(
        &mut self,
        data_type: Type,
        name: String,
    ) -> Result<Statement> {
        let mut initializer = Expression::IntegerLiteral(0); // Default initializer

        if self.match_token(TokenType::Equal) {
            initializer = self.parse_expression()?;
        }

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        Ok(Statement::VariableDeclaration {
            name,
            data_type: Some(data_type),
            initializer,
            is_global: true,
        })
    }

    /// Parse a variable declaration
    pub fn parse_variable_declaration(&mut self) -> Result<Statement> {
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

    /// Parse a struct declaration
    pub fn parse_struct(&mut self) -> Result<crate::parser::ast::Struct> {
        let name = self
            .consume(TokenType::Identifier, "Expected struct name")?
            .lexeme
            .clone();

        // Check if this is just a forward declaration
        if self.match_token(TokenType::Semicolon) {
            return Ok(crate::parser::ast::Struct {
                name,
                fields: Vec::new(),
            });
        }

        self.consume(TokenType::LeftBrace, "Expected '{' after struct name")?;

        let mut fields = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            // Parse field type
            let field_type = self.parse_type()?;

            // Parse field names (can have multiple fields of the same type)
            loop {
                let field_name = self
                    .consume(TokenType::Identifier, "Expected field name")?
                    .lexeme
                    .clone();

                // Check for array field
                let field_type = if self.match_token(TokenType::LeftBracket) {
                    let array_type = self.parse_array_type(field_type.clone())?;
                    array_type
                } else {
                    field_type.clone()
                };

                fields.push(StructField {
                    name: field_name,
                    data_type: field_type,
                });

                // Check if there are more fields of this type
                if !self.match_token(TokenType::Comma) {
                    break;
                }

                // If we see a right brace after a comma, it's a syntax error
                if self.check(TokenType::RightBrace) {
                    return Err(Error::from_token(
                        ErrorKind::InvalidDeclaration(
                            "Expected field name after comma".to_string(),
                        ),
                        &self.peek(),
                        "Expected field name after comma in struct declaration".to_string(),
                    ));
                }

                // If we see a type specifier after a comma, it's the start of a new field declaration
                if self.is_type_specifier() {
                    break;
                }
            }

            self.consume(TokenType::Semicolon, "Expected ';' after struct field")?;
        }

        self.consume(TokenType::RightBrace, "Expected '}' after struct fields")?;
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after struct declaration",
        )?;

        Ok(crate::parser::ast::Struct { name, fields })
    }

    /// Parse a typedef declaration
    pub fn parse_typedef(&mut self) -> Result<()> {
        // Skip the 'typedef' keyword (already consumed)

        // Parse the base type
        let _base_type = self.parse_type()?;

        // Parse the new type name
        let _new_type_name = self
            .consume(TokenType::Identifier, "Expected new type name")?
            .lexeme
            .clone();

        self.consume(TokenType::Semicolon, "Expected ';' after typedef")?;

        // In a full implementation, we would register this typedef in a symbol table
        // For now, we'll just acknowledge it

        Ok(())
    }

    /// Parse an enum declaration
    pub fn parse_enum(&mut self) -> Result<()> {
        // Skip the 'enum' keyword (already consumed)

        // Parse the enum name (optional)
        let _enum_name = if self.check(TokenType::Identifier) {
            Some(self.consume(TokenType::Identifier, "")?.lexeme.clone())
        } else {
            None
        };

        self.consume(TokenType::LeftBrace, "Expected '{' after enum name")?;

        let mut _value = 0; // Track the implicit enum value

        // Parse enum constants
        while !self.check(TokenType::RightBrace) {
            let _const_name = self
                .consume(TokenType::Identifier, "Expected enum constant name")?
                .lexeme
                .clone();

            // Check for explicit value
            if self.match_token(TokenType::Equal) {
                let expr = self.parse_expression()?;

                // In a full implementation, we would evaluate the constant expression
                // For now, we'll just acknowledge it
                if let Expression::IntegerLiteral(val) = expr {
                    _value = val;
                }
            }

            _value += 1; // Increment for next constant

            // Check for comma
            if !self.match_token(TokenType::Comma) {
                break;
            }

            // Allow trailing comma
            if self.check(TokenType::RightBrace) {
                break;
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after enum constants")?;
        self.consume(TokenType::Semicolon, "Expected ';' after enum declaration")?;

        Ok(())
    }
}
