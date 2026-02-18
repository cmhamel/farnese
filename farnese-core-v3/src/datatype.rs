use crate::prelude::*;
use crate::Symbol;
use inkwell::types::BasicMetadataTypeEnum;


#[derive(Clone, Debug)]
pub struct DataType {
    bits: i32,
    field_names: Vec<Symbol>,
    // field_types: Vec<Box<DataType>>,
    field_types: Vec<Symbol>,
    is_abstract: bool,
    is_mutable: bool,
    is_primitive: bool,
    pub name: Symbol,
    // supertype: Box<DataType>
    supertype: Symbol
}

impl<'a, 'b> DataType {
    pub fn create_abstract_type(name: &str, supertype: &str) -> Self {
        Self {
            bits: 8,
            field_names: vec![],
            field_types: vec![],
            is_abstract: true,
            is_mutable: false,
            is_primitive: false,
            name: Symbol::new(name),
            supertype: Symbol::new(supertype)
        }
    }

    pub fn create_concrete_type(
        name: &str, supertype: &str,
        field_names: Vec<Symbol>,
        field_types: Vec<Symbol>
    ) -> Self {
        Self {
            bits: 8,
            field_names: field_names,
            field_types: field_types,
            is_abstract: false,
            is_mutable: false,
            is_primitive: false,
            name: Symbol::new(name),
            supertype: Symbol::new(supertype)
        }
    }

    pub fn create_primitive_type(name: &str, supertype: &str, bits: i32) -> Self {
        Self {
            bits: bits,
            field_names: vec![],
            field_types: vec![],
            is_abstract: false,
            is_mutable: false,
            is_primitive: true,
            name: Symbol::new(name),
            supertype: Symbol::new(supertype)
        }
    }

    pub fn get_bit_size(&self) -> &i32 {
        &self.bits
    }

    pub fn get_field_names(&self) -> &Vec<Symbol> {
        &self.field_names
    }

    pub fn get_field_types(&self) -> &Vec<Symbol> {
        &self.field_types
    }

    pub fn get_llvm_type(&self, module: &Module<'a>) -> BasicMetadataTypeEnum<'a> {
        let context = module.get_context();
        match self.name.name() {
            "Int32" => context.i32_type().try_into().unwrap(),
            "Int64" => context.i64_type().try_into().unwrap(),
            _ => todo!()
        }
    }

    pub fn is_abstract(&self) -> &bool {
        return &self.is_abstract
    }

    pub fn is_mutable(&self) -> &bool {
        return &self.is_mutable
    }

    pub fn is_primitive(&self) -> &bool {
        return &self.is_primitive
    }

    pub fn name(&self) -> Symbol {
        return self.name.clone()
    }

    pub fn supertype(&self) -> &Symbol {
        return &self.supertype
    }
}

impl<'a, 'b> Alloca<'a, 'b> for DataType {}

impl<'a, 'b> LLVMType<'a, 'b> for DataType {
    fn llvm_type(&self, _builder: &'b Builder<'a>, module: &Module<'a>) -> BasicTypeEnum<'a> {
        module.get_struct_type("DataType")
            .unwrap().try_into().unwrap()
    }
}

impl<'a, 'b> LLVMValue<'a, 'b> for DataType {
    fn llvm_value(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> BasicValueEnum<'a> {
        let datatype = module.get_struct_type("DataType").unwrap();
        datatype.const_named_struct(&[
            self.name.llvm_value(builder, module),
            self.supertype.llvm_value(builder, module)
        ]).try_into().unwrap()
    }
}

impl<'a, 'b> OpaqueType<'a, 'b> for DataType {
    fn set_opaque_type_init(_builder: &'b Builder<'a>, module: &Module<'a>) {
        let opaque_type = module.get_struct_type("DataType")
            .expect("Failed to find opaque struct in set_opaque_type_init");
        opaque_type.set_body(
            &[
                module.get_struct_type("Symbol").unwrap().try_into().unwrap(),
                module.get_struct_type("Symbol").unwrap().try_into().unwrap()
            ],
            false
        );
    }
}

impl<'a, 'b> Store<'a, 'b> for DataType {}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_datatype_init() {
        let context = Context::create();
        let test = TestHelper::new(&context, "test_datatype_init");

        test.start();

        let dt = DataType::create_abstract_type("Any", "Any");
        let ptr = dt.alloca(&test.builder, &test.module);
        // let val = dt.llvm_value(&test.builder, &test.module);
        let _ = dt.store(&test.builder, &test.module, ptr);
        let name_sym = test.builder.build_struct_gep(
            dt.name.llvm_type(&test.builder, &test.module), ptr, 0, ""
        ).unwrap();
        let _ = dt.name.printf(&test.builder, &test.module, name_sym);

        test.end();

        // test.show_ir();
        let _results = test.run();
        // println!("Results = {}", results);
    }
}