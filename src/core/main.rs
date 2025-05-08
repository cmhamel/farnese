use crate::core::{Symbol, Value};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use std::collections::HashMap;


pub fn create_main<'a, 'b>(
  context: &'a Context, builder: &'b Builder<'a>, 
  module: &'b Module<'a>,
  other_modules: &'b HashMap<Symbol, Module<'a>>
) -> () {
  // helpers
  // let datatype_opaque = module.get_struct_type("DataType").unwrap();
  // let val_opaque = module.get_struct_type("Value").unwrap();

  // need to generalize this with export/import/using
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

  // dummy function
  // let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
  let i32_type = context.i32_type();
  let fn_type = i32_type.fn_type(&[], false);
  let function = module.add_function("main", fn_type, None);
  let basic_block = context.append_basic_block(function, "entry");
  builder.position_at_end(basic_block);

  // trying typeof
  // let a_type = builder.build_alloca(i8_ptr_type, "__type").unwrap();
  let a_type = builder.build_call(
    module.get_function("typeof_inner").unwrap(),
    &[
      // a_type.into()
      module.get_global("Number").unwrap().as_pointer_value().into()
    ],
    ""
  ).unwrap().try_as_basic_value().left().unwrap();

  // trying supertype
  let s_type = builder.build_call(
    module.get_function("supertype").unwrap(),
    &[
      module.get_global("Real").unwrap().as_pointer_value().into()
    ],
    ""
  ).unwrap().try_as_basic_value().left().unwrap();
  let s_type = builder.build_call(
    module.get_function("typeof_inner").unwrap(),
    &[
      // module.get_global("Any").unwrap().as_pointer_value().into()
      s_type.into()
    ],
    ""
  ).unwrap().try_as_basic_value().left().unwrap();
  // Create a format string for printf
  let format_string = context.const_string(b"%s\n", true);
  let global_format = module.add_global(format_string.get_type(), None, "format_string");
  global_format.set_initializer(&format_string);

  // Get a pointer to the format string
  let zero = context.i32_type().const_zero();
  let format_ptr = unsafe { global_format.as_pointer_value().const_gep(&[zero, zero]) };

  // Call `printf` to print the first field
  let _ = builder.build_call(
    module.get_function("printf").unwrap(),
    &[format_ptr.into(), a_type.into()],
    "",
  );
  let _ = builder.build_call(
    module.get_function("printf").unwrap(),
    &[format_ptr.into(), s_type.into()],
    "",
  );

  // trying some value stuff
  // builder.build_alloca(val_opaque, "").unwrap();
  let val = Value::new(69.into(), &Symbol::new("Int64"), &module, &builder);
  let _ = val.typeof_func(&module);
  let _ = val.value_func(&module);

  let t = builder.build_call(
    module.get_function("typeof").unwrap(),
    &[(*val.get_ptr()).into()],
    ""
  ).unwrap().try_as_basic_value().left().unwrap();
  let _ = builder.build_call(
    module.get_function("value").unwrap(),
    &[(*val.get_ptr()).into()],
    ""
  );
  let _ = builder.build_call(
    module.get_function("printf").unwrap(),
    &[format_ptr.into(), t.into()],
    "",
  );
  // let _ = builder.build_call(
  //   module.get_function("printf").unwrap(),
  //   &[format_ptr.into(), v.into()],
  //   "printf_call",
  // );
  // let _ = builder.build_store(a_type, module.get_global("Any").unwrap());

  // let t = module.get_global("DataType").unwrap();

  // hello world for future reference
  // let s = context.const_string(b"Hello, world!", true);
  // let ptr = builder.build_alloca(s.get_type(), "__str").unwrap();
  // let _ = builder.build_store(ptr, s);
  // let ptr_i8 = builder.build_alloca(i8_ptr_type, "__str_i8_cast").unwrap();
  // // builder.build_store(ptr, s);
  // let gep_ptr = unsafe { builder.build_in_bounds_gep(ptr, &[
  //   context.i32_type().const_zero(),
  //   context.i32_type().const_zero(),
  // ], "__gep_ptr").unwrap() };
  // let _ = builder.build_call(
  //   module.get_function("printf").unwrap(),
  //   &[gep_ptr.into()],
  //   "__printf_call"
  // );

  // let a = module.get_global("DataType").unwrap();
  
  // let a_type = builder.build_alloca(datatype_opaque, "__any_type_ptr").unwrap();
  // let a_val = builder.build_alloca(val_opaque, "__any_val_ptr").unwrap();
  // // let fn_type = datatype_opaque.fn_type(&[i8_ptr_type.into()], false);
  // let gep_ptr = unsafe { builder.build_in_bounds_gep(
  //   module.get_global("__Sym__DataType").unwrap().as_pointer_value().into(), &[
  //   context.i32_type().const_zero(),
  //   context.i32_type().const_zero(),
  // ], "__gep_ptr").unwrap() };
  // let _ = builder.build_call(
  //   module.get_function("DataType__new").unwrap(),
  //   // &[gep_ptr.into()],
  //   &[],
  //   "__any__new__call"
  // );

  // // trying to make the any type
  // let gep_ptr = unsafe { builder.build_in_bounds_gep(
  //   module.get_global("__Sym__Any").unwrap().as_pointer_value().into(), &[
  //   context.i32_type().const_zero(),
  //   context.i32_type().const_zero(),
  // ], "__gep_ptr").unwrap() };
  // let _ = builder.build_call(
  //   module.get_function("Any__new").unwrap(),
  //   // &[gep_ptr.into()],
  //   &[],
  //   "__any__new__call"
  // );
}
