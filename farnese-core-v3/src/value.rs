use crate::datatype::DataType;
use inkwell::values::{
    BasicMetadataValueEnum
};
use crate::modules::Module;
use crate::prelude::*;
use crate::primitives::Primitive;
use farnese_lexer::ast::Expr;

#[derive(Clone, Debug)]
pub enum PossibleVal<'a> {
    BasicMetadataValueEnum(BasicMetadataValueEnum<'a>),
    PointerValue(PointerValue<'a>),
    Primitive(Primitive)
}

// impl<'a> PossibleVal {
//     // pub fn get_datatype(&self, module: &Module<'a>) -> DataType {
//     //     match &self {
//     //         PossibleVal::Primitive(x) => panic!()
//     //     }
//     // }
// }
impl<'a> From<Expr> for PossibleVal<'a> {
    fn from(item: Expr) -> Self {
        match item {
            Expr::Float64(x) => PossibleVal::Primitive(x.try_into().unwrap()),
            Expr::Int(x) => PossibleVal::Primitive(x.try_into().unwrap()),
            _ => todo!("Expr = {:#?}", item)
        }
    }
}

impl<'a, 'b> Alloca<'a, 'b> for PossibleVal<'a> {
    fn alloca(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a> {
        match &self {
            PossibleVal::BasicMetadataValueEnum(_) => {
                panic!("You shouldn't need this")
            }
            PossibleVal::Primitive(x) => {
                x.alloca(builder, module)
            },
            _ => panic!()
        }
    }
}

impl<'a, 'b> FarneseType<'a, 'b> for PossibleVal<'a> {
    fn farnese_type(&self, module: &Module<'a>) -> DataType {
        match &self {
            PossibleVal::Primitive(x) => {
                x.farnese_type(module)
            },
            _ => panic!()
        }
    }
}

impl<'a, 'b> LLVMType<'a, 'b> for PossibleVal<'a> {
    fn llvm_type(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> BasicTypeEnum<'a> {
        match &self {
            PossibleVal::Primitive(x) => {
                x.llvm_type(builder, module)
            },
            _ => panic!()
        }
    }
}

impl<'a, 'b> Load<'a, 'b> for PossibleVal<'a> {
    fn load(&self, builder: &'b Builder<'a>, module: &Module<'a>, ptr: PointerValue<'a>) -> BasicValueEnum<'a> {
        match &self {
            PossibleVal::Primitive(x) => {
                x.load(builder, module, ptr).try_into().unwrap()
            },
            _ => panic!()
        }
    }
}

impl<'a, 'b> LLVMValue<'a, 'b> for PossibleVal<'a> {
    fn llvm_value(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> BasicValueEnum<'a> {
        match &self {
            PossibleVal::Primitive(x) => {
                x.llvm_value(builder, module)
            },
            _ => panic!()
        }
    }
}

impl<'a, 'b> Printf<'a, 'b> for PossibleVal<'a> {
    fn printf(&self, builder: &'b Builder<'a>, module: &Module<'a>, ptr: PointerValue<'a>) {
        match &self {
            PossibleVal::Primitive(x) => {
                x.printf(builder, module, ptr)
            },
            _ => panic!()
        }
    }
}

impl<'a, 'b> Store<'a, 'b> for PossibleVal<'a> {
    fn store(&self, builder: &'b Builder<'a>, module: &Module<'a>, ptr: PointerValue<'a>) {
        match &self {
            PossibleVal::Primitive(x) => {
                x.store(builder, module, ptr)
            },
            _ => panic!()
        }
    }
}

impl<'a> From<BasicMetadataValueEnum<'a>> for PossibleVal<'a> {
    fn from(item: BasicMetadataValueEnum<'a>) -> Self {
        Self::BasicMetadataValueEnum(item)
    }
}

impl<'a> From<Primitive> for PossibleVal<'a> {
    fn from(item: Primitive) -> Self {
        Self::Primitive(item)
    }
}

pub struct Value<'a> {
    datatype: DataType,
    value: PossibleVal<'a>
}

