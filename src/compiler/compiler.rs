use crate::compiler::Compile;
use crate::core::{Symbol, Variable, core};
use crate::parser::ast::{Node, Operator, Primitive};
use inkwell;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::AnyTypeEnum;
use inkwell::values::{AnyValueEnum, BasicValueEnum, FloatValue, IntValue, PointerValue};
use spinners::{Spinner, Spinners};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::CString;
use std::{thread, time::Duration};

// helpers
// static CORE: Symbol = Symbol::new("Core");
// const CORE: Symbol = Symbol::new("wtf");

pub struct Compiler<'a> {
  builder: Builder<'a>,
  context: &'a Context,
  modules: HashMap<Symbol, Module<'a>>,
  scope_sym_table: HashMap<Symbol, Variable<'a>>
  // scope_sym_table: HashMap<Symbol, PointerValue<'a>>
}

impl<'a> Compiler<'a> {
  pub fn new(context: &'a Context) -> Self {
    // create builder
    let builder = context.create_builder();

    // let module = context.create_module("farnese");
    let mut main_sp = Spinner::new(Spinners::Dots, "Precompiling Main...".into());
    let main_module = context.create_module("Main");

    // Create the main function
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);
    let function = main_module.add_function("main", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    // create the core module and link into main
    let mut core_sp = Spinner::new(Spinners::Dots, "Precompiling Core...".into());
    let core_module = core::create_core(context);
    core_sp.stop_and_persist("\x1b[32m✔\x1b[0m", "Precompiled Core".into());

    main_module.link_in_module(core_module.clone()).unwrap();
    // link in methods
    // let printf = core_module.get_function("printf").unwrap();
    // main_module.add_function("printf", printf.get_type(), None);
    // main_module.add_global(core_module.get_global("Any").unwrap().get_value_type(), None, "Any");
    // println!("My func = {:?}", core_module.get_function("printf").unwrap().get_type());
    // setting up basic stuff for now
    // TODO figure out the minimum set we need

    // Create the `printf` function declaration
    // let i8_type = context.i8_type();
    // let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
    // let printf_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], true);
    // let _ = main_module.add_function("printf", printf_type, None);

    // create modules hashmap
    let mut modules = HashMap::<Symbol, Module<'a>>::new();
    modules.insert(Symbol::new("Main"), main_module);
    modules.insert(Symbol::new("Core"), core_module);

    // things used to store symbols, types etc.
    let scope_sym_table = HashMap::<Symbol, Variable<'a>>::new();

    // thread::sleep(Duration::from_secs(3));
    main_sp.stop_and_persist("\x1b[32m✔\x1b[0m", "Precompiled Main".into());

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

  pub fn build_format_string(&mut self, string: &str, name: &str) -> () {
    let c_string = CString::new(string).unwrap();
    let global_string = self.context.const_string(c_string.as_bytes_with_nul(), true);
    let string_type = global_string.get_type();
    let global_var = self.modules.get(&Symbol::new("Main")).unwrap().add_global(string_type, None, name);
    global_var.set_initializer(&global_string);
    global_var.set_constant(true);
  }

  fn build_load(&mut self, sym: &Symbol, var: Variable<'a>) -> BasicValueEnum<'a> {
    self.builder.build_load(*var.get_pointer(), sym.name()).unwrap()
  }

  fn build_module(&mut self, exprs: Node) -> Module<'a> {
    match exprs {
      Node::Module { name, exprs } => {
        let mut sp = Spinner::new(Spinners::Dots, format!("Precompiling {}...", name.name()).into());
        let module = self.context.create_module(name.name());
        for expr in exprs {
          self.from_ast_inner(*expr);
        }
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

  // pub fn dump_ir(&self) -> String {
  //   let ir = self.modules.get(&Symbol::new("Main")).unwrap().print_to_string().to_string()
    
  // }

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
      Node::ImportExpr { module: module, element: element } => {
        let my_mod = self.modules.get(&module).unwrap();
        // let _ self.builder.
        let my_elem = my_mod.get_function(element.name()).unwrap();
        let main_mod = self.modules.get(&Symbol::new("Main")).unwrap();
        
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
      Node::Module { ref name, ref exprs } => {
        self.build_module(ast);
      },
      _ => panic!("Not supported {:?}", ast)
    }
  }
}
