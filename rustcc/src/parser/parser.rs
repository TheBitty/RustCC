use super::ast::{Program, Function, Statement, Expression, BinaryOp, Type};
use super::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();
        
        while !self.is_at_end() {
            if self.match_token(TokenType::Int) {
                functions.push(self.parse_function()?);
            } else {
                return Err("Expected function declaration".to_string());
            }
        }

        // No struct declarations for now
        let structs = Vec::new();

        Ok(Program { functions, structs, includes: Vec::new() })
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        // Already consumed 'int'
        let return_type = Type::Int; // Default to int for now
        
        let name_token = self.consume(TokenType::Identifier, "Expected function name")?;
        let name = name_token.lexeme.clone();
        
        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;
        
        // Parse parameters (currently we don't support parameters, just an empty list)
        let parameters = Vec::new();
        
        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;
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
        } else if self.match_token(TokenType::Int) {
            let name_token = self.consume(TokenType::Identifier, "Expected variable name")?;
            let name = name_token.lexeme.clone();
            
            self.consume(TokenType::Equal, "Expected '=' after variable name")?;
            let initializer = self.parse_expression()?;
            self.consume(TokenType::Semicolon, "Expected ';' after variable declaration")?;
            
            Ok(Statement::VariableDeclaration {
                name,
                data_type: Some(Type::Int), // Default to int for now
                initializer,
            })
        } else {
            Err("Expected statement".to_string())
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_additive()
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
        let mut expr = self.parse_primary()?;

        while self.match_any(&[TokenType::Star, TokenType::Slash]) {
            let operator = if self.previous().token_type == TokenType::Star {
                BinaryOp::Multiply
            } else {
                BinaryOp::Divide
            };
            let right = self.parse_primary()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        if self.match_token(TokenType::IntegerLiteral) {
            let value = self.previous().lexeme.parse::<i32>()
                .map_err(|_| "Invalid integer literal".to_string())?;
            Ok(Expression::IntegerLiteral(value))
        } else if self.match_token(TokenType::Identifier) {
            Ok(Expression::Variable(self.previous().lexeme.clone()))
        } else {
            Err("Expected expression".to_string())
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
        self.peek().token_type == TokenType::EOF
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
            Err(message.to_string())
        }
    }
} 