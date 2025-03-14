pub mod ast;
pub mod lexer;
pub mod token;

// New submodules
mod declarations;
mod expressions;
mod preprocessor;
mod statements;
mod utils;

use ast::Program;
use std::collections::HashMap;
use token::{Token, TokenType};

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
        let mut global_variables = Vec::new();

        while !self.is_at_end() {
            // Skip over preprocessor directives that were already processed
            if self.check(TokenType::Hash) {
                self.advance(); // Skip the # token
                if !self.is_at_end() && self.check_any(&[
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
                ]) {
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
            
            // Handle struct declarations
            if self.match_token(TokenType::Struct) {
                // Parse struct declaration
                structs.push(self.parse_struct()?);
            }
            // Handle function declarations and global variables with different types
            else if self.check(TokenType::Int)
                || self.check(TokenType::Void)
                || self.check(TokenType::Char)
                || self.check(TokenType::Const)
                || self.check(TokenType::Struct)
            {
                let return_type = self.parse_type()?;
                
                // Check if it's a function or global variable
                let name_token = self.consume(TokenType::Identifier, "Expected identifier after type")?;
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
                    self.advance();
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
}

#[cfg(test)]
mod tests {
    // Integration tests for parser components
}

// Any other modules in the parser directory 