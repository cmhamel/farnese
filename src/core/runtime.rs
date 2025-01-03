// use super::Symbol;
// use super::core;
// use inkwell::context::Context;
// use inkwell::module::Module;
// use std::collections::HashMap;
// use std::fs;
// // use std::io::Write;
// use std::path::Path;

// pub struct Runtime<'a> {  
//   // modules: Vec<Module<'a>>
//   modules: HashMap<Symbol, Module<'a>>
// }

// impl<'a> Runtime<'a> {
//   pub fn new(context: &'a Context) -> Self {
//     let mut modules = HashMap::<Symbol, Module<'a>>::new();

//     // create the core module
//     let core = core::create_core(context);
//     modules.insert(Symbol::new("Core"), core);

//     Self {
//       modules: modules
//     }
//   } 

//   pub fn add_module(&self) -> () {

//   }

//   pub fn dump_ir(&self) -> () {
//     let dir_name = "__ir_module_code";
//     if Path::new(dir_name).exists() {
//       fs::remove_dir_all(dir_name).expect("Failed to remove IR");
//     }

//     fs::create_dir(dir_name).expect("Failed to create directory for IR");

//     for (sym, module) in &self.modules {
//       let ir_file_name = format!("{}/{}.ll", dir_name, sym.name());
//       let _ = module.print_to_file(ir_file_name);
//     }
//   }
// }