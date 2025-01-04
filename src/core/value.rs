use super::datatype::create_type_tag;
use super::Symbol;
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::types::StructType;
use inkwell::AddressSpace;

// sets up a type that is { i8* Type* } where 
// Type = { i8* Type* }
pub fn create_value_type<'a>(context: &'a Context, type_tag: StructType<'a>) -> StructType<'a> {
  let value_sym = Symbol::new("Value");
  let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
let value_type = context.opaque_struct_type("Value");
  value_type.set_body(&[
    i8_ptr_type.into(), // value
    type_tag.ptr_type(AddressSpace::default()).into()
  ], false);
  value_type
}

