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
	p := Point { x: 1, y: 2 }
    hello("World")
    println("Result: ", result)
}