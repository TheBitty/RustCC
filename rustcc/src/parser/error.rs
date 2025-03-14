use crate::parser::token::Token;
use std::error::Error as StdError;
use std::fmt;

/// Represents the location of an error in the source code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub file: Option<String>,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.file {
            Some(file) => write!(f, "{}:{}:{}", file, self.line, self.column),
            None => write!(f, "line {}, column {}", self.line, self.column),
        }
    }
}

impl From<&Token> for SourceLocation {
    fn from(token: &Token) -> Self {
        SourceLocation {
            line: token.line,
            column: token.column,
            file: None,
        }
    }
}

/// Represents the different types of parser errors
#[derive(Debug)]
pub enum ErrorKind {
    /// Expected a specific token but found something else
    UnexpectedToken { expected: String, found: String },
    /// Invalid syntax in a statement
    InvalidStatement(String),
    /// Invalid syntax in an expression
    InvalidExpression(String),
    /// Invalid syntax in a declaration
    InvalidDeclaration(String),
    /// Invalid type specification
    InvalidType(String),
    /// Unexpected end of input
    UnexpectedEOF,
    /// General syntax error
    SyntaxError(String),
    /// Error in preprocessor directive
    PreprocessorError(String),
}

/// Represents a parser error with location information
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub location: SourceLocation,
    pub message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, location: SourceLocation, message: String) -> Self {
        Error {
            kind,
            location,
            message,
        }
    }

    pub fn from_token(kind: ErrorKind, token: &Token, message: String) -> Self {
        Error {
            kind,
            location: SourceLocation::from(token),
            message,
        }
    }

    pub fn unexpected_token(expected: &str, found: &Token) -> Self {
        Error {
            kind: ErrorKind::UnexpectedToken {
                expected: expected.to_string(),
                found: found.lexeme.clone(),
            },
            location: SourceLocation::from(found),
            message: format!("Expected {}, found '{}'", expected, found.lexeme),
        }
    }

    pub fn unexpected_eof(line: usize, column: usize) -> Self {
        Error {
            kind: ErrorKind::UnexpectedEOF,
            location: SourceLocation {
                line,
                column,
                file: None,
            },
            message: "Unexpected end of file".to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.location, self.message)
    }
}

impl StdError for Error {}

// Implement From<Error> for String to allow error conversion with the ? operator
impl From<Error> for String {
    fn from(error: Error) -> Self {
        error.to_string()
    }
}

/// Result type for parser operations
pub type Result<T> = std::result::Result<T, Error>;
