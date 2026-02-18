use super::{
    DataType, 
    FarneseInternal, 
    LLVMAlloca, 
    LLVMGlobal,
    Module,
    SimpleVector
};
// use dashmap::DashMap;
use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::values::{GlobalValue, PointerValue};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::ffi::CString;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
// use std::sync::atomic::{AtomicUsize, Ordering};

/// base type, holds a unique has and a name
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Symbol {
    // hash: i64,
    hash: u64,
    name: String,
}

impl<'a> Symbol {
    /// creates new symbol from a borrowed string
    pub fn new(name: &str) -> Self {
        let hash = Self::hash_name(name);
        Self {
            hash: hash,
            name: name.to_owned(),
        }
    }

    /// creates a new symbol from a String
    pub fn from_string(name: String) -> Self {
        let hash = Self::hash_name(&name);
        Self {
            hash: hash,
            name: name,
        }
    }

    /// creates a hash
    fn hash_name(name: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash: u64 = hasher.finish() as u64;
        hash
    }

    /// gets the hash TODO chanage to get_hash
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// gets the name TODO change to get_name
    pub fn name(&self) -> &str {
        &self.name
    }
}

///
impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<'a> FarneseInternal<'a> for Symbol {
    fn create_datatype() -> DataType {
        DataType::new(
            Symbol::new("Symbol"),
            Symbol::new("Any"),
            false,
            false,
            false,
            // TODO change below when we add generics
            SimpleVector::new(vec![]),
            SimpleVector::new(vec![]),
            SimpleVector::new(vec![])
        )
    }
    
    fn create_new_method(_module: &Module<'a>) {
        
    }

    fn create_opaque_type(module: &Module<'a>) {
        let context = module.get_context();
        let _ = context.opaque_struct_type("Symbol");
    }

    fn set_opaque_type_initializer(module: &Module<'a>) {
        let context = module.get_context();
        let opaque_type = module.get_struct_type("Symbol");
        opaque_type.set_body(
            &[
                context.i64_type().into(),
                context.ptr_type(AddressSpace::default()).into(),
            ],
            false,
        );
    }
}

impl<'a, 'b> LLVMAlloca<'a, 'b> for Symbol {
    fn emit_ir_alloca(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a> {
        let context = module.get_context();

        let sym_type = module.get_struct_type("Symbol");
        let ptr = builder.build_alloca(sym_type, "").unwrap();
        let glob = builder.build_global_string_ptr(&self.name.clone(), "")
            .unwrap()
            .as_pointer_value();

        // set symbol hash
        let i64_t = context.i64_type();
        let field_ptr = builder.build_struct_gep(i64_t, ptr, 0, "").unwrap();
        let hash = context
            .i64_type()
            .const_int((self.hash as u64).try_into().unwrap(), true);
        let _ = builder.build_store(field_ptr, hash);

        // set symbol string
        let field_ptr = builder.build_struct_gep(glob.get_type(), ptr, 1, "").unwrap();
        // let _ = builder.build_store(field_ptr, sym_ptr_ptr);
        let _ = builder.build_store(field_ptr, glob);

        ptr
    }
}

impl<'a> LLVMGlobal<'a> for Symbol {
    fn emit_ir_global(&self, module: &Module<'a>) -> GlobalValue<'a> {
        let context = module.get_context();
        let symbol_type = module.get_struct_type("Symbol");

        let sym_name = format!("__Sym_{}", &self.name);

        // setup necessary globals for symbol and datatype
        let sym_glob = module.add_global(
            symbol_type,
            Some(AddressSpace::default()),
            sym_name.as_str()
        );

        let name_str = context.const_string(
            &CString::new(self.name.clone()).unwrap().as_bytes_with_nul(), 
            true
        );
        // let glob = module.add_global(name_str.get_type(), None, format!("{}_str", &sym_name).as_str())
        let name_glob = module.add_global(name_str.get_type(), None, "");
        name_glob.set_initializer(&name_str);
        let name_glob = name_glob
            .as_pointer_value()
            .const_cast(context.ptr_type(AddressSpace::default()));
        let hash = context.i64_type().const_int(
            self.hash().try_into().unwrap(),
            false
        );

        let symbol_value = symbol_type.const_named_struct(&[
            hash.into(),
            name_glob.into()
        ]);
        sym_glob.set_initializer(&symbol_value);

        sym_glob
    }
}

#[derive(Clone)]
pub struct SymbolTable {
    symbols: Arc<RwLock<HashMap<String, Symbol>>>,
    counter: Arc<RwLock<i64>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Arc::new(RwLock::new(HashMap::new())),
            counter: Arc::new(RwLock::new(0)),
        }
    }

