use super::{
    DataType, 
    LLVMAlloca, 
    LLVMPrintf, 
    LLVMValue, 
    Module
};
use farnese_lexer::lexer::ast;
use inkwell::builder::Builder;
use inkwell::values::{
    BasicMetadataValueEnum,
    BasicValueEnum, 
    CallSiteValue,
    GlobalValue, 
    PointerValue
};

// TODO refactor whole file to use core::Module

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Char(char),
    // Float16(f16), // not in rust
    // Float32(f32),
    Float64(f64),
    // Int16(i16),
    // Int32(i32),
    Int64(i64),
    // String(String),
    // UInt16(u16),
    // UInt32(u32),
    // UInt64(u64)
}

impl Primitive {
    pub fn get_datatype(&self) -> DataType {
        let name = match &self {
            Primitive::Float64(_x) => "Float64",
            // Primitive::Int32(_x) => "Int32",
            Primitive::Int64(_x) => "Int64",
            _ => todo!()
        };
        let supertype = match &self {
            Primitive::Float64(_) => "AbstractFloat",
            // Primitive::Int32(_x) => "Signed",
            Primitive::Int64(_x) => "Signed",
            _ => todo!()
        };
        let bits = match &self {
            Primitive::Float64(_x) => 64,
            // Primitive::Int32(_x) => 32,
            Primitive::Int64(_x) => 64,
            _ => todo!()
        };
        DataType::new_primitive_type(name, supertype, bits)
    }
}

impl From<ast::Primitive> for Primitive {
    fn from(value: ast::Primitive) -> Self {
        match value {
            ast::Primitive::Float64(x) => Primitive::Float64(x),
            ast::Primitive::Int64(x) => Primitive::Int64(x),
            _ => todo!()
        }
    }
}

impl From<char> for Primitive {
    fn from(value: char) -> Self {
        Primitive::Char(value)
    }
}

impl From<f64> for Primitive {
    fn from(value: f64) -> Self {
        Primitive::Float64(value)
    }
}

// impl From<i32> for Primitive {
//     fn from(value: i32) -> Self {
//         Primitive::Int32(value)
//     }
// }

impl From<i64> for Primitive {
    fn from(value: i64) -> Self {
        Primitive::Int64(value)
    }
}

impl<'a, 'b> LLVMAlloca<'a, 'b> for Primitive {
    fn emit_ir_alloca(
        &self,
        builder: &'b Builder<'a>,
        module: &Module<'a>
    ) -> PointerValue<'a> {
        let context = module.get_context();
        let val = self.emit_ir_value(module);
        let ptr = builder.build_alloca(val.get_type(), "").unwrap();
        let _ = builder.build_store(ptr, val);
        ptr
    }
}

// re-write to not use globals, just allocate the fmt
// str whenever we use it.
impl<'a, 'b> LLVMPrintf<'a, 'b> for Primitive {
    fn emit_ir_printf(
        &self, 
        builder: &'b Builder<'a>, 
        module: &Module<'a>
    ) -> CallSiteValue<'a> {
        let context = module.get_context();
        let format_string = match &self {
            Primitive::Char(_) => {
                context.const_string(b"%c\n\0", false)
            },
            // Primitive::Float32(_) |
            Primitive::Float64(_) => {
                context.const_string(b"%f\n\0", false)
            },
            // Primitive::Int16(_) |
            // Primitive::Int32(_) |
            Primitive::Int64(_) => {
                context.const_string(b"%d\n\0", false)
            },
            _ => todo!()
        };
        let ptr = builder.build_alloca(format_string.get_type(), "").unwrap();
        let _ = builder.build_store(ptr, format_string);
        let gep_ptr = unsafe { builder.build_in_bounds_gep(ptr, &[
            context.i32_type().const_zero(),
            context.i32_type().const_zero(),
          ], "").unwrap() };

