use crate::parser::ast::{BinaryOp, Expression, OperatorType};
use crate::parser::Parser;
use crate::parser::token::TokenType;

impl Parser {
    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression, String> {
        let expr = self.parse_logical_or()?;

        if self.match_token(TokenType::Equal) {
            let value = self.parse_assignment()?;

            match expr {
                Expression::Variable(name) => {
                    return Ok(Expression::Assignment {
                        target: Box::new(Expression::Variable(name)),
                        value: Box::new(value),
                    });
                }
                Expression::ArrayAccess { array, index } => {
                    return Ok(Expression::Assignment {
                        target: Box::new(Expression::ArrayAccess { array, index }),
                        value: Box::new(value),
                    });
                }
                _ => return Err("Invalid assignment target".to_string()),
            }
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(TokenType::Or) {
            let right = self.parse_logical_and()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator: BinaryOp::LogicalOr,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_equality()?;

        while self.match_token(TokenType::And) {
            let right = self.parse_equality()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator: BinaryOp::LogicalAnd,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_comparison()?;

        while self.match_any(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = if self.previous().token_type == TokenType::EqualEqual {
                BinaryOp::Equal
            } else {
                BinaryOp::NotEqual
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

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_additive()?;

        while self.match_any(&[
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
        ]) {
            let operator = match self.previous().token_type {
                TokenType::Less => BinaryOp::LessThan,
                TokenType::LessEqual => BinaryOp::LessThanOrEqual,
                TokenType::Greater => BinaryOp::GreaterThan,
                TokenType::GreaterEqual => BinaryOp::GreaterThanOrEqual,
                _ => unreachable!(),
            };
            let right = self.parse_additive()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_multiplicative()?;

        while self.match_any(&[TokenType::Plus, TokenType::Minus]) {
            let operator = if self.previous().token_type == TokenType::Plus {
                BinaryOp::Add
            } else {
                BinaryOp::Subtract
            };
            let right = self.parse_multiplicative()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_unary()?;

        while self.match_any(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
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

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.match_any(&[
            TokenType::Bang,
            TokenType::Minus,
            TokenType::Increment,
            TokenType::Decrement,
        ]) {
            let operator = match self.previous().token_type {
                TokenType::Bang => OperatorType::Binary(BinaryOp::LogicalNot),
                TokenType::Minus => OperatorType::Binary(BinaryOp::Negate),
                TokenType::Increment => OperatorType::Binary(BinaryOp::PreIncrement),
                TokenType::Decrement => OperatorType::Binary(BinaryOp::PreDecrement),
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            return Ok(Expression::UnaryOperation {
                operator,
                operand: Box::new(right),
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(TokenType::LeftParen) {
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

                expr = Expression::FunctionCall {
                    name: match expr {
                        Expression::Variable(name) => name,
                        _ => return Err("Expected function name".to_string()),
                    },
                    arguments,
                };
            } else if self.match_token(TokenType::LeftBracket) {
                // Array access
                let index = self.parse_expression()?;
                self.consume(TokenType::RightBracket, "Expected ']' after array index")?;

                expr = Expression::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(TokenType::Increment) {
                // Post-increment
                expr = Expression::UnaryOperation {
                    operator: OperatorType::Binary(BinaryOp::PostIncrement),
                    operand: Box::new(expr),
                };
            } else if self.match_token(TokenType::Decrement) {
                // Post-decrement
                expr = Expression::UnaryOperation {
                    operator: OperatorType::Binary(BinaryOp::PostDecrement),
                    operand: Box::new(expr),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        if self.match_token(TokenType::IntegerLiteral) {
            let value = match self.previous().literal.clone() {
                Some(lit) => lit.parse::<i32>().unwrap_or(0),
                None => self.previous().lexeme.parse::<i32>().unwrap_or(0),
            };
            Ok(Expression::IntegerLiteral(value))
        } else if self.match_token(TokenType::StringLiteral) {
            let value = self.previous().literal.clone().unwrap_or_default();
            Ok(Expression::StringLiteral(value))
        } else if self.match_token(TokenType::CharLiteral) {
            let value = self.previous().literal.clone().unwrap_or_default();
            if value.is_empty() {
                Ok(Expression::CharLiteral('\0'))
            } else {
                Ok(Expression::CharLiteral(value.chars().next().unwrap()))
            }
        } else if self.match_token(TokenType::Identifier) {
            // Check if this identifier is a macro that should be expanded
            let name = self.previous().lexeme.clone();
            if self.defines.contains_key(&name) {
                // In a real implementation, we'd expand the macro here
                Ok(Expression::StringLiteral(
                    self.defines.get(&name).unwrap().clone(),
                ))
            } else {
                Ok(Expression::Variable(name))
            }
        } else if self.match_token(TokenType::LeftParen) {
            let expr = self.parse_expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression")?;
            Ok(expr)
        } else if self.match_token(TokenType::Sizeof) {
            // Parse sizeof operator
            self.consume(TokenType::LeftParen, "Expected '(' after sizeof")?;
            let expr = self.parse_expression()?;
            self.consume(
                TokenType::RightParen,
                "Expected ')' after sizeof expression",
            )?;
            Ok(Expression::SizeOf(Box::new(expr)))
        } else {
            Err(format!("Expected expression at line {}", self.peek().line))
        }
    }
} 