use crate::datatype::DataType;
use inkwell::values::BasicValueEnum;

#[derive(Clone, Debug, PartialEq)]
pub struct Value<'ctx> {
    pub value: BasicValueEnum<'ctx>,
    pub datatype: DataType
}

impl<'ctx> Value<'ctx> {
    pub fn new(value: BasicValueEnum<'ctx>, datatype: DataType) -> Self {
        Self {
            value: value,
            datatype: datatype
        }
    }
}
