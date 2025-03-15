use crate::parser::ast::{Expression, Statement, SwitchCase};
use crate::parser::error::Result;
use crate::parser::token::TokenType;
use crate::parser::Parser;

impl Parser {
    /// Parse a statement
    pub fn parse_statement(&mut self) -> Result<Statement> {
        if self.match_token(TokenType::If) {
            self.parse_if_statement()
        } else if self.match_token(TokenType::While) {
            self.parse_while_statement()
        } else if self.match_token(TokenType::Do) {
            self.parse_do_while_statement()
        } else if self.match_token(TokenType::For) {
            self.parse_for_statement()
        } else if self.match_token(TokenType::Return) {
            self.parse_return_statement()
        } else if self.match_token(TokenType::Break) {
            self.parse_break_statement()
        } else if self.match_token(TokenType::Continue) {
            self.parse_continue_statement()
        } else if self.match_token(TokenType::Switch) {
            self.parse_switch_statement()
        } else if self.match_token(TokenType::LeftBrace) {
            self.parse_block()
        } else if self.match_token(TokenType::Goto) {
            self.parse_goto_statement()
        } else if self.match_token(TokenType::StaticAssert) {
            self.parse_static_assert_statement()
        } else if self.match_token(TokenType::Atomic) {
            self.parse_atomic_statement()
        } else if self.match_token(TokenType::ThreadLocal) {
            self.parse_thread_local_statement()
        } else if self.match_token(TokenType::Noreturn) {
            self.parse_noreturn_statement()
        } else if self.check(TokenType::Identifier) && self.peek_next() == TokenType::Colon {
            self.parse_labeled_statement()
        } else if self.is_type_specifier() {
            self.parse_variable_declaration()
        } else {
            self.parse_expression_statement()
        }
    }

    /// Parse a block of statements
    pub fn parse_block(&mut self) -> Result<Statement> {
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

        let then_statement = Box::new(self.parse_statement()?);
        let else_statement = if self.match_token(TokenType::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_block: then_statement,
            else_block: else_statement,
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
        self.consume(TokenType::Semicolon, "Expected ';' after do-while statement")?;

        Ok(Statement::DoWhile { body, condition })
    }

    /// Parse a for statement
    fn parse_for_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'")?;

        // Parse initializer
        let initializer = if self.match_token(TokenType::Semicolon) {
            None
        } else if self.is_type_specifier() {
            // C99 style for loop with declaration in initializer
            let declaration = self.parse_variable_declaration()?;
            Some(Box::new(declaration))
        } else {
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
            // Return with no value (void)
            Expression::IntegerLiteral(0) // Placeholder
        } else {
            self.parse_expression()?
        };

        self.consume(TokenType::Semicolon, "Expected ';' after return value")?;

        Ok(Statement::Return(value))
    }

    /// Parse a break statement
    fn parse_break_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::Semicolon, "Expected ';' after 'break'")?;
        Ok(Statement::Break)
    }

    /// Parse a continue statement
    fn parse_continue_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::Semicolon, "Expected ';' after 'continue'")?;
        Ok(Statement::Continue)
    }

    /// Parse a switch statement
    fn parse_switch_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'switch'")?;
        let expression = self.parse_expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after switch expression")?;

        self.consume(TokenType::LeftBrace, "Expected '{' after switch expression")?;

        let mut cases = Vec::new();
        let mut current_case_value = None;
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
                let value = self.parse_expression()?;
                self.consume(TokenType::Colon, "Expected ':' after case value")?;
                current_case_value = Some(value);
            } else if self.match_token(TokenType::Default) {
                // If we were building a case, add it to the list
                if !current_case_statements.is_empty() {
                    cases.push(SwitchCase {
                        value: current_case_value,
                        statements: current_case_statements,
                    });
                    current_case_statements = Vec::new();
                }

                self.consume(TokenType::Colon, "Expected ':' after 'default'")?;
                current_case_value = None; // None represents default case
            } else {
                // Parse a statement for the current case
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

    /// Parse an expression statement
    fn parse_expression_statement(&mut self) -> Result<Statement> {
        let expr = self.parse_expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::ExpressionStatement(expr))
    }

    /// Parse a goto statement
    fn parse_goto_statement(&mut self) -> Result<Statement> {
        let label = self.consume(TokenType::Identifier, "Expected label name after 'goto'")?
            .lexeme
            .clone();
        self.consume(TokenType::Semicolon, "Expected ';' after goto label")?;
        Ok(Statement::Goto(label))
    }

    /// Parse a labeled statement
    fn parse_labeled_statement(&mut self) -> Result<Statement> {
        let label = self.consume(TokenType::Identifier, "Expected label name")?
            .lexeme
            .clone();
        self.consume(TokenType::Colon, "Expected ':' after label name")?;
        let statement = self.parse_statement()?;
        Ok(Statement::Label(label, Box::new(statement)))
    }

    /// Parse a _Static_assert statement (C11)
    fn parse_static_assert_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "Expected '(' after '_Static_assert'")?;
        
        // Parse the condition
        let condition = self.parse_expression()?;
        
        self.consume(TokenType::Comma, "Expected ',' after condition in _Static_assert")?;
        
        // Parse the message string
        let message_token = self.consume(TokenType::StringLiteral, "Expected string literal message in _Static_assert")?;
        let message = message_token.lexeme.clone();
        
        self.consume(TokenType::RightParen, "Expected ')' after _Static_assert")?;
        self.consume(TokenType::Semicolon, "Expected ';' after _Static_assert")?;
        
        Ok(Statement::StaticAssert {
            condition,
            message,
        })
    }

    /// Parse an atomic statement (C11)
    fn parse_atomic_statement(&mut self) -> Result<Statement> {
        // _Atomic compound statement
        self.consume(TokenType::LeftBrace, "Expected '{' after '_Atomic'")?;
        
        let mut statements = Vec::new();
        
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.consume(TokenType::RightBrace, "Expected '}' after atomic block")?;
        
        Ok(Statement::AtomicBlock(statements))
    }

    /// Parse a _Thread_local statement (C11)
    fn parse_thread_local_statement(&mut self) -> Result<Statement> {
        // _Thread_local declaration
        let declaration = self.parse_statement()?;
        
        Ok(Statement::ThreadLocal {
            declaration: Box::new(declaration),
        })
    }

    /// Parse a _Noreturn statement (C11)
    fn parse_noreturn_statement(&mut self) -> Result<Statement> {
        // _Noreturn function
        let declaration = self.parse_statement()?;
        
        Ok(Statement::NoReturn {
            declaration: Box::new(declaration),
        })
    }
}
