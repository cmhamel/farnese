use super::Symbol;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::StructType;
use inkwell::AddressSpace;

// main type that hold onto alot of things in core
// memory layout
// DataType = { i8* DataType* i64* }

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
  pub fn supertype(&self) -> &str { &self.supertype.name() }

  // main method to create the tyep for llvm ir
  // pub fn create_type_old<'a>(&self, context: &'a Context, module: Module<'a>, type_tag_type: StructType<'a>) -> Module<'a> {
  //   let type_name = context.const_string(self.name().as_bytes(), false);
  //   let type_name_global = module.add_global(type_name.get_type(), None, format!("__{}", self.name()).as_str());
  //   type_name_global.set_initializer(&type_name);

  //   let super_type_tag = module.get_global(self.supertype.name()).unwrap();
  //   let initializer = type_tag_type
  //     .const_named_struct(&[
  //       context.i8_type().ptr_type(AddressSpace::default()).const_null().into(),
  //       super_type_tag.as_pointer_value().into(), // Parent: points to itself
  //     ]);
  //   let type_tag = module.add_global(type_tag_type, None, self.name.name());
  //   type_tag.set_initializer(&initializer);

  //   module.clone()
  // }

  pub fn create_type<'a>(&self, context: &'a Context, module: Module<'a>, type_tag: StructType<'a>) -> Module<'a> {
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());

    // try to add the type
    match module.get_global(self.name()) {
      Some(_) => panic!("Type already exists"),
      _ => ()
    }
    // let type_tag = self.create_datatype_tag(context);
    let new_type = module.add_global(type_tag, None, self.name());
    
    let parent_type = match module.get_global(self.supertype()) {
      None => panic!("Supertype not found"),
      Some(x) => x
    };

    // let hash = context.i64_type().const_int(self.hash_id(), false);
    new_type.set_initializer(&type_tag.const_named_struct(&[
      i8_ptr_type.const_null().into(),
      parent_type.as_pointer_value().into(), // Parent: points to itself
      context.i64_type().ptr_type(AddressSpace::default()).const_null().into()
    ]));

    module.clone()
  }

}



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