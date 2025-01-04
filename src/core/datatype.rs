use super::Symbol;
// use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::StructType;
// use inkwell::values::GlobalValue;
use inkwell::AddressSpace;



#[derive(Clone, Debug)]
// pub struct DataType<'a> {
pub struct DataType {
  name: Symbol,
  supertype: Symbol,
  is_abstract: bool,
  is_mutable: bool,
  is_primitive: bool,
  // struct_type: StructType<'a>
}

impl DataType {
  pub fn new(
    name: &Symbol, supertype: &Symbol,
    is_abstract: bool, is_mutable: bool, is_primitive: bool,
    // struct_type: StructType<'a>
  ) -> Self {
    Self {
      name: name.clone(),
      supertype: supertype.clone(),
      is_abstract: is_abstract,
      is_mutable: is_mutable,
      is_primitive: is_primitive,
      // struct_type: struct_type
    }
  }

  // getters
  pub fn hash_id(&self) -> u64 { self.name.hash() }
  pub fn is_abstract(&self) -> bool { self.is_abstract }
  pub fn is_mutable(&self) -> bool { self.is_mutable }
  pub fn is_primitive(&self) -> bool { self.is_primitive }
  pub fn name(&self) -> &str { &self.name.name() }
  // pub fn struct_type(&self) -> StructType<'a> { self.struct_type }

  // pub fn create_type_maybe_<'a>(&self, context: &'a Context, module: Module<'a>, type_tag: StructType<'a>) -> Module<'a> {
  //   let type_name = context.const_string(self.name().as_bytes(), false);
  //   // println!("Type name = {:?}", type_name);
  //   // let type_name_global = module
  //   //   .add_global(type_name.get_type(), None, format!("__{}_str", self.name()).as_str())
  //   // type_name_global.set_initializer(&type_name);

  //   // let type_struct = context.struct_type(
  //   //   &[
  //   //     context.i8_type().ptr_type(AddressSpace::default()).into(),   // Type name
  //   //     context.i8_type().ptr_type(AddressSpace::default()).into(),   // Parent type tag (changed to avoid self-referential type creation issues)
  //   //   ],
  //   //   false,
  //   // );
  //   let type_struct_global = module.add_global(type_tag, None, self.name());
  //   // let any_type_tag = context.opaque_struct_type("Any");
  //   let any_type_tag = module.get_global("Any").unwrap();
  //   type_struct_global.set_initializer(&context.const_struct(
  //     &[
  //       // type_name_global.as_pointer_value(), // i8* (string pointer)
  //       // type_name.as_pointer_value(),
  //       type_name.try_into().unwrap(),
  //       any_type_tag.as_pointer_value().try_into().unwrap(),     // %Type* (pointer to the `Any` type tag)
  //     ],
  //     false,
  //   ));
  //   // type_struct_global.set_initializer()
  //   module.clone()
  // }

