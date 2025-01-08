
use crate::core::{DataType, Symbol};
use crate::lexer::lexer;
use crate::lexer::ast::{Node, Primitive};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::passes::PassManager;
use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::AddressSpace;
#[cfg(feature = "interactive")]
use spinners::{Spinner, Spinners};
use std::collections::HashMap;

pub struct Compiler<'a, 'b> {
  builder: &'b Builder<'a>,
  context: &'a Context,
  // core_module: Core<'a>,
  modules: HashMap<Symbol, Module<'a>>,
  // datatype_tag: StructType<'a>
  variables: HashMap<Symbol, PointerValue<'a>>
}

impl<'a, 'b> Compiler<'a, 'b> {
  // public methods
  pub fn new(context: &'a Context, builder: &'b Builder<'a>) -> Self {
    let modules = HashMap::<Symbol, Module<'a>>::new();
    let variables = HashMap::<Symbol, PointerValue<'a>>::new();
    // let datatype_tag = core::create_type_tag(context);
    Self {
      builder: builder,
      context: context,
      modules: modules,
      variables: variables
      // datatype_tag: datatype_tag
    }
  }

  pub fn create_core_module(&mut self) -> () {
    #[cfg(feature = "interactive")]
    let mut sp = Spinner::new(Spinners::Dots, "Precompiling Core...".into());
    let module = self.context.create_module("Core");

    // make datatype
    let sym = Symbol::new("DataType");
    // let new_type = DataType::new(&sym, &sym, false, false, true);

    // real datatype stuff here
    let data_type = self.context.opaque_struct_type("DataType");
    data_type.set_body(
      &[
        self.context.i8_type().ptr_type(AddressSpace::default()).into(),
        data_type.ptr_type(AddressSpace::default()).into(),
        self.context.i64_type().ptr_type(AddressSpace::default()).into(),
      ],
      false,
    );

    // let null_pointer = self.context.i8_type().ptr_type(AddressSpace::default()).const_null();
    // let data_type_global = data_type.const_named_struct(&[
    //   null_pointer.into(),
    //   data_type.as_basic_type_enum().ptr_type(AddressSpace::default()).const_null().into(),
    //   self.context.i64_type().ptr_type(AddressSpace::default()).const_null().into()
    // ]);

    // make any type
    let sym = Symbol::new("Any");
    // let any_type = DataType::new(&sym, &sym, true, false, false);
    let any_type = DataType::new(&sym, &sym);
    let module = any_type.create_type(
      self.context, module.clone(), 
      // module.get_struct_type("DataType").unwrap()
      data_type
    );
    // let any_global = module.get_global("Any").unwrap();

    // value type
    let value_type = self.context.opaque_struct_type("Value");
    value_type.set_body(
      &[
        self.context.i8_type().ptr_type(AddressSpace::default()).into(),
        data_type.into()
      ],
      false
    );
    // let value_type_global = value_type.const_named_struct(&[
    //   null_pointer.into(),
    //   data_type.as_basic_type_enum().ptr_type(AddressSpace::default()).const_null().into(),
    // ]);

    // value constructor
    let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
    let func_type = value_type.fn_type(&[i8_ptr_type.into()], false);
    let function = module.add_function("__Any__new", func_type, None);

    // Step 3: Build the function body
    let entry_block = self.context.append_basic_block(function, "entry");
    self.builder.position_at_end(entry_block);

    let input_ptr = function.get_nth_param(0).unwrap().into_pointer_value();

    // Create the %Value struct
    let value_alloca = self.builder.build_alloca(value_type, "value_alloca").unwrap();
    let value_data_ptr = self.builder.build_struct_gep(value_alloca, 0, "value_data_ptr").unwrap();
    let _ = self.builder.build_store(value_data_ptr, input_ptr);

    // let type_tag_ptr = self.builder.build_struct_gep(value_alloca, 1, "type_tag_ptr").unwrap();
    // self.builder.build_store(type_tag_ptr, data_type.const_zero());//.as_pointer_value());

    let loaded_value = self.builder.build_load(value_alloca, "loaded_value").unwrap();
    let _ = self.builder.build_return(Some(&loaded_value));
    
    // printf stuff
    let i8_type = self.context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
    let printf_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], true);
    let _ = module.add_function("printf", printf_type, Some(Linkage::External));

    //
    let _ = module.print_to_file("Core.ll");
    // module.print_to_stderr();

    self.modules.insert(Symbol::new("Core"), module);
    #[cfg(feature = "interactive")]
    sp.stop_and_persist("\x1b[32m✔\x1b[0m", "Precompiled Core".into());
    // self.modules.insert(Symbol::new("Main"), main_module);
  }

  pub fn from_source(&mut self, source: &str) -> () {
    let ast: Vec<_> = lexer::parse(source).unwrap();
    self.from_ast(ast)
  }

  // pub fn link(&mut self) -> () {
  //   let main_sym = Symbol::new("Main");
  //   let main_module = self.create_main_module();
  //   for (key, val) in (&self.modules).into_iter() {
  //     #[cfg(feature = "interactive")]
  //     let mut link_sp = Spinner::new(Spinners::Dots, format!("Linking {}...", key.name()).into());

  //     main_module.link_in_module(val.clone());      

  //     #[cfg(feature = "interactive")]
  //     link_sp.stop_and_persist("\x1b[32m✔\x1b[0m", format!("Linked {} into Main", key.name()).into());
  //   }

  //   self.modules.insert(main_sym, main_module);
  // }

  // private methods
  fn create_expr_from_ast(&mut self, module: Module<'a>, ast: &Node) -> Module<'a> {
    match ast {
      Node::AbstractType { name, params: _, supertype } => {
        let data_type = self.context.opaque_struct_type("DataType");
        data_type.set_body(
          &[
            self.context.i8_type().ptr_type(AddressSpace::default()).into(),
            data_type.ptr_type(AddressSpace::default()).into(),
            self.context.i64_type().ptr_type(AddressSpace::default()).into(),
          ],
          false,
        );
        let new_type = DataType::new(&name, &supertype, true, false, false);
        new_type.create_type(
          self.context, module.clone(), 
          // self.modules.get(&Symbol::new("Core")).unwrap().get_struct_type("DataType").unwrap()
          data_type.into()
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
      Node::PrimitiveType { name, supertype, bits: _ } => {
        println!("Here");
        // let new_type = DataType::new(&name, &supertype, false, false, true);
        let new_type = DataType::new(&name, &supertype);
        // new_type.create_type(
        //   self.context, module.clone(), 
        //   self.modules.get(&Symbol::new("Core")).unwrap().get_struct_type("DataType").unwrap()
        // )
        module
      }
      // _ => println!("AST = {:?}", ast)
      _ => panic!("Unsupported")
    }
    // module.clone()
  }

  pub fn create_basic_expr(&mut self, ast: &Node) -> bool {
    match ast {
      Node::AssignmentExpr { identifier: _, value } => {
        let get_prim = self.create_basic_expr(value);
        println!("get_prim = {:?}", get_prim);
        println!("val = {:?}", self.variables.get(&Symbol::new("__temp_prim")));
        if get_prim {
          
          // let _ = self.builder
          //   .build_call()
          // let any_new = self.modules.get(&Symbol::new("Core")).unwrap().get_function("__Any__new").unwrap();
          // // println!("Any new = {:?}", any_new);
          // // let data_type = 
          // let any_value = self.builder
          //   .build_call(
          //       any_new, &[
          //         (*self.variables.get(&Symbol::new("__temp_prim")).unwrap()).into()
          //       ], "__any__new")
          //   .unwrap()
          //   .try_as_basic_value()
          //   .left()
          //   .unwrap();
        } else {
          println!("Here {:?}", ast);
          // panic!("Not yet supported");
          return false
        }
        false
      },
      Node::Empty => false,
      Node::Primitive(x) => {
        match x {
          Primitive::Char(_) => panic!("Unsupported char type"),
          Primitive::Float(y) => {
            let value = self.context.f64_type().const_float(*y);
            let value_ptr = self.builder.build_alloca(self.context.f64_type(), "float_value_ptr").unwrap();
            let _ = self.builder.build_store(value_ptr, value);
            let value_ptr_cast = self.builder
              .build_bit_cast(value_ptr, self.context.i8_type().ptr_type(AddressSpace::default()), "int_value_ptr_cast")
              .unwrap()
              .into_pointer_value();
            self.variables.insert(Symbol::new("__temp_prim"), value_ptr_cast);
            return true
          },
          Primitive::Int(y) => {
            let value = self.context.i64_type().const_int(*y as u64, true);
            let value_ptr = self.builder.build_alloca(self.context.i64_type(), "int_value_ptr").unwrap();
            let _ = self.builder.build_store(value_ptr, value);
            let value_ptr_cast = self.builder
              .build_bit_cast(value_ptr, self.context.i8_type().ptr_type(AddressSpace::default()), "int_value_ptr_cast")
              .unwrap()
              .into_pointer_value();
            self.variables.insert(Symbol::new("__temp_prim"), value_ptr_cast);
            // create things as an any type for now...
            // TODO change to actual types like int float etc.
            return true
          }
        }
      }
      _ => {
        println!("Unsupported ast {:?}", ast);
        false
      }
    }
  }

  fn create_module_from_ast(&mut self, ast: Node) -> () {
    // println!("Node = {:?}", ast);

    let core_module = self.modules.get(&Symbol::new("Core")).unwrap();

    match ast {
      Node::Module { ref name, ref exprs } => {
        #[cfg(feature = "interactive")]
        let mut sp = Spinner::new(Spinners::Dots, format!("Precompiling {}...", name.name()).into());
        let mut module = self.context.create_module(name.name());
        let _ = module.link_in_module(core_module.clone());      

        for expr in exprs {
          module = self.create_expr_from_ast(module.clone(), &expr);
        }

        module.print_to_stderr();
        #[cfg(feature = "interactive")]
        sp.stop_and_persist("\x1b[32m✔\x1b[0m", format!("Precompiled {}", name.name()).into());
        let _ = module.print_to_file(format!("{}.ll", name.name()));
        self.modules.insert(name.clone(), module);
      },
      _ => ()
    }
  }

  fn from_ast(&mut self, ast: Vec<Node>) -> () {
    // compile modules first
    for node in &ast {
      let _ = self.module_from_ast(node.clone());
    }

    // now we can compile other stuff
    println!("Here");
    // compile main method
    for node in &ast {
      let _ = self.main_function_from_ast(node.clone());
    }
  }

  fn main_function_from_ast(&mut self, ast: Node) -> () {
    match &ast {
      Node::MainFunction { exprs } => {
        #[cfg(feature = "interactive")]
        let mut sp = Spinner::new(Spinners::Dots, "Precompiling main function...".into());

        // let main_module = self.context.create_module("Main");
        let core_module = self.modules.get(&Symbol::new("Core")).unwrap();
        let main_module = self.context.create_module("Main");
        let data_type = self.context.opaque_struct_type("DataType");
        data_type.set_body(
          &[
            self.context.i8_type().ptr_type(AddressSpace::default()).into(),
            data_type.ptr_type(AddressSpace::default()).into(),
            self.context.i64_type().ptr_type(AddressSpace::default()).into(),
          ],
          false,
        );
        let value_type = self.context.opaque_struct_type("Value");
        value_type.set_body(
          &[
            self.context.i8_type().ptr_type(AddressSpace::default()).into(),
            // main_module.get_struct_type("DataType").unwrap().ptr_type(AddressSpace::default()).into()
            // data_type.ptr_type(AddressSpace::default()).into()
            // value_type.ptr_type(AddressSpace::default()).into()
            data_type.into()
          ],
          false
        );
        let _ = main_module.add_function(
          "__Any__new", 
          // core_module.get_function("__All__new").unwrap().get_type(), 
          // core_module.get_functions().next().unwrap().get_type(),
          value_type.fn_type(&[
            self.context.i8_type().ptr_type(AddressSpace::default()).into(),
            // data_type.into(),
          ], false),
          Some(Linkage::External)
        );
        let _ = main_module.add_function(
          "printf", 
          core_module.get_function("printf").unwrap().get_type(), 
          Some(Linkage::External)
        );
        let _ = main_module.link_in_module(core_module.clone()).unwrap();
        // self.modules.insert(Symbol::new("Main"), main_module.clone());

        // Create the main function
        // TODO figure out where to put this
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let function = main_module.add_function("main", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        for expr in exprs {
          let _ = self.create_basic_expr(expr);
        }

        // dummy any init
        let _ = self.builder.build_call(
          main_module.get_function("__Any__new").unwrap(),
          &[
            (*self.variables.get(&Symbol::new("__temp_prim")).unwrap()).into()
          ],
          "__call__Any__new",
        );

        // now fetch that any
        let value_data_ptr = self.variables.get(&Symbol::new("__temp_prim")).unwrap();
        // converting register pointer to type pointer
        let value_data_ptr = self.builder.build_bit_cast(
          *value_data_ptr, 
          self.context.i64_type().ptr_type(inkwell::AddressSpace::default()), 
          "loaded_value_ptr"
        ).unwrap();
        let value_data_ptr = self.builder.build_bit_cast(
          value_data_ptr, 
          self.context.f64_type().ptr_type(inkwell::AddressSpace::default()), 
          "loaded_value_ptr"
        ).unwrap();
        let value = match value_data_ptr {
          BasicValueEnum::PointerValue(x) => {
            self.builder.build_load(x, "loaded_value")
          },
          _ => panic!("wtf")
        }.unwrap();
        
        // dummy printing
        let format_string = self.builder.build_global_string_ptr("%f\n", "format").unwrap();
        let _ = self.builder.build_call(
          main_module.get_function("printf").unwrap(),
          &[
            format_string.as_pointer_value().into(), 
            value.into()
          ],
          "printf_call",
        );

        let int = i32_type.const_int(0, false);
        let _ = self.builder.build_return(Some(&int));

        // let _ = main_module.link_in_module(core_module.clone()).unwrap();

        // main_module.print_to_stderr();
        let _ = main_module.print_to_file("Main.ll");
        self.modules.insert(Symbol::new("Main"), main_module.clone());


        #[cfg(feature = "interactive")]
        sp.stop_and_persist("\x1b[32m✔\x1b[0m", "Precompiled main".into());
      },
      _ => println!("maybe here")
    }
  }

  fn module_from_ast(&mut self, ast: Node) -> () {
    match ast {
      Node::Module { .. } => {
        self.create_module_from_ast(ast);
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
    let _ = self.modules.get(&Symbol::new("Main"))
      // .unwrap()
      .expect("You likely don't have a main method which is a bug currently")
      .print_to_file(&file_name);
  }
}
