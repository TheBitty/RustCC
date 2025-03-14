use crate::parser::ast::{Statement, SwitchCase};
use crate::parser::Parser;
use crate::parser::token::TokenType;

impl Parser {
    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        if self.match_token(TokenType::Return) {
            let expr = self.parse_expression()?;
            self.consume(TokenType::Semicolon, "Expected ';' after return statement")?;
            Ok(Statement::Return(expr))
        } else if self.match_token(TokenType::LeftBrace) {
            // Parse a block of statements
            let mut statements = Vec::new();
            while !self.check(TokenType::RightBrace) && !self.is_at_end() {
                statements.push(self.parse_statement()?);
            }
            self.consume(TokenType::RightBrace, "Expected '}' after block")?;
            Ok(Statement::Block(statements))
        } else if self.match_token(TokenType::If) {
            // Parse if statement
            self.consume(TokenType::LeftParen, "Expected '(' after 'if'")?;
            let condition = self.parse_expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after condition")?;

            let then_branch = Box::new(self.parse_statement()?);

            let else_branch = if self.match_token(TokenType::Else) {
                Some(Box::new(self.parse_statement()?))
            } else {
                None
            };

            Ok(Statement::If {
                condition,
                then_block: then_branch,
                else_block: else_branch,
            })
        } else if self.match_token(TokenType::While) {
            // Parse while loop
            self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
            let condition = self.parse_expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after condition")?;

            let body = Box::new(self.parse_statement()?);

            Ok(Statement::While { condition, body })
        } else if self.match_token(TokenType::For) {
            // Parse for loop
            self.consume(TokenType::LeftParen, "Expected '(' after 'for'")?;

            // Parse initializer
            let initializer = if !self.check(TokenType::Semicolon) {
                if self.check(TokenType::Int) || self.check(TokenType::Char) || self.check(TokenType::Const) {
                    // Variable declaration as initializer
                    Some(Box::new(self.parse_variable_declaration()?))
                } else {
                    // Expression as initializer
                    let expr = self.parse_expression()?;
                    self.consume(TokenType::Semicolon, "Expected ';' after loop initializer")?;
                    Some(Box::new(Statement::ExpressionStatement(expr)))
                }
            } else {
                self.consume(TokenType::Semicolon, "Expected ';' after loop initializer")?;
                None
            };

            // Parse condition
            let condition = if !self.check(TokenType::Semicolon) {
                let expr = self.parse_expression()?;
                Some(expr)
            } else {
                None
            };
            self.consume(TokenType::Semicolon, "Expected ';' after loop condition")?;

            // Parse increment
            let increment = if !self.check(TokenType::RightParen) {
                let expr = self.parse_expression()?;
                Some(expr)
            } else {
                None
            };
            self.consume(TokenType::RightParen, "Expected ')' after for clauses")?;

            // Parse body
            let body = Box::new(self.parse_statement()?);

            Ok(Statement::For {
                initializer,
                condition,
                increment,
                body,
            })
        } else if self.match_token(TokenType::Switch) {
            // Parse switch statement
            self.consume(TokenType::LeftParen, "Expected '(' after 'switch'")?;
            let expr = self.parse_expression()?;
            self.consume(
                TokenType::RightParen,
                "Expected ')' after switch expression",
            )?;

            self.consume(TokenType::LeftBrace, "Expected '{' before switch cases")?;

            let mut cases = Vec::new();

            while !self.check(TokenType::RightBrace) && !self.is_at_end() {
                if self.match_token(TokenType::Case) {
                    // Parse case
                    let value = self.parse_expression()?;
                    self.consume(TokenType::Colon, "Expected ':' after case value")?;

                    let mut statements = Vec::new();
                    while !self.check(TokenType::Case)
                        && !self.check(TokenType::Default)
                        && !self.check(TokenType::RightBrace)
                        && !self.is_at_end()
                    {
                        statements.push(self.parse_statement()?);
                    }

                    cases.push(SwitchCase {
                        value: Some(value),
                        statements,
                    });
                } else if self.match_token(TokenType::Default) {
                    // Parse default case
                    self.consume(TokenType::Colon, "Expected ':' after 'default'")?;

                    let mut statements = Vec::new();
                    while !self.check(TokenType::Case)
                        && !self.check(TokenType::Default)
                        && !self.check(TokenType::RightBrace)
                        && !self.is_at_end()
                    {
                        statements.push(self.parse_statement()?);
                    }

                    cases.push(SwitchCase {
                        value: None,
                        statements,
                    });
                } else {
                    return Err("Expected 'case' or 'default' in switch statement".to_string());
                }
            }

            self.consume(TokenType::RightBrace, "Expected '}' after switch cases")?;

            Ok(Statement::Switch {
                expression: expr,
                cases,
            })
        } else if self.match_token(TokenType::Break) {
            self.consume(TokenType::Semicolon, "Expected ';' after 'break'")?;
            Ok(Statement::Break)
        } else if self.match_token(TokenType::Continue) {
            self.consume(TokenType::Semicolon, "Expected ';' after 'continue'")?;
            Ok(Statement::Continue)
        } else if self.match_token(TokenType::Do) {
            // Parse do-while loop
            let body = Box::new(self.parse_statement()?);

            self.consume(TokenType::While, "Expected 'while' after do body")?;
            self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
            let condition = self.parse_expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after condition")?;
            self.consume(
                TokenType::Semicolon,
                "Expected ';' after do-while condition",
            )?;

            Ok(Statement::DoWhile { body, condition })
        } else if self.check(TokenType::Int) || self.check(TokenType::Char) || self.check(TokenType::Const) {
            // Variable declaration
            self.parse_variable_declaration()
        } else {
            // Expression statement
            let expr = self.parse_expression()?;
            self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
            Ok(Statement::ExpressionStatement(expr))
        }
    }
} 