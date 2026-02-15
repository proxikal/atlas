/**
 * C Callback Example for Atlas FFI (phase-10c)
 *
 * This C library demonstrates how C code can call Atlas functions
 * via callbacks. Compile with:
 *
 *     gcc -shared -fPIC -o libcallback_example.so c_callback_example.c
 *
 * On macOS:
 *     gcc -shared -fPIC -o libcallback_example.dylib c_callback_example.c
 *
 * On Windows:
 *     cl /LD c_callback_example.c
 */

#include <stdio.h>
#include <math.h>

// ===== Callback Type Definitions =====

// Callback that takes a double and returns a double
typedef double (*double_callback_t)(double);

// Callback that takes two doubles and returns a double
typedef double (*binary_callback_t)(double, double);

// Callback that takes an int and returns an int
typedef int (*int_callback_t)(int);

// Callback with no parameters
typedef int (*simple_callback_t)(void);

// Callback with void return
typedef void (*void_callback_t)(int);

// ===== Simple Callback Functions =====

/**
 * Call a callback with a single double argument
 */
double call_with_double(double_callback_t callback, double value) {
    printf("C: Calling callback with %.2f\n", value);
    double result = callback(value);
    printf("C: Callback returned %.2f\n", result);
    return result;
}

/**
 * Call a binary callback with two arguments
 */
double call_with_two_doubles(binary_callback_t callback, double a, double b) {
    printf("C: Calling callback with %.2f and %.2f\n", a, b);
    double result = callback(a, b);
    printf("C: Callback returned %.2f\n", result);
    return result;
}

/**
 * Call an integer callback
 */
int call_with_int(int_callback_t callback, int value) {
    printf("C: Calling callback with %d\n", value);
    int result = callback(value);
    printf("C: Callback returned %d\n", result);
    return result;
}

/**
 * Call a simple callback with no arguments
 */
int call_simple(simple_callback_t callback) {
    printf("C: Calling simple callback\n");
    int result = callback();
    printf("C: Callback returned %d\n", result);
    return result;
}

/**
 * Call a void callback
 */
void call_void_callback(void_callback_t callback, int value) {
    printf("C: Calling void callback with %d\n", value);
    callback(value);
    printf("C: Void callback completed\n");
}

// ===== Advanced Callback Examples =====

/**
 * Apply a callback to each element of an array (map operation)
 */
void map_array(double_callback_t callback, double* array, int length) {
    printf("C: Mapping over array of %d elements\n", length);
    for (int i = 0; i < length; i++) {
        array[i] = callback(array[i]);
    }
}

/**
 * Numerical integration using callbacks
 *
 * Approximates the integral of a function using the trapezoidal rule.
 */
double integrate(double_callback_t function, double a, double b, int steps) {
    double h = (b - a) / steps;
    double sum = (function(a) + function(b)) / 2.0;

    for (int i = 1; i < steps; i++) {
        double x = a + i * h;
        sum += function(x);
    }

    return sum * h;
}

/**
 * Find the root of a function using Newton's method
 *
 * Requires both function and its derivative as callbacks.
 */
double find_root(double_callback_t f, double_callback_t df, double x0, int max_iter) {
    double x = x0;

    for (int i = 0; i < max_iter; i++) {
        double fx = f(x);
        double dfx = df(x);

        if (fabs(dfx) < 1e-10) {
            break;  // Derivative too small
        }

        double x_new = x - fx / dfx;

        if (fabs(x_new - x) < 1e-10) {
            break;  // Converged
        }

        x = x_new;
    }

    return x;
}

/**
 * Call a callback multiple times and sum results
 */
double sum_callback_results(int_callback_t callback, int count) {
    double sum = 0.0;

    for (int i = 0; i < count; i++) {
        sum += callback(i);
    }

    return sum;
}

// ===== Error Handling Example =====

/**
 * Call a callback with error checking
 *
 * Returns -1 if callback returns invalid result.
 */
double call_with_validation(double_callback_t callback, double value) {
    double result = callback(value);

    if (isnan(result) || isinf(result)) {
        fprintf(stderr, "C: Callback returned invalid result\n");
        return -1.0;
    }

    return result;
}

// ===== Test Helper Functions =====

/**
 * Test function that multiplies by 2
 */
double test_double_function(double x) {
    return x * 2.0;
}

/**
 * Test function that adds two numbers
 */
double test_add_function(double a, double b) {
    return a + b;
}

/**
 * Self-test: verify the library works
 */
void self_test(void) {
    printf("=== C Callback Library Self-Test ===\n");

    // Test with internal function
    double result1 = call_with_double(test_double_function, 21.0);
    printf("Test 1: %.2f (expected 42.00)\n", result1);

    double result2 = call_with_two_doubles(test_add_function, 15.0, 27.0);
    printf("Test 2: %.2f (expected 42.00)\n", result2);

    printf("Self-test completed\n\n");
}

// ===== Main (for standalone testing) =====

#ifdef STANDALONE_TEST
int main(void) {
    self_test();
    return 0;
}
#endif
