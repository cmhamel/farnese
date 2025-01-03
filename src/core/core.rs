use inkwell;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use std::ffi::CString;

pub fn create_core<'a>(context: &'a Context) -> Module<'a> {
  // create a module
  let module = context.create_module("Core");
  let builder = context.create_builder();

  // Create the `printf` function declaration
  let i8_type = context.i8_type();
  let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
  let printf_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], true);
  let func = module.add_function("printf", printf_type, Some(Linkage::External));
  // let entry = context.append_basic_block(func, "entry");
  // builder.position_at_end(entry);

  // // some format strings
  let format_strs = vec!["%c", "%f", "%d"];
  let format_names = vec!["__format_char", "__format_f64", "__format_i64"];
  for (format, name) in format_strs.into_iter().zip(format_names.into_iter()) {
    let c_string = CString::new(format).unwrap();
    let global_string = context.const_string(c_string.as_bytes_with_nul(), true);
    let global_var = module.add_global(global_string.get_type(), None, name);
    global_var.set_initializer(&global_string);
    global_var.set_constant(true);
  }

  // test Any type
  let i64_type = context.i64_type();
  let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
  let any = context.struct_type(&[i64_type.into(), i8_ptr_type.into()], false);
  // any.set_name("Any");
  module.add_global(any, None, "Any");
  // module.add_type("Any", any);
  module
}

// fn create_format_str<'a>(
//   context: &'a Context, module: &'a mut Module<'a>,
//   string: &str, name: &str
// ) -> () {
//   let c_string = CString::new(string).unwrap();
//   let global_string = context.const_string(c_string.as_bytes_with_nul(), true);
//   // let string_type = global_string.get_type();
//   let global_var = module.add_global(global_string.get_type(), None, name);
//   global_var.set_initializer(&global_string);
//   global_var.set_constant(true);
// }