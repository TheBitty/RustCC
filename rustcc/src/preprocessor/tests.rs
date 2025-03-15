//! Tests for the preprocessor module
//!
//! This module contains tests for the preprocessor functionality.

#[cfg(test)]
mod tests {
    // These imports are only used in tests which might not be running
    #[allow(unused_imports)]
    use std::fs;
    #[allow(unused_imports)]
    use std::io::Write;
    #[allow(unused_imports)]
    use std::path::{Path, PathBuf};
    #[allow(unused_imports)]
    use tempfile::{TempDir, tempdir};

    #[allow(unused_imports)]
    use crate::preprocessor::native::NativePreprocessor;
    #[allow(unused_imports)]
    use crate::preprocessor::Preprocessor;

    #[test]
    fn test_preprocessor_availability() {
        let preprocessor = NativePreprocessor::new();
        assert!(preprocessor.is_available());
    }

    #[test]
    fn test_preprocess_simple_file() {
        let mut preprocessor = NativePreprocessor::new();
        
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join("test.c");
        
        // Write a simple C file with a define
        fs::write(&input_path, "#define VALUE 42\nint main() { return VALUE; }\n").unwrap();
        
        // Preprocess the file
        let preprocessed = preprocessor.preprocess_file(input_path.to_str().unwrap()).unwrap();
        
        // Check that the define was expanded
        assert!(preprocessed.contains("int main() { return 42; }"));
    }

    #[test]
    fn test_preprocess_with_include() {
        let mut preprocessor = NativePreprocessor::new();
        
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let header_path = temp_dir.path().join("header.h");
        let input_path = temp_dir.path().join("test.c");
        
        // Write a header file
        fs::write(&header_path, "#define HEADER_VALUE 123\n").unwrap();
        
        // Write a C file that includes the header
        fs::write(&input_path, "#include \"header.h\"\nint main() { return HEADER_VALUE; }\n").unwrap();
        
        // Add the temp directory to include paths
        preprocessor.add_include_dir(temp_dir.path().to_str().unwrap());
        
        // Preprocess the file
        let preprocessed = preprocessor.preprocess_file(input_path.to_str().unwrap()).unwrap();
        
        // Check that the include was processed and the define was expanded
        assert!(preprocessed.contains("int main() { return 123; }"));
    }

    #[test]
    fn test_preprocess_with_defines() {
        let mut preprocessor = NativePreprocessor::new();
        
        // Add some defines
        preprocessor.add_define("VERSION", "2");
        preprocessor.add_define("DEBUG", "1");
        
        // Create a source with conditional compilation
        let source = r#"
#if VERSION == 2
    #if DEBUG
        const char* build = "debug";
    #else
        const char* build = "release";
    #endif
#else
    const char* build = "unknown";
#endif
"#;
        
        // Preprocess the string
        let preprocessed = preprocessor.preprocess_string(source, "test.c").unwrap();
        
        // Check that the right branch was taken
        assert!(preprocessed.contains("const char* build = \"debug\";"));
        assert!(!preprocessed.contains("const char* build = \"release\";"));
        assert!(!preprocessed.contains("const char* build = \"unknown\";"));
    }

    #[test]
    fn test_preprocessor_error() {
        let mut preprocessor = NativePreprocessor::new();
        
        // Create a source with an error (unterminated conditional)
        let source = r#"
#if defined(DEBUG)
    const int debug_level = 2;
// Missing #endif
"#;
        
        // Preprocess should return an error
        let result = preprocessor.preprocess_string(source, "test.c");
        assert!(result.is_err());
    }

    #[test]
    fn test_preprocess_with_c11_features() {
        let mut preprocessor = NativePreprocessor::new();
        
        // Create a source with C11 features like _Static_assert
        let source = r#"
#define BUFFER_SIZE 1024
_Static_assert(BUFFER_SIZE >= 256, "Buffer too small");

#define CONCAT(a, b) a ## b
#define STRINGIFY(x) #x

int CONCAT(test_, 123) = 456;
const char* str = STRINGIFY(hello world);
"#;
        
        // Preprocess the string
        let result = preprocessor.preprocess_string(source, "test.c").unwrap();
        
        // Check that the features were processed correctly
        assert!(result.contains("_Static_assert(1024 >= 256, \"Buffer too small\");"));
        assert!(result.contains("int test_123 = 456;"));
        assert!(result.contains("const char* str = \"hello world\";"));
    }

