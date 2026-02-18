abstract type Any end
abstract type Number <: Any end
abstract type Real <: Number end
abstract type Integer <: Real end
abstract type Signed <: Integer end

primitive type Int32 <: Signed 32 end
primitive type Int64 <: Signed 64 end
