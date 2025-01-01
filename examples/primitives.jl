primitive type Float16 <: AbstractFloat 16 end
primitive type Float32 <: AbstractFloat 32 end
primitive type Float64 <: AbstractFloat 64 end

primitive type Bool    <: Integer        8 end
primitive type Char    <: AbstractChar  32 end

primitive type Int8    <: Signed         8 end
primitive type UInt8   <: Unsigned       8 end
primitive type Int16   <: Signed        16 end
primitive type UInt16  <: Unsigned      16 end
primitive type Int32   <: Signed        32 end
primitive type UInt32  <: Unsigned      32 end
primitive type Int64   <: Signed        64 end
primitive type UInt64  <: Unsigned      64 end
primitive type Int128  <: Signed       128 end
primitive type UInt128 <: Unsigned     128 end

a = 69.0
b = 23.0
c = a + b

function add(a, b)
  a + b
end

function temp
end

function sub(a, b)
end

add(a, b)

function muladd(a, b, c)
  d = add(a, b)
  d + c 
end

1
var = 69