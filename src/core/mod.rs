// pub mod any;
pub mod datatype;
pub mod module;
pub mod primitive;
pub mod symbol;

// pub use any::Any;
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
use std::ffi::CString;

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

impl<'a, 'b> LLVMPrintf<'a, 'b> for Vec<BasicMetadataValueEnum<'a>> {
    fn emit_ir_printf(
        &self, 
        builder: &'b Builder<'a>, 
        module: &Module<'a>
    ) -> CallSiteValue<'a> {
        let context = module.get_context();
        // let mut result;
        let result = self.iter()
            .map(|x| {
                let format_string = match x {
                    // how to handle chars?
                    BasicMetadataValueEnum::FloatValue(y) => {
                        context.const_string(b"%f\0", false)
                    },
                    BasicMetadataValueEnum::IntValue(y) => {
                        context.const_string(b"%d\0", false)
                    },
                    _ => todo!()
                };
                let ptr = builder.build_alloca(format_string.get_type(), "").unwrap();
                let _ = builder.build_store(ptr, format_string);
                let gep_ptr = unsafe { builder.build_in_bounds_gep(ptr, &[
                    context.i32_type().const_zero(),
                    context.i32_type().const_zero(),
                ], "").unwrap() };
                let func_args = vec![gep_ptr.into(), x.clone()];
                builder.build_call(
                    module.get_function("printf"),
                    &func_args,
                    ""
                ).unwrap()
            })
            .collect::<Vec<_>>();
        
        result[0]
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
        let _ = self.basic_c_fmts();
        let _ = self.basic_c_funcs();

        // symbol type
        let symbol_sym = Symbol::new("Symbol");
        symbol_sym.create_opaque_type(&self.module);
        let sym_type = self.module.get_struct_type("Symbol");
        let _ = self.module.add_global(sym_type, None, "Symbol");

        // data types
        let sym = Symbol::new("DataType");
        let field_names = Vec::<Symbol>::new();
        let field_types = Box::new(Vec::<DataType>::new());
        let datatype = DataType::new(sym.clone(), sym.clone(), false, false, false, field_names, field_types);
        let _ = self.module.insert_type(datatype.clone());
        let _ = self.module.push_export(sym);
        datatype.bootstrap(&self.module);

        // any type
        let sym = Symbol::new("Any");
        let field_names = Vec::<Symbol>::new();
        let field_types = Box::new(Vec::<DataType>::new());
        let datatype = DataType::new(sym.clone(), sym.clone(), false, false, false, field_names, field_types);
        let _ = self.module.insert_type(datatype.clone());
        let _ = self.module.push_export(sym);
        let _ = datatype.emit_ir_type(&self.module);

        self.module.clone()
    }

    pub fn basic_c_fmts(&self) {
        let context = self.module.get_context();

        // fmt stuff
        let fmt_strs = ["%c", "%d", "%f", "%s"];
        let fmt_globals = ["__fmt_char", "__fmt_int", "__fmt_float", "__fmt_string"];

        for (fmt_str, fmt_global) in fmt_strs.iter().zip(fmt_globals.iter()) {
            let cstr = CString::new(*fmt_str).unwrap();
            let fmt_global = self.module.module.add_global(
                context
                    .i8_type()
                    .array_type(cstr.as_bytes_with_nul().len() as u32), 
                None, 
                fmt_global
            );
            fmt_global.set_initializer(&context.i8_type().const_array(
                &cstr.as_bytes_with_nul()
                    .iter()
                    .map(|&b| context.i8_type().const_int(b as u64, false))
                    .collect::<Vec<_>>(),
            ));
        }
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

        let _ = self.module.module
            .add_function("printf", printf_type, Some(Linkage::External));
    }

    // eventually remove these but they're being heavily used in tests right now
    pub fn main_func_begin<'b>(&self, builder: &'b Builder<'a>, module: &Module<'a>) {
        let context = module.get_context();
        let fn_type = context.i32_type().fn_type(&[], false);
        let function = module.add_function("main", fn_type, None);
        let entry = context.append_basic_block(function, "entry");
        builder.position_at_end(entry);
    }

    pub fn main_func_end<'b>(&self, builder: &'b Builder<'a>, module: &Module<'a>) {
        let context = module.get_context();
        let zero = context.i32_type().const_int(0, false);
        let _ = builder.build_return(Some(&zero));
    }
}
