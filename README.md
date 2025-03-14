# RustCC

RustCC is an advanced C code obfuscation and compilation toolkit written in Rust that transforms standard C code into highly obfuscated, functionally-equivalent versions designed to resist reverse engineering. It implements state-of-the-art obfuscation techniques including control flow flattening, opaque predicates, dead code insertion, and string encryption.

## Advanced Obfuscation Features

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
- **Multiple Backends**:
  - **x86_64 Assembly**: Generate native assembly code
  - **LLVM IR (Optional)**: Generate LLVM IR for advanced optimizations

## Protection Against Reverse Engineering

RustCC's obfuscation techniques are specifically designed to defeat:
- Static analysis tools
- Disassemblers and decompilers
- Manual code analysis
- Pattern-matching approaches
- Control flow graph reconstruction

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/TheBitty/rustcc.git
cd rustcc

# Build the project
cargo build --release

# Optional: Build with LLVM backend support
cargo build --release --features llvm-backend
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

# With both (recommended for maximum protection)
./target/release/rustcc input.c output.s -O2 -obf2
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

## Obfuscation Techniques Explained

### 1. Variable Name Obfuscation

All variable names are replaced with cryptic, hard-to-follow identifiers that resemble compiler-generated symbols.

### 2. Control Flow Flattening

Restructures the code's control flow by:
- Converting structured loops and conditionals into a state machine pattern
- Adding opaque predicates to confuse analysis
- Introducing jump tables and switch-case patterns where straightforward code previously existed

### 3. Opaque Predicates

Creates complex mathematical expressions that are designed to:
- Always evaluate to a constant value (true/false)
- Be resistant to static analysis
- Require runtime execution or sophisticated SMT solvers to determine

### 4. String Encryption

All string literals are encrypted with:
- XOR cipher with random keys
- Decryption at runtime
- Fragmented string reconstruction

### 5. Dead Code Insertion

Adds semantically meaningless but syntactically complex code:
- Fake function calls
- Misleading calculations
- Dummy loops with complex termination conditions
- Never-executed branches with convincing business logic

### 6. Expression Complication

Replaces simple expressions like `x = a + b` with complex but equivalent forms such as `x = ((a ^ b) + 2 * (a & b))`.

## Obfuscation Levels

- **None (-obf0)**: No obfuscation, clean output
- **Basic (-obf1)**: Variable name obfuscation and string encryption
- **Aggressive (-obf2)**: Full suite of obfuscations including all advanced techniques

## Optimization Levels

- **None (-O0)**: No optimization, direct translation
- **Basic (-O1)**: Basic optimizations like constant folding
- **Full (-O2)**: All optimizations including dead code elimination

## Demo

You can run the example program to see the different compilation outputs:

```bash
cargo run --example compiler_demo
```

This will compile the test programs with different optimization and obfuscation settings and show the results.

## Future Development

- Enhanced control flow virtualization techniques
- Polymorphic code generation
- Anti-debugging features
- Dynamic code encryption/decryption

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

This tool is designed for legitimate security research, educational purposes, and protecting proprietary software. It is the user's responsibility to ensure compliance with applicable laws and regulations when using RustCC.