    pub fn intern(&self, name: &str) -> Symbol {
        let mut table = self.symbols.write().unwrap();
        if let Some(info) = table.get(name) {
            return info.clone();
        }

        let mut ctr = self.counter.write().unwrap();
        let id = *ctr;
        *ctr += 1;
        // let info = Symbol { name: name.to_string(), id };
        // let info = Symbol::new()
        let info = Symbol {
            name: name.to_string(),
            hash: id as u64,
        };
        table.insert(name.to_string(), info.clone());
        info
    }

    pub fn get_name(&self, id: u64) -> Option<String> {
        let table = self.symbols.read().unwrap();
        for (k, v) in table.iter() {
            if v.hash == id {
                return Some(k.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::AddressSpace;
    use inkwell::context::Context;
    use std::ffi::CString;

    #[test]
    fn test_symbol_new() {
        let sym = Symbol::new("Any");
        assert_eq!(sym.name(), "Any");
    }

    #[test]
    fn test_symbol_table() {
        let sym_table = SymbolTable::new();
        let _ = sym_table.intern("DataType");
    }

    #[test]
    fn test_symbol_table_ir_global() {
        let symbol_table = SymbolTable::new();

        // IR stuff
        let context = Context::create();
        let module = context.create_module("symbol_module");
        let builder = context.create_builder();
        let void_type = context.void_type();
        let fn_type = void_type.fn_type(&[], false);
        let function = module.add_function("main", fn_type, None);
        let entry = context.append_basic_block(function, "entry");
        builder.position_at_end(entry);

        // Intern a symbol
        let sym = symbol_table.intern("hello_world");
        let sym_str = CString::new(symbol_table.get_name(sym.hash).unwrap()).unwrap();
        let sym_global = module.add_global(
            context
                .i8_type()
                .array_type((sym_str.as_bytes_with_nul().len()) as u32),
            None,
            "sym_str",
        );
        sym_global.set_initializer(
            &context.i8_type().const_array(
                &sym_str
                    .as_bytes_with_nul()
                    .iter()
                    .map(|&b| context.i8_type().const_int(b as u64, false))
                    .collect::<Vec<_>>(),
            ),
        );

        // Printf function declaration
        let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
        let printf_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], true);
        let printf = module.add_function("printf", printf_type, None);

        // Format string
        let fmt = CString::new("Symbol: %s\n").unwrap();
        let fmt_global = module.add_global(
            context
                .i8_type()
                .array_type((fmt.as_bytes_with_nul().len()) as u32),
            None,
            "fmt",
        );
        fmt_global.set_initializer(
            &context.i8_type().const_array(
                &fmt.as_bytes_with_nul()
                    .iter()
                    .map(|&b| context.i8_type().const_int(b as u64, false))
                    .collect::<Vec<_>>(),
            ),
        );

        let fmt_ptr = unsafe {
            builder
                .build_gep(
                    fmt_global.as_pointer_value(),
                    &[
                        context.i32_type().const_zero(),
                        context.i32_type().const_zero(),
                    ],
                    "fmt_ptr",
                )
                .unwrap()
        };
        let sym_ptr = unsafe {
            builder
                .build_gep(
                    sym_global.as_pointer_value(),
                    &[
                        context.i32_type().const_zero(),
                        context.i32_type().const_zero(),
                    ],
                    "sym_ptr",
                )
                .unwrap()
        };

        let _ = builder.build_call(printf, &[fmt_ptr.into(), sym_ptr.into()], "call_printf");
        let _ = builder.build_return(None);

        // module.print_to_stderr();
    }

    // #[test]
    // fn test_alloca_and_malloc() {
    //     // let sym_table = SymbolTable::new();
    //     // sym_table.intern("Symbol");

    //     let context = Context::create();
    //     let builder = context.create_builder();
    //     let mut core = Core::new(&context);
    //     // let module = context.create_module("symbol_test_module");
    //     let module = core.bootstrap();

    //     core.main_func_begin(&builder, &module);

    //     // let sym = Symbol::new("Symbol");
    //     // let sym_ptr = sym.emit_ir_malloc(&builder, &module);
    //     // let _ = sym.emit_ir_printf(&builder, &module, &sym_ptr);
    //     // let _ = sym.emit_ir_free(&builder, &module, &sym_ptr);

    //     let sym = Symbol::new("Any");
    //     // let sym_ptr = sym.emit_ir_alloca(&builder, &module);
    //     let _ = sym.emit_ir_printf(&builder, &module);

    //     core.main_func_end(&builder, &module);

    //     let _ = module.print_to_file("sym_test.ll");
    // }
}
