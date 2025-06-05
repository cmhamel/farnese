use super::{
  DataType,
  // MethodHelper, 
  Primitive
};
use inkwell::AddressSpace;
use inkwell::module::Module;

#[derive(Clone, Debug)]
pub enum ValTypes {
  DataType(DataType),
  Primitive(Primitive)
}

#[derive(Clone, Debug)]
pub struct Val {
  datatype: DataType,
  value: ValTypes
}

impl<'a> Val {
    pub fn new(datatype: DataType, value: Primitive) -> Self {
      Self {
        datatype: datatype,
        value: ValTypes::Primitive(value.into())
      }
    }

    pub fn create_new_method(&self, module: &Module<'a>) {
      let context = module.get_context();
      let builder = context.create_builder();
      let datatype = module.get_struct_type("DataType").unwrap();
      let val_type = module.get_struct_type("Val").unwrap();
      let i64_ptr_type = context.i64_type().ptr_type(AddressSpace::default());

      let func = val_type.fn_type(&[
        i64_ptr_type.into(),
        datatype.into()
      ], false);
      let function = module.add_function("__new_Val", func, None);
      let entry = context.append_basic_block(function, "entry");
      builder.position_at_end(entry);

      // let val_ptr = function.get_nth_method_input(0);
      // let datatype_ptr = function.get_nth_method_input(1);
      // let ptr = builder.build_alloca(val_type, "");


      let _ = builder.build_return(None);

    }

    pub fn create_opaque_type(&self, module: &Module<'a>) {
      let context = module.get_context();
      let datatype = module.get_struct_type("DataType").unwrap();
      let i64_ptr_type = context.i64_type().ptr_type(AddressSpace::default());
      let opaque_type = context
        .opaque_struct_type("Val");
      opaque_type.set_body(
        &[
          i64_ptr_type.into(),
          datatype.ptr_type(AddressSpace::default()).into()
        ],
        false
      );
    }
}
