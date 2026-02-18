use farnese_core_v3::{
    datatype::DataType,
    modules::Module,
    primitives::Primitive,
    symbol::Symbol,
    traits::*,
    value::PossibleVal
};
use farnese_lexer::ast::Expr;
use farnese_lexer::parser;
use inkwell::{
    builder::Builder,
    // context::Context,
    types::BasicMetadataTypeEnum,
    values::BasicMetadataValueEnum
};
use std::collections::HashMap;
/// scope
type Scope<'a> = HashMap<Symbol, (LLVMValue<'a>, DataType)>;
/// a stack of LLVM values
type Stack<'a> = Vec<(LLVMValue<'a>, DataType)>;
/// basic value type
type LLVMValue<'a> = BasicMetadataValueEnum<'a>;

fn create_addition_methods<'a, 'b>(builder: &'b Builder<'a>, module: &'b Module<'a>) {
    let context = module.get_context();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();

    // f32 addition
    let func = f32_type.fn_type(
        &[f32_type.into(), f32_type.into()], false
    );
    let func = module.add_function("__add_float32", func, None);
    let top = context.append_basic_block(func, "top");
    builder.position_at_end(top);
    let a = func.get_nth_param(0).unwrap().into_float_value();
    let b = func.get_nth_param(1).unwrap().into_float_value();
    let sum = builder.build_float_add(a, b, "").unwrap();
    builder.build_return(Some(&sum));

    // f64 addition
    let func = f64_type.fn_type(
        &[f64_type.into(), f64_type.into()], false
    );
    let func = module.add_function("__add_float64", func, None);
    let top = context.append_basic_block(func, "top");
    builder.position_at_end(top);
    let a = func.get_nth_param(0).unwrap().into_float_value();
    let b = func.get_nth_param(1).unwrap().into_float_value();
    let sum = builder.build_float_add(a, b, "").unwrap();
    builder.build_return(Some(&sum));

    // i32 addition
    let func = i32_type.fn_type(
        &[i32_type.into(), i32_type.into()], false
    );
    let func = module.add_function("__add_int32", func, None);
    let top = context.append_basic_block(func, "top");
    builder.position_at_end(top);
    let a = func.get_nth_param(0).unwrap().into_int_value();
    let b = func.get_nth_param(1).unwrap().into_int_value();
    let sum = builder.build_int_add(a, b, "").unwrap();
    builder.build_return(Some(&sum));

    // i64 addition
    let func = i64_type.fn_type(
        &[i64_type.into(), i64_type.into()], false
    );
    let func = module.add_function("__add_int64", func, None);
    let top = context.append_basic_block(func, "top");
    builder.position_at_end(top);
    let a = func.get_nth_param(0).unwrap().into_int_value();
    let b = func.get_nth_param(1).unwrap().into_int_value();
    let sum = builder.build_int_add(a, b, "").unwrap();
    builder.build_return(Some(&sum));
}

fn create_convert_methods<'a, 'b>(builder: &'b Builder<'a>, module: &'b Module<'a>) {
    let context = module.get_context();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();

    // i32 -> f32
    let func = f32_type.fn_type(&[i32_type.into()], false);
    let func = module.add_function("__convert_i32_to_f32", func, None);
    let top = context.append_basic_block(func, "top");
    builder.position_at_end(top);
    let a = func.get_nth_param(0).unwrap().into_int_value();
    let a = builder.build_signed_int_to_float(a, f32_type, "").unwrap();    
    builder.build_return(Some(&a));

    // i64 -> f64
    let func = f64_type.fn_type(&[i64_type.into()], false);
    let func = module.add_function("__convert_i64_to_f64", func, None);
    let top = context.append_basic_block(func, "top");
    builder.position_at_end(top);
    let a = func.get_nth_param(0).unwrap().into_int_value();
    let a = builder.build_signed_int_to_float(a, f64_type, "").unwrap();    
    builder.build_return(Some(&a));
}

#[derive(Clone, Debug)]
pub enum FarneseExpr {
    DataType(DataType),
    Empty,
    Function(Function)
}

// struct MethodSignature {
//     args: Vec<(Symbol, DataType)>,
//     // body: Vec<AbstractType>,
//     ret: DataType
// }
#[derive(Clone, Debug)]
struct Function {
    name: Symbol,
    methods: HashMap<Vec<DataType>, DataType>
}

pub struct Compiler<'a> {
    pub builder: Builder<'a>,
    pub module: Module<'a>,
    scope: Scope<'a>,
    stack: Stack<'a>
}

impl<'a> Compiler<'a> {
    pub fn new(builder: Builder<'a>, module: Module<'a>) -> Self {
        create_addition_methods(&builder, &module);
        create_convert_methods(&builder, &module);

        Self {
            builder: builder,
            module: module,
            scope: Scope::<'a>::new(),
            stack: Stack::<'a>::new()
        }
    }

