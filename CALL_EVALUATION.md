# Call Evaluation and Lookup System

This document describes the comprehensive call evaluation and lookup system implemented in the Odin-style language interpreter.

## Overview

The call evaluation system handles:
- **Function calls**: Direct procedure invocations
- **Method calls**: Calls on receiver objects
- **Built-in functions**: System-provided utilities
- **Lambda expressions**: Anonymous function calls
- **Recursive calls**: Self-referential function invocations
- **Nested calls**: Function calls within expressions

## Architecture

### AST Representation

The `Call` expression in the AST includes:

```rust
Expression::Call {
    receiver: Option<Box<Expression>>,  // None for functions, Some for methods
    method: String,                      // Function or method name
    args: Vec<Expression>,               // Arguments to pass
}
```

### Call Types

#### 1. Function Calls (No Receiver)

**Syntax:**
```odin
function_name(arg1, arg2, arg3)
```

**Example:**
```odin
add :: proc(a: int, b: int) -> int {
    return a + b
}

result := add(10, 20)  // Returns 30
```

**Evaluation Process:**
1. Lookup function name in environment
2. Verify it's a callable value (Proc or Lambda)
3. Check argument count matches parameter count
4. Evaluate arguments
5. Create new scope
6. Bind arguments to parameters
7. Execute function body
8. Return result

#### 2. Method Calls (With Receiver)

**Syntax:**
```odin
receiver.method_name(arg1, arg2)
```

**Example:**
```odin
arr := [1, 2, 3, 4, 5]
length := arr.length()  // Returns 5
```

**Evaluation Process:**
1. Evaluate receiver expression
2. Match method name against built-in methods
3. Execute method-specific logic
4. Return result

### Built-in Functions

The interpreter provides several built-in functions that are handled directly without environment lookup:

#### `println(...)`
Prints arguments separated by spaces, followed by a newline.

```odin
println("Hello", "World")  // Output: Hello World\n
println(1, 2, 3)          // Output: 1 2 3\n
```

#### `print(...)`
Prints arguments separated by spaces, without a newline.

```odin
print("Hello ")
print("World")   // Output: Hello World
```

#### `len(value)`
Returns the length of a collection or string.

```odin
len([1, 2, 3])        // Returns 3
len("Hello")          // Returns 5
len({"a" => 1})       // Returns 1
```

#### `type_of(value)`
Returns the type name as a string.

```odin
type_of(42)           // Returns "int"
type_of("hello")      // Returns "string"
type_of([1, 2])       // Returns "array"
type_of(true)         // Returns "bool"
```

#### `assert(condition, [message])`
Asserts that a condition is true, optionally with a custom error message.

```odin
assert(x > 0)                    // Throws error if x <= 0
assert(x > 0, "x must be positive")  // Custom error message
```

### Built-in Methods

Methods are called on receiver objects using dot notation:

#### Array Methods

**`length()` / `size()`**
```odin
arr := [1, 2, 3, 4, 5]
len := arr.length()    // Returns 5
```

**`push(value)`**
```odin
arr := [1, 2, 3]
arr.push(4)            // arr is now [1, 2, 3, 4]
```

**`pop()`**
```odin
arr := [1, 2, 3]
val := arr.pop()       // val is 3, arr is now [1, 2]
```

#### String Methods

**`length()` / `size()`**
```odin
text := "Hello"
len := text.length()   // Returns 5
```

#### Hash Methods

**`length()` / `size()`**
```odin
hash := {"a" => 1, "b" => 2}
len := hash.length()   // Returns 2
```

#### Struct Methods

**`new()`**
```odin
Person :: struct {
    name: string,
    age: int,
}

person := Person.new()  // Creates instance with nil fields
```

## Lookup Algorithm

### Function Lookup (No Receiver)

1. **Check for built-in function**
   - If function name matches built-in, execute directly
   - Return result

2. **Search environment**
   - Look up name in current scope
   - If not found, search parent scopes
   - If still not found, return error

3. **Verify callable**
   - Ensure value is `Value::Proc` or `Value::Lambda`
   - If not, return type error

4. **Execute**
   - Call appropriate execution function
   - Return result

### Method Lookup (With Receiver)

1. **Evaluate receiver**
   - Execute receiver expression to get value
   - Handle any errors during evaluation

2. **Match method name**
   - Check against built-in methods for receiver type
   - Return error if method not found

3. **Execute method**
   - Perform method-specific operation
   - Return result

## Scoping and Environments

### Scope Management

Each function call creates a new scope:

```rust
self.env.borrow_mut().push_scope();  // Create scope

// Bind parameters
for (param, arg) in params.iter().zip(args.iter()) {
    let value = self.evaluate_expression(arg)?;
    self.env.borrow_mut().define(param.name.clone(), value);
}

// Execute body
let result = self.execute_block(&body)?;

self.env.borrow_mut().pop_scope();   // Clean up scope
```

### Variable Resolution

Variables are resolved by searching scopes from innermost to outermost:

1. Current (local) scope
2. Parent scopes (outer functions)
3. Global scope
4. Error if not found

### Closure Support

Lambda expressions capture their enclosing environment:

```rust
Value::Lambda(
    params.clone(),
    body.clone(),
    Rc::clone(&self.env),  // Capture environment
)
```

When a lambda is called, the captured environment is used instead of the current one.

## Parameter Binding

### Type Annotations

Parameters can optionally specify types:

```odin
// With types (for documentation)
add :: proc(a: int, b: int) -> int {
    return a + b
}

// Without types
add :: proc(a, b) {
    return a + b
}
```

**Note:** Type annotations are currently for documentation only. The interpreter remains dynamically typed at runtime.

### Argument Evaluation

Arguments are evaluated before being bound to parameters:

