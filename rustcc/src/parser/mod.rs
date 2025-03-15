pub mod ast;
pub mod error;
pub mod lexer;
pub mod token;

// Submodules
mod declarations;
mod expressions;
mod statements;
mod utils;

use ast::{Program, Type};
use error::Result;
use std::collections::HashMap;
use token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    // Track preprocessor definitions for macro expansion
    _defines: HashMap<String, String>,
    // Track included files
    includes: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            _defines: HashMap::new(),
            includes: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Program> {
        // Initialize program components
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut global_variables = Vec::new();

        while !self.is_at_end() {
            // Skip over preprocessor directives that were already processed
            if self.check(TokenType::Hash) {
                self.advance(); // Skip the # token
                if !self.is_at_end()
                    && self.check_any(&[
                        TokenType::PPInclude,
                        TokenType::PPDefine,
                        TokenType::PPIfDef,
                        TokenType::PPIfNDef,
                        TokenType::PPIf,
                        TokenType::PPElse,
                        TokenType::PPElif,
                        TokenType::PPEndif,
                        TokenType::PPUndef,
                        TokenType::PPPragma,
                        TokenType::PPErrorDir,
                        TokenType::PPWarning,
                    ])
                {
                    // Skip to the end of the line
                    while !self.is_at_end() && !self.is_at_new_line() {
                        self.advance();
                    }
                    if !self.is_at_end() {
                        self.advance(); // Skip the newline
                    }
                    continue;
                }
            }

            // Handle typedef declarations
            if self.match_token(TokenType::Typedef) {
                self.parse_typedef()?;
                continue;
            }

            // Handle enum declarations
            if self.match_token(TokenType::Enum) {
                self.parse_enum()?;
                continue;
            }

            // Handle struct declarations
            if self.match_token(TokenType::Struct) {
                // Parse struct declaration
                structs.push(self.parse_struct()?);
                continue;
            }

            // Handle function declarations and global variables with different types
            if self.is_type_specifier() {
                let return_type = self.parse_type()?;

                // Check if it's a function or global variable
                let name_token =
                    self.consume(TokenType::Identifier, "Expected identifier after type")?;
                let name = name_token.lexeme.clone();

                if self.check(TokenType::LeftParen) {
                    // This is a function declaration
                    functions.push(self.parse_function_with_name(return_type, name)?);
                } else {
                    // This is a global variable declaration
                    global_variables.push(self.parse_global_variable_with_name(return_type, name)?);
                }
            } else {
                // Skip unrecognized tokens
                if !self.is_at_end() {
                    return Err(self.unexpected_token_error("declaration or definition"));
                }
            }
        }

        Ok(Program {
            functions,
            structs,
            includes: self.includes.clone(),
            globals: global_variables,
        })
    }

    // Helper method to check if the current token is a type specifier
    pub fn is_type_specifier(&self) -> bool {
        self.check(TokenType::Int)
            || self.check(TokenType::Void)
            || self.check(TokenType::Char)
            || self.check(TokenType::Const)
            || self.check(TokenType::Struct)
            || self.check(TokenType::Long)
            || self.check(TokenType::Short)
            || self.check(TokenType::Unsigned)
            || self.check(TokenType::Signed)
            || self.check(TokenType::Float)
            || self.check(TokenType::Double)
    }

    // Parse a type specifier
    pub fn parse_type(&mut self) -> Result<Type> {
        // Handle basic types
        if self.match_token(TokenType::Int) {
            return Ok(Type::Int);
        } else if self.match_token(TokenType::Char) {
            return Ok(Type::Char);
        } else if self.match_token(TokenType::Void) {
            return Ok(Type::Void);
        } else if self.match_token(TokenType::Struct) {
            // Parse struct type
            let struct_name = self
                .consume(TokenType::Identifier, "Expected struct name")?
                .lexeme
                .clone();
            return Ok(Type::Struct(struct_name));
        } else if self.match_token(TokenType::Const) {
            // Parse const type
            let base_type = self.parse_type()?;
            return Ok(Type::Const(Box::new(base_type)));
        }

        // If we get here, it's an error
        Err(self.unexpected_token_error("type specifier"))
    }

    // Parse an array type (used in struct fields and variable declarations)
    pub fn parse_array_type(&mut self, base_type: Type) -> Result<Type> {
        // Parse the array size if present
        let size = if !self.check(TokenType::RightBracket) {
            if let Ok(size_token) = self.consume(TokenType::IntegerLiteral, "Expected array size") {
                let token_lexeme = size_token.lexeme.clone();
                if let Ok(size) = token_lexeme.parse::<usize>() {
                    Some(size)
                } else {
                    return Err(self.error(
                        error::ErrorKind::InvalidArraySize(token_lexeme),
                        self.current - 1
                    ));
                }
            } else {
                return Err(self.unexpected_token_error("array size"));
            }
        } else {
            None
        };

        self.consume(TokenType::RightBracket, "Expected ']' after array size")?;

        Ok(Type::Array(Box::new(base_type), size))
    }

    // Add a method to create errors with source context
    fn error(&self, kind: error::ErrorKind, token_index: usize) -> error::Error {
        let token = &self.tokens[token_index];
        let line = token.line;
        let column = token.column;
        
        // Create the basic error
        let mut err = error::Error::new(kind, line, column);
        
        // Add source context if available
        if let Some(source_line) = self.get_source_line(line) {
            err = err.with_source_line(source_line);
        }
        
        err
    }

    // Get the source line for a given line number
    fn get_source_line(&self, line: usize) -> Option<String> {
        // This is a simplified implementation
        // In a real compiler, we would store the source code and retrieve the actual line
        
        // For now, we'll reconstruct the line from tokens on the same line
        let line_tokens: Vec<&Token> = self.tokens.iter()
            .filter(|t| t.line == line)
            .collect();
        
        if line_tokens.is_empty() {
            return None;
        }
        
        // Sort tokens by column
        let mut sorted_tokens = line_tokens.clone();
        sorted_tokens.sort_by_key(|t| t.column);
        
        // Reconstruct the line
        let mut result = String::new();
        let mut last_column = 0;
        
        for token in sorted_tokens {
            // Add spaces between tokens
            if token.column > last_column {
                result.push_str(&" ".repeat(token.column - last_column));
            }
            
            // Add the token lexeme
            result.push_str(&token.lexeme);
            
            // Update last column
            last_column = token.column + token.lexeme.len();
        }
        
        Some(result)
    }

    // Helper method to create an unexpected token error
    fn unexpected_token_error(&self, expected: &str) -> error::Error {
        let token = &self.tokens[self.current];
        let kind = error::ErrorKind::UnexpectedToken(
            token.lexeme.clone(),
            expected.to_string(),
        );
        
        self.error(kind, self.current)
    }

    // Helper method to create an unexpected EOF error
    #[allow(dead_code)]
    fn unexpected_eof_error(&self, expected: &str) -> error::Error {
        let last_token = self.tokens.last().unwrap();
        let kind = error::ErrorKind::UnexpectedEOF(expected.to_string());
        
        error::Error::new(kind, last_token.line, last_token.column)
    }
}

#[cfg(test)]
mod tests {
    // Integration tests for parser components
}

// Any other modules in the parser directory
