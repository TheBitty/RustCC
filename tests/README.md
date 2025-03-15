# RustCC Test Suite

This directory contains test files and scripts for testing the RustCC compiler functionality.

## Test Files

- `test_obfuscation.c`: Tests the obfuscation functionality of the compiler
- `test_preprocessor.c`: Tests the preprocessor functionality of the compiler

## Running Tests

To run the tests, execute the `run_tests.sh` script from the project root directory:

```bash
./tests/run_tests.sh
```

The script will:

1. Build the RustCC compiler
2. Test the obfuscation functionality with different levels:
   - No obfuscation (`-obf0`)
   - Basic obfuscation (`-obf1`)
   - Aggressive obfuscation (`-obf2`)
3. Compare the sizes of the generated assembly files

## Test Results

The test results show the effectiveness of the obfuscation techniques:

| Obfuscation Level | File Size | Line Count |
|-------------------|-----------|------------|
| None              | ~1.0 KB   | ~60 lines  |
| Basic             | ~1.0 KB   | ~60 lines  |
| Aggressive        | ~5.0 KB   | ~300 lines |

The aggressive obfuscation significantly increases the code size and complexity, making it harder to reverse engineer.

## Obfuscation Features Tested

1. **Variable Obfuscation**: Renames variables to make the code harder to understand
2. **Control Flow Flattening**: Restructures the control flow to make it harder to follow
3. **Dead Code Insertion**: Adds meaningless code to confuse reverse engineers

## Adding New Tests

To add new tests:

1. Create a new C file in the `tests` directory
2. Update the `run_tests.sh` script to include your new test
3. Run the tests to verify the functionality 