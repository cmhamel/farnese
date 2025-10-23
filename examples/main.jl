abstract type Any end

# function println(a)
#   printf(a)
#   printf("\n")
# end

abstract type MyAbstractType end

function main()
  a = :some_big_long_symbol
  printf(a)
  printf("\n")
  b = 2 + 3
  printf(b)
  c = "some string"
  printf(c)
  # println(c)
end