impl<'a, 'b> Value<'a> {
    // pub fn new(module: &Module<'a>, value: PossibleVal) -> Self {
    //     let datatype = value.get_datatype(module);
    pub fn new(datatype: DataType, value: PossibleVal<'a>) -> Self {
        Self {
            datatype: datatype,
            value: value
        }
    }

    pub fn get_datatype(&self, builder: &'b Builder<'a>, module: &Module<'a>, ptr: PointerValue<'a>) -> PointerValue<'a> {
        let ptr_type = module.get_context().ptr_type(AddressSpace::default());
        let datatype_type = self.value.llvm_type(builder, module);
        let gep_ptr = builder.build_struct_gep(
            self.llvm_type(builder, module), ptr, 1, ""
        ).unwrap();
        let val_read1 = builder
            .build_load(ptr_type, gep_ptr, "")
            .unwrap();
        let _ = builder
            .build_load(datatype_type, val_read1.into_pointer_value(), "")
            .unwrap();
        val_read1.try_into().unwrap()
    }

    pub fn get_value(&self, builder: &'b Builder<'a>, module: &Module<'a>, ptr: PointerValue<'a>) -> PointerValue<'a> {
        let ptr_type = module.get_context().ptr_type(AddressSpace::default());
        let val_type = self.value.llvm_type(builder, module);
        let gep_ptr = builder.build_struct_gep(
            self.llvm_type(builder, module), ptr, 0, ""
        ).unwrap();
        let val_read1 = builder
            .build_load(ptr_type, gep_ptr, "")
            .unwrap();
        let _ = builder
            .build_load(val_type, val_read1.into_pointer_value(), "")
            .unwrap();
        val_read1.try_into().unwrap()
    }
}

impl<'a, 'b> Alloca<'a, 'b> for Value<'a> {}

impl<'a, 'b> LLVMType<'a, 'b> for Value<'a> {
    fn llvm_type(&self, _builder: &'b Builder<'a>, module: &Module<'a>) -> BasicTypeEnum<'a> {
        module.get_struct_type("Value")
            .unwrap().try_into().unwrap()
    }
}

impl<'a, 'b> LLVMValue<'a, 'b> for Value<'a> {
    fn llvm_value(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> BasicValueEnum<'a> {
        let value = module.get_struct_type("Value").unwrap();
        let value_ptr = self.value.alloca(builder, module);
        let datatype_ptr = self.datatype.alloca(builder, module);
        let _ = self.value.store(
            builder, module, value_ptr
        );
        let _ = self.datatype.store(
            builder, module, datatype_ptr
        );
        value.const_named_struct(&[
            value_ptr.into(),
            datatype_ptr.into()
        ]).try_into().unwrap()
    }
}

impl<'a, 'b> Load<'a, 'b> for Value<'a> {
    // fn load(&self, builder: &'b Builder<'a>, module: &Module<'a>, ptr: PointerValue<'a>) -> BasicValueEnum<'a> {
    //     let val_type = self.llvm_type(builder, module);
    //     let gep_ptr = builder.build_struct_gep(val_type, ptr, 0, "").unwrap();

    //     let val_type = self.value.llvm_type(builder, module);
    //     println!("Val type = {:?}", val_type);
    //     builder.build_load(val_type, gep_ptr, "").unwrap()
    // }
}

impl<'a, 'b> OpaqueType<'a, 'b> for Value<'a> {
    fn set_opaque_type_init(_builder: &'b Builder<'a>, module: &Module<'a>) {
        let opaque_type = module.get_struct_type("Value")
            .expect("Failed to find opaque struct in set_opaque_type_init");
        let ptr_type = module.get_context().ptr_type(AddressSpace::default());
        opaque_type.set_body(
            &[
                ptr_type.into(), // value for quicker access
                ptr_type.into()  // datatype
            ],
            false
        );
    }
}

impl<'a, 'b> Store<'a, 'b> for Value<'a> {}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    use super::*;

    #[test]
    fn test_value() {
        let context = Context::create();
        let test = TestHelper::new(&context, "test_value");

        test.start();

        // let ptr_type = context.ptr_type(AddressSpace::default());
        let datatype = DataType::create_abstract_type("Any", "Any");
        let val: PossibleVal = Primitive::from(69).into();
        // let val_type = val.llvm_type(&test.builder, &test.module);
        let value = Value::new(datatype, val.clone());
        let ptr = value.alloca(&test.builder, &test.module);
        let _ = value.store(
            &test.builder, &test.module, ptr
        );
        
        let val_read = value.get_value(&test.builder, &test.module, ptr);
        // let datatype_ead = value.get_datatype(&test.builder, &test.module, ptr);
        let _ = val.printf(&test.builder, &test.module, val_read);
        
        test.end();

        let results = test.run();
        let test_val = results.parse::<i32>().unwrap();
        assert_eq!(test_val, 69);
    }
}