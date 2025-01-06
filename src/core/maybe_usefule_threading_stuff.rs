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