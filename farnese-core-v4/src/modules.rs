use crate::datatype::{hash_symbol, Symbol, TypeContext};
use inkwell::{
    AddressSpace,
    context::{Context, ContextRef},
    execution_engine::ExecutionEngine,
    module::{self, Linkage},
    OptimizationLevel,
    support::LLVMString,
    types::{BasicTypeEnum, FunctionType, StructType},
    values::FunctionValue
};

pub struct Module<'a> {
    module: module::Module<'a>,
    name: Symbol,
    type_context: TypeContext
}

impl<'a> Module<'a> {
    pub fn new(context: &'a Context, name: &str) -> Self {
        let module = context.create_module(name);
        Self {
            module: module,
            name: hash_symbol(name),
            type_context: TypeContext::new()
        }
    }

    pub fn add_function(&self, name: &str, fn_type: FunctionType<'a>, link_type: Option<Linkage>) -> FunctionValue<'a> {
        self.module.add_function(name, fn_type, link_type)
    }

    pub fn add_global(&mut self, ty: BasicTypeEnum<'a>, addr_space: Option<AddressSpace>, name: &str) {
        // self.globals.push(Symbol::new(name));
        self.module.add_global(ty, addr_space, name);
    }

    pub fn create_jit_execution_engine(&self, opt_level: OptimizationLevel) -> Result<ExecutionEngine<'a>, LLVMString> {
        self.module.create_jit_execution_engine(opt_level)
    }

    pub fn get_context(&self) -> ContextRef<'a> {
        self.module.get_context()
    }

    pub fn get_function(&self, name: &str) -> Option<FunctionValue<'a>> {
        self.module.get_function(name)
    }

    pub fn get_struct_type(&self, name: &str) -> Option<StructType<'a>> {
        self.module.get_struct_type(name)
    }

    pub fn print_to_file(&self, file_name: &str) {
        self.module.print_to_file(file_name).unwrap()
    }

    pub fn print_to_stderr(&self) {
        self.module.print_to_stderr()
    }
}
