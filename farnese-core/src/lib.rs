pub mod any;
pub mod datatype;
pub mod module;
pub mod primitive;
pub mod svec;
pub mod symbol;

#[cfg(test)]
pub mod test_utils;

pub use any::Any;
pub use datatype::DataType;
pub use module::Module;
pub use primitive::Primitive;
pub use svec::SimpleVector;
pub use symbol::Symbol;

use inkwell::{
    self,
    AddressSpace,
    builder::Builder,
    context::Context,
    values::{
        BasicMetadataValueEnum, 
        BasicValueEnum, 
        CallSiteValue, 
        FunctionValue, 
        GlobalValue,
        PointerValue
    },
};

pub trait FarneseInternal<'a> {
    fn bootstrap(module: &Module<'a>) -> DataType {
        Self::create_opaque_type(module);
        // Self::create_new_method(module);
        // Self::set_opaque_type_initialize(module);
        Self::create_datatype()
    }

    fn create_datatype() -> DataType;
    fn create_new_method(module: &Module<'a>);
    fn create_opaque_type(module: &Module<'a>);
    fn set_opaque_type_initializer(module: &Module<'a>);
}

pub trait MethodHelper<'a> {
    fn get_nth_method_input(&self, n: u32) -> BasicMetadataValueEnum<'a>;
}

// some helpers to make inkwell less loud
impl<'a> MethodHelper<'a> for FunctionValue<'a> {
    fn get_nth_method_input(&self, n: u32) -> BasicMetadataValueEnum<'a> {
        self.get_nth_param(n).unwrap().try_into().unwrap()
    }
}

pub trait StructHelper<'a, 'b> {
    /// returns a pointer to the nth field in a struct
    fn get_nth_field(&self, builder: &'b Builder<'a>, n: u32) -> PointerValue<'a>;
    /// loads and returns the value to the nth field of a struct
    fn load_nth_field(&self, builder: &'b Builder<'a>, n: u32) -> BasicValueEnum<'a> {
        let field_ptr = self.get_nth_field(builder, n);
        let field_ptr = builder.build_load(field_ptr, "").unwrap();
        field_ptr
    }
    /// sets the ptr of the nth field of a struct to the supplied ptr
    fn set_nth_field(&self, builder: &'b Builder<'a>, n: u32, val: BasicMetadataValueEnum<'a>) {
        let field_ptr = self.get_nth_field(builder, n);
        match val {
            BasicMetadataValueEnum::PointerValue(x) => {
                let _ = builder.build_store(field_ptr, x);
            }
            _ => todo!("Unsupported set_nth_field type {:?}", val),
        }
    }
}

impl<'a, 'b> StructHelper<'a, 'b> for BasicMetadataValueEnum<'a> {
    fn get_nth_field(&self, builder: &'b Builder<'a>, n: u32) -> PointerValue<'a> {
        match &self {
            BasicMetadataValueEnum::PointerValue(x) => x.get_nth_field(builder, n),
            _ => todo!("unsupported"),
        }
    }
}

impl<'a, 'b> StructHelper<'a, 'b> for PointerValue<'a> {
    fn get_nth_field(&self, builder: &'b Builder<'a>, n: u32) -> PointerValue<'a> {
        builder.build_struct_gep(*self, n, "").unwrap()
    }
}

pub trait LLVMAlloca<'a, 'b> {
    fn emit_ir_alloca(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a>;
}

pub trait LLVMPrintf<'a, 'b> {
    fn emit_ir_printf(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> CallSiteValue<'a>;
}

pub trait LLVMGlobal<'a> {
    fn emit_ir_global(&self, module: &Module<'a>) -> GlobalValue<'a>;
}

// move to a value file
impl<'a, 'b> LLVMPrintf<'a, 'b> for (BasicMetadataValueEnum<'a>, DataType) {
    fn emit_ir_printf(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> CallSiteValue<'a> {
        let datatype = self.1.name().name();
        let gep_ptr = match datatype {
            "Float32" | 
            "Float64" => builder.build_global_string_ptr("%.8f\0", ""),
            "Int32" | 
            "Int64" => builder.build_global_string_ptr("%lld\0", ""),
            "String" => builder.build_global_string_ptr("%s\0", ""),
            "Symbol" => builder.build_global_string_ptr("%s\0", ""),
            _ => todo!(),
        }.unwrap().as_pointer_value();
        
        let arg_2 = match datatype {
            "String" => {
                builder.build_load(self.0.try_into().unwrap(), "")
                // builder.build_load(gep_ptr, "")
                    .unwrap()
                    .into_pointer_value()
                    .into()
            }
            "Symbol" => {
                let sym_str = self.0.get_nth_field(builder, 1);
                builder.build_load(sym_str, "")
                    .unwrap()
                    .into_pointer_value()
                    .into()
            },
            _ => self.0.clone()
        };
        let func_args = vec![gep_ptr.into(), arg_2];

        builder
            .build_call(module.get_function("printf"), &func_args, "")
            .unwrap()
    }
}

pub trait LLVMValue<'a, 'b> {
    fn emit_ir_value(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> BasicValueEnum<'a>;
}

// Core
pub struct Core<'a> {
    module: Module<'a>,
}

impl<'a> Core<'a> {
    pub fn new(context: &'a Context) -> Self {
        let module = module::Module::<'a>::new(context, "Core");
        Self { 
            module: module 
        }
    }

