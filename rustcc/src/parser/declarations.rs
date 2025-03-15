use crate::parser::ast::{Expression, Function, FunctionParameter, Statement, StructField, Type};
use crate::parser::error::{self, Result};
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
                        return Err(self.error(
                            error::ErrorKind::UnexpectedToken(
                                self.previous().lexeme.clone(),
                                "...".to_string(),
                            ),
                            self.current - 1
                        ));
                    }
                }

                // Parse parameter type
                let param_type = self.parse_type()?;
                
                // Parse parameter name (optional in C)
                let param_name = if self.check(TokenType::Identifier) {
                    self.consume(TokenType::Identifier, "Expected parameter name")?
                        .lexeme
                        .clone()
                } else {
                    // Anonymous parameter
                    String::new()
                };

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

        // Check for _Noreturn specifier (C11)
        let _is_noreturn = self.match_token(TokenType::Noreturn);

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
        // Check for _Alignas specifier (C11)
        let alignment = if self.match_token(TokenType::Alignas) {
            self.consume(TokenType::LeftParen, "Expected '(' after _Alignas")?;
            
            let alignment = if self.is_type_specifier() {
                // _Alignas(type)
                let _type_name = self.parse_type()?;
                // In a real implementation, we would compute the alignment of the type
                // For now, we'll just use a placeholder value
                Some(8)
            } else {
                // _Alignas(constant-expression)
                let expr = self.parse_expression()?;
                // In a real implementation, we would evaluate the constant expression
                // For now, we'll just use a placeholder value if it's a literal
                match expr {
                    Expression::IntegerLiteral(value) => Some(value as usize),
                    _ => Some(8), // Default alignment
                }
            };
            
            self.consume(TokenType::RightParen, "Expected ')' after _Alignas")?;
            
            alignment
        } else {
            None
        };

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
            alignment,
        })
    }

    /// Parse a variable declaration
    pub fn parse_variable_declaration(&mut self) -> Result<Statement> {
        // Parse type qualifiers and storage class specifiers
        let mut is_thread_local = false;
        
        if self.match_token(TokenType::ThreadLocal) {
            is_thread_local = true;
        }
        
        let data_type = self.parse_type()?;

        // Check for _Alignas specifier (C11)
        let alignment = if self.match_token(TokenType::Alignas) {
            self.consume(TokenType::LeftParen, "Expected '(' after _Alignas")?;
            
            let alignment = if self.is_type_specifier() {
                // _Alignas(type)
                let _type_name = self.parse_type()?;
                // In a real implementation, we would compute the alignment of the type
                // For now, we'll just use a placeholder value
                Some(8)
            } else {
                // _Alignas(constant-expression)
                let expr = self.parse_expression()?;
                // In a real implementation, we would evaluate the constant expression
                // For now, we'll just use a placeholder value if it's a literal
                match expr {
                    Expression::IntegerLiteral(value) => Some(value as usize),
                    _ => Some(8), // Default alignment
                }
            };
            
            self.consume(TokenType::RightParen, "Expected ')' after _Alignas")?;
            
            alignment
        } else {
            None
        };

        // Handle array declarations
        let name_token = self.consume(TokenType::Identifier, "Expected variable name")?;
        let name = name_token.lexeme.clone();

        // Check if it's an array declaration
        if self.match_token(TokenType::LeftBracket) {
            // Array declaration
            let size_expr = if self.match_token(TokenType::Star) {
                // VLA with unspecified size [*]
                Some(Expression::IntegerLiteral(-1)) // Special marker for [*]
            } else if !self.check(TokenType::RightBracket) {
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
                            
                            // Allow trailing comma
                            if self.check(TokenType::RightBrace) {
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

            let declaration = Statement::ArrayDeclaration {
                name,
                data_type: Some(data_type),
                size: size_expr,
                initializer,
                is_global: false,
                alignment,
            };
            
            // Wrap in ThreadLocal if needed
            if is_thread_local {
                return Ok(Statement::ThreadLocal {
                    declaration: Box::new(declaration),
                });
            } else {
                return Ok(declaration);
            }
        }

        // Regular variable declaration
        let initializer = if self.match_token(TokenType::Equal) {
            self.parse_expression()?
        } else {
            // Default initialization
            match data_type {
                Type::Int => Expression::IntegerLiteral(0),
                Type::Char => Expression::CharLiteral('\0'),
                Type::Float | Type::Double => Expression::FloatLiteral(0.0),
                _ => Expression::IntegerLiteral(0),
            }
        };

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        let declaration = Statement::VariableDeclaration {
            name,
            data_type: Some(data_type),
            initializer,
            is_global: false,
            alignment,
        };
        
        // Wrap in ThreadLocal if needed
        if is_thread_local {
            Ok(Statement::ThreadLocal {
                declaration: Box::new(declaration),
            })
        } else {
            Ok(declaration)
        }
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
            // Check for _Static_assert inside struct (C11)
            if self.match_token(TokenType::StaticAssert) {
                // Parse _Static_assert but don't add it to fields
                self.consume(TokenType::LeftParen, "Expected '(' after '_Static_assert'")?;
                self.parse_expression()?; // condition
                self.consume(TokenType::Comma, "Expected ',' after condition in _Static_assert")?;
                self.consume(TokenType::StringLiteral, "Expected string literal message in _Static_assert")?;
                self.consume(TokenType::RightParen, "Expected ')' after _Static_assert")?;
                self.consume(TokenType::Semicolon, "Expected ';' after _Static_assert")?;
                continue;
            }
            
            // Parse field type
            let field_type = self.parse_type()?;

            // Check for _Alignas specifier (C11)
            let _alignment = if self.match_token(TokenType::Alignas) {
                self.consume(TokenType::LeftParen, "Expected '(' after _Alignas")?;
                
                if self.is_type_specifier() {
                    // _Alignas(type)
                    self.parse_type()?;
                } else {
                    // _Alignas(constant-expression)
                    self.parse_expression()?;
                }
                
                self.consume(TokenType::RightParen, "Expected ')' after _Alignas")?;
                
                true
            } else {
                false
            };

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
                } else if self.match_token(TokenType::Colon) {
                    // Bit field
                    let _bit_width = self.parse_expression()?;
                    // In a real implementation, we would store the bit width
                    // For now, we'll just use the original type
                    field_type.clone()
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
                    return Err(self.error(
                        error::ErrorKind::MissingIdentifier("struct field declaration".to_string()),
                        self.current
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
