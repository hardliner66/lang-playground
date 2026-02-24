# Odin-Style Language Quick Reference

## Comments
```odin
// Single-line comment
```

## Variables
```odin
x := 10              // Type-inferred declaration
y: int = 20          // Explicit type
x = 30               // Assignment
```

## Procedures (Functions)
```odin
// Simple procedure
greet :: proc() {
    println("Hello!")
}

// With parameters and return type
add :: proc(a: int, b: int) -> int {
    return a + b
}

// Call procedures
result := add(5, 3)
greet()
```

## Structs
```odin
// Define struct
Person :: struct {
    name: string,
    age: int,
}

// Create instance
p: Person
```

## Control Flow

### If/Else
```odin
if x > 10 {
    println("Greater")
} else {
    println("Less or equal")
}
```

### For Loops
```odin
for item in [1, 2, 3, 4, 5] {
    println(item)
}

// Loop control
for i in numbers {
    if i == 0 { continue }
    if i > 10 { break }
}
```

## Data Structures

### Arrays
```odin
numbers := [1, 2, 3, 4, 5]
empty := []
mixed := [1, "hello", true]
```

### Hashes
```odin
person := {
    "name" => "Alice",
    "age" => 30
}
```

## Built-in Functions

### I/O
```odin
println("Hello", "World")     // Print with newline
print("No newline")           // Print without newline
```

### Utilities
```odin
len([1, 2, 3])               // Length of collection -> 3
len("hello")                 // String length -> 5
type_of(42)                  // Type name -> "int"
assert(x > 0)                // Assert condition
assert(x > 0, "Must be positive")  // With message
```

## Method Calls
```odin
// Array methods
arr := [1, 2, 3]
arr.length()                 // Get length
arr.push(4)                  // Add element
arr.pop()                    // Remove last element

// String methods
text := "Hello"
text.length()                // Get length

// Struct constructor
Point :: struct { x: int, y: int }
p := Point.new()             // Create instance
```

## Operators

### Arithmetic
```odin
+    // Addition
-    // Subtraction
*    // Multiplication
/    // Division
%    // Modulo
**   // Power
```

### Comparison
```odin
==   // Equal
!=   // Not equal
<    // Less than
<=   // Less than or equal
>    // Greater than
>=   // Greater than or equal
```

### Logical
```odin
&&   // AND
||   // OR
!    // NOT
```

### Bitwise
```odin
&    // AND
|    // OR
^    // XOR
~    // NOT
<<   // Left shift
>>   // Right shift
```

### Assignment
```odin
=    // Assign
:=   // Declare with inference
:    // Type annotation
```

## Imports
```odin
import "module_name"
import "path/to/module"
```

## Advanced Features

### Recursion
```odin
factorial :: proc(n: int) -> int {
    if n <= 1 {
        return 1
    } else {
        return n * factorial(n - 1)
    }
}
```

### Nested Calls
```odin
result := add(multiply(3, 4), multiply(2, 5))
```

### String Interpolation
```odin
name := "World"
println("Hello, #{name}!")
```

### Ranges
```odin
1..10      // Inclusive range [1, 2, 3, ..., 10]
1...10     // Exclusive range [1, 2, 3, ..., 9]
```

## Return Values
```odin
return value     // Return from procedure
return          // Return nil
```

## Flow Control Keywords
```odin
break           // Exit loop
continue        // Next iteration
return          // Return from procedure
defer           // Defer execution
```

## Type Names
- `int` - Integer numbers
- `float` - Floating-point numbers
- `string` - Text strings
- `bool` - Boolean (true/false)
- `array` - Array/list
- `hash` - Hash map/dictionary
- `nil` - Null/nothing value

## Examples

### Complete Program
```odin
import "math"

// Define procedure
calculate :: proc(x: int, y: int) -> int {
    return x * 2 + y
}

// Main entry point
main :: proc() {
    a := 10
    b := 20
    result := calculate(a, b)
    
    if result > 30 {
        println("Result is large:", result)
    } else {
        println("Result is small:", result)
    }
    
    // Loop through array
    for i in [1, 2, 3, 4, 5] {
        println("Number:", i)
    }
}
```

### Fibonacci
```odin
fib :: proc(n: int) -> int {
    if n <= 1 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}

println("fib(10) =", fib(10))
```

### Array Processing
```odin
double :: proc(x: int) -> int {
    return x * 2
}

numbers := [1, 2, 3, 4, 5]
for num in numbers {
    result := double(num)
    println(num, "doubled is", result)
}
```

## Common Patterns

### Max/Min
```odin
max :: proc(a: int, b: int) -> int {
    if a > b { return a } else { return b }
}

min :: proc(a: int, b: int) -> int {
    if a < b { return a } else { return b }
}
```

### Type Checking
```odin
check_type :: proc(value) {
    t := type_of(value)
    if t == "int" {
        println("Integer:", value)
    }
}
```

### Assertions
```odin
validate :: proc(x: int) {
    assert(x >= 0, "Value must be non-negative")
    assert(x <= 100, "Value must be <= 100")
}
```

## CLI Usage
```bash
# Run program
lang program.odin

# With flags
lang --debug program.odin
lang --ast program.odin
lang --log program.odin

# With search paths
lang -s ./lib program.odin
```

## Error Messages

### Common Errors
- `Undefined function or procedure 'name'` - Function not found
- `Proc 'name' expects N arguments but got M` - Wrong argument count
- `'name' is not callable` - Trying to call non-function
- `Method 'name' not found` - Unknown method
- `Assertion failed` - Assert condition was false

## Tips

1. Use `:=` for new variables, `=` for assignment
2. Functions must be declared before use or imported
3. All code blocks use `{...}` instead of `end`
4. Types are optional but help with documentation
5. Recursion is supported but watch stack depth
6. Use `assert()` for debugging and validation
7. Built-in functions don't need imports

## File Extension
- Use `.odin` for source files

## Documentation
- `README.md` - Main documentation
- `ODIN_SYNTAX.md` - Complete syntax reference
- `CALL_EVALUATION.md` - Function call system
- `MIGRATION_SUMMARY.md` - Migration guide