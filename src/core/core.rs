use super::datatype::DataType;
use super::symbol::Symbol;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::AddressSpace;

pub struct Core<'a, 'b> {
  builder: &'b Builder<'a>, 
  context: &'a Context,
  module: Module<'a>
}

impl<'a, 'b> Core<'a, 'b> {
  pub fn new(context: &'a Context, builder: &'b Builder<'a>) -> Self {
    let module = context.create_module("Core");

    // helpers
    let i8_type = context.i8_type();
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
      
    // general datatype
    let datatype_opaque = context.opaque_struct_type("DataType");
    datatype_opaque.set_body(
      &[
        context.i8_type().ptr_type(AddressSpace::default()).into(),
        datatype_opaque.ptr_type(AddressSpace::default()).into(),
        context.i64_type().ptr_type(AddressSpace::default()).into(),
      ],
      false,
    );

    // printf
    // let _ = module.add_function(
    //   "printf",
    //   i8_ptr_type.fn_type(&[
    //     i8_ptr_type.into()
    //   ], true),
    //   // None
    //   Some(Linkage::External)
    // );
    // printf_fn.set_linkage(Linkage::ExternalWeak);
    let printf_type = i8_type.fn_type(&[i8_ptr_type.into()], true);
    let printf_func = module.add_function("printf", printf_type, Some(Linkage::External));

    Self {
      builder: builder,
      context: context,
      module: module
    }
  }

  pub fn bootstrap(&mut self) -> () {

    // let datatype = self.module.add_global(datatype_opaque)
    let datatype_opaque = self.module.get_struct_type("DataType").unwrap();
    let sym = Symbol::new("DataType");
    let datatype = DataType::new(sym.clone(), sym, &self.module);
    datatype.init(&self.module);
    // let sym = Symbol::new("Any");
    // let anytype = DataType::new(sym.clone(), sym, &self.module);
    // anytype.init(&self.module);

    // adding main method here for now
    // TODO remove this
    // let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
    // let i32_type = self.context.i32_type();
    // let fn_type = i32_type.fn_type(&[], false);
    // let function = self.module.add_function("main", fn_type, None);
    // let basic_block = self.context.append_basic_block(function, "entry");
    // self.builder.position_at_end(basic_block);

    // let t = self.module.get_global("DataType").unwrap();
    // let s = self.context.const_string(b"Hello, world!", true);
    // let ptr = self.builder.build_alloca(s.get_type(), "__str").unwrap();
    // let _ = self.builder.build_store(ptr, s);
    // let ptr_i8 = self.builder.build_alloca(i8_ptr_type, "__str_i8_cast").unwrap();
    // // self.builder.build_store(ptr, s);
    // let gep_ptr = unsafe { self.builder.build_in_bounds_gep(ptr, &[
    //   self.context.i32_type().const_zero(),
    //   self.context.i32_type().const_zero(),
    // ], "__gep_ptr").unwrap() };
    // let _ = self.builder.build_call(
    //   self.module.get_function("printf").unwrap(),
    //   &[gep_ptr.into()],
    //   "__printf_call"
    // );

    // let a = self.module.get_global("DataType").unwrap();
    // let a_type = self.builder.build_alloca(datatype_opaque, "__any_type_ptr").unwrap();
    // // let fn_type = datatype_opaque.fn_type(&[i8_ptr_type.into()], false);
    // let gep_ptr = unsafe { self.builder.build_in_bounds_gep(
    //   self.module.get_global("__Sym__DataType").unwrap().as_pointer_value().into(), &[
    //   self.context.i32_type().const_zero(),
    //   self.context.i32_type().const_zero(),
    // ], "__gep_ptr").unwrap() };
    // let _ = self.builder.build_call(
    //   self.module.get_function("DataType__new").unwrap(),
    //   // &[self.module.get_global("__Sym__Any").unwrap().as_pointer_value().into()],
    //   &[gep_ptr.into()],
    //   "__any__new__call"
    // );

    // let int = i32_type.const_int(0, false);
    // let _ = self.builder.build_return(Some(&int));
  }

  pub fn module(&self) -> Module<'a> { self.module.clone() }
}