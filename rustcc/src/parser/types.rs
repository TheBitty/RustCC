use crate::parser::ast::Type;
use crate::parser::error::{self, Error, ErrorKind, Result};
use crate::parser::token::TokenType;
use crate::parser::Parser;

impl Parser {
    /// Parse a C type specification
    pub fn parse_type(&mut self) -> Result<Type> {
        // Parse type qualifiers and storage class specifiers
        let mut is_const = false;
        let mut is_volatile = false;
        let mut is_restrict = false;
        let mut is_atomic = false;
        let mut is_static = false;
        let mut is_extern = false;
        let mut is_typedef = false;
        let mut is_inline = false;
        let mut is_noreturn = false;
        let mut is_thread_local = false;

        // Parse type qualifiers and storage class specifiers
        loop {
            if self.match_token(TokenType::Const) {
                is_const = true;
            } else if self.match_token(TokenType::Volatile) {
                is_volatile = true;
            } else if self.match_token(TokenType::Restrict) {
                is_restrict = true;
            } else if self.match_token(TokenType::Atomic) {
                is_atomic = true;
            } else if self.match_token(TokenType::Static) {
                is_static = true;
            } else if self.match_token(TokenType::Extern) {
                is_extern = true;
            } else if self.match_token(TokenType::Typedef) {
                is_typedef = true;
            } else if self.match_token(TokenType::Inline) {
                is_inline = true;
            } else if self.match_token(TokenType::Noreturn) {
                is_noreturn = true;
            } else if self.match_token(TokenType::ThreadLocal) {
                is_thread_local = true;
            } else {
                break;
            }
        }

        // Parse signed/unsigned specifier
        let mut is_unsigned = false;
        if self.match_token(TokenType::Unsigned) {
            is_unsigned = true;
        } else if self.match_token(TokenType::Signed) {
            // Explicitly signed, but this is the default for integer types
        }

        // Parse the base type
        let mut base_type = if self.match_token(TokenType::Void) {
            Type::Void
        } else if self.match_token(TokenType::Char) {
            if is_unsigned {
                Type::UnsignedChar
            } else {
                Type::Char
            }
        } else if self.match_token(TokenType::Short) {
            // Handle "short int" or just "short"
            if self.match_token(TokenType::Int) {
                // "short int"
            }
            if is_unsigned {
                Type::UnsignedShort
            } else {
                Type::Short
            }
        } else if self.match_token(TokenType::Int) {
            if is_unsigned {
                Type::UnsignedInt
            } else {
                Type::Int
            }
        } else if self.match_token(TokenType::Long) {
            // Handle "long int", "long long", or just "long"
            if self.match_token(TokenType::Long) {
                // "long long"
                if self.match_token(TokenType::Int) {
                    // "long long int"
                }
                if is_unsigned {
                    Type::UnsignedLongLong
                } else {
                    Type::LongLong
                }
            } else {
                // Just "long" or "long int"
                if self.match_token(TokenType::Int) {
                    // "long int"
                }
                if is_unsigned {
                    Type::UnsignedLong
                } else {
                    Type::Long
                }
            }
        } else if self.match_token(TokenType::Float) {
            Type::Float
        } else if self.match_token(TokenType::Double) {
            if self.match_token(TokenType::Complex) {
                // "double _Complex"
                Type::Complex
            } else {
                Type::Double
            }
        } else if self.match_token(TokenType::Bool) {
            Type::Bool
        } else if self.match_token(TokenType::Complex) {
            // "_Complex" by itself is invalid, it must be preceded by a type
            return Err(self.unexpected_token_error("type specifier before _Complex"));
        } else if self.match_token(TokenType::Struct) {
            // Parse struct type
            let struct_name = self.consume(TokenType::Identifier, "Expected struct name")?
                .lexeme.clone();
            
            // Check for struct definition
            if self.match_token(TokenType::LeftBrace) {
                // This is a struct definition, not just a type reference
                // In a real implementation, we would parse the struct fields here
                // For now, we'll just skip to the matching closing brace
                let mut brace_depth = 1;
                while brace_depth > 0 && !self.is_at_end() {
                    if self.match_token(TokenType::LeftBrace) {
                        brace_depth += 1;
                    } else if self.match_token(TokenType::RightBrace) {
                        brace_depth -= 1;
                    } else {
                        self.advance();
                    }
                }
            }
            
            Type::Struct(struct_name)
        } else if self.match_token(TokenType::Union) {
            // Parse union type
            let union_name = self.consume(TokenType::Identifier, "Expected union name")?
                .lexeme.clone();
            
            // Check for union definition
            if self.match_token(TokenType::LeftBrace) {
                // This is a union definition, not just a type reference
                // In a real implementation, we would parse the union fields here
                // For now, we'll just skip to the matching closing brace
                let mut brace_depth = 1;
                while brace_depth > 0 && !self.is_at_end() {
                    if self.match_token(TokenType::LeftBrace) {
                        brace_depth += 1;
                    } else if self.match_token(TokenType::RightBrace) {
                        brace_depth -= 1;
                    } else {
                        self.advance();
                    }
                }
            }
            
            Type::Union(union_name)
        } else if self.match_token(TokenType::Enum) {
            // Parse enum type
            let enum_name = self.consume(TokenType::Identifier, "Expected enum name")?
                .lexeme.clone();
            
            // Check for enum definition
            if self.match_token(TokenType::LeftBrace) {
                // This is an enum definition, not just a type reference
                // In a real implementation, we would parse the enum values here
                // For now, we'll just skip to the matching closing brace
                let mut brace_depth = 1;
                while brace_depth > 0 && !self.is_at_end() {
                    if self.match_token(TokenType::LeftBrace) {
                        brace_depth += 1;
                    } else if self.match_token(TokenType::RightBrace) {
                        brace_depth -= 1;
                    } else {
                        self.advance();
                    }
                }
            }
            
            // For now, we'll treat enums as ints
            Type::Int
        } else if self.check(TokenType::Identifier) {
            // This could be a typedef name
            let type_name = self.advance().lexeme.clone();
            Type::TypeDef(type_name)
        } else {
            return Err(self.unexpected_token_error("type specifier"));
        };

        // Parse derived types (pointers, arrays, functions)
        loop {
            if self.match_token(TokenType::Star) {
                // Pointer type
                // Check for type qualifiers on the pointer
                let mut ptr_is_const = false;
                let mut ptr_is_volatile = false;
                let mut ptr_is_restrict = false;
                
                loop {
                    if self.match_token(TokenType::Const) {
                        ptr_is_const = true;
                    } else if self.match_token(TokenType::Volatile) {
                        ptr_is_volatile = true;
                    } else if self.match_token(TokenType::Restrict) {
                        ptr_is_restrict = true;
                    } else {
                        break;
                    }
                }
                
                // Create the pointer type
                base_type = Type::Pointer(Box::new(base_type));
                
                // Apply qualifiers to the pointer
                if ptr_is_const {
                    base_type = Type::Const(Box::new(base_type));
                }
                if ptr_is_volatile {
                    base_type = Type::Volatile(Box::new(base_type));
                }
                if ptr_is_restrict {
                    base_type = Type::Restrict(Box::new(base_type));
                }
            } else if self.match_token(TokenType::LeftBracket) {
                // Array type
                // Check for array size
                let size = if self.match_token(TokenType::RightBracket) {
                    // Unsized array []
                    None
                } else if self.match_token(TokenType::Star) {
                    // Variable-length array [*]
                    self.consume(TokenType::RightBracket, "Expected ']' after [*]")?;
                    None // We represent VLAs as arrays with no size
                } else {
                    // Parse the size expression
                    let size_expr = self.parse_expression()?;
                    
                    // Consume the closing bracket
                    self.consume(TokenType::RightBracket, "Expected ']' after array size")?;
                    
                    // For now, we only support constant integer sizes
                    // In a full implementation, we would evaluate constant expressions
                    if let crate::parser::ast::Expression::IntegerLiteral(size) = size_expr {
                        Some(size as usize)
                    } else {
                        // This is a variable-length array (VLA)
                        None
                    }
                };
                
                base_type = Type::Array(Box::new(base_type), size);
            } else if self.match_token(TokenType::LeftParen) {
                // This could be a function pointer declaration
                // Check if the next token is a right parenthesis or a type
                if self.check(TokenType::RightParen) || self.is_type_specifier() || self.check(TokenType::Identifier) {
                    // This is a function pointer declaration
                    let mut parameters = Vec::new();
                    let mut is_variadic = false;
                    
                    // Parse parameter list
                    if !self.match_token(TokenType::RightParen) {
                        loop {
                            // Check for variadic function
                            if self.match_token(TokenType::Ellipsis) {
                                is_variadic = true;
                                self.consume(TokenType::RightParen, "Expected ')' after '...'")?;
                                break;
                            }
                            
                            // Parse parameter type
                            let param_type = self.parse_type()?;
                            
                            // Parse parameter name (optional)
                            let param_name = if self.check(TokenType::Identifier) {
                                self.advance().lexeme.clone()
                            } else {
                                // Anonymous parameter
                                String::new()
                            };
                            
                            parameters.push((param_name, param_type));
                            
                            // Check for end of parameters
                            if self.match_token(TokenType::RightParen) {
                                break;
                            }
                            
                            // Expect a comma
                            self.consume(TokenType::Comma, "Expected ',' between parameters")?;
                            
                            // Check for variadic function
                            if self.match_token(TokenType::Ellipsis) {
                                is_variadic = true;
                                self.consume(TokenType::RightParen, "Expected ')' after '...'")?;
                                break;
                            }
                        }
                    }
                    
                    // Create the function type
                    base_type = Type::Function {
                        return_type: Box::new(base_type),
                        parameters,
                        is_variadic,
                    };
                } else {
                    // This is a parenthesized type, used for complex declarations
                    // For example: int (*fp)(int) - function pointer
                    // or: int (*array[10])(int) - array of function pointers
                    
                    // Parse the inner type
                    let inner_type = self.parse_type()?;
                    
                    // Consume the closing parenthesis
                    self.consume(TokenType::RightParen, "Expected ')' after type")?;
                    
                    // Continue parsing the rest of the type
                    base_type = inner_type;
                }
            } else {
                // No more derived types
                break;
            }
        }

        // Apply type qualifiers
        if is_const {
            base_type = Type::Const(Box::new(base_type));
        }
        if is_volatile {
            base_type = Type::Volatile(Box::new(base_type));
        }
        if is_restrict {
            base_type = Type::Restrict(Box::new(base_type));
        }
        if is_atomic {
            // In a real implementation, we would handle _Atomic differently
            // For now, we'll just ignore it
        }

        // Storage class specifiers and function specifiers don't affect the type itself
        // They would be handled at the declaration level

        Ok(base_type)
    }

