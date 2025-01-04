use super::any;
use super::datatype;
use super::value;
use super::{Symbol};
use inkwell;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::types::StructType;
use inkwell::values::{GlobalValue, IntValue, PointerValue};
use inkwell::AddressSpace;
use std::ffi::CString;



// fn create_box_fn<'a>(context: &'a Context, module: Module<'a>) -> Module<'a> {
//   let builder = context.create_builder();
//   let value_type = create_value_type(context);
//   let fn_type = value_type
//     .ptr_type(AddressSpace::default())
//     .fn_type(
//       &[
//         context.i8_type().ptr_type(AddressSpace::default()).into(),
//         context.i8_type().ptr_type(AddressSpace::default()).into(),
//       ],
//       false,
//     );
//   let function = module.add_function("__box", fn_type, None);
//   let entry = context.append_basic_block(function, "entry");
//   builder.position_at_end(entry);
//   let size = value_type.size_of().unwrap();
//   let malloc_fn = module.add_function(
//     "malloc",
//     context.i8_type().ptr_type(AddressSpace::default()).fn_type(&[size.get_type().into()], false),
//     None,
//   );
//   let heap_alloc = builder
//     .build_call(malloc_fn, &[size.into()], "__heap_alloc").unwrap()
//     .try_as_basic_value()
//     .left()
//     .unwrap()
//     .into_pointer_value();
//   let boxed_value = builder.build_bit_cast(heap_alloc, value_type.ptr_type(AddressSpace::default()), "__boxed_value").unwrap();
//   let ref_count_ptr = builder.build_struct_gep(boxed_value.into_pointer_value(), 0, "__ref_count_ptr").unwrap();
//   let _ = builder.build_store(ref_count_ptr, context.i64_type().const_int(1, false));

//   let type_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
//   let type_ptr_gep = builder.build_struct_gep(boxed_value.into_pointer_value(), 1, "__type_ptr_gep").unwrap();
//   let _ = builder.build_store(type_ptr_gep, type_ptr);

//   let value_ptr = function.get_nth_param(1).unwrap().into_pointer_value();
//   let value_ptr_gep = builder.build_struct_gep(boxed_value.into_pointer_value(), 2, "__value_ptr_gep").unwrap();
//   let _ = builder.build_store(value_ptr_gep, value_ptr);

//   // Return the boxed value
//   let _ = builder.build_return(Some(&boxed_value));
//   module.clone()
// }



// fn create_release_fn<'a>(context: &'a Context, module: Module<'a>) -> Module<'a> {
//   let builder = context.create_builder();
//   let jl_value_type = create_value_type(context);
//   let fn_type = context.void_type().fn_type(&[jl_value_type.ptr_type(AddressSpace::default()).into()], false);

//   let function = module.add_function("__release", fn_type, None);
//   let entry = context.append_basic_block(function, "entry");
//   let _ = builder.position_at_end(entry);

//   let boxed_value = function.get_nth_param(0).unwrap().into_pointer_value();
//   let ref_count_ptr = builder.build_struct_gep(boxed_value, 0, "__ref_count_ptr").unwrap();
//   let ref_count = builder.build_load(ref_count_ptr, "__ref_count").unwrap().into_int_value();
//   let decremented = builder.build_int_sub(ref_count, context.i64_type().const_int(1, false), "__decremented").unwrap();
//   let _ = builder.build_store::<IntValue>(ref_count_ptr, decremented);

//   // If ref_count reaches zero, free the memory
//   let zero = context.i64_type().const_int(0, false);
//   let is_zero = builder.build_int_compare(inkwell::IntPredicate::EQ, decremented, zero, "__is_zero").unwrap();

//   let free_block = context.append_basic_block(function, "free");
//   let continue_block = context.append_basic_block(function, "continue");
//   let _ = builder.build_conditional_branch(is_zero, free_block, continue_block);

//   builder.position_at_end(free_block);
//   let free_fn = module.add_function(
//     "free",
//     context.void_type().fn_type(&[context.i8_type().ptr_type(AddressSpace::default()).into()], false),
//     None,
//   );
//   let casted_ptr = builder.build_bit_cast(boxed_value, context.i8_type().ptr_type(AddressSpace::default()), "__casted_ptr").unwrap();
//   let _ = builder.build_call(free_fn, &[casted_ptr.into()], "");
//   let _ = builder.build_unconditional_branch(continue_block);

//   builder.position_at_end(continue_block);
//   let _ = builder.build_return(None);

//   module.clone()
// }

// fn create_retain_fn<'a>(context: &'a Context, module: Module<'a>) -> Module<'a> {
//   let builder = context.create_builder();
//   let jl_value_type = create_value_type(context);
//   let fn_type = context.void_type().fn_type(&[jl_value_type.ptr_type(AddressSpace::default()).into()], false);

//   let function = module.add_function("__retain", fn_type, None);
//   let entry = context.append_basic_block(function, "entry");
//   builder.position_at_end(entry);

//   let boxed_value = function.get_nth_param(0).unwrap().into_pointer_value();
//   let ref_count_ptr = builder.build_struct_gep(boxed_value, 0, "__ref_count_ptr").unwrap();
//   let ref_count = builder.build_load(ref_count_ptr, "__ref_count").unwrap().into_int_value();
//   let incremented = builder.build_int_add(ref_count, context.i64_type().const_int(1, false), "__incremented");
//   let _ = builder.build_store::<IntValue>(ref_count_ptr, incremented.unwrap());

