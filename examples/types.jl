abstract type Any end
abstract type DenseArray end
abstract type Type{T} end
abstract type AbstractArray{T, N} <: DenseArray end

struct MyType{T}
    a::Float64
    b
end

struct MyType2{T <: Number, V <: AbstractFloat}
    a::T
    b::V
end

# struct MyType3 <: AbstractFloat
#     a
# end

# struct MyType4{T <: Number, V <: AbstractFloat} <: Type{T}
#     a::T
#     b::V
# end