use super::{FarneseInternal, LLVMAlloca, LLVMPrintf, Module};
// use dashmap::DashMap;
use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::values::PointerValue;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{self, Display, Formatter};
use std::collections::HashMap;
use std::ffi::CString;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
// use std::sync::atomic::{AtomicUsize, Ordering};

/// base type, holds a unique has and a name
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Symbol {
    hash: i64,
    name: String
}

impl<'a> Symbol {
  /// creates new symbol from a borrowed string
    pub fn new(name: &str) -> Self {
        let hash = Self::hash_name(name);
        Self {
            hash: hash,
            name: name.to_owned()
        }
    }

    /// creates a new symbol from a String
    pub fn from_string(name: String) -> Self {
        let hash = Self::hash_name(&name);
        Self {
            hash: hash,
            name: name
        }
    }

    /// creates a hash
    fn hash_name(name: &str) -> i64 {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash: i64 = hasher.finish() as i64;
        hash
    }

    /// gets the hash TODO chanage to get_hash
        pub fn hash(&self) -> i64 {
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
    fn create_opaque_type(&self, module: &Module<'a>) {
        let context = module.get_context();
        let opaque_type = context
            .opaque_struct_type("Symbol");
        opaque_type.set_body(
            &[
                // context.i8_type().ptr_type(AddressSpace::default()).into(),
                // opaque_type.ptr_type(AddressSpace::default()).into()
                context.i64_type().into(),
                context.i8_type().ptr_type(AddressSpace::default()).into()
            ],
            false
        );
    }

    fn create_datatype(&self, module: &Module<'a>) {
        let context = module.get_context();
        let builder = context.create_builder();
        let sym = Symbol::new("Symbol");
        let datatype = module.get_struct_type("DataType");
        let datatype_ptr = datatype.ptr_type(AddressSpace::default());
        let func = datatype_ptr.fn_type(&[], false);
        let func = module.add_function("__DataType_Symbol", func, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        let sym_ptr = sym.emit_ir_alloca(&builder, &module);
        let supertype_ptr = module.get_global("Any").as_pointer_value();
        let func_result = builder.build_call(
            module.get_function("Datatype"),
            &[sym_ptr.into(), supertype_ptr.into()],
            ""
        )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let _ = builder.build_return(Some(&func_result));
    }

    fn create_get_methods(&self, _module: &Module<'a>) {

    }

    fn create_new_method(&self, _module: &Module<'a>) {

    }
}

impl<'a, 'b> LLVMAlloca<'a, 'b> for Symbol {
    fn emit_ir_alloca(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a> {
        let context = module.get_context();
        let i8_type = context.i8_type();
        let sym_str = CString::new(self.name.clone()).unwrap();
        let sym_bytes = sym_str.as_bytes_with_nul();
        
        // let size_val = context.i64_type().const_int(sym_bytes.len() as u64, false);
        let sym_ptr = builder.build_alloca(i8_type.array_type(sym_bytes.len() as u32), "stack_sym").unwrap();
        for (i, &b) in sym_bytes.iter().enumerate() {
            let gep = unsafe {
                builder.build_gep(sym_ptr, &[
                        context.i32_type().const_int(0, false), 
                        context.i32_type().const_int(i as u64, false)
                    ], 
                    &format!("idx_{}", i)
                )
            }.unwrap();
            let _ = builder.build_store(gep, i8_type.const_int(b as u64, false));
        }

        // now make a sym_struct
        let sym_ptr_ptr = unsafe {
            builder.build_in_bounds_gep(
                sym_ptr, 
                &[context.i32_type().const_zero(), context.i32_type().const_zero()], 
                "str_ptr"
            )
        }.unwrap();
        let sym_type = module.get_struct_type("Symbol");
        let ptr = builder.build_alloca(sym_type, "").unwrap();

        // set symbol hash
        let field_ptr = builder.build_struct_gep(ptr, 0, "").unwrap();
        let hash = context.i64_type().const_int((self.hash as u64).try_into().unwrap(), true);
        let _ = builder.build_store(field_ptr, hash);

        // set symbol string
        let field_ptr = builder.build_struct_gep(ptr, 1, "").unwrap();
        let _ = builder.build_store(field_ptr, sym_ptr_ptr);

        ptr
    }
}

// impl<'a, 'b> LLVMPrintf<'a, 'b> for Symbol {
//     fn emit_ir_printf(&self, builder: &'b Builder<'a>, module: &Module<'a>) {
//         let context = module.get_context();
//         let zero = context.i32_type().const_zero();
//         // get appropriate format string
//         let fmt_global = module.get_global("__fmt_string");
//         let fmt_ptr = unsafe {
//             builder.build_gep(fmt_global.as_pointer_value(), &[zero, zero], "fmt_ptr")
//         }.unwrap();
//         // fetch the second field
//         let field_ptr = builder.build_struct_gep(*ptr, 1, "").unwrap();
//         let sym_ptr = builder.build_load(field_ptr, "").unwrap();
//         let printf = module.get_function("printf");
//         let _ = builder.build_call(printf, &[fmt_ptr.into(), sym_ptr.into()], "call_printf");
//     }
// }

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
        let info = Symbol { name: name.to_string(), hash: id };
        table.insert(name.to_string(), info.clone());
        info
    }

    pub fn get_name(&self, id: i64) -> Option<String> {
        let table = self.symbols.read().unwrap();
        for (k, v) in table.iter() {
            if v.hash == id {
                return Some(k.clone());
            }
        }
        None
    }
}
// impl Display for Vec<Symbol> {
//   fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//     write!(f, "[");
//     for val in self.iter() {
//       write!(f, "{}, ", val);
//     }
//     write!(f, "]\n");
//   }
// }

