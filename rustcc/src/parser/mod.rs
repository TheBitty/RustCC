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
use error::{Error, ErrorKind, Result};
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
                    return Err(Error::from_token(
                        ErrorKind::SyntaxError("Unexpected token".to_string()),
                        &self.peek(),
                        format!("Unexpected token '{}' at top level", self.peek().lexeme),
                    ));
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
        Err(Error::from_token(
            ErrorKind::InvalidType("Expected type specifier".to_string()),
            &self.peek(),
            "Expected type specifier".to_string(),
        ))
    }

    // Parse an array type (used in struct fields and variable declarations)
    pub fn parse_array_type(&mut self, base_type: Type) -> Result<Type> {
        // Parse the array size if present
        let size = if !self.check(TokenType::RightBracket) {
            if let Ok(size_token) = self.consume(TokenType::IntegerLiteral, "Expected array size") {
                if let Ok(size) = size_token.lexeme.parse::<usize>() {
                    Some(size)
                } else {
                    return Err(Error::from_token(
                        ErrorKind::InvalidType("Invalid array size".to_string()),
                        size_token,
                        "Invalid array size".to_string(),
                    ));
                }
            } else {
                return Err(Error::from_token(
                    ErrorKind::InvalidType("Expected array size".to_string()),
                    &self.peek(),
                    "Expected array size".to_string(),
                ));
            }
        } else {
            None
        };

        self.consume(TokenType::RightBracket, "Expected ']' after array size")?;

        Ok(Type::Array(Box::new(base_type), size))
    }
}

#[cfg(test)]
mod tests {
    // Integration tests for parser components
}

// Any other modules in the parser directory
