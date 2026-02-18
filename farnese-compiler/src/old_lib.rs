use farnese_core_v3::{
    datatype::DataType,
    modules::Module,
    primitives::Primitive,
    symbol::Symbol,
    traits::*,
    value::PossibleVals
};
use farnese_lexer::{
    ast::Expr,
    // lexer
    parser
};
use inkwell::builder::Builder;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{
    BasicMetadataValueEnum,
    BasicValueEnum,
    PointerValue
};
use std::collections::HashMap;

/// scope
type Scope<'a> = HashMap<Symbol, (Value<'a>, DataType)>;
/// a stack of LLVM values
type Stack<'a> = Vec<(Value<'a>, DataType)>;
/// basic value type
type Value<'a> = BasicMetadataValueEnum<'a>;


pub struct Compiler<'a> {
    pub scope: Scope<'a>,
    pub stack: Stack<'a>
}

impl<'a, 'b> Compiler<'a> {
    pub fn new() -> Self {
        Self {
            scope: Scope::<'a>::new(),
            stack: Stack::<'a>::new()
        }
    }

    pub fn eval(
        &mut self, 
        builder: &'b Builder<'a>,
        module: &mut Module<'a>, 
        expr: Expr
    ) -> Option<PossibleVals<'a>> {
        // println!("Expr = {:?}", expr);
        match expr {
            // Expr::AbstractType { name, supertype } => {
            //     let datatype = DataType::create_abstract_type(&name, &supertype);
            //     module.insert_farnese_type(datatype.name(), datatype);
            //     None
            // }
            Expr::Assignment { symbol, expr } => {
                // let sym = Symbol::new(*symbol);
                let string = match *symbol {
                    Expr::Symbol(x) => x,
                    _ => unreachable!()
                };
                let sym = Symbol::new(&string);
                let val = self.eval(builder, module, *expr).unwrap();
                // println!("Expr = {:?}", expr);
                // check whether we need to create a new var
                let ptr = if self.scope.contains_key(&sym) {
                    self.scope.get(&sym).unwrap().0
                        .try_into().unwrap()
                } else {    
                    // val.alloca(builder, module)
                    match val {
                        // Some(x) => match x {
                        // PossibleVals::BasicMetadataValueEnum(x) => {
                        //     val.into_pointer_value()
                        // },
                        // _ => val.alloca(builder, module)
                        // None => val.alloca(builder, module)
                        _ => todo!()
                    }
                };
                let _ = val.store(&builder, &module, ptr);
                let datatype = val.farnese_type(module);
                let tup = (ptr.try_into().unwrap(), datatype);
                self.scope.insert(sym, tup.clone());
                self.stack.push(tup);
                None
            },
            // Expr::Empty => {
            //     // do nothing
            //     None
            // }
            // Expr::Exports { symbols } => {
            //     println!("Module exports are");
            //     for sym in symbols.iter() {
            //         println!("  {:?}", sym);
            //     }
            //     None
            // }
            // Expr::Function { name, args, return_type, body } => {
            //     self.compile_function(module, &name, *args, Symbol::new(&return_type), *body);
            //     None
            // }
            // Expr::MethodCall { name, args } => {
            //     self.compile_method_call(builder, module, &name, *args)
            // }
            // Expr::Primitive(x) => {
            //     let x: Primitive = x.into();
            //     Some(x.into())
            // }
            // Expr::PrimitiveType {
            //     name,
            //     supertype,
            //     bits,
            // } => {
            //     let datatype = DataType::create_primitive_type(&name, &supertype, bits.try_into().unwrap());
            //     module.insert_farnese_type(datatype.name(), datatype);
            //     None
            // }
            Expr::Symbol(x) => {
                let (val, ty) = self.scope.get(&Symbol::new(&x)).unwrap();
                Some((*val).into())
            }
            _ => todo!("EXPR = {:?}", expr)
        }
    }

    pub fn include(
        &mut self, 
        builder: &'b Builder<'a>,
        module: &mut Module<'a>, 
        file_name: &str
    ) {
        let ast: Vec<_> = parser::parse_file(file_name).unwrap();
        // let builder = module.get_context().create_builder();
        for expr in ast {
            panic!("TODO")
            // println!("AST = {:#?}")
            // match Expr {
            //     Expr::Module { name: _, exprs } => {
            //         println!("Got a module");
            //         for ast in exprs.iter() {
            //             self.eval(builder, module, ast.clone());
            //         }
            //     },
            //     _ => {
            //         self.eval(builder, module, Expr);
            //     }
            // }
        }
    }

    // fn compile_function(
    //     &mut self,
    //     module: &mut Module<'a>,
    //     name: &str, args: Vec<Expr>, 
    //     return_type: Symbol, body: Vec<Expr>
    // ) {
    //     self.scope.clear();
    //     let builder = module.get_context().create_builder();
    //     let void_type = module.get_context().void_type();
    //     // let arg_val_types: Vec<BasicMetadataTypeEnum> = args.iter()
    //     let arg_val_fts = args
    //         .iter()
    //         .map(|x| match x {
    //             Expr::FunctionArg { name: _, arg_type } => {
    //                 let sym = Symbol::new(arg_type);
    //                 module.get_farnese_type(sym)
    //             },
    //             _ => todo!()
    //         })
    //         .collect::<Vec<_>>();
    //     let arg_val_types = arg_val_fts
    //         .iter()
    //         .map(|x| x.get_llvm_type(module))
    //         .collect::<Vec<_>>();
    //     let fn_type = void_type.fn_type(&arg_val_types, false);
    //     // TODO need to change name since we'll have name collision

    //     let function = module.add_function(name, fn_type, None);
    //     let entry = module.get_context().append_basic_block(function, "entry");
    //     builder.position_at_end(entry);

    //     // let mut arg_vals = HashMap::<String, BasicValueEnum<'a>>::new();
    //     args
    //         .into_iter()
    //         .enumerate()
    //         .map(|(i, x)| match x {
    //             Expr::FunctionArg { name, arg_type: _ } => {
    //                 let func_inp = function.get_nth_param(i.try_into().unwrap()).unwrap();
    //                 // arg_vals.insert(name, func_inp);
    //                 // self.scope.insert(Symbol::new(&name), func_inp)
    //                 let datatype = arg_val_fts[i];
    //                 println!("Adding var {} to scope", name);
    //                 self.scope.insert(Symbol::new(&name), (func_inp.into(), datatype.clone()));
    //                 println!("Scope currently = {:?}", self.scope);
    //             },
    //             _ => todo!()
    //         })
    //         .collect::<Vec<_>>();

    //     for expr in body.iter() {
    //         println!("Expr = {:?}", expr);
    //         self.eval(&builder, module, expr.clone());
    //     }
    //     println!("Hur");
    //     // TODO hardcoding everything to return void for now
    //     let _ = builder.build_return(None);
    //     module.print_to_stderr();

    //     self.scope.clear()
    // }

    fn compile_method_call(
        &self, 
        builder: &'b Builder<'a>, module: &Module<'a>,
        name: &str, args: Vec<Expr>
    ) -> Option<PossibleVals<'a>> {
        // list of os methods that might be exposed
        let os_methods = vec![
            "fprintf",
            "printf"
        ];
        // TODO figure out return types

        if os_methods.contains(&name) {
            println!("This is a special method");
            self.compile_os_method_call(builder, module, name, args)
        } else {
            self.compile_farnese_method_call(builder, module, name, args)
        }

        None
    }

    fn compile_farnese_method_call(
        &self, 
        _builder: &'b Builder<'a>, _module: &Module<'a>,
        _name: &str, args: Vec<Expr>
    ) {
        // let arg_vals_in = self.load_method_arg_ptrs(builder, module, args);
        // let contents = module.get_context();
        // let arg_llvm_types = args.iter()
        //     .map(|x| )
        // for arg in args.iter() {
        //     println!("FUNC ARG = {:?}", arg)
        // }

        println!("Arg vals = {:?}", args);
        // let arg_vals = vec![];
        for i in 0..args.len() {
            // arg_vals.push()
        }
        // let arg_vals = args.iter()
    }

    fn compile_os_method_call(
        &self, 
        builder: &'b Builder<'a>, module: &Module<'a>,
        name: &str, args: Vec<Expr>
    ) {
        // let arg_vals = args
        //     .iter()
        //     .map(|x| match &*x {
        //         // TODO currently not using the arg type here
        //         Expr::FunctionArg { name, arg_type: _ } => Symbol::new(&name),
        //         _ => todo!("Not supported"),
        //     })
        //     .map(|x| self.scope.get(&x).unwrap().clone())
        //     // .map(|x| x.0) // only need first val
        //     .collect::<Vec<_>>();
        let arg_vals_in = self.load_method_arg_ptrs(builder, module, args);
        match name {
            "fprintf" => panic!(),
            "printf" => {
                println!("Args = {:?}", arg_vals_in);
                // panic!()
                // let arg_vals = vec![self.]
                // builder.build_call(
                //     module.get_function("printf").unwrap(),
                //     &[

                //     ]
                // )
            },
            _ => panic!()
        }
    }

    // TODO I think this one is dumb maybe
    fn load_method_arg_ptrs(
        &self,
        _builder: &'b Builder<'a>, _module: &Module<'a>,
        args: Vec<Expr>
    ) -> Vec<BasicMetadataValueEnum<'a>> {
        args
            .iter()
            .map(|x| match &*x {
                // TODO currently not using the arg type here
                // Expr::FunctionArg { name, arg_type: _ } => Symbol::new(&name),
                _ => todo!("Not supported"),
            })
            .map(|x| match self.scope.get(&x) {
                Some(contents) => contents.0,
                None => panic!("Add special case for globals")
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ffi::c_char;
    use farnese_lexer::ast::Expr;
    pub use inkwell::AddressSpace;
    pub use inkwell::builder::Builder;
    pub use inkwell::context::Context;
    use inkwell::OptimizationLevel;
    use std::fs::File;
    use std::io::Read;
    use std::ptr;
    // use std::sync::Mutex;

    // TODO move to some dev dep eventually
    pub struct TestHelper<'a> {
        pub builder: Builder<'a>,
        pub module: Module<'a>
    }

    impl<'a> TestHelper<'a> {
        pub fn new(context: &'a Context, name: &str) -> Self {
            let ptr_type = context.ptr_type(AddressSpace::default());
            let builder = context.create_builder();
            // let module = context.create_module(name);
            let module = Module::new(context, Symbol::new(name));
            
            // TODO eventually move a lot of below
            // to the module type once we figured out
            // what we need and where it should be
            let printf_type = context.i32_type()
                .fn_type(&[ptr_type.into()], true);
            module.add_function("printf", printf_type, None);

            let ptr_type = context.ptr_type(AddressSpace::default());
            let memcpy_ty = context.void_type().fn_type(
                &[
                    ptr_type.into(), // dest
                    ptr_type.into(), // src
                    context.i64_type().into(),    // size
                    context.bool_type().into(), // is_volatile
                ],
                false,
            );
            // TODO figure out what the characters mean
            module.add_function("llvm.memcpy.p0.p0.i64", memcpy_ty, None);

            Symbol::create_opaque_type(&builder, &module, "Symbol");
            Symbol::set_opaque_type_init(&builder, &module);

            DataType::create_opaque_type(&builder, &module, "DataType");
            DataType::set_opaque_type_init(&builder, &module);

            Self {
                builder: builder,
                module: module
            }
        }

        pub fn start(&self) {
            let context = self.module.get_context();
            let fn_type = context.i32_type().fn_type(&[], false);
            let function = self.module.add_function("main", fn_type, None);
            let entry = context.append_basic_block(function, "entry");
            self.builder.position_at_end(entry);
        }

        pub fn end(&self) {
            let zero = self.module.get_context().i32_type().const_int(0, false);
            let _ = self.builder.build_return(Some(&zero));
        }

        pub fn run(&self) -> String {
            let engine = self
                .module
                .create_jit_execution_engine(OptimizationLevel::None)
                .expect("Failed to create execution engine");
            let string = unsafe {
                // === Redirect stdout ===
                // Save current stdout
                let stdout_fd = libc::dup(libc::STDOUT_FILENO);
                let tmpfile = tempfile::NamedTempFile::new().unwrap();
                let tmpfile_path = tmpfile.path().to_owned();
                let tmp_fd = libc::open(
                    tmpfile_path.to_str().unwrap().as_ptr() as *const c_char,
                    libc::O_WRONLY,
                );
    
                // Redirect stdout to tmp file
                libc::dup2(tmp_fd, libc::STDOUT_FILENO);
    
                // Run JITted `main`
                let addr = engine.get_function_address("main").unwrap();
                let main_fn = std::mem::transmute::<usize, extern "C" fn() -> i32>(addr as usize);
                let _ret_code = main_fn();
    
                // Flush stdout
                libc::fflush(ptr::null_mut());
    
                // Restore original stdout
                libc::dup2(stdout_fd, libc::STDOUT_FILENO);
                libc::close(tmp_fd);
    
                // Read output from temp file
                let mut file = File::open(tmpfile_path).unwrap();
                let mut output = String::new();
                file.read_to_string(&mut output).unwrap();
    
                // assert_eq!(output.trim(), "Hello, LLVM!");
                output.trim().to_string()
            };
            string
        }

        pub fn show_ir(&self) {
            println!("Current IR:");
            self.module.print_to_stderr();
            println!("End of current IR\n\n\n")
        }
    }

    #[test]
    fn test_primitives() {
        let context = Context::create();
        let mut test = TestHelper::new(&context, "test_primitives");
        test.start();
        let expr = Expr::from_string("a = 1 + (2 + 3 * 2)");
        println!("Expr = {:#?}", expr);
        test.end();
        test.show_ir();
        let results = test.run();
        println!("Results = {:?}", results);
    }

    // #[test]
    // fn test_simple_stuff() {
    //     let context = Context::create();
    //     let mut test = TestHelper::new(&context, "test_simple_stuff");

    //     test.start();

    //     let mut compiler = Compiler::new();
    //     compiler.include(&test.builder, &mut test.module, "src/core.jl");

    //     for (_name, dt) in test.module.datatypes.iter() {
    //         // println!("Name = {:?}", name);
    //         // println!("Dt = {:?}", dt);
    //         let ptr = dt.alloca(&test.builder, &test.module);
    //         // let val = dt.llvm_value(&test.builder, &test.module);
    //         let _ = dt.store(&test.builder, &test.module, ptr);
    //         let name_sym = test.builder.build_struct_gep(
    //             dt.name.llvm_type(&test.builder, &test.module), ptr, 0, ""
    //         ).unwrap();
    //         let _ = dt.name.printf(&test.builder, &test.module, name_sym);
    //     }

    //     let val: Primitive = (69 as i64).into();
    //     let ptr = val.alloca(&test.builder, &test.module);
    //     let _ = val.store(&test.builder, &test.module, ptr);
    //     // let ft = val.farnese_type(&test.module);
    //     // println!("FT = {:?}", ft);

    //     test.end();
    //     test.show_ir();
    //     let results = test.run();
    //     println!("Results = {}", results);
    // }
}
