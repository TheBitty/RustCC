// A C program with string operations
#include <stdio.h>
#include <string.h>

int main() {
    char greeting[] = "Hello, world!";
    char name[] = "Rust";
    char message[50];
    
    // String manipulation
    strcpy(message, greeting);
    strcat(message, " I am ");
    strcat(message, name);
    
    // Using a string in a condition
    if (strcmp(name, "Rust") == 0) {
        return 0;
    } else {
        return 1;
    }
}
