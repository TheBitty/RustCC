#[cfg(test)]
mod c11_features_tests {
    use rustcc::parser::lexer::Lexer;
    use rustcc::parser::Parser;
    use rustcc::parser::ast::{Expression, Statement, Type};

    #[test]
    fn test_generic_selection() {
        let source = r#"
        int main() {
            int i = 5;
            double d = 5.0;
            void *p = &i;
            
            // Test _Generic expression
            int result = _Generic(i,
                int: 1,
                double: 2,
                default: 3
            );
            
            return 0;
        }
        "#;
        
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();
        
        let mut parser = Parser::new(tokens.clone());
        let result = parser.parse();
        
        assert!(result.is_ok(), "Failed to parse C11 _Generic expression");
    }

    #[test]
    fn test_static_assert() {
        let source = r#"
        _Static_assert(sizeof(int) >= 4, "int must be at least 32 bits");
        
        int main() {
            _Static_assert(1 + 1 == 2, "Basic arithmetic must work");
            return 0;
        }
        "#;
        
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();
        
        let mut parser = Parser::new(tokens.clone());
        let result = parser.parse();
        
        assert!(result.is_ok(), "Failed to parse C11 _Static_assert");
    }

    #[test]
    fn test_alignas() {
        let source = r#"
        struct alignas_test {
            _Alignas(16) int aligned_int;
            int regular_int;
        };
        
        int main() {
            _Alignas(8) int aligned_var = 42;
            _Alignas(double) int aligned_to_double = 42;
            return 0;
        }
        "#;
        
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();
        
        let mut parser = Parser::new(tokens.clone());
        let result = parser.parse();
        
        assert!(result.is_ok(), "Failed to parse C11 _Alignas");
    }

    #[test]
    fn test_atomic() {
        let source = r#"
        #include <stdatomic.h>
        
        int main() {
            _Atomic int atomic_var = 0;
            atomic_store(&atomic_var, 42);
            int value = atomic_load(&atomic_var);
            return value;
        }
        "#;
        
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();
        
        let mut parser = Parser::new(tokens.clone());
        let result = parser.parse();
        
        assert!(result.is_ok(), "Failed to parse C11 _Atomic");
    }

    #[test]
    fn test_thread_local() {
        let source = r#"
        _Thread_local int thread_var = 42;
        
        int main() {
            _Thread_local static int local_thread_var = 0;
            local_thread_var++;
            return local_thread_var;
        }
        "#;
        
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();
        
        let mut parser = Parser::new(tokens.clone());
        let result = parser.parse();
        
        assert!(result.is_ok(), "Failed to parse C11 _Thread_local");
    }

    #[test]
    fn test_noreturn() {
        let source = r#"
        _Noreturn void fatal_error(const char *message) {
            // Print error message
            exit(1);
        }
        
        int main() {
            if (1 > 2) {
                fatal_error("Impossible condition");
            }
            return 0;
        }
        "#;
        
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();
        
        let mut parser = Parser::new(tokens.clone());
        let result = parser.parse();
        
        assert!(result.is_ok(), "Failed to parse C11 _Noreturn");
    }

    #[test]
    fn test_unicode_literals() {
        let source = r#"
        int main() {
            char *utf8 = u8"UTF-8 string";
            char16_t *utf16 = u"UTF-16 string";
            char32_t *utf32 = U"UTF-32 string";
            wchar_t *wide = L"Wide string";
            
            char16_t c16 = u'A';
            char32_t c32 = U'B';
            wchar_t wc = L'C';
            
            return 0;
        }
        "#;
        
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();
        
        let mut parser = Parser::new(tokens.clone());
        let result = parser.parse();
        
        assert!(result.is_ok(), "Failed to parse C11 Unicode literals");
    }

    #[test]
    fn test_complex_declarations() {
        let source = r#"
        // Function pointer
        int (*fp)(int, double);
        
        // Array of function pointers
        int (*array_of_fp[10])(int);
        
        // Function returning function pointer
        int (*(*func)(int))(double);
        
        // Variable length array
        void process_vla(int n) {
            int vla[n];
            int vla2[*];  // VLA with unspecified size
        }
        
        int main() {
            return 0;
        }
        "#;
        
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();
        
        let mut parser = Parser::new(tokens.clone());
        let result = parser.parse();
        
        assert!(result.is_ok(), "Failed to parse complex declarations");
    }

    #[test]
    fn test_compound_literals() {
        let source = r#"
        struct Point {
            int x;
            int y;
        };
        
        int main() {
            // Compound literal for struct
            struct Point p = (struct Point){.x = 1, .y = 2};
            
            // Compound literal for array
            int *array = (int[]){1, 2, 3, 4};
            
            // Nested compound literals
            struct Point *points = (struct Point[]){
                {.x = 1, .y = 2},
                {.x = 3, .y = 4}
            };
            
            return 0;
        }
        "#;
        
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();
        
        let mut parser = Parser::new(tokens.clone());
        let result = parser.parse();
        
        assert!(result.is_ok(), "Failed to parse compound literals");
    }
} 