use crate::compiler::Compile;
use crate::core::value::Value;
use crate::core::{Symbol, Variable, core};
use crate::parser::ast::{Node, Operator, Primitive};
use inkwell;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Module};
use inkwell::passes::PassManager;
use inkwell::types::AnyTypeEnum;
use inkwell::values::{AnyValueEnum, BasicValueEnum, FloatValue, IntValue, PointerValue};
use inkwell::AddressSpace;
#[cfg(feature = "interactive")]
use spinners::{Spinner, Spinners};
use std::cmp::Ordering;
use std::collections::HashMap;

// TODO

// need to refactor this so we create core in these methods some how?
// or at least we need to make sure that builder and context are handled appropriately.
// it may be wise to create a seperate context/builder for creating core
// and then see if we can re-load the module
// we can always save things to ir ll files and re-load

pub struct Compiler<'a> {
  builder: Builder<'a>,
  context: &'a Context,
  modules: HashMap<Symbol, Module<'a>>,
  scope_sym_table: HashMap<Symbol, Variable<'a>>
  // scope_sym_table: HashMap<Symbol, PointerValue<'a>>
}

impl<'a> Compiler<'a> {
  pub fn new(context: &'a Context) -> Self {
    // some types
    let i32_type = context.i32_type();

    // create builder
    let builder = context.create_builder();

    // let module = context.create_module("farnese");
    #[cfg(feature = "interactive")]
    let mut main_sp = Spinner::new(Spinners::Dots, "Precompiling Main...".into());

    let main_module = context.create_module("Main");
    let _ = main_module.add_global_metadata("module", &context.metadata_string("Main"));


    // create the core module and link into main
    #[cfg(feature = "interactive")]
    let mut core_sp = Spinner::new(Spinners::Dots, "Precompiling Core...".into());

    // let core_module = core::create_core(context);
    let core_module = core::Core::new(context);
    let data_type = core_module.data_type;
    // let type_tag = core_module.type_tag;
    // let value_type = core_module.value_type;
    // let any_type = core_module.any_type;

    #[cfg(feature = "interactive")]
    core_sp.stop_and_persist("\x1b[32m✔\x1b[0m", "Precompiled Core".into());

    main_module.link_in_module(core_module.module.clone()).unwrap();

    // things used to store symbols, types etc.
    let scope_sym_table = HashMap::<Symbol, Variable<'a>>::new();

    #[cfg(feature = "interactive")]
    main_sp.stop_and_persist("\x1b[32m✔\x1b[0m", "Precompiled Main".into());

    // Create the main function
    // TODO figure out where to put this
    let fn_type = i32_type.fn_type(&[], false);
    let function = main_module.add_function("main", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    // trying to make a value

    // todo turn below into a function
    // let my_val = Value::<i32>::new(42, core_module.any_type.clone());
    // let main_module = my_val.create_value(context, builder, main_module.clone(), core_module.data_type);
    let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
    let i32_type = context.i32_type();
    // let builder = context.create_builder();
    let value_struct = context.struct_type(
      &[
        i8_ptr_type.into(),
        data_type.ptr_type(AddressSpace::default()).into(),
      ],
      false,
    );
    // Allocate space for a `Value` instance
    let value_alloca = builder.build_alloca(value_struct, "value").unwrap();

    // Initialize the `i8*` field with a pointer to an integer
    let int_value = i32_type.const_int(69, false); // todo
    let int_ptr = builder.build_alloca(i32_type, "int_ptr").unwrap();
    builder.build_store(int_ptr, int_value);
    let int_i8_ptr = builder.build_bit_cast(int_ptr, i8_ptr_type, "int_i8_ptr");

    // Set the `i8*` field of `Value`
    let value_data_ptr = unsafe {
        builder.build_struct_gep(value_alloca, 0, "value_data_ptr").unwrap()
    };
    builder.build_store(value_data_ptr, int_i8_ptr.unwrap());

    // Set the `%DataType*` field to null
    let value_type_ptr = unsafe {
        builder.build_struct_gep(value_alloca, 1, "value_type_ptr").unwrap()
    };
    builder.build_store(value_type_ptr, data_type.ptr_type(AddressSpace::default()).const_null());















    // let func = core_module.module.get_function("__Any__new").unwrap();
    // let _ = builder.build_call(func, &[], "__call__Any_new");
    // main return statement with default 0
    let _ = builder.build_return(Some(&i32_type.const_int(0, false)));

    // create modules hashmap
    let mut modules = HashMap::<Symbol, Module<'a>>::new();
    modules.insert(Symbol::new("Main"), main_module);
    modules.insert(Symbol::new("Core"), core_module.module);
        



    Self {
      builder: builder,
      context: context,
      // module: main_module,
      modules: modules,
      scope_sym_table: scope_sym_table
    }
  }

  fn build_binary_operator(&mut self, op: Operator, lhs: AnyValueEnum<'a>, rhs: AnyValueEnum<'a>) -> AnyValueEnum<'a> {
    match op {
      Operator::Minus => match lhs {
        AnyValueEnum::IntValue(x) => match rhs {
          AnyValueEnum::IntValue(y) => {
            AnyValueEnum::IntValue(self.builder.build_int_sub(x, y, "__binary_subtmp").unwrap())
          }
          _ => panic!("Unsupported operator combination")
        },
        _ => panic!("Unsupported primitive type in binary operation")
      },
      Operator::Plus => match lhs {
        AnyValueEnum::IntValue(x) => match rhs {
          AnyValueEnum::IntValue(y) => {
            AnyValueEnum::IntValue(self.builder.build_int_add(x, y, "__binary_addtmp").unwrap())
          }
          _ => panic!("Unsupported operator combination")
        },
        _ => panic!("Unsupported primitive type in binary operation")
      },
      _ => panic!("Not supported operator")
    }
  }

