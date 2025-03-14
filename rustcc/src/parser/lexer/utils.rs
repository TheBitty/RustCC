// utils.rs
// Utility functions for the lexer

/// Returns whether the given character is a digit (0-9)
pub fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// Returns whether the given character is a letter or underscore (a-z, A-Z, _)
pub fn is_alpha(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == '_'
}

/// Returns whether the given character is alphanumeric or underscore (a-z, A-Z, 0-9, _)
pub fn is_alphanumeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}

/// Returns whether the given character is a hexadecimal digit (0-9, a-f, A-F)
pub fn is_hex_digit(c: char) -> bool {
    c.is_ascii_digit() || ('a'..='f').contains(&c.to_ascii_lowercase())
}

/// Returns whether the given character is an octal digit (0-7)
pub fn is_octal_digit(c: char) -> bool {
    c.is_ascii_digit() && c <= '7'
}

/// Returns whether the given character is a whitespace character (space, tab, carriage return)
pub fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\r'
}
