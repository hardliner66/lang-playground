# Odin-Style Syntax Documentation

This document describes the Odin-inspired syntax that has been implemented in this language.

## Overview

The language syntax has been transformed from Ruby-style to Odin-style, featuring:
- `::` for declarations
- `:=` for type-inferred variable initialization
- `proc` for procedures/functions
- `struct` for data structures
- Type annotations with `:`
- Braces `{}` for code blocks
- `//` for comments

## Key Syntax Changes

### Comments

**Odin Style:**
```odin
// This is a single-line comment
```

**Old Ruby Style:**
```ruby
@ This was a comment
```

---

### Variable Declarations

**Odin Style:**
```odin
// Type-inferred declaration
x := 10

// Explicit type annotation
y: int = 20

// Regular assignment (for existing variables)
x = 30
```

**Old Ruby Style:**
```ruby
x = 10
```

---

### Procedure Definitions

**Odin Style:**
```odin
// Simple procedure
greet :: proc() {
    println("Hello!")
}

// With parameters
add :: proc(a: int, b: int) -> int {
    return a + b
}

// Parameters can optionally omit types
simple :: proc(x, y) {
    return x + y
}
```

**Old Ruby Style:**
```ruby
def greet
  println("Hello!")
end

def add(a, b)
  return a + b
end
```

---

### Struct Definitions

**Odin Style:**
```odin
Person :: struct {
    name: string,
    age: int,
    active: bool,
}

// Empty struct
EmptyStruct :: struct { }
```

**Old Ruby Style:**
```ruby
message Person
  name: String
  age: Integer
  active: Boolean
end

system MyClass
  def initialize()
  end
end
```

---

### Control Flow

#### If Statements

**Odin Style:**
```odin
if x > 10 {
    println("Greater than 10")
} else {
    println("10 or less")
}
```

**Old Ruby Style:**
```ruby
if x > 10
  println("Greater than 10")
elsif x > 5
  println("Between 5 and 10")
else
  println("5 or less")
end
```

---

#### For Loops

**Odin Style:**
```odin
for item in collection {
    println(item)
}
```

**Old Ruby Style:**
```ruby
for item in collection do
  println(item)
end
```

---

### Control Flow Keywords

**Odin Style:**
- `break` - Exit loop
- `continue` - Skip to next iteration
- `return` - Return from procedure
- `defer` - Defer statement execution

**Old Ruby Style:**
- `break` - Exit loop
- `next` - Skip to next iteration
- `return` - Return from method

---

### Imports

**Odin Style:**
```odin
import "math"
import "path/to/module"
```

**Old Ruby Style:**
```ruby
require "math"
```

---

## Complete Example

```odin
import "addition"

// Define a struct
Point :: struct {
    x: int,
    y: int,
}

// Define a procedure with return type
distance :: proc(p1: Point, p2: Point) -> int {
    dx := p1.x - p2.x
    dy := p1.y - p2.y
    return dx * dx + dy * dy
}

// Main procedure
main :: proc() {
    // Type-inferred variable
    a := 10
    b := 20
    
    // Call procedure
    sum := add(a, b)
    
    // Conditional
    if sum > 25 {
        println("Sum is large")
    } else {
        println("Sum is small")
    }
    
    // Loop
    for i in [1, 2, 3] {
        println("Number:", i)
    }
    
    println("Result:", sum)
}
```

---

## Operators

All standard operators remain the same:

### Arithmetic
- `+` Addition
- `-` Subtraction
- `*` Multiplication
- `/` Division
- `%` Modulo
- `**` Power

### Comparison
- `==` Equal
- `!=` Not equal
- `<` Less than
- `<=` Less than or equal
- `>` Greater than
- `>=` Greater than or equal

### Logical
- `&&` Logical AND
- `||` Logical OR
- `!` Logical NOT

### Bitwise
- `&` Bitwise AND
- `|` Bitwise OR
- `^` Bitwise XOR
- `~` Bitwise NOT
- `<<` Left shift
- `>>` Right shift

### Assignment
- `=` Assignment
- `:=` Declaration with type inference
- `:` Type annotation

### Other
- `::` Declaration operator
- `->` Return type indicator
- `=>` Hash arrow (for key-value pairs)
- `..` Range (inclusive)
- `...` Range (exclusive)

---

## Type System

The language supports the following type annotations:

- `int` - Integer
- `float` - Floating point
- `string` - String
- `bool` - Boolean
- Custom struct types

Types are optional in parameter lists but can be specified for clarity:

```odin
// Without types
add :: proc(a, b) -> int {
    return a + b
}

// With types
add :: proc(a: int, b: int) -> int {
    return a + b
}
```

---

## File Extension

Odin-style files use the `.odin` extension instead of `.rb`.

---

## Attributes

Attributes (similar to Rust/C# attributes) are supported:

```odin
#[inline]
#[deprecated(since="1.0")]
fast_function :: proc() {
    // implementation
}

#[derive(Debug)]
MyStruct :: struct {
    field: int,
}
```

---

## Future Enhancements

The following Odin features could be added in the future:

- Pointer types (`^T`)
- Slices (`[]T`)
- Multi-value returns
- Named return values
- `when` compile-time conditionals
- `distinct` types
- `using` imports
- Union types
- Enum types
- Switch/case statements

---

## Migration Guide

To migrate existing Ruby-style code to Odin style:

1. Replace `def` with `name :: proc`
2. Replace `end` with closing braces `}`
3. Add opening braces `{` after conditions/loops
4. Change `require` to `import`
5. Change `next` to `continue`
6. Change `message`/`system` to `name :: struct`
7. Update comments from `@` to `//`
8. Update variable declarations to use `:=` where appropriate
9. Rename files from `.rb` to `.odin`

---

## Notes

- The interpreter still supports dynamic typing at runtime
- Type annotations in function signatures are currently for documentation purposes
- The language maintains its interpreted nature with added static syntax benefits