    /// Parse a type name (used in casts and sizeof)
    pub fn parse_type_name(&mut self) -> Result<Type> {
        // Type names in casts don't have identifiers, just the type
        self.parse_type()
    }

    /// Parse an array type
    pub fn parse_array_type(&mut self, element_type: Type) -> Result<Type> {
        // Consume the opening bracket
        self.consume(TokenType::LeftBracket, "Expected '[' for array type")?;

        // Check if size is specified
        let size = if self.match_token(TokenType::RightBracket) {
            // Unsized array []
            None
        } else if self.match_token(TokenType::Star) {
            // Variable-length array [*]
            self.consume(TokenType::RightBracket, "Expected ']' after [*]")?;
            None // We represent VLAs as arrays with no size
        } else {
            // Parse the size expression
            let size_expr = self.parse_expression()?;
            
            // Consume the closing bracket
            self.consume(TokenType::RightBracket, "Expected ']' after array size")?;
            
            // For now, we only support constant integer sizes
            // In a full implementation, we would evaluate constant expressions
            if let crate::parser::ast::Expression::IntegerLiteral(size) = size_expr {
                Some(size as usize)
            } else {
                // This is a variable-length array (VLA)
                None
            }
        };

        Ok(Type::Array(Box::new(element_type), size))
    }

    /// Check if the current token is a type specifier
    pub fn is_type_specifier(&self) -> bool {
        self.check(TokenType::Void) || 
        self.check(TokenType::Char) || 
        self.check(TokenType::Short) ||
        self.check(TokenType::Int) || 
        self.check(TokenType::Long) ||
        self.check(TokenType::Float) ||
        self.check(TokenType::Double) ||
        self.check(TokenType::Signed) ||
        self.check(TokenType::Unsigned) ||
        self.check(TokenType::Bool) ||
        self.check(TokenType::Complex) ||
        self.check(TokenType::Imaginary) ||
        self.check(TokenType::Struct) ||
        self.check(TokenType::Union) ||
        self.check(TokenType::Enum) ||
        self.check(TokenType::Const) ||
        self.check(TokenType::Volatile) ||
        self.check(TokenType::Restrict) ||
        self.check(TokenType::Atomic) ||
        self.check(TokenType::Static) ||
        self.check(TokenType::Extern) ||
        self.check(TokenType::Typedef) ||
        self.check(TokenType::Inline) ||
        self.check(TokenType::Noreturn) ||
        self.check(TokenType::ThreadLocal)
        // In a real implementation, we would also check for typedef names
    }
} 