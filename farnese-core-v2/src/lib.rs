use farnese_lexer::ast::{
    BinaryExpr,
    Operator,
    Primitive
};
use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::{
        self,
        Linkage
    },
    OptimizationLevel,
    types::FunctionType,
    values::{
        BasicValueEnum,
        FunctionValue,
        PointerValue
    }
};
use std::collections::{
    HashMap,
    hash_map::DefaultHasher
};
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Index;

// type aliases
type DataTypes    = HashMap<Symbol, DataType>;
type Exports      = Vec<Symbol>;
type MethodArgs   = Vec<Symbol>;
type MethodReturn = DataType;
type Methods      = HashMap<Symbol, (MethodArgs, MethodReturn)>;

pub trait FarneseCompile<'a, 'b> {
    type Output;
    fn compile(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> Self::Output;
}

pub trait FarneseType {
    fn create_type() -> DataType;
}

// this will be for create values via llvm
pub trait FarneseValue<'a, 'b> {
    fn alloca_and_store(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a>;
    fn create_value(&self, module: &Module<'a>) -> BasicValueEnum<'a>;
}

impl<'a, 'b> FarneseCompile<'a, 'b> for BinaryExpr {
    // type Output = BasicValueEnum<'a>;
    // type Output = PointerValue<'a>;
    type Output = ();
    fn compile(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> Self::Output {
        // type promotions
        let (lhs, rhs, val_type) = match (self.lhs.clone(), self.rhs.clone()) {
            (Primitive::Int32(x), Primitive::Int32(y)) => {
                (Primitive::Int32(x), Primitive::Int32(y), module.get_context().i32_type())
            },
            _ => todo!()
        };

        let lhs_val = lhs.compile(builder, module);
        let rhs_val = rhs.compile(builder, module);
        println!("lhs = {:?}", self.lhs);

        // let ptr_type = module.get_context().i32_type().ptr_type(AddressSpace::default());
        // let val_type = module.get_context().i32_type();
        let lhs_val = builder.build_load(val_type, lhs_val, "")
            .unwrap()
            .into_int_value();
        let rhs_val = builder.build_load(val_type, rhs_val, "")
            .unwrap()
            .into_int_value();
        match &self.op {
            Operator::Plus => {
                match lhs {
                    Primitive::Int32(_) => {
                        builder.build_int_add(lhs_val, rhs_val, "").unwrap()
                    },
                    _ => todo!()
                }
            },
            _ => todo!()
        };
    }
}

#[derive(Clone)]
pub struct DataType {
    bits: i32,
    field_names: SimpleVector,
    field_types: SimpleVector,
    is_abstract: bool,
    is_mutable: bool,
    is_primitive: bool,
    name: Symbol,
    supertype: Symbol
}

impl<'a, 'b> DataType {
    pub fn create_abstract_type(name: &str, supertype: &str) -> Self {
        Self {
            bits: 8,
            field_names: SimpleVector::new(vec![]),
            field_types: SimpleVector::new(vec![]),
            is_abstract: true,
            is_mutable: false,
            is_primitive: false,
            name: Symbol::new(name),
            supertype: Symbol::new(supertype)
        }
    }

    pub fn create_concrete_type(
        name: &str, supertype: &str,
        field_names: Vec<String>,
        field_types: Vec<String>
    ) -> Self {
        let field_names = field_names
            .iter()
            .map(|x| Symbol::new(x))
            .collect::<Vec<_>>();
        let field_types = field_types
            .iter()
            .map(|x| Symbol::new(x))
            .collect::<Vec<_>>();
        Self {
            bits: 8,
            field_names: SimpleVector::new(field_names),
            field_types: SimpleVector::new(field_types),
            is_abstract: false,
            is_mutable: false,
            is_primitive: false,
            name: Symbol::new(name),
            supertype: Symbol::new(supertype)
        }
    }

    pub fn create_primitive_type(name: &str, supertype: &str, bits: i32) -> Self {
        Self {
            bits: bits,
            field_names: SimpleVector::new(vec![]),
            field_types: SimpleVector::new(vec![]),
            is_abstract: false,
            is_mutable: false,
            is_primitive: true,
            name: Symbol::new(name),
            supertype: Symbol::new(supertype)
        }
    }

    pub fn get_bit_size(&self) -> &i32 {
        &self.bits
    }

    pub fn get_field_names(&self) -> &SimpleVector {
        &self.field_names
    }

    pub fn get_field_types(&self) -> &SimpleVector {
        &self.field_types
    }

    pub fn is_abstract(&self) -> &bool {
        return &self.is_abstract
    }

    pub fn is_mutable(&self) -> &bool {
        return &self.is_mutable
    }

    pub fn is_primitive(&self) -> &bool {
        return &self.is_primitive
    }

    pub fn name(&self) -> &Symbol {
        return &self.name 
    }

    pub fn supertype(&self) -> &Symbol {
        return &self.supertype
    }
}

pub struct Module<'a> {
    pub context: &'a Context,
    data_types: DataTypes,
    exports: Exports,
    methods: Methods,
    pub module: module::Module<'a>,
    name: Symbol
}

impl<'a> Module<'a> {
    pub fn new(context: &'a Context, name: &str) -> Self {
        Self {
            context: context,
            data_types: DataTypes::new(),
            exports: Exports::new(),
            methods: Methods::new(),
            module: context.create_module(name),
            name: Symbol::new(name)
        }
    }

    pub fn create_builder(&self) -> Builder<'a> {
        self.get_context().create_builder()
    }

    pub fn create_jit_execution_engine(&self, opt_level: OptimizationLevel) -> ExecutionEngine<'_> {
        self.module.create_jit_execution_engine(opt_level).unwrap()
    }

