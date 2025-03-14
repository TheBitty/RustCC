# RustCC Preprocessor Module

This document describes the preprocessor module in RustCC, which leverages GCC's preprocessor to handle C preprocessor directives.

## Overview

The RustCC preprocessor module provides a bridge to GCC's preprocessor capabilities, allowing RustCC to handle complex C code with various preprocessor directives, macros, and includes. By delegating to GCC, we get robust support for all C11/C17 features without having to reimplement a complete preprocessor.

## Architecture

The preprocessor module consists of:

1. **Preprocessor Trait**: Defines the interface for any preprocessor implementation
2. **GccPreprocessor**: Concrete implementation that calls GCC's preprocessor
3. **PreprocessorConfig**: Configuration options for the preprocessor

## Using the Preprocessor

### Basic Usage

The simplest way to use the preprocessor is:

```rust
use crate::preprocessor::{GccPreprocessor, Preprocessor};
use std::path::Path;

// Create a preprocessor with default settings
let preprocessor = GccPreprocessor::new();

// Check if GCC is available
if !preprocessor.is_available() {
    println!("GCC preprocessor is not available");
    return;
}

// Preprocess a file
let input_path = Path::new("input.c");
let output_path = preprocessor.preprocess_file(input_path).unwrap();

// The output_path now points to a file containing the preprocessed source
```

### Configuration Options

You can customize the preprocessor behavior with `PreprocessorConfig`:

```rust
use crate::preprocessor::{GccPreprocessor, PreprocessorConfig, Preprocessor};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Create a custom configuration
let mut config = PreprocessorConfig::default();

// Add include paths
config.include_paths.push(PathBuf::from("/usr/include"));
config.include_paths.push(PathBuf::from("./include"));

// Define macros
let mut defines = HashMap::new();
defines.insert("DEBUG".to_string(), None); // #define DEBUG
defines.insert("VERSION".to_string(), Some("1.0".to_string())); // #define VERSION "1.0"
config.defines = defines;

// Keep comments in preprocessed output
config.keep_comments = true;

// Preserve line information for better error reporting
config.preserve_line_info = true;

// Add additional GCC flags
config.gcc_flags.push("-std=c11".to_string());

// Create a preprocessor with this configuration
let preprocessor = GccPreprocessor::with_config(config);

// Use it as before
let preprocessed_path = preprocessor.preprocess_file(Path::new("input.c")).unwrap();
```

### String-Based Preprocessing

For testing or special cases, you can preprocess a string directly:

```rust
let source = r#"
#define VALUE 42
int main() {
    return VALUE;
}
"#;

let preprocessed = preprocessor.preprocess_string(source).unwrap();
println!("Preprocessed source: {}", preprocessed);
```

## Compiler Integration

The preprocessor is integrated into the RustCC compiler pipeline in `compiler.rs`:

1. The compiler creates a preprocessor with appropriate configuration
2. It passes the source file to the preprocessor
3. The preprocessed output is then fed to the lexer/parser

You can customize the preprocessor behavior by:

```rust
use crate::compiler::Compiler;
use crate::preprocessor::PreprocessorConfig;

// Create a compiler
let compiler = Compiler::new("input.c".to_string(), "output.s".to_string())
    // Set a custom preprocessor configuration
    .with_preprocessor_config(PreprocessorConfig {
        include_paths: vec![PathBuf::from("./include")],
        keep_comments: true,
        ..PreprocessorConfig::default()
    });

// Compile the file
compiler.compile().unwrap();
```

## C11/C17 Support

By leveraging GCC, our preprocessor supports all C11 and C17 features, including:

- Standard includes
- Complex macro expansions
- Conditional compilation
- Support for `_Generic`, `_Atomic`, `_Static_assert`, etc.
- Unicode character support
- Include guards and once-only processing

## Error Handling

The preprocessor provides detailed error messages when preprocessing fails:

```rust
match preprocessor.preprocess_file(input_path) {
    Ok(output_path) => {
        println!("Successfully preprocessed to {:?}", output_path);
    }
    Err(error_msg) => {
        eprintln!("Preprocessing failed: {}", error_msg);
    }
}
```

## Implementation Details

### How GCC Integration Works

1. A temporary output file is created
2. GCC is invoked with appropriate flags (`-E` for preprocessor-only)
3. Include paths, macros, and other configuration is converted to GCC flags
4. The preprocessed output is captured and returned as a path or string

### Handling Complex C Features

GCC's preprocessor handles all standard C features including:

- **#include**: Resolves file inclusions
- **#define/#undef**: Defines and undefines macros
- **#if/#ifdef/#ifndef/#else/#elif/#endif**: Conditional compilation
- **#pragma**: Compiler-specific directives
- **#error/#warning**: Compilation messages

### Memory Management

Temporary files are managed using the `tempfile` crate:

- Input files (for string-based preprocessing) are automatically cleaned up
- Output files are persisted until the program ends, allowing them to be read by the compiler

## Testing the Preprocessor

The `tests.rs` module contains comprehensive tests for the preprocessor functionality. Run them with:

```bash
cargo test --package rustcc --lib preprocessor::tests
``` 