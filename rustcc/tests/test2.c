int add(int a, int b) {
    return a + b;
}

int square(int x) {
    return x * x;
}

int main() {
    int result = 0;
    
    // These calls should be inlined
    result = add(5, 7);
    
    result = square(result);
    
    return result;
} 