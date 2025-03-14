// Structure definition
struct Point {
    int x;
    int y;
};

// Global variables
int global_var = 42;

// Function declaration
int add(int a, int b);

// Main function with complex features
int main() {
    // Local variables with different types
    int x = 10;
    char c = 'A';
    int numbers[5] = {1, 2, 3, 4, 5};
    
    // Struct usage
    struct Point p1;
    p1.x = 10;
    p1.y = 20;
    
    // Control structures
    if (x > 5) {
        x = x + 1;
    } else {
        x = x - 1;
    }
    
    // For loop
    int sum = 0;
    for (int i = 0; i < 5; i++) {
        sum = sum + numbers[i];
    }
    
    // While loop
    int j = 0;
    while (j < 3) {
        j = j + 1;
    }
    
    // Do-while loop
    int k = 0;
    do {
        k = k + 1;
    } while (k < 3);
    
    // Function calls
    int result = add(10, 20);
    
    // Ternary operator
    int max = (x > j) ? x : j;
    
    return result;
}

// Function definition
int add(int a, int b) {
    return a + b;
} 