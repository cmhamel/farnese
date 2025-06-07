use super::{
    FarneseInternal,
    LLVMAlloca,
    LLVMType,
    MethodHelper, 
    Module,
    StructHelper, 
    Symbol
};
use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::PointerValue;
use std::fmt::{self, Formatter};

#[derive(Clone, Debug)]
pub struct DataType {
    pub field_names: Vec<Symbol>,
    pub field_types: Box<Vec<DataType>>,
    pub name: Symbol,
    pub supertype: Symbol,
    pub is_abstract: bool,
    pub is_mutable: bool,
    pub is_primitive: bool
}

impl<'a, 'b> DataType {
    pub fn new(
        name: Symbol,
        supertype: Symbol,
        is_abstract: bool,
        is_mutable: bool,
        is_primitive: bool,
        field_names: Vec<Symbol>,
        field_types: Box<Vec<DataType>>
    ) -> Self {
        Self {
            field_names: field_names,
            field_types: field_types,
            name: name,
            supertype: supertype,
            is_abstract: is_abstract,
            is_mutable: is_mutable,
            is_primitive: is_primitive
        }
    }

    // shoudl this go in a trait
    pub fn get_ir_value_type(&self, module: &Module<'a>) -> BasicMetadataTypeEnum<'a> {
        let context = module.get_context();
        let ir_val_type = match self.name.name() {
            "Float32" => context.i32_type().try_into().unwrap(),
            "Float64" => context.f64_type().try_into().unwrap(),
            "Int32"   => context.i32_type().try_into().unwrap(),
            "Int64"   => context.i64_type().try_into().unwrap(),
            _ => panic!("Unsupported type {}", self.name.name())
        };
        ir_val_type
    }

    pub fn from_str(
        name: &str, 
        supertype: &str, is_abstract: bool, is_mutable: bool, is_primitive: bool,
        field_names: Vec<String>,
        field_types: Box<Vec<DataType>>
    ) -> Self {
        let field_names = field_names
            .iter()
            .map(|x| Symbol::new(x))
            .collect::<Vec<_>>();
        Self::new(
            Symbol::new(name), Symbol::new(supertype), 
            is_abstract, is_mutable, is_primitive,
            field_names, field_types
        )
    }

    pub fn name(&self) -> &Symbol {
        &self.name
    }

    pub fn new_abstract_type(name: &str, supertype: &str) -> Self {
        let field_names = Vec::<Symbol>::new();
        let field_types = Box::new(Vec::<DataType>::new());
        Self::new(Symbol::new(name), Symbol::new(supertype), true, false, false, field_names, field_types)
    }

    pub fn new_primitive_type(name: &str, supertype: &str, _bits: u32) -> Self {
        let field_names = Vec::<Symbol>::new();
        let field_types = Box::new(Vec::<DataType>::new());
        Self::new(Symbol::new(name), Symbol::new(supertype), false, false, true, field_names, field_types)
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "\n\nDataType:").unwrap();
        writeln!(f, "  Name         = {}", self.name).unwrap();
        writeln!(f, "  Supertype    = {}", self.supertype).unwrap();
        writeln!(f, "  Is abstract? = {}", self.is_abstract).unwrap();
        writeln!(f, "  Is mutable?  = {}", self.is_mutable).unwrap();
        writeln!(f, "  Is primitive = {}", self.is_primitive).unwrap();
        let field_type_names: Vec<_> = self.field_types
            .iter()
            .map(|x| x.name.name())
            .collect();
        writeln!(f, "  Fields       = {:?}", field_type_names)
    }
}

