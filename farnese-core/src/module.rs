use super::{DataType, Symbol};
use inkwell::{
    self, AddressSpace, OptimizationLevel,
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::{self, FunctionIterator, Linkage},
    types::{FunctionType, IntType, PointerType, StructType},
    values::{FunctionValue, GlobalValue},
};
use std::collections::HashMap;

// type Methods<'a> = HashMap<(Symbol, FunctionType<'a>), FunctionValue<'a>>;
// type Methods<'a> = HashMap<Symbol, FunctionValue<'a>>;
type ArgTypes = Vec<DataType>;
type Exports = Vec<Symbol>;
type ReturnType = DataType;
type MethodTable = HashMap<Symbol, (ArgTypes, ReturnType)>;
// type Methods = HashMap<Symbol, Vec<
type Types = HashMap<Symbol, DataType>;

#[derive(Clone, Debug)]
pub struct Module<'a> {
    context: &'a Context,
    // dependencies: Vec<Symbol>,
    exports: Exports,
    methods: MethodTable,
    module: module::Module<'a>,
    name: Symbol,
    types: Types,
}

impl<'a> Module<'a> {
    pub fn new(context: &'a Context, name: &str) -> Self {
        let exports = Exports::new();
        let methods = MethodTable::new();
        let module = context.create_module(name);
        let symbol = Symbol::new(name);
        let types = Types::new();

        // need to add a few basic C methods to all modules I guess
        // TODO eventually move this to funcs that are stored in the Module type
        let printf_type = context.i8_type().fn_type(
            &[context.i8_type().ptr_type(AddressSpace::default()).into()],
            true,
        );

        let _ = module.add_function("printf", printf_type, Some(Linkage::External));

        Self {
            context: context,
            exports: exports,
            methods: methods,
            module: module,
            name: symbol,
            types: types,
        }
    }

    pub fn add_function(
        &self,
        name: &str,
        func: FunctionType<'a>,
        opt: Option<Linkage>,
    ) -> FunctionValue<'a> {
        self.module.add_function(name, func, opt)
    }

    pub fn add_global(
        &self,
        datatype: StructType<'a>,
        address_space: Option<AddressSpace>,
        name: &str,
    ) -> GlobalValue<'a> {
        self.module.add_global(datatype, address_space, name)
    }

    pub fn create_builder(&self) -> Builder<'a> {
        self.get_context().create_builder()
    }

    pub fn create_jit_execution_engine(&self, opt_level: OptimizationLevel) -> ExecutionEngine {
        self.module.create_jit_execution_engine(opt_level).unwrap()
    }

    pub fn get_context(&self) -> &'a Context {
        &self.context
    }

    pub fn get_function(&self, name: &str) -> FunctionValue<'a> {
        self.module.get_function(name).expect(
            format!(
                "Function {} not found in module {}.\nAvailable functions are {:?}",
                name,
                self.name,
                self.module.get_functions()
            )
            .as_str(),
        )
    }

    pub fn get_exports(&self) -> &Exports {
        &self.exports
    }

    pub fn get_functions(&self) -> FunctionIterator<'a> {
        self.module.get_functions()
    }

    pub fn get_global(&self, name: &str) -> GlobalValue<'a> {
        self.module.get_global(name).unwrap()
    }

    pub fn get_type(&self, sym: &str) -> &DataType {
        let sym = Symbol::new(sym);
        &self
            .types
            .get(&sym)
            .expect(format!("Type {} not found in module {}", sym, self.name).as_str())
    }

    pub fn get_types(&self) -> &Types {
        &self.types
    }

    pub fn get_struct_type(&self, name: &str) -> StructType<'a> {
        self.module.get_struct_type(name).unwrap()
    }

    pub fn i8_type(&self) -> IntType {
        self.context.i8_type()
    }

    pub fn i8_ptr_type(&self) -> PointerType {
        self.i8_type().ptr_type(AddressSpace::default())
    }

    pub fn i32_type(&self) -> IntType {
        self.context.i32_type()
    }

    pub fn i32_ptr_type(&self) -> PointerType {
        self.i32_type().ptr_type(AddressSpace::default())
    }

    pub fn i64_type(&self) -> IntType {
        self.context.i64_type()
    }

    pub fn i64_ptr_type(&self) -> PointerType {
        self.i64_type().ptr_type(AddressSpace::default())
    }

    pub fn insert_type(&mut self, datatype: DataType) {
        self.types.insert(datatype.name().clone(), datatype);
    }

    pub fn link(&mut self, module: &Module<'a>) {
        self.module.link_in_module(module.module.clone()).unwrap();

        for k in module.get_exports() {
            // first try assuming this key is a type
            if self.types.contains_key(k) {
                panic!(
                    "Conflicting type {} in modules {} and {}",
                    k,
                    self.name(),
                    module.name()
                )
            } else {
                // TODO note this will error if the type doesn't exist
                // need to change this to use a Result<>
                self.insert_type(module.get_type(k.name()).clone())
            }

            // TODO need to handle globals, functions, etc.
        }
    }

    pub fn module(&self) -> &module::Module<'a> {
        &self.module
    }

    pub fn name(&self) -> Symbol {
        self.name.clone()
    }

    pub fn print_to_file(&self, name: &str) {
        self.module.print_to_file(name).unwrap()
    }

    pub fn push_export(&mut self, sym: Symbol) {
        self.exports.push(sym)
    }
}
