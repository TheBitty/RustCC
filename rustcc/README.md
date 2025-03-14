# RustCC - C Compiler in Rust

RustCC is a C compiler written in Rust, designed to be educational, modular, and extensible. It follows the standard compilation pipeline while leveraging Rust's safety and modern language features.

## Features

- Modular compilation pipeline
- C11/C17 preprocessing support via GCC
- Lexical analysis
- Syntax parsing
- Semantic analysis
- Code generation
- Rich diagnostics

## Preprocessor Integration

RustCC now integrates with GCC's preprocessor to handle C11 and C17 preprocessing features. This integration provides several benefits:

1. **Full C11/C17 Compliance**: Leverages GCC's robust preprocessor implementation
2. **Macro Support**: Handles complex macro expansions and conditional compilation
3. **Include Resolution**: Properly resolves include directives with configurable search paths
4. **Reduced Implementation Burden**: Avoids reimplementing complex preprocessor logic

### How the Preprocessor Works

The preprocessor module provides a bridge between RustCC and GCC's preprocessor:

1. Input C source files are passed to GCC with the `-E` flag (preprocessor-only mode)
2. Include paths, macro definitions, and other preprocessor settings are converted to GCC flags
3. GCC processes the file, expanding macros, handling conditional compilation, and including headers
4. The preprocessed output is captured and fed to RustCC's lexer/parser for further compilation

## Project Structure

```
rustcc/
├── src/
│   ├── compiler.rs         # Main compiler interface
│   ├── config.rs           # Compiler configuration
│   ├── error.rs            # Error handling
│   ├── lexer/              # Lexical analysis
│   ├── parser/             # Syntax analysis
│   ├── preprocessor/       # Preprocessor module
│   │   ├── mod.rs          # Preprocessor trait and utilities
│   │   └── gcc.rs          # GCC preprocessor implementation
│   ├── semantic/           # Semantic analysis
│   ├── codegen/            # Code generation
│   └── lib.rs              # Library entrypoint
├── tests/                  # Test suite
│   ├── test/               # Test C files
│   │   ├── c11_features.c  # C11 feature tests
│   │   ├── c17_features.c  # C17 feature tests
│   │   └── complex.c       # Complex test cases
│   ├── test.c              # Simple test file
│   └── test2.c             # Additional test file
├── examples/               # Example programs
└── docs/                   # Documentation
    └── preprocessing.md    # Preprocessor module documentation
```

## Building and Running

### Prerequisites

- Rust toolchain (cargo, rustc)
- GCC installed on your system (for preprocessor integration)

### Building

```bash
# Build the project
cargo build

# Build with optimization
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run preprocessor tests only
cargo test --package rustcc --lib preprocessor::tests
```

### Example Usage

```rust
use rustcc::compiler::Compiler;
use rustcc::config::Config;

fn main() {
    // Create a configuration with customizations
    let config = Config::default()
        .with_include_paths(vec!["./include".into(), "/usr/include".into()])
        .with_defines([("DEBUG", None)].into());

    // Create compiler instance
    let compiler = Compiler::new("input.c".into(), "output.s".into(), config);

    // Compile the file
    match compiler.compile() {
        Ok(_) => println!("Compilation successful!"),
        Err(e) => eprintln!("Compilation failed: {}", e),
    }
}
```

## Contribution

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 