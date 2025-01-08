use crate::core::Symbol;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;
use std::collections::HashMap;


pub fn create_main<'a, 'b>(
  context: &'a Context, builder: &'b Builder<'a>, 
  module: &'b Module<'a>,
  other_modules: &'b HashMap<Symbol, Module<'a>>
) -> () {
  // helpers
  let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
  let datatype_opaque = module.get_struct_type("DataType").unwrap();

  // todo not sure why this isn't linking properly
  // let _ = module.add_function(
  //   "printf",
  //   i8_ptr_type.fn_type(&[
  //     i8_ptr_type.into()
  //   ], true),
  //   None
  // );

  let _ = module.add_function(
    "printf",
    other_modules
      .get(&Symbol::new("Core"))
      .unwrap()
      .get_function("printf")
      .unwrap()
      .get_type(),
    None
  );

  
  let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
  let i32_type = context.i32_type();
  let fn_type = i32_type.fn_type(&[], false);
  let function = module.add_function("main", fn_type, None);
  let basic_block = context.append_basic_block(function, "entry");
  builder.position_at_end(basic_block);

  let t = module.get_global("DataType").unwrap();
  let s = context.const_string(b"Hello, world!", true);
  let ptr = builder.build_alloca(s.get_type(), "__str").unwrap();
  let _ = builder.build_store(ptr, s);
  let ptr_i8 = builder.build_alloca(i8_ptr_type, "__str_i8_cast").unwrap();
  // builder.build_store(ptr, s);
  let gep_ptr = unsafe { builder.build_in_bounds_gep(ptr, &[
    context.i32_type().const_zero(),
    context.i32_type().const_zero(),
  ], "__gep_ptr").unwrap() };
  let _ = builder.build_call(
    module.get_function("printf").unwrap(),
    &[gep_ptr.into()],
    "__printf_call"
  );

  let a = module.get_global("DataType").unwrap();
  let a_type = builder.build_alloca(datatype_opaque, "__any_type_ptr").unwrap();
  // let fn_type = datatype_opaque.fn_type(&[i8_ptr_type.into()], false);
  let gep_ptr = unsafe { builder.build_in_bounds_gep(
    module.get_global("__Sym__DataType").unwrap().as_pointer_value().into(), &[
    context.i32_type().const_zero(),
    context.i32_type().const_zero(),
  ], "__gep_ptr").unwrap() };
  let _ = builder.build_call(
    module.get_function("DataType__new").unwrap(),
    // &[module.get_global("__Sym__Any").unwrap().as_pointer_value().into()],
    &[gep_ptr.into()],
    "__any__new__call"
  );

  let int = i32_type.const_int(0, false);
  let _ = builder.build_return(Some(&int));
}