    pub fn ast_to_core(&self, expr: Expr) -> FarneseExpr {
        match expr {
            Expr::AbstractType { name, supertype } => {
                let supertype = match *supertype {
                    Expr::Symbol(x) => x,
                    _ => todo!()
                };
                FarneseExpr::DataType(
                    DataType::create_abstract_type(name.as_str(), &supertype)
                )
            }
            Expr::Empty => {
                FarneseExpr::Empty
            }
            Expr::PrimitiveType { name, supertype, bits } => {
                let supertype = match *supertype {
                    Expr::Symbol(x) => x,
                    _ => todo!()
                };
                FarneseExpr::DataType(
                    DataType::create_primitive_type(name.as_str(), &supertype, bits)
                )
            }
            _ => todo!("Expr = {:#?}", expr)
        }
    }

    pub fn compile_primitive(&mut self, expr: Expr) {
        let val: PossibleVal = expr.into();
        let ty = match val {
            PossibleVal::Primitive(ref x) => {
                match x {
                    Primitive::Float64(_) => self.module.datatypes.get(&Symbol::new("Float64")),
                    Primitive::Int32(_) => self.module.datatypes.get(&Symbol::new("Int32")),
                    Primitive::Int64(_) => self.module.datatypes.get(&Symbol::new("Int64")),
                }
            }
            _ => panic!()
        }.unwrap();
        let ptr = val.alloca(&self.builder, &self.module);
        let _ = val.store(&self.builder, &self.module, ptr);
        self.stack.push((ptr.try_into().unwrap(), ty.clone()))
    }

    pub fn compile_symbol(&self, expr: Expr) -> Symbol {
        match expr {
            Expr::Symbol(x) => Symbol::new(&x),
            _ => todo!()
        }
    }

    pub fn eval(&mut self, expr: &str) {
        let expr = Expr::from_string(expr);
        self.eval_expr(expr)
    }

    fn eval_expr(&mut self, expr: Expr) {
        let context = self.module.get_context();
        match expr {
            Expr::AbstractType { name, supertype } => {
                // let name = self.compile_symbol(*name);
                let supertype = self.compile_symbol(*supertype);
                let dt = DataType::create_abstract_type(name.as_str(), supertype.name());
                self.module.datatypes.insert(Symbol::new(&name), dt);  
            }
            Expr::Assignment { symbol, expr } => {
                let sym = self.compile_symbol(*symbol);
                self.eval_expr(*expr);
                self.scope.insert(sym, self.stack.pop().unwrap());
            }
            Expr::Call { func, args } => {
                // self.compile_method_call(expr);
                let func = self.module.get_function(&func)
                    .expect(format!("Failed to load function {}", func).as_str());
                let args = args.iter()
                    .map(|x| match x {
                        Expr::Call { .. } => {
                            self.eval_expr(x.clone());
                            self.stack.pop().unwrap().0
                        },
                        Expr::Symbol(y) => self.scope.get(&Symbol::new(&y)).unwrap().0,
                        _ => todo!("Arg = {:#?}", x)
                    })
                    .collect::<Vec<_>>();
                let call: LLVMValue = self.builder
                    .build_call(func, &args, "")
                    .expect("Call failed")
                    .try_as_basic_value()
                    .basic().unwrap().into();
                
                let any_type = self.module.datatypes.get(&Symbol::new("Any")).unwrap();
                
                println!("Call = {:#?}", call);
                // panic!()
            }
            Expr::Empty => {

            }
            Expr::Method { func, args, body, ret } => {
                // let's empty scope here...
                self.scope.clear();

                let func = self.compile_symbol(Expr::Symbol(func));
                let arg_types = args.iter()
                    .map(|x| match x {
                        Expr::Val { val, ty } => {
                            match &**ty {
                                Expr::Type(y) => y,
                                _ => todo!()
                            }
                        }
                        _ => todo!("arg = {:#?}", x)
                    })
                    .collect::<Vec<_>>();
                let llvm_types: Vec<BasicMetadataTypeEnum> = arg_types.iter()
                    .map(|x| match x.as_str() {
                        "Int64" => context.i64_type().try_into().unwrap(),
                        _ => todo!()
                    })
                    .collect();
                let mut arg_map = HashMap::<Symbol, usize>::new();
                args.iter().enumerate()
                    .map(|(i, x)| match x {
                        Expr::Val { val, ty } => {
                            let val = match &**val {
                                Expr::Symbol(x) => Symbol::new(&x),
                                _ => todo!()
                            };
                            println!("Val = {:#?}", val);
                            arg_map.insert(val, i);
                        }
                        _ => todo!()
                    })
                    .collect::<Vec<_>>();

                // TODO create return type
                // let i64_type = context.i64_typ`e();
                // println!("Arg_name = {:#?}", arg_names);
                let fn_type = context.void_type().fn_type(&llvm_types, false);
                let new_fn = self.module.add_function(func.name(), fn_type, None);
                let entry = context.append_basic_block(new_fn, "top");
                let _ = self.builder.position_at_end(entry);

                // load up all inputs into scope
                for (name, index) in arg_map.iter() {
                    let val = new_fn.get_nth_param(*index as u32).unwrap();
                    let ty = self.module.datatypes.get(&Symbol::new(arg_types[*index])).unwrap();
                    self.scope.insert(name.clone(), (val.into(), ty.clone()));
                }

                for st in body.iter() {
                    println!("ST = {:#?}", st);
                    self.eval_expr(st.clone());
                }
                let _ = self.builder.build_return(None);
                // panic!()
            }

            Expr::PrimitiveType { name, supertype, bits } => {
                // let name = self.compile_symbol(*name);
                let supertype = self.compile_symbol(*supertype);
                let dt = DataType::create_primitive_type(&name, supertype.name(), bits);
                self.module.datatypes.insert(Symbol::new(&name), dt); 
            }
            Expr::Val { ty, val } => {
                let val = match *ty {
                    Expr::Symbol(ref x) => {
                        match x.as_str() {
                            "Float64" |
                            "Int64" => {
                                self.compile_primitive(*val);
                            },
                            _ => todo!("Type unsupported {:#?}", ty)
                        }
                    }
                    _ => panic!()
                };
                // self.stack.push((val, ty))
            }
            _ => todo!("Expr = {:#?}", expr)
        };
    }

