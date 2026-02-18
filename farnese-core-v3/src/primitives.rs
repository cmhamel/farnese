use crate::datatype::DataType;
use crate::prelude::*;
use crate::symbol::Symbol;

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Float64(f64),
    Int32(i32),
    Int64(i64)
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

impl<'a, 'b> Alloca<'a, 'b> for Primitive {}

impl<'a, 'b> FarneseType<'a, 'b> for Primitive {
    fn farnese_type(&self, module: &Module<'a>) -> DataType {
        match &self {
            Primitive::Float64(_) => module.get_farnese_type(Symbol::new("Float64")).clone(),
            Primitive::Int32(_x)  => module.get_farnese_type(Symbol::new("Int32")).clone(),
            Primitive::Int64(_x)  => module.get_farnese_type(Symbol::new("Int64")).clone()
        }
    }
}

impl<'a, 'b> LLVMType<'a, 'b> for Primitive {
    fn llvm_type(&self, _builder: &'b Builder<'a>, module: &Module<'a>) -> BasicTypeEnum<'a> {
        let context = module.get_context();
        match &self {
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

impl<'a, 'b> Load<'a, 'b> for Primitive {}

impl<'a, 'b> Printf<'a, 'b> for Primitive {
    fn fmt_str(
        &self, 
        builder: &'b Builder<'a>, 
        _module: &Module<'a>
    ) -> PointerValue<'a> {
        match &self {
            Primitive::Float64(_x) => builder
                .build_global_string_ptr("%d\0", ""),
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
        // let gep_ptr = match &self {
        //     Primitive::Int32(_x) => builder
        //         .build_global_string_ptr("%lld\0", ""),
        //     Primitive::Int64(_x) => builder
        //         .build_global_string_ptr("%lld\0", "")
        // }.unwrap().as_pointer_value();
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
    use crate::tests::TestHelper;
    use super::*;
    use inkwell::context::Context;

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
        test.show_ir();
        let results = test.run();
        let val = results.parse::<i32>().unwrap();
        assert_eq!(val, 69);

        // let farnese_type = x.farnese_type(&test.module);
        // println!("farnese type = {:?}", farnese_type);
    }
}