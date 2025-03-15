#include "myheader.h"

// Implementation of test_function
int test_function(void) {
    return TEST_MACRO;
}

int main() {
    return test_function() - 42;
}
