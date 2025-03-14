// Complex C program with various control structures and operations
#include <stdio.h>

int factorial(int n) {
    if (n <= 1)
        return 1;
    return n * factorial(n - 1);
}

int fibonacci(int n) {
    if (n <= 0)
        return 0;
    if (n == 1)
        return 1;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int calculate_sum(int arr[], int size) {
    int sum = 0;
    for (int i = 0; i < size; i++) {
        sum += arr[i];
    }
    return sum;
}

int main() {
    // Variable declarations and initializations
    int x = 42;
    int y = 10;
    int z = 0;
    
    // Array initialization
    int numbers[5] = {5, 10, 15, 20, 25};
    
    // Conditional branching
    if (x > y) {
        z = x - y;
    } else {
        z = y - x;
    }
    
    // Loop structure
    for (int i = 0; i < 3; i++) {
        z += i * 2;
    }
    
    // While loop
    int counter = 5;
    while (counter > 0) {
        z += counter;
        counter--;
    }
    
    // Function calls
    int fact = factorial(4);
    int fib = fibonacci(7);
    int sum = calculate_sum(numbers, 5);
    
    // Compound calculation
    int result = z + (fact * fib) / (sum % 10 + 1);
    
    // Switch statement
    switch (result % 3) {
        case 0:
            result += 5;
            break;
        case 1:
            result *= 2;
            break;
        default:
            result -= 1;
            break;
    }
    
    return result;
}