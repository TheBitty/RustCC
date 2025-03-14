# RustCC: Git Setup and Project Usage Guide

## 1. Setting Up Git for RustCC

### Why Use Git for This Project
Git helps manage your RustCC codebase by tracking changes, enabling collaboration, and protecting your work. It's essential for any software project, especially a complex compiler like RustCC.

### Initial Git Setup

1. **Configure Git Identity**:
   ```bash
   git config --global user.name "Your Name"
   git config --global user.email "your.email@example.com"
   ```

2. **Create a .gitignore file** (already present in your project):
   The existing .gitignore properly excludes:
   - Build artifacts in `/target/` and `rustcc/target/`
   - IDE configuration files (.idea/, .vscode/)
   - Generated files (.o, .a, .so, .bc, .ll)
   - Temporary files

3. **Initialize a Repository** (if not already done):
   Your project already has a Git repository initialized.

4. **Remote Repository Setup** (for collaboration):
   ```bash
   # If you want to push to a remote repository (e.g., GitHub):
   git remote add origin https://github.com/yourusername/rustcc.git
   ```

### Git Workflow for RustCC

1. **Make Changes**:
   Edit code in the rustcc directory as needed.

2. **Check Status**:
   ```bash
   git status
   ```

3. **Stage Changes**:
   ```bash
   git add rustcc/src/file_you_changed.rs
   # Or stage all changes:
   git add .
   ```

4. **Commit Changes**:
   ```bash
   git commit -m "Detailed description of your changes"
   ```

5. **Push Changes** (if using a remote repository):
   ```bash
   git push origin main
   ```

6. **Create Feature Branches** (for new features):
   ```bash
   git checkout -b feature/new-optimization-pass
   # Make changes, commit them
   git push origin feature/new-optimization-pass
   # Then merge via pull request
   ```

## 2. Project Structure and Organization

RustCC is organized into logical components that follow the typical compiler pipeline:

```
rustcc/
├── src/
│   ├── main.rs           # Entry point for the CLI application
│   ├── lib.rs            # Library interface
│   ├── config.rs         # Configuration handling
│   ├── compiler.rs       # Main compiler implementation
│   ├── cli.rs            # Command-line interface
│   ├── parser/           # Lexical analysis and parsing
│   ├── analyzer/         # Semantic analysis
│   ├── transforms/       # Code transformations (optimizations, obfuscation)
│   └── codegen/          # Code generation (x86_64, LLVM)
├── tests/                # Integration tests
├── examples/             # Example C programs
└── target/               # Build artifacts (generated)
```

### Key Components

1. **Parser**: Converts C code into an Abstract Syntax Tree (AST)
2. **Analyzer**: Performs semantic analysis and type checking
3. **Transforms**: Applies optimizations (constant folding, dead code elimination) and obfuscation techniques
4. **Code Generator**: Produces assembly code from the transformed AST

## 3. Building and Using the Compiler

### Prerequisites

1. **Install Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env  # For Unix systems
   ```

2. **Clone the Repository** (if you haven't already):
   ```bash
   git clone https://github.com/yourusername/rustcc.git
   cd rustcc
   ```

### Building RustCC

```bash
# Standard build
cd rustcc
cargo build --release

# Build with LLVM backend support (optional)
cargo build --release --features llvm-backend
```

The compiled binary will be available at `target/release/rustcc`.

### Using RustCC Step-by-Step

1. **Create a Simple C Program** (e.g., `example.c`):
   ```c
   #include <stdio.h>
   
   int factorial(int n) {
       if (n <= 1) return 1;
       return n * factorial(n-1);
   }
   
   int main() {
       printf("Factorial of 5 is %d\n", factorial(5));
       return 0;
   }
   ```

2. **Compile with RustCC**:
   ```bash
   # Basic compilation (no optimization or obfuscation)
   ./target/release/rustcc example.c output.s
   
   # With optimization
   ./target/release/rustcc example.c output.s -O2
   
   # With obfuscation
   ./target/release/rustcc example.c output.s -obf2
   
   # With both optimization and obfuscation
   ./target/release/rustcc example.c output.s -O2 -obf2
   ```

3. **Assemble and Link** (using GCC as the assembler/linker):
   ```bash
   gcc output.s -o example
   ```

4. **Run the Compiled Program**:
   ```bash
   ./example
   ```

### Command-Line Options

- `-O0`, `-O1`, `-O2`: Set optimization level (none, basic, full)
- `-obf0`, `-obf1`, `-obf2`: Set obfuscation level (none, basic, aggressive)
- `--emit=<format>`: Specify output format (asm, llvm, obj, c)
- `-v`, `--verbose`: Enable verbose output
- `--config=<file>`: Use a custom configuration file

## 4. Detailed Workflows

### Basic Compilation Workflow

1. **Parse**: C code is tokenized and parsed into an AST
2. **Analyze**: Type checking and semantic analysis are performed
3. **Transform**: Optional optimizations are applied
4. **Generate**: Assembly code is generated
5. **Output**: The resulting assembly is written to the output file

### Advanced Use Cases

#### Using Configuration Files

Create a `rustcc.toml` file for custom settings:

```toml
[optimization]
level = "full"
inline_threshold = 100
constant_folding = true
dead_code_elimination = true

[obfuscation]
level = "aggressive"
control_flow_flattening = true
string_encryption = true
opaque_predicates = true
```

Then use it:
```bash
./target/release/rustcc example.c output.s --config=rustcc.toml
```

#### Integration with Build Systems

For Make:
```makefile
CC=path/to/rustcc
CFLAGS=-O2 -obf2

example: example.c
	$(CC) $(CFLAGS) $< example.s
	gcc example.s -o $@
```

#### Using as a Library

```rust
use rustcc::compiler::{Compiler, OptimizationLevel, ObfuscationLevel};

fn main() {
    let compiler = Compiler::new("input.c".to_string(), "output.s".to_string())
        .with_optimization(OptimizationLevel::Full)
        .with_obfuscation(ObfuscationLevel::Aggressive);
    
    compiler.compile().unwrap();
}
```

## 5. Troubleshooting

### Common Issues

1. **Parsing Errors**:
   - Check your C code syntax
   - RustCC may not support all C language features

2. **Missing Headers**:
   - Include standard headers may not be fully supported
   - Try using `-I` flag to specify include directories

3. **Build Errors**:
   - Ensure you have the latest Rust toolchain
   - For LLVM backend issues, verify LLVM is properly installed

### Debug Mode

For more detailed information, use:
```bash
./target/release/rustcc example.c output.s -v
```
