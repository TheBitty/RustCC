// mod.rs
// Main entry point for the lexer module

// Private module imports
mod token_definitions;
mod utils;
mod scanner;
mod literals;
mod identifiers;
mod preprocessor;
mod operators;

// Re-exports
pub use token_definitions::*;
pub use self::utils::*;

use std::collections::HashMap;
use crate::parser::token::{Token, TokenType};

/// The Lexer struct is responsible for tokenizing C source code.
/// It scans a source string and produces a sequence of tokens that
/// represent the lexical structure of the code.
pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    keywords: HashMap<String, TokenType>,
    // Flag to indicate if we're at the start of a line (for preprocessor directives)
    at_line_start: bool,
}

impl Lexer {
    /// Creates a new lexer for the given source code
    pub fn new(source: String) -> Self {
        let keywords = token_definitions::init_keywords();
        
        Lexer {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            keywords,
            at_line_start: true,
        }
    }
}
