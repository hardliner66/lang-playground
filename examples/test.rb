require "addition"

result = add(1, 2)
message Blub
  a
  b
end

system Blah
  def hello(s)
    println( "Hello, #{s}!")
  end
end

b = Blah.new
b.hello("World")

result