  pub fn build_default_return(&self) -> () {
    let int = self.context.i32_type().const_int(0, false);
    let _ = self.builder.build_return(Some(&int));
  }

  fn build_load(&mut self, sym: &Symbol, var: Variable<'a>) -> BasicValueEnum<'a> {
    self.builder.build_load(*var.get_pointer(), sym.name()).unwrap()
  }

  fn build_module(&mut self, exprs: Node) -> Module<'a> {
    match exprs {
      Node::Module { name, exprs } => {
        #[cfg(feature = "interactive")]
        let mut sp = Spinner::new(Spinners::Dots, format!("Precompiling {}...", name.name()).into());
        let module = self.context.create_module(name.name());
        for expr in exprs {
          self.from_ast_inner(*expr);
        }
        let main_module = self.modules.get(&Symbol::new("Main")).unwrap();
        main_module.link_in_module(module.clone()).unwrap();
        #[cfg(feature = "interactive")]
        sp.stop_and_persist("\x1b[32m✔\x1b[0m", format!("Precompiled {}", name.name()).into());
        module
      },
      _ => panic!("Bad node type in build_module")
    }
  }

  fn build_pointer(&mut self, sym: &Symbol, val_type: AnyTypeEnum<'a>) -> PointerValue<'a> {
    match val_type {
      AnyTypeEnum::FloatType(t) => self.builder.build_alloca(t, sym.name()).unwrap(),
      AnyTypeEnum::IntType(t) => self.builder.build_alloca(t, sym.name()).unwrap(),
      _ => panic!("wtf")
    }
  }

  fn build_primitive(&mut self, sym: Symbol, val: Primitive) -> Variable<'a> {
    let (val_type, val): (AnyTypeEnum, BasicValueEnum) = match val {
      Primitive::Char(x) => {
        let val_type = self.context.i8_type();
        (val_type.try_into().unwrap(), val_type.const_int((x as u8).into(), false).try_into().unwrap())
      },
      Primitive::Float(x) => {
        let val_type = self.context.f64_type();
        (val_type.try_into().unwrap(), val_type.const_float(x).try_into().unwrap())
      },
      Primitive::Int(x) => {
        let val_type = self.context.i64_type();
        (val_type.try_into().unwrap(), val_type.const_int(x as u64, true).try_into().unwrap())
      }
    };


    // try ing out general type stuff
    let ptr = self.build_pointer(&sym, val_type);
    let _ = self.build_store(ptr, val.into());
    let var = Variable::new(val.get_type(), ptr, val.into());
    // self.scope_sym_table.insert(sym, var.clone());
    var
  }

  fn build_store(&mut self, ptr: PointerValue<'a>, val: AnyValueEnum) -> () {
    let _ = match val {
      AnyValueEnum::FloatValue(x) => self.builder.build_store::<FloatValue>(ptr, x),
      AnyValueEnum::IntValue(x) => self.builder.build_store::<IntValue>(ptr, x),
      _ => panic!("Not supported store.")
    };
  }

  fn build_unary_operator(&mut self, op: Operator, child: AnyValueEnum<'a>) -> AnyValueEnum<'a> {
    let expr: AnyValueEnum = match op {
      Operator::Minus => match child {
        AnyValueEnum::FloatValue(x) => {
          let zero = x.get_type().const_float(0.0);
          self.builder.build_float_sub(zero, x, "__unary_neg").unwrap().try_into().unwrap()
        },
        AnyValueEnum::IntValue(x) => {
          let zero = x.get_type().const_int(0, true);
          self.builder.build_int_sub(zero, x, "__unary_neg").unwrap().try_into().unwrap()
        },
        _ => panic!("Unsupported type in unary operator")
      },
      Operator::Plus => child,
      _ => panic!("Unsupported operator in unary operator")
    };
    expr
  }

  pub fn clear_scope(&mut self) -> () {
    self.scope_sym_table.clear();
  }

  fn evaluate_expr(&mut self, sym: Symbol, expr: Node) -> Variable<'a> {
    match expr {
      Node::BinaryExpr { op, lhs, rhs } => {
        let lhs_sym = Symbol::new("__lhstmp");
        let rhs_sym = Symbol::new("__rhstmp");
        let lhs_expr = self.evaluate_expr(lhs_sym.clone(), *lhs);
        let rhs_expr = self.evaluate_expr(rhs_sym.clone(), *rhs);
        let binary_expr = self.build_binary_operator(op, *lhs_expr.get_value(), *rhs_expr.get_value());
        let ptr = self.build_pointer(&sym, binary_expr.get_type());
        let _ = self.build_store(ptr, binary_expr);
        let var = Variable::new(binary_expr.get_type().try_into().unwrap(), ptr, binary_expr);
        var
      },
      Node::ParenthesesExpr { expr } => self.evaluate_expr(sym, *expr),
      Node::Primitive(x) => self.build_primitive(sym, x),
      Node::Symbol(x) => {
        let var = self.scope_sym_table.get(&x).unwrap().clone();
        let _ = self.build_load(&sym, var.clone());
        var
      },
      Node::UnaryExpr { op, child } => {
        let child_sym = Symbol::new("__unarytmp");
        let child_expr = self.evaluate_expr(child_sym.clone(), *child);
        let unary_expr = self.build_unary_operator(op, *child_expr.get_value());
        let ptr = self.build_pointer(&sym, unary_expr.get_type());
        let _ = self.build_store(ptr, unary_expr);
        let var = Variable::new(unary_expr.get_type().try_into().unwrap(), ptr, unary_expr);
        var
      }
      _ => panic!("Invalid expression type sym = {}, expr = {:?}", sym, expr)
    }
  }

