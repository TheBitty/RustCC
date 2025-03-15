use crate::preprocessor::{NativePreprocessor, Preprocessor};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_preprocessor_availability() {
    let preprocessor = NativePreprocessor::new();
    assert!(preprocessor.is_available());
}

#[test]
fn test_preprocess_simple_file() {
    let preprocessor = NativePreprocessor::new();
    
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("test.c");
    
    // Write a simple C file with a define
    fs::write(&input_path, "#define VALUE 42\nint main() { return VALUE; }\n").unwrap();
    
    // Preprocess the file
    let output_path = preprocessor.preprocess_file(&input_path).unwrap();
    
    // Read the preprocessed content
    let preprocessed = fs::read_to_string(&output_path).unwrap();
    
    // Verify that the preprocessor expanded the macro
    assert!(preprocessed.contains("return 42"));
    
    // Clean up
    fs::remove_file(output_path).unwrap();
}

#[test]
fn test_preprocess_with_include() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let header_path = temp_dir.path().join("header.h");
    let input_path = temp_dir.path().join("test.c");
    
    // Write a header file and a C file that includes it
    fs::write(&header_path, "#define TEST_VALUE 123\n").unwrap();
    fs::write(&input_path, "#include \"header.h\"\nint main() { return TEST_VALUE; }\n").unwrap();
    
    // Configure the preprocessor with the include path
    let preprocessor = NativePreprocessor::new()
        .add_include_dir(temp_dir.path());
    
    // Preprocess the file
    let output_path = preprocessor.preprocess_file(&input_path).unwrap();
    
    // Read the preprocessed content
    let preprocessed = fs::read_to_string(&output_path).unwrap();
    
    // Verify that the preprocessor resolved the include and expanded the macro
    assert!(preprocessed.contains("return 123"));
    
    // Clean up
    fs::remove_file(output_path).unwrap();
}

#[test]
fn test_preprocess_with_defines() {
    // Add some predefined macros
    let preprocessor = NativePreprocessor::new()
        .add_define("DEBUG", "")
        .add_define("VERSION", "1.0");
    
    // Create a simple C file that uses the defines
    let source = r#"
    #ifdef DEBUG
    const char* mode = "debug";
    #else
    const char* mode = "release";
    #endif

    const char* version = VERSION;
    
    int main() {
        return 0;
    }
    "#;
    
    // Preprocess the string
    let preprocessed = preprocessor.preprocess_string(source).unwrap();
    
    // Verify that the preprocessor handled the conditional compilation
    assert!(preprocessed.contains("const char* mode = \"debug\""));
    assert!(preprocessed.contains("const char* version = \"1.0\""));
}

#[test]
fn test_preprocessor_error() {
    let preprocessor = NativePreprocessor::new();
    
    // Create a C file with a preprocessing error
    let source = r#"
    #include "non_existent_file.h"
    
    int main() {
        return 0;
    }
    "#;
    
    // Preprocess the string, which should fail
    let result = preprocessor.preprocess_string(source);
    
    // Verify that we get an error
    assert!(result.is_err());
}

#[test]
fn test_preprocess_with_c11_features() {
    let preprocessor = NativePreprocessor::new();
    
    // C11 features: _Generic, _Atomic, anonymous structures
    let source = r#"
    // C11 _Generic feature
    #define typename(x) _Generic((x),                                                 \
                          char: "char",                                               \
                         float: "float",                                              \
                         double: "double",                                            \
                         default: "other")
    
    // C11 anonymous struct
    struct Point {
        int x;
        int y;
        union {
            struct { char r, g, b; };  // Anonymous struct
            int color;
        };
    };
    
    int main() {
        struct Point p = {1, 2, .color = 0xFFFFFF};
        const char* type = typename(p.x);
        return 0;
    }
    "#;
    
    // Preprocess the string
    let result = preprocessor.preprocess_string(source);
    
    // This should succeed with C11 features
    assert!(result.is_ok());
}

#[test]
fn test_preprocess_with_keep_comments() {
    let preprocessor = NativePreprocessor::new()
        .keep_comments(true);
    
    // Create a simple C file with comments
    let source = r#"
    /* This is a multi-line
       comment that should be preserved */
    
    // This is a single-line comment
    
    int main() {
        return 0; // End of main
    }
    "#;
    
    // Preprocess the string
    let preprocessed = preprocessor.preprocess_string(source).unwrap();
    
    // Verify that comments are preserved
    assert!(preprocessed.contains("/* This is a multi-line"));
    assert!(preprocessed.contains("// This is a single-line comment") || 
            preprocessed.contains("/* This is a single-line comment */"));
} 