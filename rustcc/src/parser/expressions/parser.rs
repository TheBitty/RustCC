use crate::parser::ast::{BinaryOp, Expression, OperatorType, UnaryOp};
use crate::parser::error::{self, Result};
use crate::parser::token::TokenType;
use crate::parser::Parser;

impl Parser {
    /// Parse an expression
    pub fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_assignment()
    }

    /// Parse an assignment expression
    fn parse_assignment(&mut self) -> Result<Expression> {
        let expr = self.parse_ternary()?;

        if self.match_token(TokenType::Equal) {
            let value = self.parse_assignment()?;

            // Validate that the left side is a valid assignment target
            match expr {
                Expression::Variable(_)
                | Expression::ArrayAccess { .. }
                | Expression::StructFieldAccess { .. }
                | Expression::PointerFieldAccess { .. } => {
                    return Ok(Expression::Assignment {
                        target: Box::new(expr),
                        value: Box::new(value),
                    });
                }
                _ => {
                    return Err(self.error(
                        error::ErrorKind::InvalidAssignment("Invalid assignment target".to_string()),
                        self.current - 1
                    ));
                }
            }
        }

        Ok(expr)
    }

    /// Parse a ternary conditional expression
    fn parse_ternary(&mut self) -> Result<Expression> {
        let expr = self.parse_logical_or()?;

        if self.match_token(TokenType::Question) {
            let then_expr = self.parse_expression()?;
            self.consume(TokenType::Colon, "Expected ':' in ternary expression")?;
            let else_expr = self.parse_ternary()?;

            return Ok(Expression::TernaryIf {
                condition: Box::new(expr),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            });
        }

        Ok(expr)
    }

    /// Parse a logical OR expression
    fn parse_logical_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(TokenType::Or) || self.match_token(TokenType::LogicalOr) {
            let right = self.parse_logical_and()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator: BinaryOp::LogicalOr,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parse a logical AND expression
    fn parse_logical_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_bitwise_or()?;

        while self.match_token(TokenType::And) || self.match_token(TokenType::LogicalAnd) {
            let right = self.parse_bitwise_or()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator: BinaryOp::LogicalAnd,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parse a bitwise OR expression
    fn parse_bitwise_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_bitwise_xor()?;

        while self.match_token(TokenType::Pipe) || self.match_token(TokenType::BitwiseOr) {
            let right = self.parse_bitwise_xor()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator: BinaryOp::BitwiseOr,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parse a bitwise XOR expression
    fn parse_bitwise_xor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_bitwise_and()?;

        while self.match_token(TokenType::Caret) || self.match_token(TokenType::BitwiseXor) {
            let right = self.parse_bitwise_and()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator: BinaryOp::BitwiseXor,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parse a bitwise AND expression
    fn parse_bitwise_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;

        while self.match_token(TokenType::Ampersand) || self.match_token(TokenType::BitwiseAnd) {
            let right = self.parse_equality()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator: BinaryOp::BitwiseAnd,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parse an equality expression
    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;

        while self.match_token(TokenType::EqualEqual) || self.match_token(TokenType::BangEqual) {
            let operator = match self.previous().token_type {
                TokenType::EqualEqual => BinaryOp::Equal,
                TokenType::BangEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parse a comparison expression
    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_shift()?;

        while self.match_token(TokenType::Less)
            || self.match_token(TokenType::LessEqual)
            || self.match_token(TokenType::Greater)
            || self.match_token(TokenType::GreaterEqual)
        {
            let operator = match self.previous().token_type {
                TokenType::Less => BinaryOp::LessThan,
                TokenType::LessEqual => BinaryOp::LessThanOrEqual,
                TokenType::Greater => BinaryOp::GreaterThan,
                TokenType::GreaterEqual => BinaryOp::GreaterThanOrEqual,
                _ => unreachable!(),
            };
            let right = self.parse_shift()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }
    
    /// Parse a shift expression
    fn parse_shift(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;

        while self.match_token(TokenType::ShiftLeft)
            || self.match_token(TokenType::LeftShift)
            || self.match_token(TokenType::ShiftRight)
            || self.match_token(TokenType::RightShift)
        {
            let operator = match self.previous().token_type {
                TokenType::ShiftLeft | TokenType::LeftShift => BinaryOp::LeftShift,
                TokenType::ShiftRight | TokenType::RightShift => BinaryOp::RightShift,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parse a term expression
    fn parse_term(&mut self) -> Result<Expression> {
        let mut expr = self.parse_factor()?;

        while self.match_token(TokenType::Plus) || self.match_token(TokenType::Minus) {
            let operator = match self.previous().token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parse a factor expression
    fn parse_factor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;

        while self.match_token(TokenType::Star)
            || self.match_token(TokenType::Slash)
            || self.match_token(TokenType::Percent)
        {
            let operator = match self.previous().token_type {
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parse a unary expression
    fn parse_unary(&mut self) -> Result<Expression> {
        if self.match_token(TokenType::Bang)
            || self.match_token(TokenType::Minus)
            || self.match_token(TokenType::Tilde)
            || self.match_token(TokenType::Star)
            || self.match_token(TokenType::Ampersand)
            || self.match_token(TokenType::Increment)
            || self.match_token(TokenType::PlusPlus)
            || self.match_token(TokenType::Decrement)
            || self.match_token(TokenType::MinusMinus)
        {
            let operator = match self.previous().token_type {
                TokenType::Bang => OperatorType::Unary(UnaryOp::LogicalNot),
                TokenType::Minus => OperatorType::Unary(UnaryOp::Negate),
                TokenType::Tilde => OperatorType::Unary(UnaryOp::BitwiseNot),
                TokenType::Star => OperatorType::Unary(UnaryOp::Dereference),
                TokenType::Ampersand => OperatorType::Unary(UnaryOp::AddressOf),
                TokenType::Increment | TokenType::PlusPlus => {
                    OperatorType::Unary(UnaryOp::PreIncrement)
                }
                TokenType::Decrement | TokenType::MinusMinus => {
                    OperatorType::Unary(UnaryOp::PreDecrement)
                }
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOperation {
                operator,
                operand: Box::new(operand),
            });
        }

        if self.match_token(TokenType::Sizeof) {
            self.consume(TokenType::LeftParen, "Expected '(' after sizeof")?;

            // Check if it's a type or an expression
            if self.is_type_specifier() {
                let type_name = self.parse_type()?;
                self.consume(TokenType::RightParen, "Expected ')' after sizeof type")?;

                // Create a dummy expression for the type
                let dummy_expr = Expression::Cast {
                    target_type: type_name,
                    expr: Box::new(Expression::IntegerLiteral(0)),
                };

                return Ok(Expression::SizeOf(Box::new(dummy_expr)));
            } else {
                let expr = self.parse_expression()?;
                self.consume(
                    TokenType::RightParen,
                    "Expected ')' after sizeof expression",
                )?;
                return Ok(Expression::SizeOf(Box::new(expr)));
            }
        }

        self.parse_postfix()
    }

    /// Parse a postfix expression
    fn parse_postfix(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(TokenType::LeftBracket) {
                // Array access
                let index = self.parse_expression()?;
                self.consume(TokenType::RightBracket, "Expected ']' after array index")?;
                expr = Expression::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(TokenType::Dot) {
                // Struct field access
                let field = self
                    .consume(TokenType::Identifier, "Expected field name after '.'")?
                    .lexeme
                    .clone();
                expr = Expression::StructFieldAccess {
                    object: Box::new(expr),
                    field,
                };
            } else if self.match_token(TokenType::Arrow) {
                // Pointer field access
                let field = self
                    .consume(TokenType::Identifier, "Expected field name after '->'")?
                    .lexeme
                    .clone();
                expr = Expression::PointerFieldAccess {
                    pointer: Box::new(expr),
                    field,
                };
            } else if self.match_token(TokenType::LeftParen) {
                // Function call
                let mut arguments = Vec::new();

                if !self.check(TokenType::RightParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                    }
                }

                self.consume(
                    TokenType::RightParen,
                    "Expected ')' after function arguments",
                )?;

                // Extract function name from expression
                let name = match expr {
                    Expression::Variable(name) => name,
                    _ => {
                        return Err(self.error(
                            error::ErrorKind::NotCallable("Expression is not callable".to_string()),
                            self.current - 1
                        ));
                    }
                };

                expr = Expression::FunctionCall { name, arguments };
            } else if self.match_token(TokenType::Increment)
                || self.match_token(TokenType::PlusPlus)
            {
                // Post-increment
                expr = Expression::UnaryOperation {
                    operator: OperatorType::Unary(UnaryOp::PostIncrement),
                    operand: Box::new(expr),
                };
            } else if self.match_token(TokenType::Decrement)
                || self.match_token(TokenType::MinusMinus)
            {
                // Post-decrement
                expr = Expression::UnaryOperation {
                    operator: OperatorType::Unary(UnaryOp::PostDecrement),
                    operand: Box::new(expr),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse a primary expression
    fn parse_primary(&mut self) -> Result<Expression> {
        if self.match_token(TokenType::IntegerLiteral) {
            let value = self.previous().lexeme.parse::<i32>().unwrap_or(0);
            return Ok(Expression::IntegerLiteral(value));
        }

        if self.match_token(TokenType::FloatLiteral) {
            let value = self.previous().lexeme.parse::<f64>().unwrap_or(0.0);
            return Ok(Expression::FloatLiteral(value));
        }

        if self.match_token(TokenType::StringLiteral) {
            let value = self.previous().lexeme.clone();
            return Ok(Expression::StringLiteral(value));
        }

        if self.match_token(TokenType::CharLiteral) {
            let value = self.previous().lexeme.chars().next().unwrap_or('\0');
            return Ok(Expression::CharLiteral(value));
        }

        if self.match_token(TokenType::Identifier) {
            return Ok(Expression::Variable(self.previous().lexeme.clone()));
        }

        if self.match_token(TokenType::LeftParen) {
            // Check for cast expression: (type)expr
            if self.is_type_specifier() || self.check(TokenType::Const) || self.check(TokenType::Volatile) {
                // This is likely a cast
                let cast_type = self.parse_type()?;
                self.consume(TokenType::RightParen, "Expected ')' after cast type")?;
                
                // Parse the expression being cast
                let expr = self.parse_unary()?;
                
                return Ok(Expression::Cast {
                    target_type: cast_type,
                    expr: Box::new(expr),
                });
            }
            
            // Check for compound literal: (type){initializers}
            if (self.is_type_specifier() || self.check(TokenType::Const) || self.check(TokenType::Volatile)) 
                && self.peek_ahead(1).token_type == TokenType::LeftBrace {
                // This is a compound literal
                let type_name = self.parse_type()?;
                self.consume(TokenType::RightParen, "Expected ')' after compound literal type")?;
                
                // Parse initializers
                self.consume(TokenType::LeftBrace, "Expected '{' after compound literal type")?;
                
                let mut initializers = Vec::new();
                
                if !self.check(TokenType::RightBrace) {
                    loop {
                        initializers.push(self.parse_expression()?);
                        
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                        
                        // Allow trailing comma
                        if self.check(TokenType::RightBrace) {
                            break;
                        }
                    }
                }
                
                self.consume(TokenType::RightBrace, "Expected '}' after compound literal initializers")?;
                
                return Ok(Expression::CompoundLiteral {
                    type_name,
                    initializers,
                });
            }

            // Regular parenthesized expression
            let expr = self.parse_expression()?;
            self.consume(
                TokenType::RightParen,
                "Expected ')' after expression",
            )?;
            return Ok(expr);
        }

        if self.match_token(TokenType::Sizeof) {
            // sizeof operator
            if self.match_token(TokenType::LeftParen) {
                // Check if it's sizeof(type) or sizeof(expr)
                if self.is_type_specifier() || self.check(TokenType::Const) || self.check(TokenType::Volatile) {
                    // sizeof(type)
                    let type_name = self.parse_type()?;
                    self.consume(TokenType::RightParen, "Expected ')' after type in sizeof")?;
                    return Ok(Expression::SizeOfType(type_name));
                } else {
                    // sizeof(expr)
                    let expr = self.parse_expression()?;
                    self.consume(TokenType::RightParen, "Expected ')' after expression in sizeof")?;
                    return Ok(Expression::SizeOf(Box::new(expr)));
                }
            } else {
                // sizeof expr (without parentheses)
                let expr = self.parse_unary()?;
                return Ok(Expression::SizeOf(Box::new(expr)));
            }
        }

        if self.match_token(TokenType::Alignof) {
            // _Alignof operator (C11)
            self.consume(TokenType::LeftParen, "Expected '(' after _Alignof")?;
            let type_name = self.parse_type()?;
            self.consume(TokenType::RightParen, "Expected ')' after type in _Alignof")?;
            return Ok(Expression::AlignOf(type_name));
        }

        if self.match_token(TokenType::Generic) {
            // _Generic expression (C11)
            self.consume(TokenType::LeftParen, "Expected '(' after _Generic")?;
            
            // Parse the controlling expression
            let controlling_expr = self.parse_expression()?;
            
            self.consume(TokenType::Comma, "Expected ',' after controlling expression in _Generic")?;
            
            // Parse the association list
            let mut associations = Vec::new();
            let mut default_expr = None;
            
            loop {
                if self.match_token(TokenType::Default) {
                    // default: expr
                    self.consume(TokenType::Colon, "Expected ':' after 'default' in _Generic")?;
                    default_expr = Some(Box::new(self.parse_expression()?));
                } else {
                    // type: expr
                    let type_name = self.parse_type()?;
                    self.consume(TokenType::Colon, "Expected ':' after type in _Generic")?;
                    let expr = self.parse_expression()?;
                    associations.push((type_name, expr));
                }
                
                if !self.match_token(TokenType::Comma) {
                    break;
                }
                
                // Check for end of associations
                if self.check(TokenType::RightParen) {
                    break;
                }
            }
            
            self.consume(TokenType::RightParen, "Expected ')' after _Generic associations")?;
            
            return Ok(Expression::GenericSelection {
                controlling_expr: Box::new(controlling_expr),
                associations,
                default_expr,
            });
        }

        if self.match_token(TokenType::StaticAssert) {
            // _Static_assert (C11)
            self.consume(TokenType::LeftParen, "Expected '(' after _Static_assert")?;
            
            // Parse the condition
            let condition = self.parse_expression()?;
            
            self.consume(TokenType::Comma, "Expected ',' after condition in _Static_assert")?;
            
            // Parse the message string
            let message_token = self.consume(TokenType::StringLiteral, "Expected string literal message in _Static_assert")?;
            let message = message_token.lexeme.clone();
            
            self.consume(TokenType::RightParen, "Expected ')' after _Static_assert")?;
            
            return Ok(Expression::StaticAssert {
                condition: Box::new(condition),
                message,
            });
        }

        if self.match_token(TokenType::LeftBrace) {
            // Array initializer: {1, 2, 3}
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
            return Ok(Expression::ArrayLiteral(elements));
        }

        Err(self.unexpected_token_error("expression"))
    }

    /// Helper method to peek ahead by a specific number of tokens
    fn peek_ahead(&self, offset: usize) -> &Token {
        if self.current + offset >= self.tokens.len() {
            // Return EOF token if we're past the end
            &self.tokens[self.tokens.len() - 1]
        } else {
            &self.tokens[self.current + offset]
        }
    }
} 