  pub fn optimize_ir(&self) -> () {
    let module = self.modules.get(&Symbol::new("Main")).unwrap();
    module.print_to_stderr();
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

  pub fn write_ir_to_file(&self, file_name: &str) -> () {
    let _ = self.modules.get(&Symbol::new("Main")).unwrap().print_to_file(&file_name);
  }
}

impl<'a> Compile for Compiler<'a> {
  type Output = anyhow::Result<i32>; 

  fn from_ast(&mut self, ast: Vec<Node>) -> () {
    for node in ast {
      let _ = self.from_ast_inner(node);
    }
  }

  fn from_ast_inner(&mut self, ast: Node) -> () {
    match ast {
      Node::AssignmentExpr { identifier, value } => {
        let expr = self.evaluate_expr(identifier.clone(), *value); 
        self.scope_sym_table.insert(identifier, expr);
      },
      Node::Empty => (),
      Node::ImportExpr { module, element } => {
        let my_mod = self.modules.get(&module).unwrap();
        let my_elem = my_mod.get_function(element.name()).unwrap();
        // try import globals first
        // let my_elem = match my_mod.get_global(element.name()) {
        //   Some(x) => my_mod.get_global("Any").unwrap(),
        //   // _ => ()
        //   _ => Err(my_mod.get_global("Any").unwrap())
        // };
        // let my_glob = my_mod.get_global(element.name());
        // let my_func = my_mod.get_function(element.name());
        // let my_struct = my_mod.get_struct_type(element.name());
        let main_mod = self.modules.get(&Symbol::new("Main")).unwrap();
        
        // match my_glob {
        //   Some(x) => {
        //     // main_mod.add_global(element.name(), my_glob.get_type(), None);
        //     // main_mod.add_global(x, None, element.name());
        //   },
        //   _ => ()
        // }
        main_mod.add_function(element.name(), my_elem.get_type(), None);
      },
      Node::MethodCall { name, args } => {
        // check for the few built in methods like print
        match name.name().cmp("printf") {
          Ordering::Equal => {
            for arg in args {
              match *arg {
                Node::Symbol(x) => {
                  let loaded_value = self.builder.build_load(
                    // *self.scope_sym_table.get(&x).unwrap().get_pointer(),
                    *self.scope_sym_table.get(&x).expect(format!("Variable not found {}", x).as_str()).get_pointer(),
                    "__loaded_value"
                  );

                  // Call `printf` with the format string pointer and loaded value
                  let printf = self.modules.get(&Symbol::new("Main")).unwrap().get_function("printf").unwrap();
                  let zero = self.context.i32_type().const_int(0, false);

                  // get the format string based on type
                  let fmt_str = match loaded_value {
                    Ok(BasicValueEnum::FloatValue(_x)) => self.modules.get(&Symbol::new("Main")).unwrap().get_global("__format_f64").unwrap(),
                    Ok(BasicValueEnum::IntValue(_x)) => self.modules.get(&Symbol::new("Main")).unwrap().get_global("__format_i64").unwrap(),
                    _ => panic!("unsupported type in get_format_string")
                  };
                  
                  // now build the gep and call
                  let format_string_ptr = unsafe {
                    self.builder.build_gep(
                        // self.module.get_global("format").unwrap().as_pointer_value(),
                        fmt_str.as_pointer_value(),
                        &[zero, zero],
                        "format_ptr",
                    )
                  };
                  let _ = self.builder.build_call(
                    printf, 
                    &[format_string_ptr.unwrap().into(), loaded_value.unwrap().into()], 
                    "printf_call"
                  );
                },
                _ => panic!("unsupported printf arg")
              }
            }
          }
          _ => panic!("Unsupported method call")
        }
      },
      Node::Module { .. } => {
        // self.build_module(ast);
        println!("skipping building modules for now. Go to from_ast_inner to undo this");
      },
      _ => panic!("Not supported {:?}", ast)
    }
  }
}
