use crate::modules::Module;
use inkwell::{
    builder::Builder,
    types::BasicTypeEnum,
    values::{BasicValueEnum, PointerValue}
};

// basic LLVM stuff for convert types to llvm
// e.g. primitives and structs
pub trait LLVMType<'a, 'b> {
    fn llvm_type(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> BasicTypeEnum<'a>;
}

pub trait LLVMValue<'a, 'b> {
    fn llvm_value(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> BasicValueEnum<'a>;
}

// specific llvm ops
pub trait Alloca<'a, 'b>: LLVMType<'a, 'b> {
    fn alloca(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a> {
        let val_type = self.llvm_type(builder, module);
        builder.build_alloca(val_type, "").unwrap()
    }
}

pub trait Load<'a, 'b>: LLVMType<'a, 'b> {
    fn load(&self, builder: &'b Builder<'a>, module: &Module<'a>, ptr: PointerValue<'a>) -> BasicValueEnum<'a> {
        let val_type = self.llvm_type(builder, module);
        builder.build_load(val_type, ptr, "").unwrap()
    }
}

pub trait Printf<'a, 'b>: LLVMType<'a, 'b> {
    // fn fmt_str(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a>;
    fn fmt_str(
        &self,
        builder: &'b Builder<'a>, 
        _module: &Module<'a>
    ) -> PointerValue<'a> {
        builder
            .build_global_string_ptr("%s\0", "")
            .unwrap()
            .as_pointer_value()
    }

    fn printf(&self, builder: &'b Builder<'a>, module: &Module<'a>, ptr: PointerValue<'a>);
}

pub trait Store<'a, 'b>: LLVMValue<'a, 'b> {
    fn store(
        &self, 
        builder: &'b Builder<'a>, 
        module: &Module<'a>,
        ptr: PointerValue<'a>
    ) {
        let val = self.llvm_value(builder, module);
        builder.build_store(ptr, val).unwrap();
    }
}
