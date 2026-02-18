use crate::prelude::*;

impl<'a, 'b> Alloca<'a, 'b> for &str {}

impl<'a, 'b> LLVMType<'a, 'b> for &str {
    fn llvm_type(&self, _builder: &'b Builder<'a>, module: &Module<'a>) -> BasicTypeEnum<'a> {
        let context = module.get_context();
        context
            .i8_type()
            .array_type((self.len() + 1).try_into().unwrap())
            .try_into()
            .unwrap()
    }
}

impl<'a, 'b> LLVMValue<'a, 'b> for &str {
    fn llvm_value(&self, builder: &'b Builder<'a>, _module: &Module<'a>) -> BasicValueEnum<'a> {
        builder.build_global_string_ptr(&self, "")
            .unwrap()
            .as_pointer_value()
            .try_into()
            .unwrap()
    }
}

// impl<'a, 'b> Load<'a, 'b> for &str {}


impl<'a, 'b> Printf<'a, 'b> for &str {
    // fn fmt_str(
    //     &self,
    //     builder: &'b Builder<'a>, 
    //     module: &Module<'a>
    // ) -> PointerValue<'a> {
    //     builder
    //         .build_global_string_ptr("%s\0", "")
    //         .unwrap()
    //         .as_pointer_value()
    // }

    fn printf(
        &self, 
        builder: &'b Builder<'a>, 
        module: &Module<'a>,
        ptr: PointerValue<'a>
    ) {
        let context = module.get_context();
        let zero = context.i32_type().const_zero();
        let array_type = self.llvm_type(builder, module);
        // let gep_ptr = builder
        //     .build_global_string_ptr("%s\0", "")
        //     .unwrap()
        //     .as_pointer_value();
        let gep_ptr = self.fmt_str(builder, module);
        let val = unsafe {
            builder.build_gep(array_type, ptr, &[zero, zero], "")
        }.unwrap();
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

impl<'a, 'b> Store<'a, 'b> for &str {
    fn store(
        &self, 
        builder: &'b Builder<'a>, 
        module: &Module<'a>,
        ptr: PointerValue<'a>
    ) {
        let context = module.get_context();
        let zero = context.i32_type().const_zero();
        let len = context.i64_type().const_int(self.len().try_into().unwrap(), false);
        let val = self.llvm_value(builder, module);
        let buf_ptr = unsafe {
            builder.build_gep(self.llvm_type(builder, module), ptr, &[zero, zero], "")
        }.unwrap();
        builder.build_call(
            module.get_function("llvm.memcpy.p0.p0.i64").unwrap(),
            &[
                buf_ptr.into(),
                val.into(),
                len.into(),
                context.bool_type().const_zero().into(),
            ],
            "copy",
        ).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    use super::*;

    #[test]
    fn test_string() {
        let context = Context::create();
        let test = TestHelper::new(&context, "test_string");

        test.start();

        let string = "Some long string";
        let _ = string.llvm_value(&test.builder, &test.module);
        let ptr = string.alloca(&test.builder, &test.module);
        let _ = string.store(&test.builder, &test.module, ptr);
        let _ = string.printf(&test.builder, &test.module, ptr);
    
        test.end();
        let results = test.run();
        assert_eq!("Some long string", results);
    }
}