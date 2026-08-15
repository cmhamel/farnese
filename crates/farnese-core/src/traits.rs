use crate::codegen::*;
use crate::program::InferenceHelper;
use inkwell::values::{BasicValueEnum, PointerValue};

pub trait Compile<'ctx> {
    type Output;
    fn compile(&self, codegen: &mut CodeGen<'ctx>) -> Self::Output;
}

pub trait Inference {
    type Output;
    fn inference(&self, helper: &mut InferenceHelper) -> Self::Output;
}

pub trait Interpret {
    type Output;
    fn interpret(&self) -> Self::Output;
}

pub trait LLVMAlloca<'ctx> {
    fn emit_ir_alloca(&self, codegen: &mut CodeGen<'ctx>, value: &BasicValueEnum<'ctx>) -> PointerValue<'ctx>;
}

pub trait LLVMGlobal<'ctx> {
    fn emit_ir_global(&self);
}

pub trait LLVMPrintf<'ctx> {
    fn emit_ir_printf(&self);
}

pub trait LLVMStore<'ctx> {
    fn emit_ir_store(&self);
}

pub trait LLVMValue<'ctx> {
    fn emit_ir_value(&self, codegen: &CodeGen<'ctx>) -> BasicValueEnum<'ctx>;
}