    pub fn get_context(&self) -> &'a Context {
        &self.context
    }

    pub fn get_datatype(&self, sym: &str) -> &DataType {
        let sym = Symbol::new(sym);
        &self
            .data_types
            .get(&sym)
            .expect(format!("DataType {} not found in module {}", sym, self.name).as_str())
    }

    pub fn get_exports(&self) -> &Exports {
        &self.exports
    }

    pub fn get_methods(&self) -> &Methods {
        &self.methods
    }

    pub fn insert_datatype(&mut self, datatype: DataType) {
        self.data_types.insert(datatype.name().clone(), datatype);
    }

    pub fn insert_method(
        &self,
        name: &str,
        func: FunctionType<'a>,
        opt: Option<Linkage>
    ) -> FunctionValue<'a> {
        self.module.add_function(name, func, opt)
    }

    pub fn link(&mut self, module: &Module<'a>) {
        self.module.link_in_module(module.module.clone()).unwrap();
        for k in module.get_exports() {
            // first try assuming this key is a type
            if self.data_types.contains_key(k) {
                panic!(
                    "Conflicting type {} in modules {} and {}",
                    k,
                    self.name(),
                    module.name()
                )
            } else {
                // TODO note this will error if the type doesn't exist
                // need to change this to use a Result<>
                self.insert_datatype(module.get_datatype(k.name()).clone());
            }

            // TODO need to handle globals, functions, etc.
        }
    }

    pub fn module(&self) -> &module::Module<'a> {
        &self.module
    }

    pub fn name(&self) -> &Symbol {
        &self.name
    }
}

// primitives bindings below

