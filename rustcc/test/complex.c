#include <stdio.h>
#include <stdlib.h>

// Structure definition
struct Point {
    int x;
    int y;
    char label[20];
};

// Typedef example
typedef struct {
    int width;
    int height;
} Rectangle;

// Enum example
enum Color {
    RED,
    GREEN,
    BLUE = 5,
    YELLOW
};

// Global variables
int global_var = 42;
char global_array[10] = {'H', 'e', 'l', 'l', 'o'};

// Function declaration
int add(int a, int b);

// Function with array parameter
void print_array(int arr[], int size);

// Function with variadic arguments
int sum_values(int count, ...);

// Main function with complex features
int main(int argc, char *argv[]) {
    // Local variables with different types
    int x = 10;
    char c = 'A';
    int *ptr = &x;
    int numbers[5] = {1, 2, 3, 4, 5};
    
    // Struct usage
    struct Point p1 = {10, 20, "Point 1"};
    Rectangle rect = {30, 40};
    
    // Enum usage
    enum Color color = GREEN;
    
    // Control structures
    if (x > 5) {
        printf("x is greater than 5\n");
    } else {
        printf("x is not greater than 5\n");
    }
    
    // For loop
    for (int i = 0; i < 5; i++) {
        printf("%d ", numbers[i]);
    }
    printf("\n");
    
    // While loop
    int j = 0;
    while (j < 3) {
        printf("j = %d\n", j);
        j++;
    }
    
    // Do-while loop
    int k = 0;
    do {
        printf("k = %d\n", k);
        k++;
    } while (k < 3);
    
    // Switch statement
    switch (color) {
        case RED:
            printf("Color is red\n");
            break;
        case GREEN:
            printf("Color is green\n");
            break;
        case BLUE:
            printf("Color is blue\n");
            break;
        default:
            printf("Unknown color\n");
            break;
    }
    
    // Function calls
    int sum = add(10, 20);
    printf("Sum: %d\n", sum);
    
    print_array(numbers, 5);
    
    // Pointer operations
    *ptr = 20;
    printf("x = %d\n", x);
    
    // Struct member access
    p1.x = 15;
    printf("p1.x = %d\n", p1.x);
    
    // Ternary operator
    int max = (x > j) ? x : j;
    printf("Max: %d\n", max);
    
    // Sizeof operator
    printf("Size of int: %zu\n", sizeof(int));
    printf("Size of Point: %zu\n", sizeof(struct Point));
    
    return 0;
}

// Function definition
int add(int a, int b) {
    return a + b;
}

// Function with array parameter implementation
void print_array(int arr[], int size) {
    for (int i = 0; i < size; i++) {
        printf("%d ", arr[i]);
    }
    printf("\n");
} 