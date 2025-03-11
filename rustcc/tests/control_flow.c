// A C program with various control flow statements
int main() {
    int sum = 0;
    
    // For loop
    for (int i = 0; i < 10; i++) {
        sum += i;
    }
    
    // While loop
    int j = 0;
    while (j < 5) {
        sum += j;
        j++;
    }
    
    // If-else statements
    if (sum > 50) {
        return 1;
    } else if (sum > 30) {
        return 2;
    } else {
        return 0;
    }
}
