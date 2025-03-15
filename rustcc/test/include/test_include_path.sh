#!/bin/bash

set -e

cd "$(dirname "$0")"

# Build the preprocessor if needed
cd ..
cargo build

cd test/include

# Run the preprocessor with include path
../../target/debug/rustcc -E test.c -I. -o test_preprocessed.c

# Check if the preprocessed file contains the expected macro expansion
if grep -q "return 42" test_preprocessed.c; then
    echo "✅ Include path test passed: TEST_MACRO was correctly expanded"
    exit 0
else
    echo "❌ Include path test failed: TEST_MACRO was not correctly expanded"
    cat test_preprocessed.c
    exit 1
fi
