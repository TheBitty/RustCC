// operators.rs
// Operator recognition for the lexer

use crate::parser::lexer::Lexer;
use crate::parser::token::TokenType;

impl Lexer {
    /// Handles the dot character (. or ...)
    pub(crate) fn handle_dot(&mut self) {
        // Check for ellipsis (...) for variadic functions
        if self.peek() == '.' && self.peek_next() == '.' {
            self.advance(); // Consume the second '.'
            self.advance(); // Consume the third '.'
            self.add_token(TokenType::Ellipsis);
        } else {
            self.add_token(TokenType::Dot);
        }
    }

    /// Handles the plus character (+ or ++ or +=)
    pub(crate) fn handle_plus(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenType::PlusEqual)
        } else if self.match_char('+') {
            self.add_token(TokenType::Increment)
        } else {
            self.add_token(TokenType::Plus)
        }
    }

    /// Handles the minus character (- or -- or -= or ->)
    pub(crate) fn handle_minus(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenType::MinusEqual)
        } else if self.match_char('-') {
            self.add_token(TokenType::Decrement)
        } else if self.match_char('>') {
            self.add_token(TokenType::Arrow)
        } else {
            self.add_token(TokenType::Minus)
        }
    }

    /// Handles the star character (* or *=)
    pub(crate) fn handle_star(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenType::StarEqual)
        } else {
            self.add_token(TokenType::Star)
        }
    }

    /// Handles the slash character (/ or /= or comments)
    pub(crate) fn handle_slash(&mut self) {
        if self.match_char('/') {
            // A comment goes until the end of the line
            while self.peek() != '\n' && !self.is_at_end() {
                self.advance();
            }
        } else if self.match_char('*') {
            // A block comment goes until */
            while !(self.is_at_end() || self.peek() == '*' && self.peek_next() == '/') {
                if self.peek() == '\n' {
                    self.line += 1;
                    self.column = 0;
                    self.at_line_start = true;
                }
                self.advance();
            }

            // Consume the */
            if !self.is_at_end() {
                self.advance(); // *
                self.advance(); // /
            }
        } else if self.match_char('=') {
            self.add_token(TokenType::SlashEqual)
        } else {
            self.add_token(TokenType::Slash)
        }
    }

    /// Handles the percent character (% or %=)
    pub(crate) fn handle_percent(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenType::PercentEqual)
        } else {
            self.add_token(TokenType::Percent)
        }
    }

    /// Handles the equal character (= or ==)
    pub(crate) fn handle_equal(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenType::EqualEqual)
        } else {
            self.add_token(TokenType::Equal)
        }
    }

    /// Handles the bang character (! or !=)
    pub(crate) fn handle_bang(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenType::BangEqual)
        } else {
            self.add_token(TokenType::Bang)
        }
    }

    /// Handles the less-than character (< or <= or << or <<=)
    pub(crate) fn handle_less(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenType::LessEqual)
        } else if self.match_char('<') {
            if self.match_char('=') {
                self.add_token(TokenType::ShiftLeftEqual)
            } else {
                self.add_token(TokenType::ShiftLeft)
            }
        } else {
            self.add_token(TokenType::Less)
        }
    }

    /// Handles the greater-than character (> or >= or >> or >>=)
    pub(crate) fn handle_greater(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenType::GreaterEqual)
        } else if self.match_char('>') {
            if self.match_char('=') {
                self.add_token(TokenType::ShiftRightEqual)
            } else {
                self.add_token(TokenType::ShiftRight)
            }
        } else {
            self.add_token(TokenType::Greater)
        }
    }

    /// Handles the ampersand character (& or && or &=)
    pub(crate) fn handle_ampersand(&mut self) {
        if self.match_char('&') {
            self.add_token(TokenType::And)
        } else if self.match_char('=') {
            self.add_token(TokenType::AmpersandEqual)
        } else {
            self.add_token(TokenType::Ampersand)
        }
    }

    /// Handles the pipe character (| or || or |=)
    pub(crate) fn handle_pipe(&mut self) {
        if self.match_char('|') {
            self.add_token(TokenType::Or)
        } else if self.match_char('=') {
            self.add_token(TokenType::PipeEqual)
        } else {
            self.add_token(TokenType::Pipe)
        }
    }

    /// Handles the caret character (^ or ^=)
    pub(crate) fn handle_caret(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenType::CaretEqual)
        } else {
            self.add_token(TokenType::Caret)
        }
    }

    /// Handles the hash character (#)
    pub(crate) fn handle_hash(&mut self) {
        // Check for token concatenation (##) in preprocessor
        if self.match_char('#') {
            self.add_token(TokenType::PPHashHash);
        } else {
            // This is the stringification operator (#) in preprocessor
            self.add_token(TokenType::PPHash);
        }
    }
}
