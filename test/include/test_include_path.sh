#!/bin/bash

set -e

# Get the absolute path of the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Build the preprocessor if needed
cd "$REPO_ROOT"
cargo build

# Run the preprocessor with include path
"$REPO_ROOT/target/debug/rustcc" -E "$SCRIPT_DIR/test.c" -I"$SCRIPT_DIR" -o "$SCRIPT_DIR/test_preprocessed.c"

# Check if the preprocessed file contains the expected macro expansion
if grep -q "return 42" "$SCRIPT_DIR/test_preprocessed.c"; then
    echo "✅ Include path test passed: TEST_MACRO was correctly expanded"
    exit 0
else
    echo "❌ Include path test failed: TEST_MACRO was not correctly expanded"
    cat "$SCRIPT_DIR/test_preprocessed.c"
    exit 1
fi 