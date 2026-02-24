package main

// Test file for call evaluation and lookup

// Simple function with no parameters
greet :: proc() {
    println("Hello from greet!")
}

// Function with parameters
add :: proc(a: int, b: int) -> int {
    return a + b
}

// Function with multiple parameters
calculate :: proc(x: int, y: int, z: int) -> int {
    result := x + y * z
    return result
}

// Nested function calls
multiply :: proc(a: int, b: int) -> int {
    return a * b
}

nested_call :: proc() -> int {
    x := add(5, 3)
    y := multiply(2, 4)
    return add(x, y)
}

// Array methods
test_array_methods :: proc() {
    arr := [1, 2, 3]

    println("Array length:", arr.length())

    // This would modify the array if push worked with method syntax
    // arr.push(4)
    // println("After push:", arr.length())
}

// String methods
test_string_methods :: proc() {
    text := "Hello, World!"
    println("String length:", text.length())
}

// Struct definition
Point :: struct {
    x: int,
    y: int,
}

// Function using struct
distance_squared :: proc(p1: Point, p2: Point) -> int {
    dx := p1.x - p2.x
    dy := p1.y - p2.y
    return dx * dx + dy * dy
}

// Test variable assignment and function calls
test_variables :: proc() {
    a := 10
    b := 20
    sum := add(a, b)
    println("Sum of", a, "and", b, "is", sum)

    product := multiply(a, b)
    println("Product of", a, "and", b, "is", product)
}

// Test conditionals with function calls
test_conditionals :: proc() {
    x := add(5, 10)

    if x > 10 {
        println("Result is greater than 10:", x)
    } else {
        println("Result is 10 or less:", x)
    }
}

// Test loops with function calls
test_loops :: proc() {
    for i in [1, 2, 3, 4, 5] {
        result := multiply(i, 2)
        println("Double of", i, "is", result)
    }
}

// Recursive function
factorial :: proc(n: int) -> int {
    if n <= 1 {
        return 1
    } else {
        return n * factorial(n - 1)
    }
}

test_recursion :: proc() {
    result := factorial(5)
    println("Factorial of 5 is:", result)
}

// Main entry point
main :: proc() {
    println("=== Call Evaluation Test Suite ===")
    println()

    // Test simple function call
    println("1. Simple function call:")
    greet()
    println()

    // Test function with parameters
    println("2. Function with parameters:")
    sum := add(10, 20)
    println("10 + 20 =", sum)
    println()

    // Test nested calls
    println("3. Nested function calls:")
    nested_result := nested_call()
    println("Nested call result:", nested_result)
    println()

    // Test variables
    println("4. Variables and function calls:")
    test_variables()
    println()

    // Test conditionals
    println("5. Conditionals with function calls:")
    test_conditionals()
    println()

    // Test loops
    println("6. Loops with function calls:")
    test_loops()
    println()

    // Test array methods
    println("7. Array methods:")
    test_array_methods()
    println()

    // Test string methods
    println("8. String methods:")
    test_string_methods()
    println()

    // Test recursion
    println("9. Recursive function:")
    test_recursion()
    println()

    // Test multiple expressions
    println("10. Complex expressions:")
    a := 5
    b := 3
    c := 2
    result := add(multiply(a, b), multiply(b, c))
    println("(5 * 3) + (3 * 2) =", result)
    println()

    println("=== All tests completed! ===")
}