1. Evaluate each argument expression in the caller's environment
2. Check argument count matches parameter count
3. Bind evaluated values to parameter names in new scope

### Variadic Functions

Not currently supported. All functions must receive exactly the number of arguments specified in their parameter list.

## Recursion

The interpreter fully supports recursive function calls:

```odin
factorial :: proc(n: int) -> int {
    if n <= 1 {
        return 1
    } else {
        return n * factorial(n - 1)
    }
}

result := factorial(5)  // Returns 120
```

### Tail Call Optimization

**Status:** Not implemented.

Tail-recursive calls will build up the call stack normally. Deep recursion may cause stack overflow.

## Error Handling

### Common Errors

**Undefined Function:**
```
Error: Undefined function or procedure 'foo'
```

**Wrong Argument Count:**
```
Error: Proc 'add' expects 2 arguments but got 3
```

**Not Callable:**
```
Error: 'x' is not callable
```

**Method Not Found:**
```
Error: Method 'foo' not found on Integer(42)
```

**Type Error:**
```
Error: len() not supported for Integer(42)
```

### Error Propagation

Errors bubble up through the call stack:

1. Error occurs in nested call
2. Function returns `Err(message)`
3. Caller receives error and propagates
4. Continues until caught or reaches top level

## Performance Considerations

### Function Call Overhead

Each function call involves:
- Scope creation
- Argument evaluation
- Parameter binding
- Scope cleanup

For performance-critical code, consider:
- Reducing call depth
- Inlining simple operations
- Using local variables instead of repeated calls

### Recursion Depth

Without tail call optimization, deep recursion can cause:
- Stack overflow
- Memory pressure
- Performance degradation

Recommended maximum recursion depth: ~1000 calls

## Examples

### Basic Function Call

```odin
square :: proc(x: int) -> int {
    return x * x
}

result := square(5)  // Returns 25
```

### Nested Function Calls

```odin
add :: proc(a: int, b: int) -> int {
    return a + b
}

multiply :: proc(a: int, b: int) -> int {
    return a * b
}

result := add(multiply(3, 4), multiply(2, 5))
// multiply(3, 4) = 12
// multiply(2, 5) = 10
// add(12, 10) = 22
```

### Method Chaining (Limited)

```odin
// Single method call
arr := [1, 2, 3]
len := arr.length()

// Multiple operations require intermediate variables
arr.push(4)
new_len := arr.length()
```

### Function Composition

```odin
double :: proc(x: int) -> int {
    return x * 2
}

apply_twice :: proc(x: int) -> int {
    return double(double(x))
}

result := apply_twice(5)  // Returns 20
```

### Higher-Order Patterns

```odin
apply :: proc(f, x) {
    return f(x)
}

// Note: Currently limited due to no first-class function passing
```

### Recursive Fibonacci

```odin
fib :: proc(n: int) -> int {
    if n <= 1 {
        return n
    } else {
        return fib(n - 1) + fib(n - 2)
    }
}

println(fib(10))  // Returns 55
```

## Implementation Details

### Call Evaluation Function

```rust
fn evaluate_call(
    &mut self,
    receiver: &Option<Box<Expression>>,
    method: &str,
    args: &[Expression],
) -> Result<Value, String>
```

**Parameters:**
- `receiver`: Optional receiver for method calls
- `method`: Function or method name
- `args`: Argument expressions

**Returns:**
- `Ok(Value)`: Successful execution result
- `Err(String)`: Error message

### Proc Execution

```rust
fn call_proc(
    &mut self,
    proc_def: &ProcDef,
    args: &[Expression],
) -> Result<Value, String>
```

**Process:**
1. Validate argument count
2. Push new scope
3. Bind parameters
4. Execute body
5. Handle return value
6. Pop scope
7. Return result

### Lambda Execution

```rust
fn call_lambda(
    &mut self,
    params: &[String],
    body: &[Statement],
    closure_env: &Rc<RefCell<Environment>>,
    args: &[Expression],
) -> Result<Value, String>
```

**Key Difference from Proc:**
- Switches to captured closure environment
- Restores original environment after execution

### Method Call Dispatch

```rust
fn evaluate_method_call(
    &mut self,
    receiver_expr: &Expression,
    method: &str,
    args: &[Expression],
) -> Result<Value, String>
```

**Dispatch based on:**
1. Receiver type (Array, String, Hash, Struct, Instance)
2. Method name
3. Built-in method implementations

## Future Enhancements

### Planned Features

1. **First-class Functions**
   - Pass functions as arguments
   - Return functions from functions
   - Store functions in variables

2. **Method Chaining**
   - Fluent API support
   - Chainable method returns

3. **Variadic Functions**
   - Variable argument lists
   - `args...` syntax

4. **Named Arguments**
   - `func(name: value)` syntax
   - Optional parameters

5. **Default Parameters**
   - Parameter default values
   - Partial application

6. **Tail Call Optimization**
   - Optimize tail-recursive calls
   - Prevent stack overflow

7. **Function Overloading**
   - Multiple signatures
   - Type-based dispatch

8. **Generic Functions**
   - Type parameters
   - Polymorphic procedures

## Testing

See `examples/call_test.odin` and `examples/advanced_calls.odin` for comprehensive test suites demonstrating all call evaluation features.

## Summary

The call evaluation system provides:
- ✅ Direct function calls
- ✅ Method calls with receivers
- ✅ Built-in functions (println, print, len, type_of, assert)
- ✅ Built-in methods (length, push, pop, new)
- ✅ Recursive calls
- ✅ Nested calls
- ✅ Lambda expressions with closures
- ✅ Proper scoping and environment management
- ✅ Comprehensive error handling

The system is fully functional and supports the complete range of call patterns needed for practical programming in the Odin-style language.