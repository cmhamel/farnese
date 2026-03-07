use crate::modules::Module;
use crate::traits::*;
use inkwell::{
    builder::Builder,
    types::BasicTypeEnum,
    values::{BasicValueEnum, PointerValue}
};

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Float32(f32),
    Float64(f64),
    Int32(i32),
    Int64(i64)
}

impl From<f32> for Primitive {
    fn from(item: f32) -> Self { 
        Primitive::Float32(item) 
    }
}

impl From<f64> for Primitive {
    fn from(item: f64) -> Self { 
        Primitive::Float64(item) 
    }
}

impl From<i32> for Primitive {
    fn from(item: i32) -> Self { 
        Primitive::Int32(item) 
    }
}

impl From<i64> for Primitive {
    fn from(item: i64) -> Self { 
        Primitive::Int64(item) 
    }
}

// basic llvm wrappers
impl<'a, 'b> LLVMType<'a, 'b> for Primitive {
    fn llvm_type(&self, _builder: &'b Builder<'a>, module: &Module<'a>) -> BasicTypeEnum<'a> {
        let context = module.get_context();
        match &self {
            Primitive::Float32(_x) => context
                .f32_type().try_into().unwrap(),
            Primitive::Float64(_x) => context
                .f64_type().try_into().unwrap(),
            Primitive::Int32(_x) => context
                .i32_type().try_into().unwrap(),
            Primitive::Int64(_x) => context
                .i64_type().try_into().unwrap()
        }
    }
}

impl<'a, 'b> LLVMValue<'a, 'b> for Primitive {
    fn llvm_value(&self, _builder: &'b Builder<'a>, module: &Module<'a>) -> BasicValueEnum<'a> {
        let context = module.get_context();
        match &self {
            Primitive::Float32(x) => context
                .f32_type()
                .const_float((*x).try_into().unwrap())
                .try_into()
                .unwrap(),
            Primitive::Float64(x) => context
                .f64_type()
                .const_float((*x).try_into().unwrap())
                .try_into()
                .unwrap(),
            Primitive::Int32(x) => context
                .i32_type()
                .const_int((*x).try_into().unwrap(), false)
                .try_into()
                .unwrap(),
            Primitive::Int64(x) => context
                .i64_type()
                .const_int((*x).try_into().unwrap(), false)
                .try_into()
                .unwrap()
        }
    }
}

// specific llvm ops
impl<'a, 'b> Alloca<'a, 'b> for Primitive {}

impl<'a, 'b> Load<'a, 'b> for Primitive {}

impl<'a, 'b> Printf<'a, 'b> for Primitive {
    fn fmt_str(
        &self, 
        builder: &'b Builder<'a>, 
        _module: &Module<'a>
    ) -> PointerValue<'a> {
        match &self {
            Primitive::Float32(_x) => builder
                .build_global_string_ptr("%.4f\0", ""),
            Primitive::Float64(_x) => builder
                .build_global_string_ptr("%.4f\0", ""),
            Primitive::Int32(_x) => builder
                .build_global_string_ptr("%lld\0", ""),
            Primitive::Int64(_x) => builder
                .build_global_string_ptr("%lld\0", "")
        }.unwrap().as_pointer_value()
    }

    fn printf(
        &self, 
        builder: &'b Builder<'a>, 
        module: &Module<'a>,
        ptr: PointerValue<'a>
    ) {
        let gep_ptr = self.fmt_str(builder, module);
        let val = self.load(builder, module, ptr);
        builder
            .build_call(
                module.get_function("printf")
                    .expect("Failed to load printf"), 
                &[
                    gep_ptr.into(),
                    val.into()
                ], 
                ""
            )
            .unwrap();
    }
}

impl<'a, 'b> Store<'a, 'b> for Primitive {}

#[cfg(test)]
mod tests {
    use crate::test_utils::TestHelper;
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_f32() {
        let context = Context::create();
        let test = TestHelper::new(&context, "test_f32");
        test.start();
        let x = Primitive::Float64(420.69);
        let ptr = x.alloca(&test.builder, &test.module);
        let _ = x.store(&test.builder, &test.module, ptr);
        let _ = x.printf(&test.builder, &test.module, ptr);
        test.end();
        let results = test.run();
        let val = results.parse::<f32>().unwrap();
        assert_eq!(val, 420.69 as f32);
    }

    #[test]
    fn test_f64() {
        let context = Context::create();
        let test = TestHelper::new(&context, "test_f64");
        test.start();
        let x = Primitive::Float64(420.69 as f64);
        let ptr = x.alloca(&test.builder, &test.module);
        let _ = x.store(&test.builder, &test.module, ptr);
        let _ = x.printf(&test.builder, &test.module, ptr);
        test.end();
        let results = test.run();
        let val = results.parse::<f64>().unwrap();
        assert_eq!(val, 420.69 as f64);
    }

    #[test]
    fn test_i32() {
        let context = Context::create();
        let test = TestHelper::new(&context, "test_i32");
        test.start();
        let x = Primitive::Int32(69);
        let ptr = x.alloca(&test.builder, &test.module);
        let _ = x.store(&test.builder, &test.module, ptr);
        let _ = x.printf(&test.builder, &test.module, ptr);
        test.end();
        let results = test.run();
        let val = results.parse::<i32>().unwrap();
        assert_eq!(val, 69);
    }

    #[test]
    fn test_i64() {
        let context = Context::create();
        let test = TestHelper::new(&context, "test_i64");
        test.start();
        let x = Primitive::Int64(420);
        let ptr = x.alloca(&test.builder, &test.module);
        let _ = x.store(&test.builder, &test.module, ptr);
        let _ = x.printf(&test.builder, &test.module, ptr);
        test.end();
        let results = test.run();
        let val = results.parse::<i64>().unwrap();
        assert_eq!(val, 420);
    }
}
