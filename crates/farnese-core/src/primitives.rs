use crate::codegen::*;
use crate::datatype::DataType;
use crate::program::{InferenceHelper, TypedExpr};
use crate::traits::*;
use farnese_parser::*;
use inkwell::values::BasicValueEnum;

impl<'ctx> Compile<'ctx> for Primitive {
    type Output = BasicValueEnum<'ctx>;
    fn compile(&self, codegen: &mut CodeGen<'ctx>) -> Self::Output {
        self.emit_ir_value(codegen)
    }
}

impl Inference for Primitive {
    type Output = TypedExpr;
    fn inference(&self, _: &mut InferenceHelper) -> Self::Output {
        let datatype = match self {
            Primitive::Float64(_x) => DataType::new_primitive("Float64", 64),
            Primitive::Int64(_x) => DataType::new_primitive("Int64", 64),
            _ => todo!()
        };
        TypedExpr::Primitive { val: self.clone(), datatype: datatype }
    }
}

#[cfg(feature = "interpreter")]
impl Interpret for Primitive {
    type Output = ();
    fn interpret(&self) -> Self::Output {
        match self {
            Primitive::Float64(x) => x,
            _ => todo!("Unsupported primitive in interpret")
        }
    }
}

impl<'ctx> LLVMValue<'ctx> for f64 {
    fn emit_ir_value(&self, codegen: &CodeGen<'ctx>) -> BasicValueEnum<'ctx> {
        codegen.context.f64_type()
            .const_float((*self).try_into().unwrap())
            .try_into()
            .unwrap()
    }
}

impl<'ctx> LLVMValue<'ctx> for i64 {
    fn emit_ir_value(&self, codegen: &CodeGen<'ctx>) -> BasicValueEnum<'ctx> {
        codegen.context.i64_type()
            .const_int((*self).try_into().unwrap(), true)
            .try_into()
            .unwrap()
    }
}

impl<'ctx> LLVMValue<'ctx> for Primitive {
    fn emit_ir_value(&self, codegen: &CodeGen<'ctx>) -> BasicValueEnum<'ctx> {
        match self {
            Primitive::Float64(x) => x.emit_ir_value(codegen),
            Primitive::Int64(x) => x.emit_ir_value(codegen),
            _ => todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::TestHelper;
    use super::*;
    use farnese_derive::*;

    #[farnese_component_test]
    fn test_f64() {
        let val = Primitive::Float64(69.0);
        // val.emit_ir_alloca(codegen);
        codegen.module.print_to_stderr();
    }

    #[farnese_component_test]
    fn test_f64_from_expr() {
        let val = parse_program("69.0");
        println!("val = {:?}", val);
    }
}