        let func_args: Vec<BasicMetadataValueEnum<'a>> = vec![
            gep_ptr.into(),
            self.emit_ir_alloca(builder, module).into()
        ];
        let func_result = builder.build_call(
            module.get_function("printf"),
            &func_args,
            ""
        ).unwrap();
        func_result
    }
    // fn emit_ir_printf(&self, builder: &'b Builder<'a>, module: &Module<'a>, ptr: &'b PointerValue<'a>) {
    //     let context = module.get_context();
    //     let zero = context.i32_type().const_zero();
    //     let (loaded_val, fmt_global): (BasicValueEnum<'a>, GlobalValue<'a>) = match &self {
    //         Primitive::Char(_x) => {
    //             (
    //                 builder.build_load(*ptr, "").unwrap().into_int_value().try_into().unwrap(),
    //                 module.get_global("__fmt_char")
    //             )
    //         },
    //         Primitive::Int32(_x) => {
    //             (
    //                 builder.build_load(*ptr, "").unwrap().into_int_value().try_into().unwrap(), 
    //                 module.get_global("__fmt_int")
    //             )
    //         },
    //         Primitive::Int64(_x) => {
    //             (
    //                 builder.build_load(*ptr, "").unwrap().into_int_value().try_into().unwrap(), 
    //                 module.get_global("__fmt_int")
    //             )
    //         },
    //         Primitive::Float64(_x) => {
    //             (
    //                 builder.build_load(*ptr, "").unwrap().into_float_value().try_into().unwrap(), 
    //                 module.get_global("__fmt_float")
    //             )
    //         },
    //         _ => todo!()
    //     };
    //     let fmt_ptr = unsafe {
    //         builder.build_gep(fmt_global.as_pointer_value(), &[zero, zero], "fmt_ptr")
    //     }.unwrap();
    //     let printf = module.get_function("printf");
    //     let _ = builder.build_call(printf, &[fmt_ptr.into(), loaded_val.into()], "call_printf");
    // }
}

impl<'a, 'b> LLVMValue<'a> for Primitive {
    fn emit_ir_value(&self, module: &Module<'a>) -> BasicValueEnum<'a> {
        let context = module.get_context();
        match &self {
            Primitive::Char(x) => {
                context.i8_type().const_int((*x).try_into().unwrap(), false)
                    .try_into().unwrap()
            }
            Primitive::Float64(x) => {
                context.f64_type().const_float((*x).try_into().unwrap())
                    .try_into().unwrap()
            },
            // Primitive::Int32(x) => {
            //     context.i32_type().const_int((*x).try_into().unwrap(), false)
            //         .try_into().unwrap()
            // },
            Primitive::Int64(x) => {
                context.i64_type().const_int((*x).try_into().unwrap(), false)
                    .try_into().unwrap()
            },
            _ => todo!("Unsupported primitive LLVMValue {:?}", self)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::Core;
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_float64() {
        let context = Context::create();
        let builder = context.create_builder();
        let mut core = Core::new(&context);
        let module = core.bootstrap();
        core.main_func_begin(&builder, &module);

        let prim_val: Primitive = 69.0.into();
        // let prim_ptr = prim_val.emit_ir_alloca(&builder, &module);
        let _ = prim_val.emit_ir_printf(&builder, &module);

        core.main_func_end(&builder, &module);

        let _ = module.print_to_file("primitive_float64_test.ll");
    }

    #[test]
    fn test_int32() {
        let context = Context::create();
        let builder = context.create_builder();
        let mut core = Core::new(&context);
        let module = core.bootstrap();
        core.main_func_begin(&builder, &module);
        let prim_val: Primitive = (32 as i32).into();
        // let prim_ptr = prim_val.emit_ir_alloca(&builder, &module);
        let _ = prim_val.emit_ir_printf(&builder, &module);

        core.main_func_end(&builder, &module);

        let _ = module.print_to_file("primitive_int64_test.ll");
    }

    #[test]
    fn test_int64() {
        let context = Context::create();
        let builder = context.create_builder();
        let mut core = Core::new(&context);
        let module = core.bootstrap();
        core.main_func_begin(&builder, &module);
        let prim_val: Primitive = (32 as i64).into();
        // let prim_ptr = prim_val.emit_ir_alloca(&builder, &module);
        let _ = prim_val.emit_ir_printf(&builder, &module);

        core.main_func_end(&builder, &module);

        let _ = module.print_to_file("primitive_int64_test.ll");
    }

    #[test]
    fn test_primitive() {
        let context = Context::create();
        let builder = context.create_builder();
        let mut core = Core::new(&context);
        let module = core.bootstrap();
        core.main_func_begin(&builder, &module);

        // char
        let prim_val: Primitive = 'c'.into();
        // let prim_ptr = prim_val.emit_ir_alloca(&builder, &module);
        let _ = prim_val.emit_ir_printf(&builder, &module);

        // double
        let prim_val: Primitive = 69.0.into();
        // let prim_ptr = prim_val.emit_ir_alloca(&builder, &module);
        let _ = prim_val.emit_ir_printf(&builder, &module);

        // int
        let prim_val: Primitive = 32.into();
        // let prim_ptr = prim_val.emit_ir_alloca(&builder, &module);
        let _ = prim_val.emit_ir_printf(&builder, &module);

        core.main_func_end(&builder, &module);

        let _ = module.print_to_file("primitive_test.ll");
    }
}
