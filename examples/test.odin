import "addition"

result := add(1, 2)

Message :: struct {
    a: string,
    b: int,
}

Point :: struct {
    x: int,
    y: int,
}

hello :: proc(name: string) {
    println("Hello, ", name, "!")
}

main :: proc() {
    hello("World")
    println("Result: ", result)
}