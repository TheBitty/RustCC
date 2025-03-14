/**
 * String encryption example
 * 
 * This file demonstrates how RustCC's string encryption obfuscation works.
 * It contains several string literals that will be encrypted during obfuscation.
 */

#include <stdio.h>
#include <string.h>

/**
 * Check if a username and password are valid
 * This function would be a target for reverse engineering in real applications
 */
int authenticate(const char* username, const char* password) {
    // These strings would be vulnerable to static analysis without obfuscation
    const char* valid_username = "admin";
    const char* valid_password = "supersecret123";
    
    // Simple authentication check
    if (strcmp(username, valid_username) == 0 && 
        strcmp(password, valid_password) == 0) {
        return 1; // Authentication successful
    }
    
    return 0; // Authentication failed
}

/**
 * Function with several string literals that would be obfuscated
 */
void print_messages() {
    printf("Welcome to the secure system!\n");
    printf("--------------------------------\n");
    printf("This system contains sensitive information.\n");
    printf("All access attempts are logged and monitored.\n");
    printf("Unauthorized access is strictly prohibited.\n");
}

/**
 * Main function to demonstrate string usage
 */
int main() {
    // Example strings that would be obfuscated
    char username[64];
    char password[64];
    
    print_messages();
    
    printf("Enter username: ");
    scanf("%63s", username);
    
    printf("Enter password: ");
    scanf("%63s", password);
    
    if (authenticate(username, password)) {
        printf("Authentication successful! Welcome, %s.\n", username);
        printf("Access granted to secure data.\n");
    } else {
        printf("Authentication failed. Invalid credentials.\n");
        printf("This attempt has been logged.\n");
    }
    
    return 0;
} 