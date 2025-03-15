
// Test header file

int test_function(void);


// Implementation of test_function
int test_function(void) {
    return 42;
}

int main() {
    return test_function() - 42;
}
