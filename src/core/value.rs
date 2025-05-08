use super::symbol::Symbol;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::AddressSpace;

pub enum ValueTypes {
  Float(f64),
  Int(i64)
}

impl From<i64> for ValueTypes {
  fn from(value: i64) -> Self {
    ValueTypes::Int(value)
  }
}

impl From<f64> for ValueTypes {
  fn from(value: f64) -> Self {
    ValueTypes::Float(value)
  }
}

#[derive(Clone, Debug)]
pub struct Value<'a> {
  val: PointerValue<'a>
}

impl<'a, 'b> Value<'a> {
  pub fn new(
    value: ValueTypes, type_name: &Symbol, 
    module: &'b Module<'a>, builder: &'b Builder<'a>
  ) -> Self {
    // setup
    let context = module.get_context();
    // let builder = context.create_builder();

    // helpers
    let i64_ptr_type = context.i64_type().ptr_type(AddressSpace::default());
    let datatype_opaque = module.get_struct_type("DataType").unwrap();
    let val_type = context.struct_type(&[
      i64_ptr_type.into(), 
      datatype_opaque.ptr_type(AddressSpace::default()).into()
    ], false);

    // allocate Val memory
    let val = builder.build_alloca(val_type, "").unwrap();

    // allocate memory for val
    let val_alloca = match value {
      // ValueTypes::Float(x) => println!("hur"),
      ValueTypes::Float(x) => {
        let temp = context.f64_type().const_float(x.try_into().unwrap());
        let ptr = builder.build_alloca(context.f64_type(), "").unwrap();
        let _ = builder.build_store(ptr, temp);
        ptr
      },
      ValueTypes::Int(x) => {
        let temp = context.i64_type().const_int(x.try_into().unwrap(), true);
        let ptr = builder.build_alloca(context.i64_type(), "").unwrap();
        let _ = builder.build_store(ptr, temp);
        ptr
      }
    };

    let val_ptr = builder.build_struct_gep(val, 0, "").unwrap();
    let _ = builder.build_store(val_ptr, val_alloca);

    let type_ptr = builder.build_struct_gep(val, 1, "").unwrap();
    let _ = builder.build_store(type_ptr, module.get_global(type_name.name()).unwrap());

    // let val = module.add_global(value_opaque, None, format!("Val__{}", type_name.name()).as_str());
    // let _ = val.set_initializer(&value_opaque.const_named_struct(&[
    //   context.i64_type().ptr_type(AddressSpace::default()).const_null().into(),
    //   // datatype.as_pointer_value().into()
    //   module.get_global(type_name.name()).unwrap().as_pointer_value().into()
    // ]));

    Self {
      val: val
    }
  }

  pub fn get_ptr(&self) -> &PointerValue<'a> {
    &self.val
  }

  // below methods only need to be called once
  pub fn typeof_func(&self, module: &'b Module<'a>) -> BasicValueEnum<'a> {
    // setup
    let context = module.get_context();
    let builder = context.create_builder();

    // helpers
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
    let i64_ptr_type = context.i64_type().ptr_type(AddressSpace::default());
    let datatype_opaque = module.get_struct_type("DataType").unwrap();
    let val_type = context.struct_type(&[
      i64_ptr_type.into(), 
      datatype_opaque.ptr_type(AddressSpace::default()).into()
    ], false);

    let func = i8_ptr_type.fn_type(&[
      val_type.ptr_type(AddressSpace::default()).into()
    ], false);
    let func = module.add_function("typeof", func, None);
    let basic_block = context.append_basic_block(func, "entry");
    builder.position_at_end(basic_block);

    let struct_ptr = func.get_first_param().unwrap().into_pointer_value();
    let type_ptr = builder
      .build_struct_gep(struct_ptr, 1, "")
      .unwrap();
    let type_ptr = builder.build_load(type_ptr, "").unwrap();
    let t = builder.build_call(
      module.get_function("typeof_inner").unwrap(),
      &[type_ptr.into()],
      ""
    ).unwrap().try_as_basic_value().left().unwrap();
    println!("T = {:?}", t);
    // let int = context.i32_type().const_int(0, false);
    let _ = builder.build_return(Some(&t));

    t
  }

  pub fn value_func(&self, module: &'b Module<'a>) -> BasicValueEnum<'a> {
    // setup
    let context = module.get_context();
    let builder = context.create_builder();

    // helpers
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
    let i64_ptr_type = context.i64_type().ptr_type(AddressSpace::default());
    let datatype_opaque = module.get_struct_type("DataType").unwrap();
    let val_type = context.struct_type(&[
      i64_ptr_type.into(), 
      datatype_opaque.ptr_type(AddressSpace::default()).into()
    ], false);

    let func = i64_ptr_type.fn_type(&[
      val_type.ptr_type(AddressSpace::default()).into()
    ], false);
    let func = module.add_function("value", func, None);
    let basic_block = context.append_basic_block(func, "entry");
    builder.position_at_end(basic_block);

    let struct_ptr = func.get_first_param().unwrap().into_pointer_value();
    let val_ptr = builder
      .build_struct_gep(struct_ptr, 0, "")
      .unwrap();
    let val_ptr = builder.build_load(val_ptr, "").unwrap();
    
    // val_ptr.into()
    // let type_ptr = builder.build_load(type_ptr, "").unwrap();
    // let t = builder.build_call(
    //   module.get_function("typeof_inner").unwrap(),
    //   &[type_ptr.into()],
    //   ""
    // ).unwrap().try_as_basic_value().left().unwrap();
    // println!("T = {:?}", t);
    // // let int = context.i32_type().const_int(0, false);
    let _ = builder.build_return(Some(&val_ptr));

    val_ptr.into()
  }
}
