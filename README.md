# RustCC

RustCC is a C code obfuscation toolkit written in Rust that transforms standard C code into functionally-equivalent versions resistant to reverse engineering. It implements string encryption, control flow flattening, and API hiding for security research and penetration testing.

## Features

- **Complete C Compiler**: Parse, analyze, and compile a subset of C to x86_64 assembly
- **Code Obfuscation**: Transform code to resist reverse engineering
  - Variable Name Obfuscation: Replace variable names with random strings
  - Control Flow Obfuscation: Add complex but equivalent expressions
  - Dead Code Insertion: Add meaningless but valid code to confuse analysis
- **Optimization Passes**: Improve code performance
  - Constant Folding: Evaluate constant expressions at compile time
  - Dead Code Elimination: Remove unused variables and code
- **Multiple Backends**:
  - x86_64 Assembly: Generate native assembly code
  - LLVM IR (Work in Progress): Generate LLVM IR for advanced optimizations

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/your-username/rustcc.git
cd rustcc

# Build the project
cargo build --release
```

### Usage

RustCC can be used as a command-line tool:

```bash
# Basic compilation
./target/release/rustcc input.c output.s

# With optimization
./target/release/rustcc input.c output.s -O2

# With obfuscation
./target/release/rustcc input.c output.s -obf2

# With both
./target/release/rustcc input.c output.s -O2 -obf2
```

Or as a library in your Rust projects:

```rust
use rustcc::{compile, OptimizationLevel, ObfuscationLevel};

fn main() {
    compile(
        "input.c",
        "output.s",
        OptimizationLevel::Full,
        ObfuscationLevel::Aggressive
    ).unwrap();
}
```

## Demo

You can run the example program to see the different compilation outputs:

```bash
cargo run --example compiler_demo
```

This will compile the test programs with different optimization and obfuscation settings and show the results.

## Understanding Obfuscation Levels

- **None (-obf0)**: No obfuscation, clean output
- **Basic (-obf1)**: Variable name obfuscation only
- **Aggressive (-obf2)**: Full suite of obfuscations including variable renaming, control flow obfuscation, and dead code insertion

## Optimization Levels

- **None (-O0)**: No optimization, direct translation
- **Basic (-O1)**: Basic optimizations like constant folding
- **Full (-O2)**: All optimizations including dead code elimination

## Future Work

- Complete LLVM backend integration
- Additional obfuscation techniques
- Support for more C language features

## License

This project is licensed under the MIT License - see the LICENSE file for details.
