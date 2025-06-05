use super::{DataType, Symbol};
use inkwell::{
    self, 
    AddressSpace,
    context::Context,
    module::{
        self,
        FunctionIterator,
        Linkage
    },
    types::{
        FunctionType,
        IntType,
        PointerType,
        StructType
    },
    values::{
        FunctionValue,
        GlobalValue
    }
};
use std::collections::HashMap;

// type Methods<'a> = HashMap<(Symbol, FunctionType<'a>), FunctionValue<'a>>;
// type Methods<'a> = HashMap<Symbol, FunctionValue<'a>>;
// type Types<'a> = HashMap<Symbol, StructType<'a>>;
type Types = HashMap<Symbol, DataType>;


#[derive(Clone, Debug)]
// pub struct Module<'a, 'b> {
//     builder: &'b Builder<'a>,
pub struct Module<'a> {
    context: &'a Context,
    // dependencies: Vec<Symbol>,
    exports: Vec<Symbol>,
    // methods: Methods<'a>,
    pub module: module::Module<'a>,
    name: Symbol,
    types: Types
}

// impl<'a, 'b> Module<'a, 'b> {
impl<'a> Module<'a> {
    // pub fn new(builder: &'b Builder<'a>, context: &'a Context, name: &str) -> Self {
    pub fn new(context: &'a Context, name: &str) -> Self {
        // let methods = Methods::<'a>::new();
        // let dependencies = Vec::<Symbol>::new();
        let exports = Vec::<Symbol>::new();
        let module = context.create_module(name);
        let symbol = Symbol::new(name);
        let types = Types::new();

        // need to add a few basic C methods to all modules I guess
        // TODO eventually move this to funcs that are stored in the Module type
        let printf_type = context
            .i8_type()
            .fn_type(&[
                    context.i8_type().ptr_type(AddressSpace::default()).into()
                ], 
                true
            );

        let _ = module
            .add_function("printf", printf_type, Some(Linkage::External));

        Self {
            // builder: builder,
            context: context,
            // dependencies: dependencies,
            exports: exports,
            // methods: methods,
            module: module,
            name: symbol,
            types: types
        }
    }

    pub fn add_function(&self, name: &str, func: FunctionType<'a>, opt: Option<Linkage>) -> FunctionValue<'a> {
        self.module.add_function(name, func, opt)
    }

    pub fn add_global(
        &self, 
        datatype: StructType<'a>, 
        address_space: Option<AddressSpace>,
        name: &str
    ) -> GlobalValue<'a> {
        self.module.add_global(datatype, address_space, name)
    }

    // pub fn get_builder(&self) -> &'b Builder<'a> {
    //     &self.builder
    // }

    pub fn get_context(&self) -> &'a Context {
        &self.context
    }

    pub fn get_exports(&self) -> &Vec<Symbol> {
        &self.exports
    }

    pub fn get_function(&self, name: &str) -> FunctionValue<'a> {
        self.module.get_function(name)
            .expect(
                format!(
                    "Function {} not found in module {}.\nAvailable functions are {:?}", 
                    name, self.name, self.module.get_functions()
                ).as_str()
            )
    }

    pub fn get_functions(&self) -> FunctionIterator<'a> {
        self.module.get_functions()
    }

    pub fn get_global(&self, name: &str) -> GlobalValue<'a> {
        self.module.get_global(name).unwrap()
    }

    pub fn get_type(&self, sym: &str) -> &DataType {
        let sym = Symbol::new(sym);
        &self.types.get(&sym)
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

    pub fn link(&self, module: &Module<'a>) {
        self.module.link_in_module(module.module.clone()).unwrap();
        // types first
        // for export in module.get_exports().iter() {
        //     let datatype = module.get_types().get(export);
        //     let _ = match datatype {
        //         Some(x) => {
                    
        //         },
        //         None => {}
        //     };
        // }
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

    pub fn push_export(&mut self, name: Symbol) {
        self.exports.push(name);
    }
}
