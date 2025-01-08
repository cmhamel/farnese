use crate::core::Symbol;
use inkwell::module::Module;
use inkwell::AddressSpace;


#[derive(Clone, Debug)]
// pub struct DataType<'a> {
pub struct DataType {
  name: Symbol,
  supertype: Symbol,
  // is_abstract: bool,
  // is_mutable: bool,
  // is_primitive: bool,
  // struct_type: StructType<'a>
}

impl<'a, 'b> DataType {
  pub fn new(name: Symbol, supertype: Symbol, module: &'b Module<'a>) -> Self {
    let datatype_opaque = module.get_struct_type("DataType").unwrap();
    let context = module.get_context();
    let builder = context.create_builder();

    // add type name as a global in this module
    let datatype = module.add_global(datatype_opaque, None, name.name()); 
    println!("Supertype = {:?}", supertype);
    let supertype_glob = module.get_global(supertype.name()).unwrap();
    let _ = datatype.set_initializer(&datatype_opaque.const_named_struct(&[
      context.i8_type().ptr_type(AddressSpace::default()).const_null().into(),
      supertype_glob.as_pointer_value().into(),
      context.i64_type().ptr_type(AddressSpace::default()).const_null().into(),
    ]));

    // Self::init(module);
    Self {
      name: name,
      supertype: supertype
    }

    // temp.init(module);
    // temp
    // Self {
    //   name: name
    // }
  }

  pub fn init(&self, module: &'b Module<'a>) -> () {
    let context = module.get_context();
    let builder = context.create_builder();
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
    let datatype_opaque = module
      .get_struct_type("DataType")
      .unwrap();
    let fn_type = datatype_opaque.fn_type(&[
      i8_ptr_type.into()
    ], false);
    let function = module.add_function(format!("{}__new", self.name()).as_str(), fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);
    let sym = context.const_string(self.name().as_bytes(), true);
    let sym_glob = module.add_global(sym.get_type(), None, format!("__Sym__{}", self.name()).as_str());
    sym_glob.set_initializer(&sym);
    let val = builder.build_alloca(datatype_opaque, "__ptr").unwrap();
    let ptr = builder.build_struct_gep(val, 0, "__name").unwrap();

    let string_ptr = unsafe { builder.build_in_bounds_gep(
      sym_glob.as_pointer_value(),
      &[context.i32_type().const_zero(), context.i32_type().const_zero()],
      "string_ptr",
    ).unwrap() };
    let _ = builder.build_store(ptr, string_ptr);

    // let _ = builder.build_store(ptr, sym_glob.as_pointer_value());
    let val = builder.build_load(val, "__val").unwrap();
    let _ = builder.build_return(Some(&val));
  }

  pub fn name(&self) -> &str { &self.name.name() }
  // pub fn supertype(&self) -> &str { &self.supertype.name() }
}