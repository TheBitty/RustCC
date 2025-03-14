use super::ast::{BinaryOp, Expression, Function, FunctionParameter, Program, Statement, Type};
use super::token::{Token, TokenType};
use std::collections::HashMap;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    // Track preprocessor definitions for macro expansion
    defines: HashMap<String, String>,
    // Track included files
    includes: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            defines: HashMap::new(),
            includes: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        // First process all preprocessor directives
        self.process_preprocessor_directives()?;

        let mut functions = Vec::new();
        let mut structs = Vec::new();

        while !self.is_at_end() {
            // Handle struct declarations
            if self.match_token(TokenType::Struct) {
                // Parse struct declaration
                structs.push(self.parse_struct()?);
            }
            // Handle function declarations with different return types
            else if self.check(TokenType::Int)
                || self.check(TokenType::Void)
                || self.check(TokenType::Char)
                || self.check(TokenType::Struct)
            {
                let return_type = self.parse_type()?;
                functions.push(self.parse_function(return_type)?);
            } else {
                // Skip unrecognized tokens
                if !self.is_at_preprocessor_directive() && !self.is_at_end() {
                    return Err(format!(
                        "Expected type declaration at line {}",
                        self.peek().line
                    ));
                }
                if !self.is_at_end() {
                    self.advance();
                }
            }
        }

        Ok(Program {
            functions,
            structs,
            includes: self.includes.clone(),
        })
    }

    // Process preprocessor directives first
    fn process_preprocessor_directives(&mut self) -> Result<(), String> {
        let mut i = 0;
        while i < self.tokens.len() {
            if self.tokens[i].token_type == TokenType::Hash {
                // Look for preprocessor directive after hash
                if i + 1 < self.tokens.len() {
                    match self.tokens[i + 1].token_type {
                        TokenType::PPInclude => {
                            // Handle include directive
                            if i + 2 < self.tokens.len()
                                && self.tokens[i + 2].token_type == TokenType::StringLiteral
                            {
                                if let Some(path) = &self.tokens[i + 2].literal {
                                    self.includes.push(path.clone());
                                }
                            }
                            // Skip this directive in subsequent parsing
                            i += 3; // Skip #include "file.h"
                            continue;
                        }
                        TokenType::PPDefine => {
                            // Handle define directive
                            if i + 2 < self.tokens.len()
                                && self.tokens[i + 2].token_type == TokenType::Identifier
                            {
                                let name = self.tokens[i + 2].lexeme.clone();
                                if i + 3 < self.tokens.len() {
                                    // Handle various define value formats
                                    if self.tokens[i + 3].token_type == TokenType::StringLiteral
                                        || self.tokens[i + 3].token_type
                                            == TokenType::IntegerLiteral
                                    {
                                        if let Some(value) = &self.tokens[i + 3].literal {
                                            self.defines.insert(name, value.clone());
                                        }
                                    }
                                }
                            }
                            // Skip to end of line
                            while i < self.tokens.len()
                                && self.tokens[i].token_type != TokenType::Eof
                            {
                                i += 1;
                                if i < self.tokens.len()
                                    && self.tokens[i].line > self.tokens[i - 1].line
                                {
                                    break;
                                }
                            }
                            continue;
                        }
                        // Other preprocessor directives
                        TokenType::PPIfDef
                        | TokenType::PPIfNDef
                        | TokenType::PPIf
                        | TokenType::PPElse
                        | TokenType::PPElif
                        | TokenType::PPEndif
                        | TokenType::PPUndef
                        | TokenType::PPPragma
                        | TokenType::PPErrorDir
                        | TokenType::PPWarning => {
                            // Skip to end of line
                            while i < self.tokens.len()
                                && self.tokens[i].token_type != TokenType::Eof
                            {
                                i += 1;
                                if i < self.tokens.len()
                                    && self.tokens[i].line > self.tokens[i - 1].line
                                {
                                    break;
                                }
                            }
                            continue;
                        }
                        _ => {}
                    }
                }
            }
            i += 1;
        }

        // Reset current position after preprocessing
        self.current = 0;
        Ok(())
    }

    fn is_at_preprocessor_directive(&self) -> bool {
        if self.current > 0 {
            return self.tokens[self.current - 1].token_type == TokenType::Hash;
        }
        false
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        if self.match_token(TokenType::Int) {
            // Check for pointer types
            if self.match_token(TokenType::Star) {
                return Ok(Type::Pointer(Box::new(Type::Int)));
            }
            Ok(Type::Int)
        } else if self.match_token(TokenType::Void) {
            if self.match_token(TokenType::Star) {
                return Ok(Type::Pointer(Box::new(Type::Void)));
            }
            Ok(Type::Void)
        } else if self.match_token(TokenType::Char) {
            if self.match_token(TokenType::Star) {
                return Ok(Type::Pointer(Box::new(Type::Char)));
            }
            Ok(Type::Char)
        } else if self.match_token(TokenType::Struct) {
            // Handle struct types
            let name = self
                .consume(TokenType::Identifier, "Expected struct name")?
                .lexeme
                .clone();
            if self.match_token(TokenType::Star) {
                return Ok(Type::Pointer(Box::new(Type::Struct(name))));
            }
            Ok(Type::Struct(name))
        } else {
            Err("Expected type".to_string())
        }
    }

    fn parse_function(&mut self, return_type: Type) -> Result<Function, String> {
        let name_token = self.consume(TokenType::Identifier, "Expected function name")?;
        let name = name_token.lexeme.clone();

        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;

        // Parse parameters list
        let mut parameters = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
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
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
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
                if self.check(TokenType::Int) || self.check(TokenType::Char) {
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

                    cases.push(crate::parser::ast::SwitchCase {
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

                    cases.push(crate::parser::ast::SwitchCase {
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
        } else if self.check(TokenType::Int) || self.check(TokenType::Char) {
            // Variable declaration
            self.parse_variable_declaration()
        } else {
            // Expression statement
            let expr = self.parse_expression()?;
            self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
            Ok(Statement::ExpressionStatement(expr))
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, String> {
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
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
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
                TokenType::Bang => super::ast::OperatorType::Binary(BinaryOp::LogicalNot),
                TokenType::Minus => super::ast::OperatorType::Binary(BinaryOp::Negate),
                TokenType::Increment => super::ast::OperatorType::Binary(BinaryOp::PreIncrement),
                TokenType::Decrement => super::ast::OperatorType::Binary(BinaryOp::PreDecrement),
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
                    operator: super::ast::OperatorType::Binary(BinaryOp::PostIncrement),
                    operand: Box::new(expr),
                };
            } else if self.match_token(TokenType::Decrement) {
                // Post-decrement
                expr = Expression::UnaryOperation {
                    operator: super::ast::OperatorType::Binary(BinaryOp::PostDecrement),
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

    // Helper method to synchronize after error
    #[allow(dead_code)]
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Int
                | TokenType::Void
                | TokenType::Char
                | TokenType::If
                | TokenType::While
                | TokenType::For
                | TokenType::Return
                | TokenType::Struct => return,
                _ => {}
            }

            self.advance();
        }
    }

    // Helper methods
    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_any(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(format!("{} at line {}", message, self.peek().line))
        }
    }

    fn parse_struct(&mut self) -> Result<crate::parser::ast::Struct, String> {
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

            fields.push(crate::parser::ast::StructField {
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