    pub fn include(&mut self, file_name: &str) {
        let ast: Vec<_> = parser::parse_file(file_name).unwrap();
        let mut cores = Vec::<FarneseExpr>::new();
        for expr in ast {
            // self.eval_expr(expr)
            let core = self.ast_to_core(expr);
            cores.push(core.clone());
            println!("Core = {:#?}", core);
        }
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

    pub struct TestHelper<'a> {
        // pub builder: Builder<'a>,
        pub compiler: Compiler<'a>,
        // pub module: Module<'a>
    }

    impl<'a> TestHelper<'a> {
        pub fn new(context: &'a Context, name: &str) -> Self {
            let ptr_type = context.ptr_type(AddressSpace::default());
            let builder = context.create_builder();
            let module = Module::new(context, Symbol::new(name));
            let compiler = Compiler::new(builder, module);
            // let module = context.create_module(name);
            
            // TODO eventually move a lot of below
            // to the module type once we figured out
            // what we need and where it should be
            let printf_type = context.i32_type()
                .fn_type(&[ptr_type.into()], true);
            compiler.module.add_function("printf", printf_type, None);

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
            compiler.module.add_function("llvm.memcpy.p0.p0.i64", memcpy_ty, None);

            Symbol::create_opaque_type(&compiler.builder, &compiler.module, "Symbol");
            Symbol::set_opaque_type_init(&compiler.builder, &compiler.module);

            DataType::create_opaque_type(&compiler.builder, &compiler.module, "DataType");
            DataType::set_opaque_type_init(&compiler.builder, &compiler.module);

            Self {
                // builder: builder,
                compiler: compiler,
                // module: module
            }
        }

        pub fn start(&self) {
            let context = self.compiler.module.get_context();
            let fn_type = context.i32_type().fn_type(&[], false);
            let function = self.compiler.module.add_function("main", fn_type, None);
            let entry = context.append_basic_block(function, "entry");
            self.compiler.builder.position_at_end(entry);
        }

        pub fn end(&self) {
            let zero = self.compiler.module.get_context().i32_type().const_int(0, false);
            let _ = self.compiler.builder.build_return(Some(&zero));
        }

        pub fn run(&self) -> String {
            let engine = self.compiler
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
            self.compiler.module.print_to_stderr();
            println!("End of current IR\n\n\n")
        }
    }

    macro_rules! farnese_test {
        (
            $name:ident, $expected:expr,
            |$ctx:ident, $test:ident, $results:ident| $body:block
        ) => {
            #[test]
            fn $name() -> () {
                use inkwell::context::Context;
    
                let $ctx = Context::create();
                let mut $test = TestHelper::new(&$ctx, "some test name");
                $test.start();

                $body

                $test.end();
                $test.show_ir();
                let $results = $test.run();
                assert_eq!($expected, $results);
                println!("Results = {:?}", $results);
            }
        };
    }

    farnese_test!(test_const_int, "", |ctx, test, results| {
        test.compiler.eval("1");
    });

    farnese_test!(test_add_two_ints, "", |ctx, test, results| {
        test.compiler.eval("1 + 2 * (3 + 4.)");
    });

    farnese_test!(test_printf_int, "1", |ctx, test, results| {
        // let expr = Expr::from_string("1");
        // let val: PossibleVal = expr.clone().into();
        // let ptr = val.alloca(&test.builder, &test.module);
        // let _ = val.store(&test.builder, &test.module, ptr);
        // let _ = val.printf(&test.builder, &test.module, ptr);
        // println!("Expr = {:#?}", expr);
        // println!("Val  = {:#?}", val);
        // let compiler = Compiler::new();
        // compiler.eval("1");
        test.compiler.eval("printf(1)");
    });
}
