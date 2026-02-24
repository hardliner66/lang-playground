// Advanced call evaluation examples

// ============================================
// Function Composition and Higher-Order Patterns
// ============================================

double :: proc(x: int) -> int {
    return x * 2
}

triple :: proc(x: int) -> int {
    return x * 3
}

apply_twice :: proc(x: int) -> int {
    return double(double(x))
}

// ============================================
// Mathematical Functions
// ============================================

square :: proc(x: int) -> int {
    return x * x
}

cube :: proc(x: int) -> int {
    return x * x * x
}

sum_of_squares :: proc(a: int, b: int) -> int {
    return square(a) + square(b)
}

// ============================================
// String and Array Operations
// ============================================

create_array :: proc() {
    numbers := [1, 2, 3, 4, 5]
    println("Array:", numbers)
    println("Length:", numbers.length())

    text := "Hello"
    println("String:", text)
    println("String length:", text.length())
}

// ============================================
// Conditional Logic with Function Calls
// ============================================

max :: proc(a: int, b: int) -> int {
    if a > b {
        return a
    } else {
        return b
    }
}

min :: proc(a: int, b: int) -> int {
    if a < b {
        return a
    } else {
        return b
    }
}

clamp :: proc(value: int, min_val: int, max_val: int) -> int {
    return max(min_val, min(value, max_val))
}

// ============================================
// Complex Expression Evaluation
// ============================================

evaluate_expression :: proc() -> int {
    a := 5
    b := 10
    c := 15

    // Nested function calls in expression
    result := double(a) + triple(b) - square(2)
    return result
}

// ============================================
// Fibonacci (Recursive)
// ============================================

fib :: proc(n: int) -> int {
    if n <= 1 {
        return n
    } else {
        return fib(n - 1) + fib(n - 2)
    }
}

// ============================================
// Struct and Instance Methods
// ============================================

Rectangle :: struct {
    width: int,
    height: int,
}

Point3D :: struct {
    x: int,
    y: int,
    z: int,
}

compute_area :: proc(width: int, height: int) -> int {
    return width * height
}

// ============================================
// Lookup and Scoping Tests
// ============================================

global_var := 100

test_scoping :: proc() {
    local_var := 50
    println("Local variable:", local_var)
    println("Global variable:", global_var)

    nested_proc :: proc() {
        inner_var := 25
        println("Inner variable:", inner_var)
    }

    nested_proc()
}

// ============================================
// Array Processing
// ============================================

sum_array :: proc(arr) {
    total := 0
    for num in arr {
        total = total + num
    }
    return total
}

process_array :: proc() {
    data := [10, 20, 30, 40, 50]

    println("Processing array:", data)

    for val in data {
        doubled := double(val)
        println("  Value:", val, "Doubled:", doubled)
    }
}

// ============================================
// Control Flow with Calls
// ============================================

is_even :: proc(n: int) -> bool {
    remainder := n % 2
    if remainder == 0 {
        return true
    } else {
        return false
    }
}

filter_evens :: proc() {
    numbers := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

    println("Finding even numbers:")
    for num in numbers {
        if is_even(num) {
            println("  Even:", num)
        }
    }
}

// ============================================
// Main Test Suite
// ============================================

main :: proc() {
    println("╔══════════════════════════════════════╗")
    println("║  Advanced Call Evaluation Tests     ║")
    println("╚══════════════════════════════════════╝")
    println()

    // Test 1: Simple function composition
    println("━━━ Test 1: Function Composition ━━━")
    x := 5
    result := apply_twice(x)
    println("apply_twice(5) =", result)
    println()

    // Test 2: Mathematical functions
    println("━━━ Test 2: Mathematical Operations ━━━")
    println("square(7) =", square(7))
    println("cube(3) =", cube(3))
    println("sum_of_squares(3, 4) =", sum_of_squares(3, 4))
    println()

    // Test 3: Min/Max/Clamp
    println("━━━ Test 3: Min/Max/Clamp ━━━")
    println("max(10, 20) =", max(10, 20))
    println("min(10, 20) =", min(10, 20))
    println("clamp(25, 0, 20) =", clamp(25, 0, 20))
    println("clamp(5, 0, 20) =", clamp(5, 0, 20))
    println()

    // Test 4: Complex expressions
    println("━━━ Test 4: Complex Expressions ━━━")
    expr_result := evaluate_expression()
    println("Complex expression result:", expr_result)
    println()

    // Test 5: Recursion
    println("━━━ Test 5: Fibonacci ━━━")
    println("fib(0) =", fib(0))
    println("fib(1) =", fib(1))
    println("fib(5) =", fib(5))
    println("fib(7) =", fib(7))
    println("fib(10) =", fib(10))
    println()

    // Test 6: Array and string methods
    println("━━━ Test 6: Array and String Methods ━━━")
    create_array()
    println()

    // Test 7: Array processing
    println("━━━ Test 7: Array Processing ━━━")
    process_array()
    println()

    // Test 8: Filtering
    println("━━━ Test 8: Even Number Filter ━━━")
    filter_evens()
    println()

    // Test 9: Nested calls
    println("━━━ Test 9: Deeply Nested Calls ━━━")
    nested := max(min(100, 50), min(30, 20))
    println("max(min(100, 50), min(30, 20)) =", nested)
    println()

    // Test 10: Expression with multiple function calls
    println("━━━ Test 10: Multiple Function Calls ━━━")
    a := 2
    b := 3
    c := 4
    complex := square(a) + cube(b) + double(c)
    println("square(2) + cube(3) + double(4) =", complex)
    println()

    println("╔══════════════════════════════════════╗")
    println("║  All Advanced Tests Passed! ✓        ║")
    println("╚══════════════════════════════════════╝")
}