  // main method to create the tyep for llvm ir
  pub fn create_type<'a>(&self, context: &'a Context, module: Module<'a>, type_tag_type: StructType<'a>) -> Module<'a> {
    let type_name = context.const_string(self.name().as_bytes(), false);
    let type_name_global = module.add_global(type_name.get_type(), None, format!("__{}", self.name()).as_str());
    type_name_global.set_initializer(&type_name);

    // let type_tag = context.struct_type(
    //   &[
    //     context.i8_type().ptr_type(AddressSpace::default()).into(),   // Type name
    //     context.i8_type().ptr_type(AddressSpace::default()).into(),   // Parent type tag (changed to avoid self-referential type creation issues)
    //   ],
    //   false,
    // );
    let super_type_tag = module.get_global(self.supertype.name()).unwrap();
    let initializer = type_tag_type
      .const_named_struct(&[
        context.i8_type().ptr_type(AddressSpace::default()).const_null().into(),
        // context.i64_type().ptr_type(AddressSpace::default()).const_null().into(),       // Type name (null for now)
        // use below if you want a name...
        // context.const_string(self.name().as_bytes(), false).try_into().unwrap(),
        // type_name_global.as_pointer_value().into(),
        super_type_tag.as_pointer_value().into(), // Parent: points to itself
      ]);
    let type_tag = module.add_global(type_tag_type, None, self.name.name());
    type_tag.set_initializer(&initializer);

    
    // build teh gep
    // let gep_index = context.i32_type().const_int(1, false);
    // let gep_index = 1;
    // let struct_ptr = builder.build_struct_gep(type_ptr, gep_index, "type_ptr").unwrap();
    // module.print_to_stderr();
    module.clone()
  }

  // pub fn create_value<'a>(&self, context: &'a Context, builder: &'a Builder, type_tag: StructType<'a>) -> () {
  //   let builder = context.create_builder();
  //   // let basic_block = context.append_basic_block(function, "entry");
  //   // builder.position_at_end(basic_block);
  //   let alloca = builder.build_alloca(type_tag, "__newtype").unwrap();
  //   // let field1_ptr = unsafe {
  //   //   builder.build_gep(
  //   //       alloca,  // The pointer to the struct
  //   //       &[context.i32_type().const_int(0, false)],  // Field index 0 for type name
  //   //       "field1_ptr",
  //   //   )
  //   // };
  // }

  pub fn type_name<'a>(&self, context: &'a Context, module: Module<'a>, _type_tag: StructType<'a>) -> Module<'a> {
    let builder = context.create_builder();
    // let type_global = module.get_global(self.name()).unwrap();
    let return_type = context.i8_type().ptr_type(AddressSpace::default());
    let fn_type = return_type.fn_type(&[], false); // No parameters
    let function = module.add_function("get_first_field", fn_type, None);
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);
    // let struct_ptr = builder.build_load(type_global.as_pointer_value(), "struct_ptr"); // Load the struct pointer
    // let first_field_ptr = builder.build_gep(struct_ptr.unwrap(), &[context.i32_type().const_int(0, false)], "first_field_ptr");

    // Step 7: Return the first field's pointer
    // builder.build_return(Some(&first_field_ptr.unwrap()));


    // let fn_type = return_type.fn_type(&[], false); // No parameters
    // let function = module.add_function("typename", fn_type, None);
    // let entry = context.append_basic_block(function, "entry");
    // builder.position_at_end(entry);

    // // Step 5: Return the global variable (cast to a pointer)
    // builder.build_return(Some(&type_global.as_pointer_value()));
    // let return_type = context.i8_type().ptr_type(AddressSpace::default());
    // let fn_type = return_type.fn_type(&[], false);
    // let function = module.add_function("typename", fn_type, None);
    // let entry = context.append_basic_block(function, "entry");
    // builder.position_at_end(entry); // Correcting the reference issue
    // let type_global = module.get_global(self.name()).unwrap();
    // println!("Type global = {:?}", type_global);
    // println!("Type ptr = {:?}", type_global.as_pointer_value());
    // let builder = context.create_builder();
    // let index = vec!(context.i32_type().const_int(0, false));
    // let first_field = unsafe {
    //   builder.build_gep(type_global.as_pointer_value(), &index, "__typename")
    // }.unwrap();
    // let name_pointer = builder.build_load(first_field, "__name_pointer");
    // println!("first_field = {:?}", first_field);
    // builder.build_return(Some(&first_field)); // Fixing the unwrap issue

    module.clone()
  }

  pub fn type_of<'a>(&self, context: &'a Context, module: Module<'a>) -> Module<'a> {
    // let type_tag = module.get_struct_type(self.name()).unwrap();
    let builder = context.create_builder();

    let return_type = create_type_tag(context); // The type we're returning
    let fn_type = return_type.fn_type(&[], false); // Function takes no arguments

    // Create the function in the module
    let function = module.add_function("typeof", fn_type, None);

    // Create a basic block for the function
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry); // Correcting the reference issue

    // Allocate space for the type structure
    let type_tag_alloca = builder.build_alloca(return_type, "__type").unwrap();

    // Store some data in the allocated space (for example, a null pointer for now)
    // let null_ptr = context.i8_type().ptr_type(AddressSpace::default()).const_null();
    let null_ptr = return_type.const_zero();
    let _ = builder.build_store(type_tag_alloca, null_ptr);
    // Load the value from the allocated space
    let loaded_value = builder.build_load(type_tag_alloca, "loaded_type_tag");

    // Return the allocated structure (pointer to it)
    let _ = builder.build_return(Some(&loaded_value.unwrap())); // Fixing the unwrap issue

    module.clone()
  }
}

// pub fn supertype<'a>(module: &'a Module, child: &str, parent: &str) -> () {
//   let context = module.get_context();
//   let child = module.get_struct_type(child);
// }

// helper method probably only called once or twice maybe?
pub fn create_type_tag<'a>(context: &'a Context) -> StructType<'a> {
  let type_tag_type = context.opaque_struct_type("Type");
  // let type_tag_name = context.const_string("TypeName".as_bytes(), false);
  type_tag_type.set_body(
    &[
      // type_tag_name.ptr_type(AddressSpace::default()).into(),
      // type_tag_name.get_type().try_into().unwrap(),
      // context.i8_type().ptr_type(AddressSpace::default()).into(),
      // context.string_type().ptr_type(AddressSpace::default()).into(),
      context.i8_type().ptr_type(AddressSpace::default()).into(),
      // context.i64_type().into(),
      type_tag_type.ptr_type(AddressSpace::default()).into()
    ],
    false,
  );
  type_tag_type
}

// pub fn create_concrete_type_tag<'a>(context: &'a Context) -> StructType<'a> {
//     // Define the fields of the struct
//     let i8_ptr = context.i8_type().ptr_type(AddressSpace::default());
    
//     // Create a concrete struct with the specified fields
//     let type_tag_type = context.struct_type(
//         &[
//             i8_ptr.into(),  // First field: i8* (pointer to i8)
//             i8_ptr.into(),  // Second field: i8* (pointer to i8)
//         ],
//         false,  // Not packed
//     );
    
//     // Return the created struct type
//     type_tag_type
// }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_datatype_new() {
    let sym = Symbol::new("Any");
    let any = DataType::new(&sym, &sym, true, false, false);
    assert_eq!(any.name(), "Any");
    assert_eq!(any.is_abstract(), true);
    assert_eq!(any.is_mutable(), false);
    assert_eq!(any.is_primitive(), false);
    // assert_eq!(any.supertype(), sym);
  }

  #[test]
  fn test_create_type_tag() {
    let context = Context::create();
    let type_tag = create_type_tag(&context);
  }
}