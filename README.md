# Odin-Style Programming Language

A dynamically-typed interpreted programming language with Odin-inspired syntax.

## Overview

This language combines the simplicity of dynamic typing with the clean, modern syntax of Odin. It features:

- **Odin-style syntax**: `::` declarations, `{}` blocks, `//` comments
- **Type annotations**: Optional type hints for documentation
- **Simple semantics**: Easy to learn and use
- **Dynamic typing**: Flexibility at runtime
- **Built-in features**: Arrays, hashes, strings, and more

## Quick Start

### Installation

```bash
cargo build --release
```

### Running a Program

```bash
cargo run -- examples/addition.odin
```

Or with the compiled binary:

```bash
./target/release/lang examples/addition.odin
```

## Syntax Guide

### Comments

```odin
// This is a single-line comment
```

### Variables

```odin
// Type-inferred declaration
x := 10

// Explicit type annotation (for documentation)
y: int = 20

// Regular assignment
x = 30
```

### Procedures (Functions)

```odin
// Simple procedure
greet :: proc() {
    println("Hello, World!")
}

// With parameters and return type
add :: proc(a: int, b: int) -> int {
    return a + b
}

// Parameters can omit types
simple :: proc(x, y) {
    return x + y
}
```

### Structs

```odin
// Define a struct
Person :: struct {
    name: string,
    age: int,
    active: bool,
}

// Create instance
p: Person
```

### Control Flow

#### If Statements

```odin
if x > 10 {
    println("Greater than 10")
} else {
    println("10 or less")
}
```

#### For Loops

```odin
// Iterate over array
for item in [1, 2, 3, 4, 5] {
    println(item)
}

// Loop control
for i in numbers {
    if i == 0 {
        continue  // Skip this iteration
    }
    if i > 10 {
        break     // Exit loop
    }
    println(i)
}
```

### Data Structures

#### Arrays

```odin
numbers := [1, 2, 3, 4, 5]
mixed := [1, "hello", true]
```

#### Hashes

```odin
person := {
    "name" => "Alice",
    "age" => 30
}
```

### Imports

```odin
import "math"
import "path/to/module"
```

### Operators

#### Arithmetic

- `+` Addition
- `-` Subtraction
- `*` Multiplication
- `/` Division
- `%` Modulo
- `**` Power

#### Comparison

- `==` Equal
- `!=` Not equal
- `<` Less than
- `<=` Less than or equal
- `>` Greater than
- `>=` Greater than or equal

#### Logical

- `&&` Logical AND
- `||` Logical OR
- `!` Logical NOT

#### Bitwise

- `&` Bitwise AND
- `|` Bitwise OR
- `^` Bitwise XOR
- `~` Bitwise NOT
- `<<` Left shift
- `>>` Right shift

## Complete Example

```odin
import "addition"

// Define a struct
Point :: struct {
    x: int,
    y: int,
}

// Define procedures
distance_squared :: proc(p1: Point, p2: Point) -> int {
    dx := p1.x - p2.x
    dy := p1.y - p2.y
    return dx * dx + dy * dy
}

main :: proc() {
    // Variables
    a := 10
    b := 20

    // Call imported function
    sum := add(a, b)

    // Conditionals
    if sum > 25 {
        println("Sum is large:", sum)
    } else {
        println("Sum is small:", sum)
    }

    // Arrays
    numbers := [1, 2, 3, 4, 5]

    // Loops
    for num in numbers {
        println("Number:", num)
    }

    // Return result
    println("Final result:", sum)
}
```

### Built-in Functions

The language provides several built-in functions:

```odin
// Print with newline
println("Hello", "World")      // Output: Hello World

// Print without newline
print("Hello ")
print("World")                 // Output: Hello World

// Get length of collections
len([1, 2, 3])                // Returns 3
len("Hello")                  // Returns 5

// Get type name
type_of(42)                   // Returns "int"
type_of("hello")              // Returns "string"

// Assertions
assert(x > 0)                 // Throws error if false
assert(x > 0, "x must be positive")
```

### Method Calls

Objects support method calls using dot notation:

```odin
// Array methods
arr := [1, 2, 3]
length := arr.length()        // Returns 3
arr.push(4)                   // Adds 4 to array

// String methods
text := "Hello"
size := text.length()         // Returns 5

// Struct constructors
Person :: struct {
    name: string,
    age: int,
}
person := Person.new()        // Creates instance
```

## Examples

See the `examples/` directory for more examples:

- `addition.odin` - Simple arithmetic function
- `test.odin` - Basic language features
- `comprehensive.odin` - More complete example

## Language Features

### Currently Supported

- ✅ Variables with type inference (`:=`)
- ✅ Procedures with optional type annotations
- ✅ Structs
- ✅ Arrays and hashes
- ✅ If/else conditionals
- ✅ For loops
- ✅ Break and continue
- ✅ String interpolation
- ✅ Comments (`//`)
- ✅ Import system
- ✅ Arithmetic, comparison, logical, and bitwise operators
- ✅ Attributes (`#[attr]`)
- ✅ **Comprehensive call evaluation system**
  - Direct function calls
  - Method calls with receivers (e.g., `arr.length()`)
  - Built-in functions: `println`, `print`, `len`, `type_of`, `assert`
  - Built-in methods: `length`, `size`, `push`, `pop`, `new`
  - Recursive function calls
  - Nested function calls
  - Lambda expressions with closures

### Planned Features

- ⏳ While loops
- ⏳ Pointer types (`^T`)
- ⏳ Slices (`[]T`)
- ⏳ Switch/case statements
- ⏳ Enum types
- ⏳ Union types
- ⏳ Pattern matching
- ⏳ Multiple return values
- ⏳ Named return values

## Documentation

- **[ODIN_SYNTAX.md](ODIN_SYNTAX.md)** - Complete syntax reference
- **[MIGRATION_SUMMARY.md](MIGRATION_SUMMARY.md)** - Details of syntax transformation
- **[CALL_EVALUATION.md](CALL_EVALUATION.md)** - Call evaluation and lookup system

## CLI Options

```bash
lang [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the source file

Options:
  -s, --search-paths <SEARCH_PATHS>  Additional search paths for imports
  -a, --ast                          Print the AST
  -l, --log                          Enable logging
  -d, --debug                        Enable debug mode
  -h, --help                         Print help
```

## Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with example
cargo run -- examples/addition.odin
```

## Architecture

The language consists of:

1. **Lexer** (`src/lexer.rs`) - Tokenizes source code
2. **Parser** (`src/parser.rs`) - Builds AST from tokens
3. **AST** (`src/ast.rs`) - Abstract syntax tree definitions
4. **Interpreter** (`src/interpreter.rs`) - Executes the AST

## Contributing

This is a personal project demonstrating Odin-style syntax in an interpreted language.

## License

See LICENSE file for details.

## Acknowledgments

- Inspired by the [Odin programming language](https://odin-lang.org/)
- Uses Rust for implementation
- Built with `clap` for CLI parsing

---

**Note**: This is an educational/experimental language. Type annotations are currently for documentation purposes only, as the language remains dynamically typed at runtime.
