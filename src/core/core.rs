use super::datatype::DataType;
use super::symbol::Symbol;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

pub struct Core<'a> {
  // builder: &'b Builder<'a>, 
  // context: &'a Context,
  module: Module<'a>
}

impl<'a> Core<'a> {
  pub fn new(context: &'a Context) -> Self {
    let module = context.create_module("Core");

    // helpers
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(AddressSpace::default());
    let i64_type = context.i64_type();
    let i64_ptr_type = i64_type.ptr_type(AddressSpace::default());  

    // general datatype
    let datatype_opaque = context.opaque_struct_type("DataType");
    datatype_opaque.set_body(
      &[
        i8_ptr_type.into(),
        datatype_opaque.ptr_type(AddressSpace::default()).into(),
        i64_ptr_type.into(),
      ],
      false,
    );

    // let valuetype_opaque = context.opaque_struct_type("Value");
    // valuetype_opaque.set_body(
    //   &[
    //     i64_ptr_type.into(),
    //     datatype_opaque.ptr_type(AddressSpace::default()).into()
    //   ],
    //   false,
    // );
    // let value = module.add_global(valuetype_opaque, None, "Value");

    let printf_type = i8_type.fn_type(&[i8_ptr_type.into()], true);
    let _ = module.add_function("printf", printf_type, None);
    let printf_type = i8_type.fn_type(&[
      i8_ptr_type.into(),
      // i64_type.into(),
      ], true
    );
    let _ = module.add_function("printf", printf_type, None);


    Self {
      // builder: builder,
      // context: context,
      module: module
    }
  }

  pub fn bootstrap(&mut self) -> () {
    // setup datatype
    let sym = Symbol::new("DataType");
    let datatype = DataType::new(sym.clone(), sym, &self.module);
    datatype.create_supertype(&self.module);
    datatype.create_typeof(&self.module);
    // datatype.create_new_type(&self.module);
    // datatype.create_new_val(&self.module);

    // // setup value
    // let sym = Symbol::new("Value");
    // let value = Value::new(sym, datatype.name, &self.module);

  }

  pub fn module(&self) -> Module<'a> { self.module.clone() }
}