pub mod datatype;
pub mod module;
pub mod primitive;
pub mod symbol;

#[cfg(test)]
pub mod test_utils;

pub use datatype::DataType;
pub use module::Module;
pub use primitive::Primitive;
pub use symbol::Symbol;

use inkwell::{
    self, 
    AddressSpace, 
    builder::Builder, 
    context::Context,
    module::Linkage,
    values::{
        BasicMetadataValueEnum,
        BasicValueEnum,
        CallSiteValue,
        FunctionValue,
        PointerValue
    }
};

pub trait FarneseInternal<'a> {
    fn bootstrap(&self, module: &Module<'a>) {
        self.create_opaque_type(module);
        self.create_new_method(module);
        self.create_get_methods(module);
    }

    fn create_datatype(&self, _module: &Module<'a>) {

    }

    fn create_get_methods(&self, module: &Module<'a>);
    fn create_new_method(&self, module: &Module<'a>);
    fn create_opaque_type(&self, module: &Module<'a>);
}

pub trait MethodHelper<'a> {
    fn get_nth_method_input(&self, n: u32) -> BasicMetadataValueEnum<'a>;
}

// some helpers to make inkwell less loud
impl<'a> MethodHelper<'a> for FunctionValue<'a> {
    fn get_nth_method_input(&self, n: u32) -> BasicMetadataValueEnum<'a> {
        self.get_nth_param(n)
            .unwrap()
            .try_into()
            .unwrap()
    }
}

pub trait StructHelper<'a, 'b> {
    /// returns a pointer to the nth field in a struct
    fn get_nth_field(&self, builder: &'b Builder<'a>, n: u32) -> PointerValue<'a>;
    /// loads and returns the value to the nth field of a struct
    fn load_nth_field(&self, builder: &'b Builder<'a>, n: u32) -> BasicValueEnum<'a> {
        let field_ptr = self.get_nth_field(builder, n);
        let field_ptr = builder
            .build_load(field_ptr, "")
            .unwrap();
        field_ptr
    }
    /// sets the ptr of the nth field of a struct to the supplied ptr
    fn set_nth_field(&self, builder: &'b Builder<'a>, n: u32, val: BasicMetadataValueEnum<'a>) {
        let field_ptr = self.get_nth_field(builder, n);
        match val {
            BasicMetadataValueEnum::PointerValue(x) => {
                let _ = builder.build_store(field_ptr, x);
            },
            _ => todo!("Unsupported set_nth_field type {:?}", val)
        }
    }
}

impl<'a, 'b> StructHelper<'a, 'b> for BasicMetadataValueEnum<'a> {
    fn get_nth_field(&self, builder: &'b Builder<'a>, n: u32) -> PointerValue<'a> {
        match &self {
            BasicMetadataValueEnum::PointerValue(x) => {
                x.get_nth_field(builder, n)
            },
            _ => todo!("unsupported")
        }
    }
}

impl<'a, 'b> StructHelper<'a, 'b> for PointerValue<'a> {
    fn get_nth_field(&self, builder: &'b Builder<'a>, n: u32) -> PointerValue<'a> {
        builder
            .build_struct_gep(*self, n, "")
            .unwrap()
    }
}

pub trait LLVMAlloca<'a, 'b> {
    fn emit_ir_alloca(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a>;
}

pub trait LLVMPrintf<'a, 'b> {
    fn emit_ir_printf(
        &self, 
        builder: &'b Builder<'a>, 
        module: &Module<'a>
    ) -> CallSiteValue<'a>;
}

// move to a value file
impl<'a, 'b> LLVMPrintf<'a, 'b> for (BasicMetadataValueEnum<'a>, DataType) {
    fn emit_ir_printf(
        &self,
        builder: &'b Builder<'a>,
        module: &Module<'a>
    ) -> CallSiteValue<'a> {
        let context = module.get_context();
        let datatype = self.1.name().name();
        let format_string = match datatype {
            "Float32" | "Float64" => {
                context.const_string(b"%.8f\0", false)
            },
            "Int32" | "Int64" => {
                context.const_string(b"%lld\0", false)
            },
            "String" => {
                context.const_string(b"%s\0", false)
            }
            _ => todo!()
        };
        let ptr = builder.build_alloca(format_string.get_type(), "").unwrap();
        let _ = builder.build_store(ptr, format_string);
        let gep_ptr = unsafe { builder.build_in_bounds_gep(ptr, &[
            context.i32_type().const_zero(),
            context.i32_type().const_zero(),
        ], "").unwrap() };
        let func_args = vec![gep_ptr.into(), self.0.clone()];
        builder.build_call(
            module.get_function("printf"),
            &func_args,
            ""
        ).unwrap()
    }
}

