/**
 * Fibonacci sequence example
 * 
 * This file demonstrates a simple recursive implementation of the Fibonacci sequence.
 * It will be used to show different optimization and obfuscation techniques.
 */

#include <stdio.h>

/**
 * Calculate the nth Fibonacci number recursively
 */
int fibonacci(int n) {
    // Base case
    if (n <= 1) {
        return n;
    }
    
    // Recursive case
    return fibonacci(n-1) + fibonacci(n-2);
}

/**
 * Calculate Fibonacci numbers iteratively (more efficient)
 */
int fibonacci_iterative(int n) {
    if (n <= 1) {
        return n;
    }
    
    int a = 0, b = 1, c;
    for (int i = 2; i <= n; i++) {
        c = a + b;
        a = b;
        b = c;
    }
    
    return b;
}

/**
 * Main function to demonstrate Fibonacci calculations
 */
int main() {
    int n = 10;
    
    printf("Fibonacci(%d) recursive = %d\n", n, fibonacci(n));
    printf("Fibonacci(%d) iterative = %d\n", n, fibonacci_iterative(n));
    
    return 0;
} 