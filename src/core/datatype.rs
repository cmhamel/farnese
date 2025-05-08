use crate::core::Symbol;
use inkwell::module::Module;
use inkwell::AddressSpace;


#[derive(Clone, Debug)]
// pub struct DataType<'a> {
pub struct DataType {
  pub name: Symbol,
  // supertype: Symbol,
  // is_abstract: bool,
  // is_mutable: bool,
  // is_primitive: bool,
}

impl<'a, 'b> DataType {
  pub fn new(name: Symbol, supertype: Symbol, module: &'b Module<'a>) -> Self {
    // setup
    let context = module.get_context();
    // let builder = context.create_builder();

    // helpers
    // let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
    let i64_ptr_type = context.i64_type().ptr_type(AddressSpace::default());
    let zero = context.i32_type().const_zero();

    // globals
    let datatype_opaque = module.get_struct_type("DataType").unwrap();

    // make a sym
    let sym = context.const_string(name.name().as_bytes(), true);
    let sym_glob = module.add_global(sym.get_type(), None, format!("__Sym__{}", name.name()).as_str());
    sym_glob.set_initializer(&sym);

    // add type name as a global in this module
    let datatype = module.add_global(datatype_opaque, None, name.name()); 
    let supertype_glob = module.get_global(supertype.name()).unwrap();
    let _ = unsafe { datatype.set_initializer(&datatype_opaque.const_named_struct(&[
      sym_glob.as_pointer_value().const_gep(&[zero, zero]).into(),
      supertype_glob.as_pointer_value().into(),
      i64_ptr_type.const_null().into(),
    ]))};

    // create a value type for this
    // let val = Value::new(&name, module, &builder);

    // // create typeof method
    // let func = i8_ptr_type.fn_type(&[
    //   datatype_opaque.ptr_type(AddressSpace::default()).into()
    // ], false);
    // let func = module.add_function(format!("typeof__{}", name.name()).as_str(), func, None);
    // let basic_block = context.append_basic_block(func, "entry");
    // builder.position_at_end(basic_block);

    // let ret_val = builder.build_alloca(context.i8_type(), "__ptr").unwrap();
    // let struct_ptr = func.get_first_param().unwrap().into_pointer_value();
    // let sym_ptr = builder
    //   .build_struct_gep(struct_ptr, 0, "__sym_ptr")
    //   .unwrap();
    // let sym_ptr = builder.build_load(sym_ptr, "__sym").unwrap();
    // let _ = builder.build_return(Some(&sym_ptr));

    // create supertype method
    // let func = datatype_opaque.ptr_type(AddressSpace::default()).fn_type(&[
    //   datatype_opaque.ptr_type(AddressSpace::default()).into()
    // ], false);
    // let func = module.add_function(format!("supertype__{}", name.name()).as_str(), func, None);
    // let basic_block = context.append_basic_block(func, "entry");
    // builder.position_at_end(basic_block);

    // // let ret_val = builder.build_alloca(datatype_opaque.ptr_type(AddressSpace::default()), "__ptr").unwrap();
    // let struct_ptr = func.get_first_param().unwrap().into_pointer_value();
    // let type_ptr = builder
    //   .build_struct_gep(struct_ptr, 1, "__sym_ptr")
    //   .unwrap();
    // let type_ptr = builder.build_load(type_ptr, "__sym").unwrap();
    // // let type_ptr = builder.build_load(type_ptr, "__sym").unwrap();
    // let _ = builder.build_return(Some(&type_ptr));


    Self {
      name: name,
      // supertype: supertype
    }
  }

  pub fn create_supertype(&self, module: &'b Module<'a>) -> () {
    // setup
    let context = module.get_context();
    let builder = context.create_builder();

    // helpers
    // let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());

    // globals
    let datatype_opaque = module.get_struct_type("DataType").unwrap();

    let func = datatype_opaque.ptr_type(AddressSpace::default()).fn_type(&[
      datatype_opaque.ptr_type(AddressSpace::default()).into()
    ], false);
    let func = module.add_function("supertype", func, None);
    let basic_block = context.append_basic_block(func, "entry");
    builder.position_at_end(basic_block);

    // let ret_val = builder.build_alloca(datatype_opaque.ptr_type(AddressSpace::default()), "__ptr").unwrap();
    let struct_ptr = func.get_first_param().unwrap().into_pointer_value();
    let type_ptr = builder
      .build_struct_gep(struct_ptr, 1, "")
      .unwrap();
    let type_ptr = builder.build_load(type_ptr, "").unwrap();
    // let type_ptr = builder.build_load(type_ptr, "__sym").unwrap();
    let _ = builder.build_return(Some(&type_ptr));
  }

  pub fn create_typeof(&self, module: &'b Module<'a>) -> () {
    // setup
    let context = module.get_context();
    let builder = context.create_builder();

    // helpers
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());

    // globals
    let datatype_opaque = module.get_struct_type("DataType").unwrap();

    let func = i8_ptr_type.fn_type(&[
      datatype_opaque.ptr_type(AddressSpace::default()).into()
    ], false);
    let func = module.add_function("typeof_inner", func, None);
    let basic_block = context.append_basic_block(func, "entry");
    builder.position_at_end(basic_block);

    // let ret_val = builder.build_alloca(context.i8_type(), "").unwrap();
    let struct_ptr = func.get_first_param().unwrap().into_pointer_value();
    let sym_ptr = builder
      .build_struct_gep(struct_ptr, 0, "")
      .unwrap();
    let sym_ptr = builder.build_load(sym_ptr, "").unwrap();
    let _ = builder.build_return(Some(&sym_ptr));
  }

  pub fn name(&self) -> &str { &self.name.name() }
  // pub fn supertype(&self) -> &str { &self.supertype.name() }
}
