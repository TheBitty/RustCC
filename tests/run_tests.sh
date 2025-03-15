#!/bin/bash

# Script to run tests for RustCC compiler
# This will test various aspects of the compiler's functionality

set -e  # Exit on any error
cd "$(dirname "$0")/../rustcc"  # Change to the rustcc directory

echo "Building RustCC compiler..."
cargo build

RUSTCC="./target/debug/rustcc"
INCLUDE_DIR="./include"

echo "======================================================================="
echo "Testing Obfuscation Functionality"
echo "======================================================================="

# Test with no obfuscation
echo "1. Testing compilation with no obfuscation..."
$RUSTCC ../tests/test_obfuscation.c -o ../tests/test_obfuscation_none.s -obf0 -I$INCLUDE_DIR
echo "Compilation successful!"
echo ""

# Test with basic obfuscation
echo "2. Testing compilation with basic obfuscation..."
$RUSTCC ../tests/test_obfuscation.c -o ../tests/test_obfuscation_basic.s -obf1 -I$INCLUDE_DIR
echo "Compilation successful!"
echo ""

# Test with aggressive obfuscation
echo "3. Testing compilation with aggressive obfuscation..."
$RUSTCC ../tests/test_obfuscation.c -o ../tests/test_obfuscation_aggressive.s -obf2 -I$INCLUDE_DIR
echo "Compilation successful!"
echo ""

echo "======================================================================="
echo "Comparing Obfuscation Levels"
echo "======================================================================="
echo "File sizes:"
ls -lh ../tests/test_obfuscation_*.s | awk '{print $9 ": " $5}'
echo ""

echo "All tests completed successfully!" 