use super::{
    DataType, 
    LLVMAlloca, 
    LLVMPrintf, 
    LLVMValue, 
    Module
};
use farnese_lexer::ast;
use inkwell::builder::Builder;
use inkwell::values::{
    BasicMetadataValueEnum,
    BasicValueEnum, 
    CallSiteValue,
    PointerValue
};

// TODO refactor whole file to use core::Module

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Char(char),
    // Float16(f16), // not in rust
    Float32(f32),
    Float64(f64),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    String(String),
    // UInt16(u16),
    // UInt32(u32),
    // UInt64(u64)
}

impl Primitive {
    pub fn get_datatype(&self) -> DataType {
        let name = match &self {
            Primitive::Float32(_) => "Float32",
            Primitive::Float64(_) => "Float64",
            Primitive::Int16(_) => "Int16",
            Primitive::Int32(_) => "Int32",
            Primitive::Int64(_) => "Int64",
            Primitive::String(_) => "String",
            _ => todo!()
        };
        let supertype = match &self {
            Primitive::Float32(_) => "AbstractFloat",
            Primitive::Float64(_) => "AbstractFloat",
            Primitive::Int16(_) => "Signed",
            Primitive::Int32(_) => "Signed",
            Primitive::Int64(_) => "Signed",
            Primitive::String(_) => "AbstractString",
            _ => todo!()
        };
        let bits = match &self {
            Primitive::Float32(_) => 32,
            Primitive::Float64(_) => 64,
            Primitive::Int16(_) => 16,
            Primitive::Int32(_) => 32,
            Primitive::Int64(_) => 64,
            Primitive::String(_) => 8, // do we want this to be primitive?
            _ => todo!()
        };
        DataType::new_primitive_type(name, supertype, bits)
    }
}

impl From<ast::Primitive> for Primitive {
    fn from(value: ast::Primitive) -> Self {
        match value {
            ast::Primitive::Float32(x) => Primitive::Float32(x),
            ast::Primitive::Float64(x) => Primitive::Float64(x),
            ast::Primitive::Int16(x) => Primitive::Int16(x),
            ast::Primitive::Int32(x) => Primitive::Int32(x),
            ast::Primitive::Int64(x) => Primitive::Int64(x),
            ast::Primitive::String(x) => Primitive::String(x),
            _ => todo!()
        }
    }
}

impl From<char> for Primitive {
    fn from(value: char) -> Self {
        Primitive::Char(value)
    }
}

impl From<f32> for Primitive {
    fn from(value: f32) -> Self {
        Primitive::Float32(value)
    }
}

impl From<f64> for Primitive {
    fn from(value: f64) -> Self {
        Primitive::Float64(value)
    }
}

impl From<i16> for Primitive {
    fn from(value: i16) -> Self {
        Primitive::Int16(value)
    }
}

impl From<i32> for Primitive {
    fn from(value: i32) -> Self {
        Primitive::Int32(value)
    }
}

impl From<i64> for Primitive {
    fn from(value: i64) -> Self {
        Primitive::Int64(value)
    }
}

impl From<String> for Primitive {
    fn from(value: String) -> Self {
        Primitive::String(value)
    }
}

impl<'a, 'b> LLVMAlloca<'a, 'b> for Primitive {
    fn emit_ir_alloca(
        &self,
        builder: &'b Builder<'a>,
        module: &Module<'a>
    ) -> PointerValue<'a> {
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
                context.const_string(b"%c\0", false)
            },
            Primitive::Float32(_) |
            Primitive::Float64(_) => {
                context.const_string(b"%.8f\0", false)
            },
            Primitive::Int16(_) |
            Primitive::Int32(_) |
            Primitive::Int64(_) => {
                context.const_string(b"%lld\0", false)
            },
            Primitive::String(_) => {
                context.const_string(b"%s\0", false)
            }
        };
        let ptr = builder.build_alloca(format_string.get_type(), "").unwrap();
        let _ = builder.build_store(ptr, format_string);
        let gep_ptr = unsafe { builder.build_in_bounds_gep(ptr, &[
            context.i32_type().const_zero(),
            context.i32_type().const_zero(),
          ], "").unwrap() 
        };

        let val = self.emit_ir_value(module);
        // let val = match self {
        //     Primitive::String(_) => {
        //         self.emit_ir_alloca(builder, module).into()
        //     },
        //     _ => self.emit_ir_value(module)
        // };

        let val: BasicValueEnum = match self {
            Primitive::Int16(_) => {
                match val {
                    BasicValueEnum::IntValue(y) => {
                        builder.build_int_z_extend(y, context.i32_type(), "")
                            .unwrap()
                            .try_into()
                            .unwrap()
                    },
                    _ => panic!()
                }
            },
            Primitive::Float32(_) => {
                match val {
                    BasicValueEnum::FloatValue(y) => {
                        builder.build_float_ext(y, context.f64_type(), "")
                            .unwrap()
                            .try_into()
                            .unwrap()
                    },
                    _ => panic!()
                }
            },
            _ => val
        };

        let func_args: Vec<BasicMetadataValueEnum<'a>> = vec![
            gep_ptr.into(),
            val.into()
        ];
        let func_result = builder.build_call(
            module.get_function("printf"),
            &func_args,
            ""
        ).unwrap();
        func_result
    }
}

