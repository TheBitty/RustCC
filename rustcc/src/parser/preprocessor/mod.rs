use crate::parser::Parser;
use crate::parser::token::TokenType;

impl Parser {
    // Process preprocessor directives first
    pub fn process_preprocessor_directives(&mut self) -> Result<(), String> {
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
                                        || self.tokens[i + 3].token_type == TokenType::IntegerLiteral
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

    pub fn is_at_preprocessor_directive(&self) -> bool {
        if self.current > 0 {
            return self.tokens[self.current - 1].token_type == TokenType::Hash;
        }
        false
    }
} 