//   let _ = builder.build_return(None);

//   // function
//   module.clone()
// }

// fn create_value_type<'a>(context: &'a Context) -> StructType<'a> {
//   let ref_count = context.i64_type();
//   let type_ptr = context.i8_type().ptr_type(AddressSpace::default());
//   let value_ptr = context.i8_type().ptr_type(AddressSpace::default());
//   context.struct_type(&[ref_count.into(), type_ptr.into(), value_ptr.into()], false)
// }

fn create_formatted_strings<'a>(context: &'a Context, module: Module<'a>) -> Module<'a> {
  // add printf method
  let i8_type = context.i8_type();
  let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
  let printf_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], true);
  let _ = module.add_function("printf", printf_type, Some(Linkage::External));
  // let func = module.add_function("printf", printf_type, None);
  // let _ = module.add_global(func.get_type(), None, "printf");
  // formatted strings
  let format_strs = vec!["%c", "%f", "%d"];
  let format_names = vec!["__format_char", "__format_f64", "__format_i64"];
  for (format, name) in format_strs.into_iter().zip(format_names.into_iter()) {
    let c_string = CString::new(format).unwrap();
    let global_string = context.const_string(c_string.as_bytes_with_nul(), true);
    let global_var = module.add_global(global_string.get_type(), None, name);
    global_var.set_initializer(&global_string);
    // global_var.set_metadata(context.metadata_string("module: Core"), 0);
    global_var.set_constant(true);
  }

  // Add some data to ensure things are used (to prevent optimization from removing it)
  let _dummy_string = context.const_string(b"__dummy_str", false);

  module.clone()
}

pub struct Core<'a> {
  pub module: Module<'a>,
  pub type_tag: StructType<'a>,
  pub value_type: StructType<'a>,
  pub any_type: GlobalValue<'a>
}

impl<'a> Core<'a> {
  pub fn new(context: &'a Context) -> Self {
    let module = context.create_module("Core");
    let _ = module.add_global_metadata("module", &context.metadata_string("Core"));
    // let _ = module.add_global("__ModuleName");
    // let module_na
    let builder = context.create_builder();

    // set up basic stuff for string formatting
    let module = create_formatted_strings(context, module.clone());

    // type system
    let type_tag = datatype::create_type_tag(context);
    let value_type = value::create_value_type(context, type_tag);

    // any type
    let module = any::create_any_type(context, module.clone(), type_tag);
    let any_type = module.get_global("Any").unwrap();

    // // any type constructor
    // let fn_type = context.i32_type().fn_type(&[], false);
    // let function = module.add_function("__Any__new", fn_type, None);
    // let basic_block = context.append_basic_block(function, "entry");
    // builder.position_at_end(basic_block);
    // let any_value = any::AnyValue::new_on_stack(context, &builder, type_tag, value_type, any_type);
    // // builder.build_return(None);
    // builder.build_return(Some(&context.i32_type().const_int(0, false)));

    // type name
    let fn_type = context.i8_type()
      .ptr_type(AddressSpace::default())
      .fn_type(&[any_type.as_pointer_value().get_type().into()], false);
    let function = module.add_function("__Type__name", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);
    let any_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
    let first_field_ptr = builder
        .build_struct_gep(any_ptr, 0, "__name")
        .unwrap();
    let first_field = builder.build_load(first_field_ptr, "__name__load").unwrap().into_pointer_value();
    builder.build_return(Some(&first_field));

    // super type
    let fn_type = type_tag
      .ptr_type(AddressSpace::default())
      .fn_type(&[any_type.as_pointer_value().get_type().into()], false);
    let function = module.add_function("__Type__supertype", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);
    let any_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
    let second_field_ptr = builder
        .build_struct_gep(any_ptr, 1, "__supertype")
        .unwrap();
    let second_field = builder.build_load(second_field_ptr, "__supertype__load").unwrap().into_pointer_value();
    builder.build_return(Some(&second_field));

    // any type constructor
    let fn_type = context.i32_type().fn_type(&[], false);
    let function = module.add_function("__Any__new", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);
    let any_value = any::AnyValue::new_on_stack(context, &builder, type_tag, value_type, any_type);
    let supertype_fn = module.get_function("__Type__supertype").unwrap();
    builder.build_call(super_type_fn, &[any_value.into()], "__call_Type_supertype");
    // builder.build_return(None);
    builder.build_return(Some(&context.i32_type().const_int(0, false)));

    // module.print_to_stderr();
    Self {
      module: module,
      type_tag: type_tag,
      value_type: value_type,
      any_type: any_type
    }
  } 
}

// pub fn create_core<'a>(context: &'a Context) -> Module<'a> {
//   // create a module
//   let module = context.create_module("Core");
//   let _ = module.add_global_metadata("module", &context.metadata_string("Core"));
//   let builder = context.create_builder();

//   // type system
//   let type_tag = datatype::create_type_tag(context);
//   let value_type = value::create_value_type(context, type_tag);

//   // set up basic stuff for string formatting
//   let module = create_formatted_strings(context, module.clone());

//   module
// }
