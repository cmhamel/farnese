module Base

abstract type Any <: Any end
abstract type Type{T} end
abstract type Vararg{T} end


abstract type Number end
abstract type Real <: Number end
abstract type AbstractFloat <: Real end

primitive type Float32 <: AbstractFloat 32 end
primitive type Float64 <: AbstractFloat 64 end

struct Nothing
end

struct Pair
  a
  b
end

struct FloatPair
  a::Float64
  b::Float64
end

end # module
