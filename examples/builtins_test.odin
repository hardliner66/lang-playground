// Built-in Functions Test Suite
package main

// Test println - prints with newline
test_println :: proc() {
    println("=== Testing println ===")
    println("Hello, World!")
    println("Multiple", "arguments", "on", "one", "line")
    println("Numbers:", 1, 2, 3, 4, 5)
    println()
}

// Test print - prints without newline
test_print :: proc() {
    println("=== Testing print ===")
    print("This ")
    print("is ")
    print("printed ")
    print("without ")
    println("newlines!")
    println()
}

// Test len - returns length of collections
test_len :: proc() {
    println("=== Testing len ===")

    // Array length
    arr := [1, 2, 3, 4, 5]
    println("Array:", arr)
    println("Length:", len(arr))

    // String length
    text := "Hello, World!"
    println("String:", text)
    println("Length:", len(text))

    // Empty array
    empty := []
    println("Empty array length:", len(empty))

    println()
}

// Test type_of - returns type as string
test_type_of :: proc() {
    println("=== Testing type_of ===")

    // Integer
    x := 42
    println("Value:", x, "Type:", type_of(x))

    // String
    s := "hello"
    println("Value:", s, "Type:", type_of(s))

    // Boolean
    b := true
    println("Value:", b, "Type:", type_of(b))

    // Array
    arr := [1, 2, 3]
    println("Value:", arr, "Type:", type_of(arr))

    // Nil
    n := nil
    println("Value:", n, "Type:", type_of(n))

    println()
}

// Test assert - assertion checking
test_assert :: proc() {
    println("=== Testing assert ===")

    // Passing assertions
    assert(true)
    println("✓ assert(true) passed")

    assert(5 > 3)
    println("✓ assert(5 > 3) passed")

    assert(10 == 10)
    println("✓ assert(10 == 10) passed")

    // Custom message (will fail if uncommented)
    // assert(false, "This is a custom error message")

    println("All assertions passed!")
    println()
}

// Complex usage examples
test_complex :: proc() {
    println("=== Complex Usage ===")

    // Using len in expressions
    data := [10, 20, 30, 40, 50]
    if len(data) > 3 {
        println("Data has more than 3 elements")
    }

    // Type checking
    value := 100
    if type_of(value) == "int" {
        println("Value is an integer:", value)
    }

    // Nested function calls
    numbers := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    println("Array has", len(numbers), "elements")

    println()
}

// Helper function to demonstrate function calls
double :: proc(x: int) -> int {
    return x * 2
}

test_with_helpers :: proc() {
    println("=== Using Built-ins with Helper Functions ===")

    values := [5, 10, 15, 20]
    println("Original values:", values)
    println("Array length:", len(values))

    for val in values {
        result := double(val)
        println("  double(" + type_of(val) + "):", result)
    }

    println()
}

// Main test runner
main :: proc() {
    println("╔════════════════════════════════════════╗")
    println("║   Built-in Functions Test Suite       ║")
    println("╚════════════════════════════════════════╝")
    println()

    test_println()
    test_print()
    test_len()
    test_type_of()
    test_assert()
    test_complex()
    test_with_helpers()

    println("╔════════════════════════════════════════╗")
    println("║   All Built-in Tests Completed! ✓     ║")
    println("╚════════════════════════════════════════╝")
}