impl<'a, 'b> FarneseInternal<'a> for DataType {
    /// only called once when core is bootstrapped
    fn create_opaque_type(&self, module: &Module<'a>) -> () {
        let context = module.get_context();
        let sym_type = module.get_struct_type("Symbol");
        let opaque_type = context
            .opaque_struct_type("DataType");
        opaque_type.set_body(
            &[
                sym_type.ptr_type(AddressSpace::default()).into(),
                opaque_type.ptr_type(AddressSpace::default()).into()
            ],
            false
        );
    }
 
    fn create_new_method(&self, module: &Module<'a>) {
        let symbol = module.get_struct_type("Symbol");
        let datatype = module.get_struct_type("DataType");
        let symbol_ptr_type = symbol.ptr_type(AddressSpace::default());
        let datatype_ptr_type = datatype.ptr_type(AddressSpace::default());
        let context = module.get_context();
        let builder = context.create_builder();

        let func = datatype_ptr_type.fn_type(&[
            symbol_ptr_type.into(),
            datatype_ptr_type.into()
        ], false);
        let func = module.add_function("Datatype", func, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        let sym_ptr = func.get_nth_method_input(0);
        let supertype_ptr = func.get_nth_method_input(1);
        let struct_ptr = builder.build_alloca(datatype, "").unwrap();
        let _ = struct_ptr.set_nth_field(&builder, 0, sym_ptr);
        let _ = struct_ptr.set_nth_field(&builder, 1, supertype_ptr);
        let _ = builder.build_return(Some(&struct_ptr));
    }

    fn create_get_methods(&self, module: &Module<'a>) {
        let context = module.get_context();
        let builder = context.create_builder();
        let symbol = module.get_struct_type("Symbol");
        let datatype = module.get_struct_type("DataType");
        let symbol_ptr_type = symbol.ptr_type(AddressSpace::default());
        let datatype_ptr_type = datatype.ptr_type(AddressSpace::default());

        // create get_name function
        let func = symbol_ptr_type.fn_type(&[
            datatype_ptr_type.into()
        ], false);
        let func = module.add_function("name", func, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        let struct_ptr = func.get_nth_method_input(0);
        let field_ptr = struct_ptr.load_nth_field(&builder, 0);
        let _ = builder.build_return(Some(&field_ptr));

        // create get supertype function
        let func = datatype_ptr_type.fn_type(&[
            datatype_ptr_type.into()
        ], false);
        let func = module.add_function("supertype", func, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        let struct_ptr = func.get_nth_method_input(0);
        let field_ptr = struct_ptr.load_nth_field(&builder, 1);
        let _ = builder.build_return(Some(&field_ptr));
    }
}

impl<'a, 'b> LLVMAlloca<'a, 'b> for DataType {
    fn emit_ir_alloca(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a> {
        // setup datatype struct
        let datatype_opaque = module.get_struct_type("DataType");
        let _datatype = module.add_global(datatype_opaque, None, self.name.name());
        let _supertype_glob = module.get_global(self.supertype.name());
        // println!("supertype = {:?}", self.supertype.name());
        let ptr = builder.build_alloca(datatype_opaque, "").unwrap();

        // setup a fresh symbol
        let sym_ptr = self.name.emit_ir_alloca(&builder, &module);

        // set the fields where the supertype of DataType is DataType
        let _ = ptr.set_nth_field(&builder, 0, sym_ptr.try_into().unwrap());
        let _ = ptr.set_nth_field(&builder, 1, ptr.try_into().unwrap());
        // let _ = ptr.set_nth_field(&builder, 1, supertype_glob.as_pointer_value());
        // ptr

        // let _ = datatype.set_initializer(&datatype_opaque.const_named_struct(&[
        //     sym_ptr.into(),
        //     ptr.into()
        // ]));
        ptr
    }
}

impl<'a> LLVMType<'a> for DataType {
    fn emit_ir_type(&self, module: &Module<'a>) {
        let datatype_opaque = module.get_struct_type("DataType");
        let _ = module.add_global(datatype_opaque, None, self.name.name());
        // let supertype_glob = module.get_global(self.supertype.name());
        // let _ = datatype.set_initializer(&datatype_opaque.const_named_struct(&[

        // ]))
    }
}

#[cfg(test)]
mod tests {
    use crate::{DataType, LLVMPrintf};
    use crate::test_utils::TestHelper;
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_datatype() {
        let context = Context::create();
        let builder = context.create_builder();
        let tester = TestHelper::new("test_datatype", &builder, &context);
        tester.start();

        let sym = Symbol::new("DataType");
        let sym_super = Symbol::new("DataType");
        let field_names = Vec::<Symbol>::new();
        let field_types = Box::new(Vec::<DataType>::new());
        let datatype = DataType::new(
            sym.clone(), sym_super, 
            false, false, false, 
            field_names, field_types
        );
        let ptr = datatype.emit_ir_alloca(&builder, &tester.module);

        let func = tester.module.get_function("name");
        let _ = builder.build_call(func, &[ptr.into()], "call_name")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let _ = sym.emit_ir_printf(&builder, &tester.module);

        let func = tester.module.get_function("supertype");
        let func_result = builder.build_call(func, &[ptr.into()], "call_supertype")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        let func = tester.module.get_function("name");
        let _ = builder.build_call(func, &[func_result.into()], "call_name")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let _ = sym.emit_ir_printf(&builder, &tester.module);

        // now make Any
        let sym = Symbol::new("Any");
        let sym_super = Symbol::new("Any");
        let field_names = Vec::<Symbol>::new();
        let field_types = Box::new(Vec::<DataType>::new());
        let datatype = DataType::new(
            sym.clone(), sym_super, 
            false, false, false, 
            field_names, field_types
        );
        let ptr = datatype.emit_ir_alloca(&builder, &tester.module);

        let func = tester.module.get_function("name");
        let _ = builder.build_call(func, &[ptr.into()], "call_name")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let _ = sym.emit_ir_printf(&builder, &tester.module);

        let func = tester.module.get_function("supertype");
        let func_result = builder.build_call(func, &[ptr.into()], "call_supertype")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        let func = tester.module.get_function("name");
        let _ = builder.build_call(func, &[func_result.into()], "call_name")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let _ = sym.emit_ir_printf(&builder, &tester.module);

        // now make some other type
        let sym = Symbol::new("Number");
        let sym_super = Symbol::new("Any");
        let field_names = Vec::<Symbol>::new();
        let field_types = Box::new(Vec::<DataType>::new());
        let datatype = DataType::new(
            sym.clone(), sym_super, 
            false, false, false, 
            field_names, field_types
        );
        let ptr = datatype.emit_ir_alloca(&builder, &tester.module);

        let func = tester.module.get_function("name");
        let _ = builder.build_call(func, &[ptr.into()], "call_name")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let _ = sym.emit_ir_printf(&builder, &tester.module);

        let func = tester.module.get_function("supertype");
        let func_result = builder.build_call(func, &[ptr.into()], "call_supertype")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        let func = tester.module.get_function("name");
        let _ = builder.build_call(func, &[func_result.into()], "call_name")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let _ = sym.emit_ir_printf(&builder, &tester.module);

        tester.end();
    }
}
