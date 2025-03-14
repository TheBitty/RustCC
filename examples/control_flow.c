/**
 * Control flow example
 * 
 * This file demonstrates complex control flow that will be flattened
 * during the obfuscation process.
 */

#include <stdio.h>

/**
 * A function with nested conditionals that will be flattened during obfuscation
 */
int process_value(int value) {
    int result = 0;
    
    // Complex nested conditional logic
    if (value > 100) {
        if (value % 2 == 0) {
            result = value / 2;
        } else {
            result = (value * 3) + 1;
        }
    } else if (value > 50) {
        if (value % 3 == 0) {
            result = value * 3;
        } else {
            result = value - 10;
        }
    } else {
        if (value % 2 == 0) {
            result = value * value;
        } else {
            result = value + 1;
        }
    }
    
    return result;
}

/**
 * A function with loops that will be flattened during obfuscation
 */
int complex_loops(int n) {
    int sum = 0;
    
    // Nested loops
    for (int i = 0; i < n; i++) {
        if (i % 3 == 0) {
            // Skip some iterations
            continue;
        }
        
        for (int j = 0; j < i; j++) {
            sum += i * j;
            
            if (sum > 1000) {
                // Early exit
                break;
            }
        }
        
        if (sum > 2000) {
            // Early exit from outer loop
            break;
        }
    }
    
    return sum;
}

/**
 * A function with switch statements that will be flattened
 */
int complex_switch(int code) {
    int result = 0;
    
    switch (code) {
        case 1:
            result = 100;
            break;
        case 2:
            result = 200;
            // Fallthrough intentional
        case 3:
            result += 300;
            break;
        case 4:
        case 5:
            result = 500;
            break;
        default:
            result = -1;
    }
    
    // Nested switch
    if (result > 0) {
        switch (result) {
            case 100:
                return result * 2;
            case 200:
            case 500:
                return result / 2;
            case 600:
                return result - 100;
            default:
                return result;
        }
    }
    
    return result;
}

/**
 * Main function to demonstrate control flow obfuscation
 */
int main() {
    printf("Processing 120: %d\n", process_value(120));
    printf("Processing 75: %d\n", process_value(75));
    printf("Processing 30: %d\n", process_value(30));
    
    printf("Complex loops with n=10: %d\n", complex_loops(10));
    
    printf("Switch with code 1: %d\n", complex_switch(1));
    printf("Switch with code 2: %d\n", complex_switch(2));
    printf("Switch with code 4: %d\n", complex_switch(4));
    
    return 0;
} 