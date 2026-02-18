use crate::datatype::DataType;
use crate::prelude::*;
use crate::symbol::Symbol;
use inkwell::context::{
    Context,
    ContextRef
};
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::{
    self,
    Linkage
};
use inkwell::OptimizationLevel;
use inkwell::support::LLVMString;
use std::collections::HashMap;

pub struct Module<'a> {
    pub datatypes: HashMap<Symbol, DataType>,
    globals: Vec<Symbol>,
    module: module::Module<'a>,
    name: Symbol
}

impl<'a> Module<'a> {
    pub fn new(context: &'a Context, name: Symbol) -> Self {
        let datatypes = HashMap::<Symbol, DataType>::new();
        let globals = Vec::<Symbol>::new();
        let module = context.create_module(name.name());
        Self {
            datatypes: datatypes,
            globals: globals,
            module: module,
            name: name
        }
    }

    pub fn add_function(&self, name: &str, fn_type: FunctionType<'a>, link_type: Option<Linkage>) -> FunctionValue<'a> {
        self.module.add_function(name, fn_type, link_type)
    }

    pub fn add_global(&mut self, ty: BasicTypeEnum<'a>, addr_space: Option<AddressSpace>, name: &str) {
        self.globals.push(Symbol::new(name));
        self.module.add_global(ty, addr_space, name);
    }

    pub fn create_jit_execution_engine(&self, opt_level: OptimizationLevel) -> Result<ExecutionEngine<'a>, LLVMString> {
        self.module.create_jit_execution_engine(opt_level)
    }

    pub fn get_context(&self) -> ContextRef<'a> {
        self.module.get_context()
    }

    pub fn get_farnese_type(&self, name: Symbol) -> &DataType {
        self.datatypes.get(&name).unwrap()
    }

    pub fn get_function(&self, name: &str) -> Option<FunctionValue<'a>> {
        self.module.get_function(name)
    }

    pub fn get_module(&self) -> &module::Module<'a> {
        &self.module
    }

    pub fn get_name(&self) -> &Symbol {
        &self.name
    }

    pub fn get_struct_type(&self, name: &str) -> Option<StructType<'a>> {
        self.module.get_struct_type(name)
    }

    pub fn insert_farnese_type(&mut self, name: Symbol, ty: DataType) {
        self.datatypes.insert(name.clone(), ty);
    }

    pub fn print_to_file(&self, file_name: &str) {
        self.module.print_to_file(file_name).unwrap()
    }

    pub fn print_to_stderr(&self) {
        self.module.print_to_stderr()
    }
}