/// symbol table of symbols that have unique hashes
// #[derive(Debug)]
// pub struct SymbolTable {
//   table: DashMap<String, Arc<Symbol>>
// }

// impl SymbolTable {
//   /// creates a new symbol table that is empty by default
//   pub fn new() -> Self {
//     Self {
//       table: DashMap::new()
//     }
//   }

//   /// pushes a new symbol, ensuring uniqueness.
//   pub fn push(&self, name: &str) -> Arc<Symbol> {
//     if let Some(existing) = self.table.get(name) {
//       return Arc::clone(&existing);
//     }

//     let symbol = Arc::new(Symbol::new(name));
//     self.table.insert(name.to_owned(), Arc::clone(&symbol));
//     symbol
//   }

//   /// Looks up a symbol without adding it.
//   pub fn lookup(&self, name: &str) -> Option<Arc<Symbol>> {
//     self.table.get(name).map(|entry| Arc::clone(&entry))
//   }
// }

// // probably need below for some thread safefty stuff
// #[derive(Debug)]
// pub struct UniqueSymbolGenerator {
//   counter: AtomicUsize,
//   table: SymbolTable,
// }

// impl UniqueSymbolGenerator {
//   pub fn new() -> Self {
//     Self {
//       counter: AtomicUsize::new(0),
//       table: SymbolTable::new(),
//     }
//   }

//   /// Generates a unique symbol with a specified prefix.
//   pub fn generate(&self, prefix: &str) -> Arc<Symbol> {
//     let id = self.counter.fetch_add(1, Ordering::Relaxed);
//     let name = format!("{}{}", prefix, id);
//     self.table.push(&name)
//   }
// }

#[cfg(test)]
mod tests {
    use crate::core::Core;
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
        let sym_global = module.add_global(context.i8_type().array_type((sym_str.as_bytes_with_nul().len()) as u32), None, "sym_str");
        sym_global.set_initializer(&context.i8_type().const_array(
            &sym_str.as_bytes_with_nul().iter().map(|&b| context.i8_type().const_int(b as u64, false)).collect::<Vec<_>>()
        ));

        // Printf function declaration
        let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
        let printf_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], true);
        let printf = module.add_function("printf", printf_type, None);

        // Format string
        let fmt = CString::new("Symbol: %s\n").unwrap();
        let fmt_global = module.add_global(context.i8_type().array_type((fmt.as_bytes_with_nul().len()) as u32), None, "fmt");
        fmt_global.set_initializer(&context.i8_type().const_array(
            &fmt.as_bytes_with_nul().iter().map(|&b| context.i8_type().const_int(b as u64, false)).collect::<Vec<_>>()
        ));

        let fmt_ptr = unsafe {
            builder.build_gep(fmt_global.as_pointer_value(), &[context.i32_type().const_zero(), context.i32_type().const_zero()], "fmt_ptr").unwrap()
        };
        let sym_ptr = unsafe {
            builder.build_gep(sym_global.as_pointer_value(), &[context.i32_type().const_zero(), context.i32_type().const_zero()], "sym_ptr").unwrap()
        };

        let _ = builder.build_call(printf, &[fmt_ptr.into(), sym_ptr.into()], "call_printf");
        let _ = builder.build_return(None);

        // module.print_to_stderr();
    }

    #[test]
    fn test_alloca_and_malloc() {
        // let sym_table = SymbolTable::new();
        // sym_table.intern("Symbol");

        let context = Context::create();
        let builder = context.create_builder();
        let mut core = Core::new(&context);
        // let module = context.create_module("symbol_test_module");
        let module = core.bootstrap();

        core.main_func_begin(&builder, &module);

        // let sym = Symbol::new("Symbol");
        // let sym_ptr = sym.emit_ir_malloc(&builder, &module);
        // let _ = sym.emit_ir_printf(&builder, &module, &sym_ptr);
        // let _ = sym.emit_ir_free(&builder, &module, &sym_ptr);

        let sym = Symbol::new("Any");
        let sym_ptr = sym.emit_ir_alloca(&builder, &module);
        // let _ = sym.emit_ir_printf(&builder, &module);

        core.main_func_end(&builder, &module);

        let _ = module.print_to_file("sym_test.ll");
    }
}