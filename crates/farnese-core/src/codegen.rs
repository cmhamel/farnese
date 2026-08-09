use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    values::{FunctionValue, PointerValue}
};
use std::collections::HashMap;

pub struct CodeGen<'ctx> {
    pub builder: Builder<'ctx>,
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    /// Current function being compiled
    pub current_fn: Option<FunctionValue<'ctx>>,
    /// Map from function names to LLVM functions
    pub functions: HashMap<String, FunctionValue<'ctx>>,
    /// Map from variable names to their stack allocations
    pub variables: HashMap<String, PointerValue<'ctx>>,

}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        Self {
            builder: builder,
            context: context,
            module: module,
            current_fn: None,
            functions: HashMap::<String, FunctionValue<'ctx>>::new(),
            variables: HashMap::<String, PointerValue<'ctx>>::new()
        }
    }

    // pub fn get_builder(&mut self) -> &mut Builder<'ctx> { &mut self.builder }
}