    #[test]
    fn test_preprocess_with_keep_comments() {
        let mut preprocessor = NativePreprocessor::new();
        
        // Create a source with comments
        let source = r#"
/* This is a multi-line comment
   that spans multiple lines */
#define MAX_SIZE 100 // Define maximum size

// This is a single-line comment
int main() {
    // Another comment
    return MAX_SIZE; /* End of main */
}
"#;
        
        // Preprocess the string
        let preprocessed = preprocessor.preprocess_string(source, "test.c").unwrap();
        
        // Check that the define was expanded but comments were preserved
        assert!(preprocessed.contains("return 100;"));
        assert!(preprocessed.contains("/* This is a multi-line comment"));
        assert!(preprocessed.contains("// This is a single-line comment"));
    }

    #[test]
    fn test_function_like_macros() {
        let mut preprocessor = NativePreprocessor::new();
        
        // Create a source with function-like macros
        let source = r#"
#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define SUM(a, b) (a + b)
#define PRINT(msg) printf("%s\n", msg)

int main() {
    int x = MAX(5, 10);
    int y = SUM(3, 4);
    PRINT("Hello");
    return 0;
}
"#;
        
        // Preprocess the string
        let result = preprocessor.preprocess_string(source, "test.c").unwrap();
        
        // Check that the macros were expanded correctly
        assert!(result.contains("int x = ((5) > (10) ? (5) : (10));"));
        assert!(result.contains("int y = (3 + 4);"));
        assert!(result.contains("printf(\"%s\\n\", \"Hello\");"));
    }

    #[test]
    fn test_complex_conditionals() {
        let mut preprocessor = NativePreprocessor::new();
        
        // Add some defines
        preprocessor.add_define("VERSION", "2");
        preprocessor.add_define("PLATFORM", "LINUX");
        
        // Create a source with complex conditionals
        let source = r#"
#if defined(VERSION) && VERSION > 1
    #if defined(PLATFORM) && (PLATFORM == LINUX || PLATFORM == MAC)
        #define SUPPORTED 1
    #else
        #define SUPPORTED 0
    #endif
#elif defined(VERSION) && VERSION == 1
    #define SUPPORTED 1
#else
    #define SUPPORTED 0
#endif

#if SUPPORTED
const char* status = "supported";
#else
const char* status = "unsupported";
#endif
"#;
        
        // Preprocess the string
        let result = preprocessor.preprocess_string(source, "test.c").unwrap();
        
        // Check that the conditionals were evaluated correctly
        assert!(result.contains("const char* status = \"supported\";"));
        assert!(!result.contains("const char* status = \"unsupported\";"));
    }

    #[test]
    fn test_standard_predefined_macros() {
        let mut preprocessor = NativePreprocessor::new();
        
        // Create a source that uses standard predefined macros
        let source = r#"
#if defined(__FILE__)
    const char* current_file = __FILE__;
#endif

#if defined(__LINE__)
    const int current_line = __LINE__;
#endif

#if defined(__DATE__) && defined(__TIME__)
    const char* build_timestamp = __DATE__ " " __TIME__;
#endif

#if defined(__STDC__)
    const int standard_c = __STDC__;
#endif
"#;
        
        // Preprocess the string
        let result = preprocessor.preprocess_string(source, "test.c").unwrap();
        
        // Check that the predefined macros were expanded
        assert!(result.contains("const char* current_file = \"test.c\";"));
        assert!(result.contains("const int current_line = "));
        assert!(result.contains("const char* build_timestamp = "));
        assert!(result.contains("const int standard_c = 1;"));
    }

    #[test]
    fn test_nested_includes() {
        let mut preprocessor = NativePreprocessor::new();
        
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let header1_path = temp_dir.path().join("header1.h");
        let header2_path = temp_dir.path().join("header2.h");
        let input_path = temp_dir.path().join("test.c");
        
        // Write header files with nested includes
        fs::write(&header1_path, "#define HEADER1_VALUE 100\n#include \"header2.h\"\n").unwrap();
        fs::write(&header2_path, "#define HEADER2_VALUE 200\n").unwrap();
        
        // Write a C file that includes header1
        fs::write(&input_path, "#include \"header1.h\"\nint main() { return HEADER1_VALUE + HEADER2_VALUE; }\n").unwrap();
        
        // Add the temp directory to include paths
        preprocessor.add_include_dir(temp_dir.path().to_str().unwrap());
        
        // Preprocess the file
        let result = preprocessor.preprocess_file(input_path.to_str().unwrap()).unwrap();
        
        // Check that both headers were included and defines were expanded
        assert!(result.contains("int main() { return 100 + 200; }"));
    }
} 