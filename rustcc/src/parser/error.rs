use crate::parser::token::Token;
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

/// Error type for the parser
#[derive(Debug, Clone)]
pub struct Error {
    /// The kind of error
    pub kind: ErrorKind,
    /// The line number where the error occurred
    pub line: usize,
    /// The column number where the error occurred
    pub column: usize,
    /// The file name where the error occurred
    pub file: Option<String>,
    /// Additional context for the error
    pub context: Option<String>,
    /// The source code line where the error occurred
    pub source_line: Option<String>,
}

/// Error kind for the parser
#[derive(Debug, Clone)]
pub enum ErrorKind {
    /// Lexical errors
    InvalidCharacter(char),
    UnterminatedString,
    UnterminatedComment,
    
    /// Parsing errors
    UnexpectedToken(String, String), // (found, expected)
    UnexpectedEOF(String), // expected
    InvalidType(String), // message
    InvalidArraySize(String), // size string
    MissingIdentifier(String), // context
    MissingSemicolon,
    MissingClosingBrace,
    MissingClosingParen,
    
    /// Semantic errors
    UndefinedVariable(String),
    UndefinedFunction(String),
    UndefinedType(String),
    TypeMismatch(String, String), // (expected, found)
    InvalidOperator(String, String), // (operator, type)
    NotCallable(String),
    WrongArgumentCount(String, usize, usize), // (function, expected, found)
    InvalidAssignment(String), // message
    BreakOutsideLoop,
    ContinueOutsideLoop,
    ReturnOutsideFunction,
    ReturnTypeMismatch(String, String), // (expected, found)
    
    /// Other errors
    IOError(String),
    PreprocessorError(String),
    InternalError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let location = if let Some(file) = &self.file {
            format!("{}:{}:{}", file, self.line, self.column)
        } else {
            format!("line {}, column {}", self.line, self.column)
        };

        let error_message = match &self.kind {
            // Lexical errors
            ErrorKind::InvalidCharacter(c) => {
                format!("Invalid character: '{}'", c)
            }
            ErrorKind::UnterminatedString => {
                "Unterminated string literal".to_string()
            }
            ErrorKind::UnterminatedComment => {
                "Unterminated comment".to_string()
            }
            
            // Parsing errors
            ErrorKind::UnexpectedToken(found, expected) => {
                format!("Unexpected token '{}', expected {}", found, expected)
            }
            ErrorKind::UnexpectedEOF(expected) => {
                format!("Unexpected end of file, expected {}", expected)
            }
            ErrorKind::InvalidType(message) => {
                format!("Invalid type: {}", message)
            }
            ErrorKind::InvalidArraySize(size) => {
                format!("Invalid array size: {}", size)
            }
            ErrorKind::MissingIdentifier(context) => {
                format!("Missing identifier in {}", context)
            }
            ErrorKind::MissingSemicolon => {
                "Missing semicolon".to_string()
            }
            ErrorKind::MissingClosingBrace => {
                "Missing closing brace '}'".to_string()
            }
            ErrorKind::MissingClosingParen => {
                "Missing closing parenthesis ')'".to_string()
            }
            
            // Semantic errors
            ErrorKind::UndefinedVariable(name) => {
                format!("Undefined variable: '{}'", name)
            }
            ErrorKind::UndefinedFunction(name) => {
                format!("Undefined function: '{}'", name)
            }
            ErrorKind::UndefinedType(name) => {
                format!("Undefined type: '{}'", name)
            }
            ErrorKind::TypeMismatch(expected, found) => {
                format!("Type mismatch: expected '{}', found '{}'", expected, found)
            }
            ErrorKind::InvalidOperator(op, type_name) => {
                format!("Invalid operator '{}' for type '{}'", op, type_name)
            }
            ErrorKind::NotCallable(expr) => {
                format!("Expression is not callable: '{}'", expr)
            }
            ErrorKind::WrongArgumentCount(func, expected, found) => {
                format!("Wrong number of arguments for function '{}': expected {}, found {}", 
                        func, expected, found)
            }
            ErrorKind::InvalidAssignment(message) => {
                format!("Invalid assignment: {}", message)
            }
            ErrorKind::BreakOutsideLoop => {
                "Break statement outside of loop".to_string()
            }
            ErrorKind::ContinueOutsideLoop => {
                "Continue statement outside of loop".to_string()
            }
            ErrorKind::ReturnOutsideFunction => {
                "Return statement outside of function".to_string()
            }
            ErrorKind::ReturnTypeMismatch(expected, found) => {
                format!("Return type mismatch: expected '{}', found '{}'", expected, found)
            }
            
            // Other errors
            ErrorKind::IOError(message) => {
                format!("I/O error: {}", message)
            }
            ErrorKind::PreprocessorError(message) => {
                format!("Preprocessor error: {}", message)
            }
            ErrorKind::InternalError(message) => {
                format!("Internal compiler error: {}", message)
            }
        };

        // Format the error message with location
        let mut result = format!("{}: {}", location, error_message);

        // Add context if available
        if let Some(context) = &self.context {
            result.push_str(&format!("\nContext: {}", context));
        }

        // Add source line if available
        if let Some(source_line) = &self.source_line {
            result.push_str(&format!("\n{}", source_line));
            // Add caret pointing to the error position
            if self.column > 0 {
                result.push_str(&format!("\n{}^", " ".repeat(self.column - 1)));
            }
        }

        write!(f, "{}", result)
    }
}

impl std::error::Error for Error {}

impl Error {
    /// Create a new error
    pub fn new(kind: ErrorKind, line: usize, column: usize) -> Self {
        Error {
            kind,
            line,
            column,
            file: None,
            context: None,
            source_line: None,
        }
    }

    /// Add file information to the error
    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    /// Add context to the error
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// Add source line to the error
    pub fn with_source_line(mut self, source_line: String) -> Self {
        self.source_line = Some(source_line);
        self
    }
}

/// Result type for the parser
pub type Result<T> = std::result::Result<T, Error>;
