use crate::parser::ast::{Expression, Statement, SwitchCase};
use crate::parser::error::Result;
use crate::parser::token::TokenType;
use crate::parser::Parser;

impl Parser {
    /// Parse a statement
    pub fn parse_statement(&mut self) -> Result<Statement> {
        // Check for different statement types
        if self.match_token(TokenType::LeftBrace) {
            return self.parse_block();
        } else if self.match_token(TokenType::If) {
            return self.parse_if_statement();
        } else if self.match_token(TokenType::While) {
            return self.parse_while_statement();
        } else if self.match_token(TokenType::Do) {
            return self.parse_do_while_statement();
        } else if self.match_token(TokenType::For) {
            return self.parse_for_statement();
        } else if self.match_token(TokenType::Return) {
            return self.parse_return_statement();
        } else if self.match_token(TokenType::Break) {
            self.consume(TokenType::Semicolon, "Expected ';' after 'break'")?;
            return Ok(Statement::Break);
        } else if self.match_token(TokenType::Continue) {
            self.consume(TokenType::Semicolon, "Expected ';' after 'continue'")?;
            return Ok(Statement::Continue);
        } else if self.match_token(TokenType::Switch) {
            return self.parse_switch_statement();
        } else if self.is_type_specifier() {
            // Variable declaration
            return self.parse_variable_declaration();
        }

        // If none of the above, it's an expression statement
        let expr = self.parse_expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::ExpressionStatement(expr))
    }

    /// Parse a block of statements
    fn parse_block(&mut self) -> Result<Statement> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block")?;
        Ok(Statement::Block(statements))
    }

    /// Parse an if statement
    fn parse_if_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after if condition")?;

        let then_block = Box::new(self.parse_statement()?);

        let else_block = if self.match_token(TokenType::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    /// Parse a while statement
    fn parse_while_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after while condition")?;

        let body = Box::new(self.parse_statement()?);

        Ok(Statement::While { condition, body })
    }

    /// Parse a do-while statement
    fn parse_do_while_statement(&mut self) -> Result<Statement> {
        let body = Box::new(self.parse_statement()?);

        self.consume(TokenType::While, "Expected 'while' after do block")?;
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after while condition")?;
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after do-while statement",
        )?;

        Ok(Statement::DoWhile { body, condition })
    }

    /// Parse a for statement
    fn parse_for_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'")?;

        // Parse initializer
        let initializer = if self.match_token(TokenType::Semicolon) {
            None
        } else if self.is_type_specifier() {
            // Variable declaration as initializer
            let init_stmt = self.parse_variable_declaration()?;
            Some(Box::new(init_stmt))
        } else {
            // Expression as initializer
            let expr = self.parse_expression()?;
            self.consume(TokenType::Semicolon, "Expected ';' after for initializer")?;
            Some(Box::new(Statement::ExpressionStatement(expr)))
        };

        // Parse condition
        let condition = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(TokenType::Semicolon, "Expected ';' after for condition")?;

        // Parse increment
        let increment = if self.check(TokenType::RightParen) {
            None
        } else {
            Some(self.parse_expression()?)
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
    }

    /// Parse a return statement
    fn parse_return_statement(&mut self) -> Result<Statement> {
        let value = if self.check(TokenType::Semicolon) {
            // Return without value (void)
            Expression::IntegerLiteral(0) // Placeholder for void return
        } else {
            self.parse_expression()?
        };

        self.consume(TokenType::Semicolon, "Expected ';' after return value")?;
        Ok(Statement::Return(value))
    }

    /// Parse a switch statement
    fn parse_switch_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'switch'")?;
        let expression = self.parse_expression()?;
        self.consume(
            TokenType::RightParen,
            "Expected ')' after switch expression",
        )?;

        self.consume(TokenType::LeftBrace, "Expected '{' after switch expression")?;

        let mut cases = Vec::new();
        let mut current_case_value: Option<Expression> = None;
        let mut current_case_statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if self.match_token(TokenType::Case) {
                // If we were building a case, add it to the list
                if !current_case_statements.is_empty() {
                    cases.push(SwitchCase {
                        value: current_case_value,
                        statements: current_case_statements,
                    });
                    current_case_statements = Vec::new();
                }

                // Parse the case value
                current_case_value = Some(self.parse_expression()?);
                self.consume(TokenType::Colon, "Expected ':' after case value")?;
            } else if self.match_token(TokenType::Default) {
                // If we were building a case, add it to the list
                if !current_case_statements.is_empty() {
                    cases.push(SwitchCase {
                        value: current_case_value,
                        statements: current_case_statements,
                    });
                    current_case_statements = Vec::new();
                }

                // Default case has no value
                current_case_value = None;
                self.consume(TokenType::Colon, "Expected ':' after 'default'")?;
            } else {
                // Parse statement for the current case
                current_case_statements.push(self.parse_statement()?);
            }
        }

        // Add the last case if there is one
        if !current_case_statements.is_empty() {
            cases.push(SwitchCase {
                value: current_case_value,
                statements: current_case_statements,
            });
        }

        self.consume(TokenType::RightBrace, "Expected '}' after switch cases")?;

        Ok(Statement::Switch { expression, cases })
    }
}
