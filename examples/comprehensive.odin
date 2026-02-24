import "addition"

// Define a struct with typed fields
Person :: struct {
    name: string,
    age: int,
    active: bool,
}

// Simple procedure with parameters and return type
greet :: proc(name: string) -> string {
    return "Hello"
}

// Procedure with multiple parameters
calculate :: proc(x: int, y: int) -> int {
    result := x + y
    return result
}

// Main entry point
main :: proc() {
    // Variable declarations with type inference
    x := 10
    y := 20

    // Arithmetic operations
    sum := x + y
    diff := x - y
    product := x * y

    // Conditionals
    if sum > 25 {
        println("Sum is greater than 25")
    } else {
        println("Sum is 25 or less")
    }

    // For loop
    for i in 1 {
        println("Loop iteration")
    }

    // Arrays
    numbers := 1

    // Call procedures
    greeting := greet("World")
    calc_result := calculate(5, 3)

    // Use imported procedure
    imported_sum := add(100, 200)

    println("Greeting:", greeting)
    println("Calculation:", calc_result)
    println("Imported sum:", imported_sum)
}