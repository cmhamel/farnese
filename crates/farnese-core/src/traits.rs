use crate::codegen::*;
use farnese_parser::*;
use inkwell::values::{BasicValueEnum, PointerValue};

pub trait Compile<'ctx> {
    type Output;
    fn compile(&self, codegen: &mut CodeGen<'ctx>) -> Self::Output;
}

pub trait Inference {
    fn inference(&self);
}

pub trait Interpret {
    type Output;
    fn interpret(&self) -> Self::Output;
}

#[cfg(feature = "compiler")]
pub trait LLVMAlloca<'ctx> {
    fn emit_ir_alloca(&self, codegen: &CodeGen<'ctx>) -> PointerValue<'ctx>;
}

#[cfg(feature = "compiler")]
pub trait LLVMGlobal<'ctx> {
    fn emit_ir_global(&self);
}

#[cfg(feature = "compiler")]
pub trait LLVMPrintf<'ctx> {
    fn emit_ir_printf(&self);
}

#[cfg(feature = "compiler")]
pub trait LLVMStore<'ctx> {
    fn emit_ir_store(&self);
}

#[cfg(feature = "compiler")]
pub trait LLVMValue<'ctx> {
    fn emit_ir_value(&self, codegen: &CodeGen<'ctx>) -> BasicValueEnum<'ctx>;
}