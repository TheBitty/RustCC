// token_definitions.rs
// Contains token type definitions and helpers for the lexer

use std::collections::HashMap;
use crate::parser::token::{Token, TokenType};

/// Initializes the keywords HashMap for the lexer
pub fn init_keywords() -> HashMap<String, TokenType> {
    let mut keywords = HashMap::new();
    
    // Basic types
    keywords.insert("int".to_string(), TokenType::Int);
    keywords.insert("char".to_string(), TokenType::Char);
    keywords.insert("short".to_string(), TokenType::Short);
    keywords.insert("long".to_string(), TokenType::Long);
    keywords.insert("float".to_string(), TokenType::Float);
    keywords.insert("double".to_string(), TokenType::Double);
    keywords.insert("void".to_string(), TokenType::Void);
    keywords.insert("_Bool".to_string(), TokenType::Bool);
    keywords.insert("_Complex".to_string(), TokenType::Complex);
    keywords.insert("_Imaginary".to_string(), TokenType::Imaginary);
    
    // Type qualifiers
    keywords.insert("const".to_string(), TokenType::Const);
    keywords.insert("volatile".to_string(), TokenType::Volatile);
    keywords.insert("restrict".to_string(), TokenType::Restrict);
    keywords.insert("_Atomic".to_string(), TokenType::Atomic);
    
    // Storage class specifiers
    keywords.insert("auto".to_string(), TokenType::Auto);
    keywords.insert("register".to_string(), TokenType::Register);
    keywords.insert("static".to_string(), TokenType::Static);
    keywords.insert("extern".to_string(), TokenType::Extern);
    keywords.insert("typedef".to_string(), TokenType::Typedef);
    keywords.insert("_Thread_local".to_string(), TokenType::ThreadLocal);
    
    // Control flow
    keywords.insert("if".to_string(), TokenType::If);
    keywords.insert("else".to_string(), TokenType::Else);
    keywords.insert("while".to_string(), TokenType::While);
    keywords.insert("for".to_string(), TokenType::For);
    keywords.insert("return".to_string(), TokenType::Return);
    keywords.insert("break".to_string(), TokenType::Break);
    keywords.insert("continue".to_string(), TokenType::Continue);
    keywords.insert("switch".to_string(), TokenType::Switch);
    keywords.insert("case".to_string(), TokenType::Case);
    keywords.insert("default".to_string(), TokenType::Default);
    keywords.insert("do".to_string(), TokenType::Do);
    keywords.insert("goto".to_string(), TokenType::Goto);
    
    // Other keywords
    keywords.insert("sizeof".to_string(), TokenType::Sizeof);
    keywords.insert("_Alignas".to_string(), TokenType::Alignas);
    keywords.insert("_Alignof".to_string(), TokenType::Alignof);
    keywords.insert("_Generic".to_string(), TokenType::Generic);
    keywords.insert("_Noreturn".to_string(), TokenType::Noreturn);
    keywords.insert("_Static_assert".to_string(), TokenType::StaticAssert);
    
    // Struct/Union/Enum
    keywords.insert("struct".to_string(), TokenType::Struct);
    keywords.insert("union".to_string(), TokenType::Union);
    keywords.insert("enum".to_string(), TokenType::Enum);
    
    // Function specifiers
    keywords.insert("inline".to_string(), TokenType::Inline);
    
    // Unsigned/signed
    keywords.insert("unsigned".to_string(), TokenType::Unsigned);
    keywords.insert("signed".to_string(), TokenType::Signed);

    // Preprocessor keywords - these are handled differently
    // as they only have special meaning after a # symbol
    // But we'll add them anyway for completeness
    keywords.insert("include".to_string(), TokenType::PPInclude);
    keywords.insert("define".to_string(), TokenType::PPDefine);
    keywords.insert("undef".to_string(), TokenType::PPUndef);
    keywords.insert("ifdef".to_string(), TokenType::PPIfDef);
    keywords.insert("ifndef".to_string(), TokenType::PPIfNDef);
    keywords.insert("if".to_string(), TokenType::PPIf); // Note: duplicate with control flow "if"
    keywords.insert("else".to_string(), TokenType::PPElse); // Note: duplicate with control flow "else"
    keywords.insert("elif".to_string(), TokenType::PPElif);
    keywords.insert("endif".to_string(), TokenType::PPEndif);
    keywords.insert("pragma".to_string(), TokenType::PPPragma);
    keywords.insert("error".to_string(), TokenType::PPErrorDir);
    keywords.insert("warning".to_string(), TokenType::PPWarning);
    keywords.insert("line".to_string(), TokenType::PPLine);

    keywords
}

/// Creates a token with the given type
pub fn create_token(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Token {
    Token {
        token_type,
        lexeme,
        line,
        column,
        literal: None,
    }
}

/// Creates a token with the given type and literal value
pub fn create_token_with_literal(
    token_type: TokenType, 
    lexeme: String, 
    line: usize, 
    column: usize, 
    literal: String
) -> Token {
    Token {
        token_type,
        lexeme,
        line,
        column,
        literal: Some(literal),
    }
} 