require "addition"

result = add(1, 2)
message Blub
  a: String
  b: Integer
end

system Blah
  def hello(s)
    println( "Hello, #{s}!")
  end
end

b = Blah.new
b.hello("World")

result