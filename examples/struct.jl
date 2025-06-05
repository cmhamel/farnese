module MyModule

abstract type Any end
abstract type Number end
abstract type AbstractFloat <: Number end
abstract type Integer <: Number end

primitive type Float64 <: AbstractFloat 64 end
primitive type Int64 <: Integer 64 end

struct Pair <: Number
    a::Float64
    b::Float64
end

function add(a::Int64, b::Int64)::Int64
    a + b
end

# function add(a::Float64, b::Float64)::Float64
#     a + b
# end 

function main()
    a = 1
    b = 2
    printf(a)
    printf(b)
    # c = add(a, b)
    c = (a + b)
    d = c + 69
    printf(d, a, a, b)

    # pair = Pair(a, b)
    e = 1.
    f = 2.
    printf(e)
    printf(f)
    # g = add(e, f)
    printf(e, f)
end

end # module
