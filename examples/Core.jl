abstract type Any end
abstract type Number <: Any end
abstract type Real <: Number end
abstract type Integer <: Real end
abstract type Signed <: Integer end
abstract type Unsigned <: Integer end
abstract type AbstractFloat <: Real end

primitive type Int32 <: Signed 32 end
primitive type Int64 <: Signed 64 end
primitive type Float64 <: AbstractFloat 64 end

# a = 1
# b = 2.0
# b = 2

# c = 1 + 2 * (a + b)

# printf(a)
# # some comment

# function func(a::Int64, b)
#     c = a + b

#     d = e + f
# end

+(a::Int64, b::Int64) = __add_int64(a, b)
# *(a::Int64, b::Int64) = __

function my_func(a::Int64, b::Int64)
    c = a + b + (3 + 4)
end
