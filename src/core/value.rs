use super::DataType;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::StructType;
use inkwell::AddressSpace;

// sets up a type that is { i8* Type* } where 
// Type = { i8* Type* }
pub fn create_value_type<'a>(context: &'a Context, type_tag: StructType<'a>) -> StructType<'a> {
  // let value_sym = Symbol::new("Value"); // eventually w'ell use this
  let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
  let value_type = context.opaque_struct_type("Value");
  value_type.set_body(&[
    i8_ptr_type.into(), // value
    type_tag.ptr_type(AddressSpace::default()).into()
  ], false);
  value_type
}

pub struct Value<T> {
  value: T,
  data_type: DataType
}

impl<T> Value<T> {
  pub fn new(value: T, data_type: DataType) -> Self {
    Self {
      value: value,
      data_type: data_type
    }
  }

  pub fn create_constructor<'a>(
    &self, 
    context: &'a Context, module: Module<'a>, 
    type_tag: StructType<'a>
  ) -> Module<'a> {
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
    let any_type = module.get_global("Any").unwrap();
    // let value_type = module.get_global("Value").unwrap();
    // value_type.set_initializer(&type_tag.const_named_struct(&[
    //   i8_ptr_type.const_null().into(),
    //   any_type.as_pointer_value().into(),
    // ]));
    // let value_type = context.struct_type(&[
    //   i8_ptr_type.into(), // value
    //   type_tag.ptr_type(AddressSpace::default()).into()
    // ], false);
    module.clone()
  }

  pub fn create_type<'a>(&self, context: &'a Context, module: Module<'a>, data_type: StructType<'a>) -> Module<'a> {
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
    let builder = context.create_builder();
    // let value_type = self.create_datatype_tag(context, module.clone(), type_tag);
    // let any_type = module.get_global("Any").unwrap();
    // todo need to check if type exists...

    // let value = value_type.const_named_struct(&[
    //   i8_ptr_type.const_null().into(),
    //   // type_tag.as_pointer_value().into()
    //   any_type.as_pointer_value().into()
    // ]);
    // any_type.set_initializer(&value);

    let value_type = context.struct_type(&[
      i8_ptr_type.into(), // value
      data_type.ptr_type(AddressSpace::default()).into()
    ], false);
    let value_ptr_type = value_type.ptr_type(AddressSpace::default());
    let fn_type = value_ptr_type.fn_type(&[
      i8_ptr_type.into(), 
      data_type.ptr_type(AddressSpace::default()).into()
    ], false);
    let function = module.add_function("__Value__constructor", fn_type, None);
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);
    // Allocate space for the Value struct
    let value_ptr = builder.build_alloca(value_type, "__value_ptr").unwrap();

    // Store the i8* data
    let data_param = function.get_nth_param(0).unwrap().into_pointer_value();
    let data_field_ptr = unsafe {
        builder.build_struct_gep(value_ptr, 0, "__data_ptr").unwrap()
    };
    builder.build_store(data_field_ptr, data_param);

    // Store the %DataType* pointer
    let type_param = function.get_nth_param(1).unwrap().into_pointer_value();
    let type_field_ptr = unsafe {
        builder.build_struct_gep(value_ptr, 1, "__type_ptr_ptr").unwrap()
    };
    builder.build_store(type_field_ptr, type_param);

    // Return the constructed Value pointer
    builder.build_return(Some(&value_ptr));
    module.clone()
  }

  pub fn create_value<'a, 'b>(&self, context: &'a Context, builder: &'b Builder, module: Module<'a>, data_type: StructType<'a>) -> Module<'a> {
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
    let i32_type = context.i32_type();
    // let builder = context.create_builder();
    let value_struct = context.struct_type(
      &[
        i8_ptr_type.into(),
        data_type.ptr_type(AddressSpace::default()).into(),
      ],
      false,
    );
    // Allocate space for a `Value` instance
    let value_alloca = builder.build_alloca(value_struct, "value").unwrap();

    // Initialize the `i8*` field with a pointer to an integer
    let int_value = i32_type.const_int(69, false); // todo
    let int_ptr = builder.build_alloca(i32_type, "int_ptr").unwrap();
    builder.build_store(int_ptr, int_value);
    let int_i8_ptr = builder.build_bit_cast(int_ptr, i8_ptr_type, "int_i8_ptr");

    // Set the `i8*` field of `Value`
    let value_data_ptr = unsafe {
        builder.build_struct_gep(value_alloca, 0, "value_data_ptr").unwrap()
    };
    builder.build_store(value_data_ptr, int_i8_ptr.unwrap());

    // Set the `%DataType*` field to null
    let value_type_ptr = unsafe {
        builder.build_struct_gep(value_alloca, 1, "value_type_ptr").unwrap()
    };
    builder.build_store(value_type_ptr, data_type.ptr_type(AddressSpace::default()).const_null());

    // todo remove me
    // let _ = builder.build_return(Some(&i32_type.const_int(0, false)));

    module.clone()
  }

  pub fn create_datatype_tag<'a>(&self, context: &'a Context, module: Module<'a>, type_tag: StructType<'a>) -> StructType<'a> {
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
    let value_type = context.opaque_struct_type("Value");
    // let value_type = context.opaque_struct_type("Value");
    value_type.set_body(&[
      i8_ptr_type.into(), // value
      type_tag.ptr_type(AddressSpace::default()).into()
    ], false);
    // let value_type = context.struct_type(&[
    //   i8_ptr_type.into(), // value
    //   type_tag.ptr_type(AddressSpace::default()).into()
    // ], false);
    // let value_type_global = module.add_global(value_type, None, "Value");
    // module.clone()
    value_type
  }
}
