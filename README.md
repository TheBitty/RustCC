# RustCC

[![Build Status](https://github.com/TheBitty/rustcc/workflows/Rust%20CI/badge.svg)](https://github.com/TheBitty/rustcc/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/rustcc.svg)](https://crates.io/crates/rustcc)

RustCC is an advanced C code obfuscation and compilation toolkit written in Rust that transforms standard C code into highly obfuscated, functionally-equivalent versions designed to resist reverse engineering. It implements state-of-the-art obfuscation techniques including control flow flattening, opaque predicates, dead code insertion, and string encryption.

<p align="center">
  <img src="docs/images/rustcc_logo.png" alt="RustCC Logo" width="200"/>
</p>

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
  - [Linux](#linux)
  - [macOS](#macos)
  - [Windows](#windows)
- [Quick Start](#quick-start)
- [How To Guide](#how-to-guide)
  - [Git Setup](#git-setup)
  - [Project Structure](#project-structure)
  - [Complete Workflow](#complete-workflow)
  - [Advanced Use Cases](#advanced-use-cases)
- [Examples](#examples)
- [Configuration Options](#configuration-options)
- [Obfuscation Techniques Explained](#obfuscation-techniques-explained)
- [Troubleshooting](#troubleshooting)
- [Future Development](#future-development)
- [Contributing](#contributing)
- [License](#license)
- [Disclaimer](#disclaimer)

## Overview

RustCC is not just a standard C compiler - it's a comprehensive toolkit for code obfuscation and optimization. Whether you're trying to protect intellectual property, prevent reverse engineering, or just learn about compiler techniques, RustCC provides powerful tools to transform your code while preserving its functionality.

![Before and After Obfuscation](docs/images/before_after_obfuscation.gif)

## Features

- **Complete C Compiler**: Parse, analyze, and compile a subset of C to x86_64 assembly
- **Industrial-Strength Code Obfuscation**: Transform code to be virtually impossible to reverse engineer
  - **Advanced Variable Name Obfuscation**: Replace variable names with cryptic patterns designed to confuse static analysis
  - **Control Flow Flattening**: Restructure code flow to hide the original algorithmic structure
  - **Opaque Predicates**: Insert mathematically complex expressions that always evaluate to true/false but are difficult to statically analyze
  - **String Encryption**: Encrypt string literals to prevent easy identification
  - **Dead Code Insertion**: Add complex but meaningless code that looks functional
  - **Expression Complication**: Replace simple expressions with complex equivalent forms
- **Optimization Passes**: Improve code performance and size
  - **Constant Folding**: Evaluate constant expressions at compile time
  - **Dead Code Elimination**: Remove unused variables and code
  - **Function Inlining**: Replace function calls with the actual function body
- **Multiple Backends**:
  - **x86_64 Assembly**: Generate native assembly code
  - **LLVM IR (Optional)**: Generate LLVM IR for advanced optimizations

## Installation

### Prerequisites

- Rust toolchain (1.65.0 or later)
- For LLVM backend: LLVM 15.0 or later
- For assembly output: appropriate assembler for your platform (e.g., GCC, Clang)

### Linux

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install LLVM dependencies (for LLVM backend)
sudo apt-get update
sudo apt-get install -y llvm-dev libclang-dev clang

# Clone the repository
git clone https://github.com/TheBitty/rustcc.git
cd rustcc

# Build the project
cargo build --release

# Optional: Build with LLVM backend support
cargo build --release --features llvm-backend

# Install to your system (optional)
cargo install --path .
```

### macOS

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install LLVM dependencies (for LLVM backend)
brew install llvm

# Clone the repository
git clone https://github.com/TheBitty/rustcc.git
cd rustcc

# Build the project
cargo build --release

# Optional: Build with LLVM backend support
LLVM_SYS_150_PREFIX=$(brew --prefix llvm) cargo build --release --features llvm-backend

# Install to your system (optional)
cargo install --path .
```

### Windows

```powershell
# Install Rust if not already installed
# Download and run rustup-init.exe from https://rustup.rs/

# Install LLVM dependencies (for LLVM backend)
# Download and install LLVM from https://releases.llvm.org/

# Clone the repository
git clone https://github.com/TheBitty/rustcc.git
cd rustcc

# Build the project
cargo build --release

# Optional: Build with LLVM backend support (set LLVM_SYS_150_PREFIX to your LLVM installation)
$env:LLVM_SYS_150_PREFIX = "C:\Program Files\LLVM"
cargo build --release --features llvm-backend

# Install to your system (optional)
cargo install --path .
```

## Quick Start

RustCC can be used as a command-line tool:

```bash
# Basic compilation
rustcc input.c output.s

# With optimization
rustcc input.c output.s -O2

# With obfuscation
rustcc input.c output.s -obf2

# With both (recommended for maximum protection)
rustcc input.c output.s -O2 -obf2

# With verbose output
rustcc input.c output.s -v

# Generate LLVM IR output (requires llvm-backend feature)
rustcc input.c output.ll --emit=llvm
```

Or as a library in your Rust projects:

```rust
use rustcc::compiler::{Compiler, OptimizationLevel, ObfuscationLevel};

fn main() {
    let compiler = Compiler::new("input.c".to_string(), "output.s".to_string())
        .with_optimization(OptimizationLevel::Full)
        .with_obfuscation(ObfuscationLevel::Aggressive);
    
    compiler.compile().unwrap();
}
```

## How To Guide

### Git Setup

#### Setting Up Git for RustCC

1. **Configure Git Identity**:
   ```bash
   git config --global user.name "Your Name"
   git config --global user.email "your.email@example.com"
   ```

2. **Working with the Repository**:
   ```bash
   # Check status
   git status
   
   # Stage changes
   git add rustcc/src/file_you_changed.rs
   
   # Commit changes
   git commit -m "Detailed description of your changes"
   
   # Push changes (if using a remote repository)
   git push origin main
   
   # Create a feature branch
   git checkout -b feature/new-optimization-pass
   ```

### Project Structure

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

#### Key Components

1. **Parser**: Converts C code into an Abstract Syntax Tree (AST)
2. **Analyzer**: Performs semantic analysis and type checking
3. **Transforms**: Applies optimizations and obfuscation techniques
4. **Code Generator**: Produces assembly code from the transformed AST

### Complete Workflow

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
   # Basic compilation
   ./target/release/rustcc example.c output.s
   
   # With optimization and obfuscation
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

## Examples

### Simple Example: Fibonacci Function

**Input C Code:**
```c
int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n-1) + fibonacci(n-2);
}
```

**After Compilation (-O0):**
```assembly
fibonacci:
    push    rbp
    mov     rbp, rsp
    sub     rsp, 16
    mov     DWORD PTR [rbp-4], edi
    cmp     DWORD PTR [rbp-4], 1
    jg      .L2
    mov     eax, DWORD PTR [rbp-4]
    jmp     .L3
.L2:
    mov     eax, DWORD PTR [rbp-4]
    sub     eax, 1
    mov     edi, eax
    call    fibonacci
    mov     DWORD PTR [rbp-8], eax
    mov     eax, DWORD PTR [rbp-4]
    sub     eax, 2
    mov     edi, eax
    call    fibonacci
    add     eax, DWORD PTR [rbp-8]
.L3:
    leave
    ret
```

**After Obfuscation (-obf2):**
```assembly
# Output too complex to display here - see complete example in the examples directory
```

See more examples in the [examples](examples/) directory.

## Configuration Options

RustCC offers various configuration options through command-line flags and a configuration file:

### Command-Line Options

| Flag | Description |
|------|-------------|
| `-O0` | No optimization |
| `-O1` | Basic optimizations |
| `-O2` | Full optimizations |
| `-obf0` | No obfuscation |
| `-obf1` | Basic obfuscation |
| `-obf2` | Aggressive obfuscation |
| `--emit=<format>` | Output format: `asm` (default), `llvm`, `obj`, `c` |
| `-v`, `--verbose` | Enable verbose output |
| `-q`, `--quiet` | Suppress non-error messages |
| `--config=<file>` | Use custom configuration file |

### Configuration File

You can create a `rustcc.toml` file to customize options:

```toml
[optimization]
level = "full"
inline_threshold = 100
constant_folding = true
dead_code_elimination = true

[obfuscation]
level = "aggressive"
variable_rename_style = "random"
string_encryption = true
control_flow_flattening = true
dead_code_insertion_ratio = 0.3
opaque_predicate_complexity = "high"

[output]
format = "asm"
debug_info = false
```

## Obfuscation Techniques Explained

### 1. Variable Name Obfuscation

All variable names are replaced with cryptic, hard-to-follow identifiers that resemble compiler-generated symbols.

**Before:**
```c
int calculate(int x, int y) {
    int result = x * y;
    return result;
}
```

**After:**
```c
int calculate(int _a7fe42, int _b9d31c) {
    int _e82f91 = _a7fe42 * _b9d31c;
    return _e82f91;
}
```

### 2. Control Flow Flattening

Restructures the code's control flow by converting structured loops and conditionals into a state machine pattern.

**Before:**
```c
int process(int value) {
    if (value > 10) {
        return value * 2;
    } else {
        return value / 2;
    }
}
```

**After:**
```c
int process(int value) {
    int state = 0;
    int result = 0;
    
    while (1) {
        switch (state) {
            case 0:
                if (value > 10) {
                    state = 1;
                } else {
                    state = 2;
                }
                break;
            case 1:
                result = value * 2;
                state = 3;
                break;
            case 2:
                result = value / 2;
                state = 3;
                break;
            case 3:
                return result;
        }
    }
}
```

### 3-6. Additional Techniques

See the full documentation for detailed explanations of opaque predicates, string encryption, dead code insertion, and expression complication.

## Troubleshooting

### Common Issues

#### LLVM Backend Issues

**Problem**: `error: Failed to find libLLVM.so`

**Solution**: Make sure LLVM is installed and set the LLVM_SYS_150_PREFIX environment variable:

```bash
# Linux/macOS
export LLVM_SYS_150_PREFIX=/path/to/llvm

# Windows
set LLVM_SYS_150_PREFIX=C:\path\to\llvm
```

#### Linker Errors

**Problem**: `error: linking with 'cc' failed: exit status: 1`

**Solution**: Make sure you have GCC or Clang installed and in your PATH:

```bash
# Linux
sudo apt-get install build-essential

# macOS
xcode-select --install

# Windows
# Install MinGW or MSVC
```

#### Verbose Debugging

For detailed debugging information, use the verbose flag:

```bash
rustcc input.c output.s -v
```

For more troubleshooting tips, see the [wiki](https://github.com/TheBitty/rustcc/wiki/Troubleshooting).

## Future Development

- Enhanced control flow virtualization techniques
- Polymorphic code generation
- Anti-debugging features
- Dynamic code encryption/decryption
- WebAssembly backend support
- More language extensions

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This tool is designed for legitimate security research, educational purposes, and protecting proprietary software. It is the user's responsibility to ensure compliance with applicable laws and regulations when using RustCC.
