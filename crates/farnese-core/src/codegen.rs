use crate::datatype::DataType;
use crate::value::Value;
use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::Module,
    values::FunctionValue
};
use std::collections::HashMap;

pub struct CodeGen<'ctx> {
    pub builder: Builder<'ctx>,
    pub context: &'ctx Context,
    /// Current function being compiled
    pub current_fn: Option<FunctionValue<'ctx>>,
    pub datatypes: HashMap<String, DataType>,
    /// Map from function names to LLVM functions
    pub functions: HashMap<String, FunctionValue<'ctx>>,
    pub module: Module<'ctx>,
    /// Map from variable names to their stack allocations
    // pub variables: HashMap<String, PointerValue<'ctx>>,
    pub variables: HashMap<String, Value<'ctx>>

}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let mut functions = HashMap::<String, FunctionValue<'ctx>>::new();
        let printf_fn = context.i32_type().fn_type(
            &[context.ptr_type(AddressSpace::default()).into()],
            true
        );
        let printf_fn = module.add_function("printf", printf_fn, None);
        functions.insert("printf".to_string(), printf_fn);

        // helpful strs
        // builder.build_global_string_ptr("%.4f\0", "");
        // builder.build_global_string_ptr("%ldd\0", "");

        Self {
            builder: builder,
            context: context,
            current_fn: None,
            datatypes: HashMap::<String, DataType>::new(),
            // functions: HashMap::<String, FunctionValue<'ctx>>::new(),
            functions: functions,
            module: module,
            // variables: HashMap::<String, PointerValue<'ctx>>::new()
            variables: HashMap::<String, Value<'ctx>>::new()
        }
    }
}
