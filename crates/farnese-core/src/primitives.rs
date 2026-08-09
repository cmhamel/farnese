use crate::codegen::*;
use crate::traits::*;
use farnese_parser::*;
use inkwell::values::{BasicValueEnum, PointerValue};

#[cfg(feature = "compiler")]
impl<'ctx> Compile<'ctx> for Primitive {
    type Output = BasicValueEnum<'ctx>;
    fn compile(&self, codegen: &mut CodeGen<'ctx>) -> Self::Output {
        self.emit_ir_value(codegen)
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

// impl<'ctx> LLVMAlloca<'ctx> for Primitive {
//     fn emit_ir_alloca(&self, codegen: &'ctx CodeGen) -> PointerValue<'ctx> {
//         let val = self.emit_ir_value(codegen);
//         let ptr = codegen.builder.build_alloca(val.get_type(), "").unwrap();
//         let _ = codegen.builder.build_store(ptr, val);
//         ptr
//     }
// }

impl<'ctx> LLVMValue<'ctx> for f64 {
    fn emit_ir_value(&self, codegen: &CodeGen<'ctx>) -> BasicValueEnum<'ctx> {
        codegen.context.f64_type()
            .const_float((*self).try_into().unwrap())
            .try_into()
            .unwrap()
    }
}

impl<'ctx> LLVMValue<'ctx> for Primitive {
    fn emit_ir_value(&self, codegen: &CodeGen<'ctx>) -> BasicValueEnum<'ctx> {
        match self {
            Primitive::Float64(x) => x.emit_ir_value(codegen),
            _ => todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::TestHelper;
    use super::*;
    use farnese_derive::*;
    use farnese_parser::*;

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
