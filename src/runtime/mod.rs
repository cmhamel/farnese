use crate::core::{
    Core,
    Module,
    Symbol
};
use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::context::Context;
use std::collections::HashMap;


pub struct Runtime<'a, 'b> {
    modules: HashMap<Symbol, Module<'a, 'b>>
}

// when booting up the runtime, we should first
// load core
// then for each module
// first load up the types into the type cache
//  - maybe organize this by module somehow.. maybe just append 
//    the module symble to the type name
// then compile the methods 

impl<'a, 'b> Runtime<'a, 'b> {
    pub fn new(builder: &'b Builder<'a>, context: &'a Context) -> Self {
        let mut modules = HashMap::<Symbol, Module<'a, 'b>>::new();
        // add core module
        let mut core = Core::new(&builder, &context);
        let _ = core.bootstrap();
        modules.insert(core.module.name(), core.module);

        // create main module
        let main = Module::<'a, 'b>::new(builder, context, "Main");
        let _ = main.link(modules.get(&Symbol::new("Core")).unwrap());
        modules.insert(main.name(), main);
        Self {
            modules: modules
        }
    }

    pub fn initialize_type_table(&self, builder: &'b Builder<'a>, context: &'a Context) {
        let module = self.modules.get(&Symbol::new("Main")).unwrap();

        let datatype = module.get_struct_type("DataType");
        // let datatype_ptr = datatype.ptr_type(AddressSpace::default());
        let datatype_array = datatype.array_type(1024);
        let datatype_array_ptr = datatype_array.ptr_type(AddressSpace::default());
        let func = datatype_array_ptr.fn_type(&[], false);
        let func = module.add_function("__init_type_table", func, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        let ptr = builder.build_malloc(datatype_array, "").unwrap();

        let _ = builder.build_return(Some(&ptr));
    }

    pub fn main_func_begin(&self, builder: &'b Builder<'a>, module: &Module<'a, 'b>) {
        let context = module.get_context();
        let fn_type = context.void_type().fn_type(&[], false);
        let function = module.add_function("main", fn_type, None);
        let entry = context.append_basic_block(function, "entry");
        builder.position_at_end(entry);
    }

    pub fn main_func_end(&self, builder: &'b Builder<'a>) {
        let _ = builder.build_return(None);
    }

    pub fn modules(&self) -> &HashMap<Symbol, Module<'a, 'b>> {
        &self.modules
    }

    pub fn using_module(&self) -> () {

    }
}

#[cfg(test)]
mod tests {
    use crate::core::{DataType, LLVMAlloca};
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_runtime_new() {
        let context = Context::create();
        let builder = context.create_builder();
        let runtime = Runtime::new(&builder, &context);
    }

    #[test]
    fn test_runtime_main() {
        let context = Context::create();
        let builder = context.create_builder();
        let zero = context.i32_type().const_zero();
        let runtime = Runtime::new(&builder, &context);
        let _ = runtime.initialize_type_table(&builder, &context);
        let core = runtime.modules().get(&Symbol::new("Core")).unwrap();
        let module = runtime.modules().get(&Symbol::new("Main")).unwrap();
        let _ = runtime.main_func_begin(&builder, &module);

        // let sym = Symbol::new("Any");
        // let sym_ptr = sym.emit_ir_alloca(&builder, &module.module);
        // let _ = sym.emit_ir_printf(&builder, &module.module, &sym_ptr);

        // let datatype = DataType::new(sym.clone(), sym, false, false, false);
        // let ptr = datatype.emit_ir_alloca(&builder, &module.module);
        let func = module.get_function("__init_type_table");
        let type_table_ptr = builder.build_call(func, &[], "")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // make initial stuff
        let field_types = Box::new(Vec::<DataType>::new());
        let datatype = DataType::from_str("DataType", "DataType", false, false, false, field_types.clone());
        let any = DataType::from_str("Any", "Any", false, false, false, field_types.clone());
        let symbol = DataType::from_str("Symbol", "Any", false, false, false, field_types.clone());

        let datatype_ptr = datatype.emit_ir_alloca(&module);
        let any_ptr = any.emit_ir_alloca(&module);
        let symbol_ptr = symbol.emit_ir_alloca(&module);

        let type_ptrs = [datatype_ptr, any_ptr, symbol_ptr];

        for (n, type_ptr) in type_ptrs.iter().enumerate() {
            let struct_ptr = unsafe { builder
                .build_in_bounds_gep(
                    type_table_ptr,
                    &[
                        zero, context.i32_type().const_int(
                        n.try_into().unwrap(), false
                    )],
                    ""
                )
                .unwrap()
                // .build_struct_gep(type_table_ptr, n.try_into().unwrap(), "")
                // .unwrap();
            };
        }

        let _ = builder.build_free(type_table_ptr).unwrap();
        let _ = runtime.main_func_end(&builder);
        let _ = module.module.print_to_file("runtime.ll");
    }
}