    pub fn bootstrap(&mut self) -> Module<'a> {
        let context = self.module.get_context();
        // // symbol
        // let sym_datatype = Symbol::bootstrap(&self.module);
        // let _ = self.module.push_export(sym_datatype.name().clone());
        // let _ = self.module.insert_type(sym_datatype.clone());

        // // svec
        // let svec_datatype = SimpleVector::bootstrap(&self.module);
        // let _ = self.module.insert_type(svec_datatype.clone());

        // // data types
        // let datatype = DataType::bootstrap(&self.module);
        // let _ = self.module.push_export(datatype.name().clone());
        // let _ = self.module.insert_type(datatype.clone());

        // // any type
        // let any = Any::bootstrap(&self.module);
        // let _ = self.module.push_export(any.name().clone());
        // let _ = self.module.insert_type(any.clone());

        // // emit globals - order matters here
        // Symbol::set_opaque_type_initializer(&self.module);
        // SimpleVector::set_opaque_type_initializer(&self.module);
        // DataType::set_opaque_type_initializer(&self.module);
        // Any::set_opaque_type_initializer(&self.module);

        // any.emit_ir_global(&self.module);
        // println!("Symbol initialized");
        // svec_datatype.emit_ir_global(&self.module);
        // datatype.emit_ir_global(&self.module);
        // sym_datatype.emit_ir_global(&self.module);
        // self.module.clone()

        // setup opaque types
        let sym_opaque_type = context.opaque_struct_type("Symbol");
        let datatype_opaque_type = context.opaque_struct_type("DataType");
        let svec_opaque_type = context.opaque_struct_type("SimpleVector");
        let any_opaque_type = context.opaque_struct_type("Any");

        // fill opaque types
        sym_opaque_type.set_body(&[
            context.i64_type().into(),
            context.i8_type().ptr_type(AddressSpace::default()).into(),
        ], false);
        datatype_opaque_type.set_body(&[
            sym_opaque_type.ptr_type(AddressSpace::default()).into(),
            datatype_opaque_type.ptr_type(AddressSpace::default()).into(),
            svec_opaque_type.ptr_type(AddressSpace::default()).into(),
            svec_opaque_type.ptr_type(AddressSpace::default()).into()
        ], false);
        svec_opaque_type.set_body(&[
            context.i64_type().into(),
            datatype_opaque_type.ptr_type(AddressSpace::default())
                .ptr_type(AddressSpace::default()).into()
        ], false);
        any_opaque_type.set_body(&[
            context.i64_type().ptr_type(AddressSpace::default()).into(),
            datatype_opaque_type.ptr_type(AddressSpace::default()).into()
        ], false);

        // setup types
        let sym_type = Symbol::create_datatype();
        let datatype_type = DataType::create_datatype();
        let svec_type = SimpleVector::create_datatype();
        let any_type = Any::create_datatype();

        // add exports
        self.module.push_export(sym_type.name().clone());
        self.module.push_export(datatype_type.name().clone());
        self.module.push_export(svec_type.name().clone());
        self.module.push_export(any_type.name().clone());

        // add types to dict
        self.module.insert_type(sym_type.clone());
        self.module.insert_type(datatype_type.clone());
        self.module.insert_type(svec_type.clone());
        self.module.insert_type(any_type.clone());

        // emit global stuff
        datatype_type.emit_ir_global(&self.module);
        any_type.emit_ir_global(&self.module);
        sym_type.emit_ir_global(&self.module);
        svec_type.emit_ir_global(&self.module);

        self.module.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{Core, Module};
    use inkwell::context::Context;

    #[test]
    fn test_bootstrap_core() {
        let context = Context::create();
        let _ = Core::new(&context);
    }

    #[test]
    fn test_add_base() {
        let context = Context::create();
        let _ = Module::new(&context, "Base");
    }
}