pub trait LLVMType<'a> {
    fn emit_ir_type(&self, module: &Module<'a>);
}

pub trait LLVMValue<'a> {
    fn emit_ir_value(&self, module: &Module<'a>) -> BasicValueEnum<'a>;
}


// Core
pub struct Core<'a> {
    module: Module<'a>
}

impl<'a> Core<'a> {
    pub fn new(context: &'a Context) -> Self {
        let module = module::Module::<'a>::new(context, "Core");
        Self {
            module: module
        }
    }

    pub fn bootstrap(&mut self) -> Module<'a> {
        let _ = self.basic_c_funcs();

        // symbol type
        let symbol_sym = Symbol::new("Symbol");

        symbol_sym.create_opaque_type(&self.module);
        let sym_type = self.module.get_struct_type("Symbol");
        let _ = self.module.add_global(sym_type, None, "Symbol");

        // self.module.insert_type(sym_type.clone());     
        let datatype = DataType::new(
            Symbol::new("Symbol"), Symbol::new("Any"), 
            false, false, false, 
            vec![], Box::new(vec![])
        );
        self.module.insert_type(datatype);   
        self.module.push_export(symbol_sym);

        // data types
        let sym = Symbol::new("DataType");
        let field_names = Vec::<Symbol>::new();
        let field_types = Box::new(Vec::<DataType>::new());
        let datatype = DataType::new(
            sym.clone(), sym.clone(), 
            false, false, false, 
            field_names, field_types
        );
        let _ = self.module.insert_type(datatype.clone());
        // let _ = self.module.push_export(sym);
        datatype.bootstrap(&self.module);
        self.module.push_export(sym);

        // any type
        let sym = Symbol::new("Any");
        let field_names = Vec::<Symbol>::new();
        let field_types = Box::new(Vec::<DataType>::new());
        let datatype = DataType::new(
            sym.clone(), sym.clone(), 
            false, false, false, 
            field_names, field_types
        );
        let _ = self.module.insert_type(datatype.clone());
        // let _ = self.module.push_export(sym);
        let _ = datatype.emit_ir_type(&self.module);
        self.module.push_export(sym);

        // symbol datatype - TODO move this to above
        

        // basic abstract types needed for primitive types
        self.module.insert_type(DataType::new_abstract_type("AbstractString", "Any"));
        self.module.push_export(Symbol::new("AbstractString"));
        self.module.insert_type(DataType::new_abstract_type("Number", "Any"));
        self.module.push_export(Symbol::new("Number"));
        self.module.insert_type(DataType::new_abstract_type("Real", "Number"));
        self.module.push_export(Symbol::new("Real"));

        self.module.insert_type(DataType::new_abstract_type("AbstractFloat", "Real"));
        self.module.push_export(Symbol::new("AbstractFloat"));
        self.module.insert_type(DataType::new_abstract_type("Integer", "Real"));
        self.module.push_export(Symbol::new("Integer"));
        self.module.insert_type(DataType::new_abstract_type("Signed", "Integer"));
        self.module.push_export(Symbol::new("Signed"));
        self.module.insert_type(DataType::new_abstract_type("Unsigned", "Integer"));
        self.module.push_export(Symbol::new("Unsigned"));

        // all the primitive types
        self.module.insert_type(DataType::new_primitive_type("Float32", "AbstractFloat", 32));
        self.module.push_export(Symbol::new("Float32"));
        self.module.insert_type(DataType::new_primitive_type("Float64", "AbstractFloat", 64));
        self.module.push_export(Symbol::new("Float64"));
        self.module.insert_type(DataType::new_primitive_type("Int32", "Signed", 32));
        self.module.push_export(Symbol::new("Int32"));
        self.module.insert_type(DataType::new_primitive_type("Int64", "Signed", 64));
        self.module.push_export(Symbol::new("Int64"));
        self.module.insert_type(DataType::new_primitive_type("String", "AbstractString", 8));
        self.module.push_export(Symbol::new("String"));

        self.module.clone()
    }

    pub fn basic_c_funcs(&self) {
        let context = self.module.get_context();
        let printf_type = context
            .i8_type()
            .fn_type(&[
                    self.module.get_context()
                        .i8_type()
                        .ptr_type(AddressSpace::default()).into()
                ], 
                true
            );

        let _ = self.module
            .add_function("printf", printf_type, Some(Linkage::External));
    }
}
