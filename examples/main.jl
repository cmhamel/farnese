# struct Pair
#   a
#   b
# end

abstract type Any end
abstract type Float64 end

function add(a::Float64, b::Float64)::Float64
  a + b
end

a = 79
printf(a)
b = -10
printf(b)
a = 79
b = 69
c = 6
d = 12
e = 7
f = d + e
printf(a)
# value(a)
printf(f)
# t = typeof(a)
# # typeof(a)
# printf(c)
# printf(supertype(Number))
# typeof(a)
# typeof_inner(a)
# wtf(a)
# typeof(a, b)
# typeofaaadfadfadfadsfasdofijawefoijdsf(a)
# typeof(a)

# g = 128.0
# # printf(g)
# # printf(a, b)
# h = f + ((2 + a) + (b + 3))
# i = -h
# # printf(h)
# printf(i)
# j = add(a, b)
# printf(j)