impl<'a, 'b> LLVMValue<'a> for Primitive {
    fn emit_ir_value(&self, module: &Module<'a>) -> BasicValueEnum<'a> {
        let context = module.get_context();
        match &self {
            Primitive::Char(x) => {
                context.i8_type().const_int((*x).try_into().unwrap(), false)
                    .try_into().unwrap()
            },
            Primitive::Float32(x) => {
                context.f32_type().const_float((*x).try_into().unwrap())
                    .try_into().unwrap()
            },
            Primitive::Float64(x) => {
                context.f64_type().const_float((*x).try_into().unwrap())
                    .try_into().unwrap()
            },
            Primitive::Int16(x) => {
                context.i16_type().const_int((*x).try_into().unwrap(), false)
                    .try_into().unwrap()
            },
            Primitive::Int32(x) => {
                context.i32_type().const_int((*x).try_into().unwrap(), false)
                    .try_into().unwrap()
            },
            Primitive::Int64(x) => {
                context.i64_type().const_int((*x).try_into().unwrap(), false)
                    .try_into().unwrap()
            },
            Primitive::String(x) => {
                context.const_string(x.as_bytes(), true)
                    .try_into().unwrap()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::TestHelper;
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_float32_printf() {
        let context = Context::create();
        let builder = context.create_builder();
        let tester = TestHelper::new("test_float32", &builder, &context);
        tester.start();
        let prim_val: Primitive = (69.0 as f32).into();
        let _ = prim_val.emit_ir_printf(&tester.builder, &tester.module);
        tester.end();

        let test_output = tester.run();
        assert_eq!(69.0, test_output.parse::<f32>().unwrap());
    }

    #[test]
    fn test_float64_printf() {
        let context = Context::create();
        let builder = context.create_builder();
        let tester = TestHelper::new("test_float64", &builder, &context);
        tester.start();
        let prim_val: Primitive = (69.0 as f64).into();
        let _ = prim_val.emit_ir_printf(&tester.builder, &tester.module);
        tester.end();

        let test_output = tester.run();
        assert_eq!(69.0, test_output.parse::<f64>().unwrap())
    }

    #[test]
    fn test_int32_printf() {
        let context = Context::create();
        let builder = context.create_builder();
        let tester = TestHelper::new("test_int32", &builder, &context);
        tester.start();
        let prim_val: Primitive = (69 as i32).into();
        let _ = prim_val.emit_ir_printf(&tester.builder, &tester.module);
        tester.end();

        let test_output = tester.run();
        assert_eq!(69, test_output.parse::<i32>().unwrap())
    }

    #[test]
    fn test_int64_printf() {
        let context = Context::create();
        let builder = context.create_builder();
        let tester = TestHelper::new("test_int64", &builder, &context);
        tester.start();
        let prim_val: Primitive = (69 as i64).into();
        let _ = prim_val.emit_ir_printf(&tester.builder, &tester.module);
        tester.end();

        let test_output = tester.run();
        assert_eq!(69, test_output.parse::<i64>().unwrap())
    }

    #[test]
    fn test_string_printf() {
        let context = Context::create();
        let builder = context.create_builder();
        let tester = TestHelper::new("test_string", &builder, &context);
        tester.start();
        let prim_val: Primitive = "MyTestString".to_string().into();
        let prim_val = prim_val.emit_ir_alloca(&tester.builder, &tester.module);
        let prim_val: BasicMetadataValueEnum = prim_val.into();
        let datatype = tester.module.get_type("String");
        let _ = (prim_val, datatype.clone()).emit_ir_printf(&tester.builder, &tester.module);
        tester.end();

        let test_output = tester.run();
        assert_eq!("MyTestString", test_output.parse::<String>().unwrap())
    }
}
