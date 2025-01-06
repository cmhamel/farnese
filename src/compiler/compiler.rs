
use crate::core::{self, DataType, Symbol};
use crate::parser::parser;
use crate::parser::ast::Node;
use super::Compile;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::passes::PassManager;
use inkwell::types::StructType;
use inkwell::AddressSpace;
#[cfg(feature = "interactive")]
use spinners::{Spinner, Spinners};
use std::collections::HashMap;
use std::ffi::CString;

pub struct Compiler<'a, 'b> {
  builder: &'b Builder<'a>,
  context: &'a Context,
  // core_module: Core<'a>,
  modules: HashMap<Symbol, Module<'a>>,
  // datatype_tag: StructType<'a>
}

impl<'a, 'b> Compiler<'a, 'b> {
  // public methods
  pub fn new(context: &'a Context, builder: &'b Builder<'a>) -> Self {
    let mut modules = HashMap::<Symbol, Module<'a>>::new();
    // let datatype_tag = core::create_type_tag(context);
    Self {
      builder: builder,
      context: context,
      modules: modules,
      // datatype_tag: datatype_tag
    }
  }

  pub fn create_core_module(&mut self) -> () {
    #[cfg(feature = "interactive")]
    let mut sp = Spinner::new(Spinners::Dots, "Precompiling Core...".into());
    let module = self.context.create_module("Core");

    // make datatype
    let sym = Symbol::new("DataType");
    let new_type = DataType::new(&sym, &sym, false, false, true);
    let type_tag_type = self.context.opaque_struct_type("DataType");
    type_tag_type.set_body(
      &[
        self.context.i8_type().ptr_type(AddressSpace::default()).into(),
        type_tag_type.ptr_type(AddressSpace::default()).into()
      ],
      false,
    );
    let module = new_type.create_type(self.context, module.clone(), type_tag_type);

    // make any type
    let sym = Symbol::new("Any");
    let any_type = DataType::new(&sym, &sym, true, false, false);
    let module = any_type.create_type(
      self.context, module.clone(), 
      module.get_struct_type("DataType").unwrap()
    );
    let any_global = module.get_global("Any").unwrap();

    // any constructor
    let value_type = self.context.opaque_struct_type("Value");
    value_type.set_body(
      &[
        self.context.i8_type().ptr_type(AddressSpace::default()).into(),
        module.get_struct_type("DataType").unwrap().ptr_type(AddressSpace::default()).into()
      ],
      false
    );
    let fn_type = value_type.fn_type(&[
        self.context.i8_type().ptr_type(AddressSpace::default()).into()
      ], 
      false
    );
    let function = module.add_function("__Any__new", fn_type, Some(Linkage::External));
    let entry_block = self.context.append_basic_block(function, "entry");
    self.builder.position_at_end(entry_block);
    // Step 5: Allocate memory for the Value struct
    let value_alloca = self.builder.build_alloca(value_type, "value_alloca").unwrap();
    // Step 6: Get function argument (i8* value to store)
    let input_value = function.get_nth_param(0).unwrap().into_pointer_value();
    // Step 7: Store the i8* in the first field of the Value struct
    let value_data_ptr = self.builder
        .build_struct_gep(value_alloca, 0, "value_data_ptr")
        .unwrap();
    self.builder.build_store(value_data_ptr, input_value);
    // Step 8: Store the "Any" type tag in the second field
    let type_tag_ptr = self.builder
        .build_struct_gep(value_alloca, 1, "type_tag_ptr")
        .unwrap();
    let any_type_value = any_global.as_pointer_value();
    // let any_type_value = main_module.get_struct_type("Any").unwrap().as_pointer_value();
    self.builder.build_store(type_tag_ptr, any_type_value);
    // Step 9: Return the constructed Value
    let loaded_value = self.builder.build_load(value_alloca, "loaded_value").unwrap();
    self.builder.build_return(Some(&loaded_value));

    // printf stuff
    let i8_type = self.context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
    let printf_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], true);
    let _ = module.add_function("printf", printf_type, Some(Linkage::External));

    //
    module.print_to_file("Core.ll");
    // module.print_to_stderr();

    self.modules.insert(Symbol::new("Core"), module);
    #[cfg(feature = "interactive")]
    sp.stop_and_persist("\x1b[32m✔\x1b[0m", "Precompiled Core".into());
    // self.modules.insert(Symbol::new("Main"), main_module);
  }

  pub fn create_main_module(&mut self) -> Module<'a> {
    #[cfg(feature = "interactive")]
    let mut sp = Spinner::new(Spinners::Dots, "Precompiling Main...".into());
    let core_module = self.modules.get(&Symbol::new("Core")).unwrap();
    let main_module = self.context.create_module("Main");
    let _ = main_module.add_function(
      "__All__new", 
      // core_module.get_function("__All__new").unwrap().get_type(), 
      core_module.get_functions().next().unwrap().get_type(),
      Some(Linkage::External)
    );
    let _ = main_module.add_function(
      "printf", 
      core_module.get_function("printf").unwrap().get_type(), 
      Some(Linkage::External)
    );
    let _ = main_module.link_in_module(core_module.clone()).unwrap();

    // trying something out
    let any_global = main_module.get_global("Any").unwrap();

    // Create the main function
    // TODO figure out where to put this
    let i32_type = self.context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);
    let function = main_module.add_function("main", fn_type, None);
    let basic_block = self.context.append_basic_block(function, "entry");
    self.builder.position_at_end(basic_block);

    // trying things out
    // let int_value = self.context.i64_type().const_int(69, false);
    let float_value = self.context.f64_type().const_float(69.0);
    let float_value_ptr = self.builder.build_alloca(self.context.f64_type(), "int_value_ptr").unwrap();
    self.builder.build_store(float_value_ptr, float_value);
    let float_value_ptr_cast = self.builder
      .build_bit_cast(float_value_ptr, self.context.i8_type().ptr_type(AddressSpace::default()), "int_value_ptr_cast")
      .unwrap()
      .into_pointer_value();
    let any_new = main_module.get_function("__Any__new").unwrap();
    let any_value = self.builder
      .build_call(any_new, &[float_value_ptr_cast.into()], "__any__new")
      .unwrap()
      .try_as_basic_value()
      .left()
      .unwrap();

    // TODO need to hook with parser to read in main method

    // Create a format string for printf
    let format_string = self.builder.build_global_string_ptr("%f\n", "format").unwrap();
    self.builder.build_call(
      // printf_func,
      main_module.get_function("printf").unwrap(),
      &[format_string.as_pointer_value().into(), float_value.into()],
      "printf_call",
    );

    let int = i32_type.const_int(0, false);
    let _ = self.builder.build_return(Some(&int));

    #[cfg(feature = "interactive")]
    sp.stop_and_persist("\x1b[32m✔\x1b[0m", "Precompiled Main".into());
    self.modules.insert(Symbol::new("Main"), main_module.clone());
    main_module
  }

  pub fn from_source(&mut self, source: &str) -> () {
    let ast: Vec<_> = parser::parse(source).unwrap();
    self.from_ast(ast)
  }

  pub fn link(&mut self) -> () {
    let main_sym = Symbol::new("Main");
    let main_module = self.create_main_module();
    for (key, val) in (&self.modules).into_iter() {
      #[cfg(feature = "interactive")]
      let mut link_sp = Spinner::new(Spinners::Dots, format!("Linking {}...", key.name()).into());

      main_module.link_in_module(val.clone());      

      #[cfg(feature = "interactive")]
      link_sp.stop_and_persist("\x1b[32m✔\x1b[0m", format!("Linked {} into Main", key.name()).into());
    }

    self.modules.insert(main_sym, main_module);
  }

  // private methods
  fn create_expr_from_ast(&mut self, module: Module<'a>, ast: &Node) -> Module<'a> {
    match ast {
      Node::AbstractType { name, params, supertype } => {
        let new_type = DataType::new(&name, &supertype, true, false, false);
        new_type.create_type(
          self.context, module.clone(), 
          self.modules.get(&Symbol::new("Core")).unwrap().get_struct_type("DataType").unwrap()
        )
      }
      Node::AssignmentExpr { identifier, value } => {
        println!("iden = {:?}, val = {:?}", identifier, value);
        module
      },
      Node::Empty => module,
      Node::Primitive(x) => {
        println!("Got a primitive {:?}", x);
        module
      }
      Node::PrimitiveType { name, supertype, bits } => {
        println!("Here");
        let new_type = DataType::new(&name, &supertype, false, false, true);
        new_type.create_type(
          self.context, module.clone(), 
          self.modules.get(&Symbol::new("Core")).unwrap().get_struct_type("DataType").unwrap()
        )
      }
      // _ => println!("AST = {:?}", ast)
      _ => panic!("Unsupported")
    }
    // module.clone()
  }

  fn create_module_from_ast(&mut self, ast: Node) -> () {
    // println!("Node = {:?}", ast);

    let core_module = self.modules.get(&Symbol::new("Core")).unwrap();

    match ast {
      Node::Module { ref name, ref exprs } => {
        #[cfg(feature = "interactive")]
        let mut sp = Spinner::new(Spinners::Dots, format!("Precompiling {}...", name.name()).into());
        let mut module = self.context.create_module(name.name());
        module.link_in_module(core_module.clone());      

        for expr in exprs {
          module = self.create_expr_from_ast(module.clone(), &expr);
        }

        module.print_to_stderr();
        #[cfg(feature = "interactive")]
        sp.stop_and_persist("\x1b[32m✔\x1b[0m", format!("Precompiled {}", name.name()).into());
        module.print_to_file(format!("{}.ll", name.name()));
        self.modules.insert(name.clone(), module);
      },
      _ => ()
    }
  }

  fn from_ast(&mut self, ast: Vec<Node>) -> () {
    // compile modules first
    for node in ast {
      let _ = self.module_from_ast(node);
    }

    // now we can compile other stuff
  }

  fn module_from_ast(&mut self, ast: Node) -> () {
    match ast {
      Node::Module { .. } => {
        self.create_module_from_ast(ast);
        println!("Got a module here")
      },
      _ => ()
    }
  }

  // end result methods
  pub fn optimize_ir(&self) -> () {
    let module = self.modules.get(&Symbol::new("Main")).unwrap();
    // module.print_to_stderr();
    let fpm = PassManager::create(());
    // Add some optimization passes
    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();
    fpm.add_gvn_pass();
    fpm.add_cfg_simplification_pass();
    fpm.add_basic_alias_analysis_pass();
    // run optimization passes
    fpm.run_on(module);
  }  

  pub fn write_ir_to_file(&self, file_name: &str, ) -> () {
    let _ = self.modules.get(&Symbol::new("Main")).unwrap().print_to_file(&file_name);
  }
}
