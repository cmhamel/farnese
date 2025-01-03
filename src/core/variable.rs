use inkwell::types::BasicTypeEnum;
use inkwell::values::{AnyValueEnum, PointerValue};

// move to base
#[derive(Clone, Debug)]
pub struct Variable<'a> {
  // val_type: AnyTypeEnum<'a>,
  val_type: BasicTypeEnum<'a>,
  ptr: PointerValue<'a>,
  val: AnyValueEnum<'a>
}

impl<'a> Variable<'a> {
  pub fn new(val_type: BasicTypeEnum<'a>, ptr: PointerValue<'a>, val: AnyValueEnum<'a>) -> Self {
    Self { val_type: val_type, ptr: ptr, val: val }
  }
  // pub fn new(val_type: AnyTypeEnum<'a>, ptr: PointerValue<'a>) -> Self {
  //   Self { val_type: val_type, ptr: ptr }
  // }

  pub fn get_pointer(&self) -> &PointerValue<'a> {
    &self.ptr
  }

  pub fn get_type(&self) -> &BasicTypeEnum<'a> {
    &self.val_type
  }

  pub fn get_value(&self) -> &AnyValueEnum<'a> {
    &self.val
  }

  pub fn set_value(&mut self, new_val: AnyValueEnum<'a>) -> () {
    self.val = new_val;
  }
}