impl<'a, 'b> FarneseCompile<'a, 'b> for Primitive {
    // type Output = BasicValueEnum<'a>;
    type Output = PointerValue<'a>;
    fn compile(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> Self::Output {
        self.alloca_and_store(builder, module)
        // self.create_value(module)
    }
}

impl FarneseType for f32 {
    fn create_type() -> DataType {
        DataType::create_primitive_type("Float32", "AbstractFloat", 32)
    }
}

impl FarneseType for i32 {
    fn create_type() -> DataType {
        DataType::create_primitive_type("Int", "Signed", 32)
    }
}

impl<'a, 'b> FarneseValue<'a, 'b> for Primitive {
    fn alloca_and_store(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a> {
        let val = self.create_value(module);
        let ptr = builder.build_alloca(val.get_type(), "").unwrap();
        let _ = builder.build_store(ptr, val);
        ptr
    }

    fn create_value(&self, module: &Module<'a>) -> BasicValueEnum<'a> {
        let context = module.get_context();
        match &self {
            Primitive::Float32(x) => {
                context
                    .f32_type()
                    .const_float((*x).into())
                    .try_into()
                    .unwrap()
            },
            Primitive::Int32(x) => {
                context
                    .i32_type()
                    .const_int((*x).try_into().unwrap(), true)
                    .try_into()
                    .unwrap()
            },
            _ => todo!()
        }
    }
}

#[derive(Clone)]
pub struct SimpleVector {
    data: Vec<Symbol>
}

impl SimpleVector {
    pub fn new(data: Vec<Symbol>) -> SimpleVector {
        Self {
            data: data
        }
    }
}

impl Index<usize> for SimpleVector {
    type Output = Symbol;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.data[idx]
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Symbol {
    hash: u64,
    name: String
}

impl Symbol {
    pub fn new(name: &str) -> Self {
        let hash = Self::hash_name(name);
        Self {
            hash: hash,
            name: name.to_owned()
        }
    }

    fn hash_name(name: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash: u64 = hasher.finish() as u64;
        hash
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl FarneseType for Symbol {
    fn create_type() -> DataType {
        DataType::create_concrete_type("Symbol", "Any", vec![], vec![])
    }
}

// impl FarneseValue for Symbol {
//     fn create_value() {

//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Symbol;
    use core::ffi::c_char;
    use inkwell::OptimizationLevel;
    use once_cell::sync::Lazy;
    use std::fs::File;
    use std::io::Read;
    use std::ptr;
    use std::sync::Mutex;

    static STDOUT_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    pub struct TestHelper<'a, 'b> {
        pub builder: &'b Builder<'a>,
        pub module: Module<'a>
    }

    impl<'a, 'b> TestHelper<'a, 'b> {
        pub fn new(name: &str, builder: &'b Builder<'a>, context: &'a Context) -> Self {
            // let core = Core::new(&context).bootstrap();
            let mut module = Module::new(&context, name);
            
            // module.link(&core);
            Self {
                builder: &builder,
                module: module,
            }
        }
    
        pub fn end(&self) {
            let zero = self.module.context.i32_type().const_int(0, false);
            let _ = self.builder.build_return(Some(&zero));
        }
    
        pub fn module(&self) -> &Module<'a> {
            &self.module
        }

        pub fn run(&self) -> String {
            let _lock = STDOUT_LOCK.lock().unwrap();
    
            let engine = self
                .module()
                .create_jit_execution_engine(OptimizationLevel::None);
    
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
    
        pub fn start(&self) {
            let context = self.module.get_context();
            let fn_type = context.i32_type().fn_type(&[], false);
            let function = self.module.insert_method("main", fn_type, None);
            let entry = context.append_basic_block(function, "entry");
            self.builder.position_at_end(entry);
        }
    }

    #[test]
    fn test_module() {
        let context = Context::create();
        let module = Module::new(&context, "MyModule");
    }

    #[test]
    fn test_primitive_datatypes() {
        let f32_type = f32::create_type();
        let i32_type = i32::create_type();

        let prim_val: Primitive = (69.0 as f32).into();
        println!("prim val = {:?}", prim_val)
    }

    #[test]
    fn test_symbol_datatype() {
        let sym_type = Symbol::create_type();
        assert_eq!(sym_type.name(), &Symbol::new("Symbol"));
        assert_eq!(sym_type.supertype(), &Symbol::new("Any"));
        assert_eq!(sym_type.is_abstract, false);
        assert_eq!(sym_type.is_mutable, false);
        assert_eq!(sym_type.is_primitive, false);
    }

    #[test]
    fn test_primitive_runtime() {
        let context = Context::create();
        let builder = context.create_builder();
        let tester = TestHelper::new("prim_test", &builder, &context);
        tester.start();
        let val = Primitive::Float32(69.0);
        let ptr = val.compile(&builder, tester.module());
        let val = Primitive::Int32(4);
        let ptr = val.compile(&builder, tester.module());

        let bin_expr = BinaryExpr { 
            lhs: Primitive::Int32(1),
            op: Operator::Plus,
            rhs: Primitive::Int32(2)
        };
        let res = bin_expr.compile(&builder, tester.module());

        tester.end();

        tester.module().module().print_to_file("./temp.ll");
        let test_output = tester.run();
        println!("Test output = {:?}", test_output);
    }
}
