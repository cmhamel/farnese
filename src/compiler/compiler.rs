use crate::ast::{Node, Operator, Primitive};
use crate::base::Symbol;
use crate::compiler::Compile;
use inkwell;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::AnyTypeEnum;
use inkwell::values::{AnyValueEnum, BasicValueEnum, FloatValue, IntValue, PointerValue};

use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::CString;

pub enum Expression<'a> {
//   BinaryExpr(Node::BinaryExpr),
//   Symbol(Symbol),
//   UnaryExpr(Node::UnaryExpr),
  Primitive(AnyTypeEnum<'a>, AnyValueEnum<'a>),
}

#[derive(Clone, Debug)]
pub struct Variable<'a> {
  val_type: AnyTypeEnum<'a>,
  ptr: PointerValue<'a>,
  val: AnyValueEnum<'a>
}

impl<'a> Variable<'a> {
  pub fn new(val_type: AnyTypeEnum<'a>, ptr: PointerValue<'a>, val: AnyValueEnum<'a>) -> Self {
    Self { val_type: val_type, ptr: ptr, val: val }
  }

  pub fn get_pointer(&self) -> &PointerValue<'a> {
    &self.ptr
  }

  pub fn get_type(&self) -> &AnyTypeEnum<'a> {
    &self.val_type
  }

  pub fn get_value(&self) -> &AnyValueEnum<'a> {
    &self.val
  }
}

pub struct Compiler<'a> {
  builder: Builder<'a>,
  context: &'a Context,
  module: Module<'a>,
  scope_sym_table: HashMap<Symbol, Variable<'a>>
}

impl<'a> Compiler<'a> {
  pub fn new(context: &'a Context) -> Self {
    let module = context.create_module("farnese");
    let builder = context.create_builder();

    // setting up basic stuff for now
    // TODO figure out the minimum set we need

    // Create the `printf` function declaration
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
    let printf_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], true);
    let _ = module.add_function("printf", printf_type, None);

    // Create the main function
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);
    let function = module.add_function("main", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    // things used to store symbols, types etc.
    let scope_sym_table = HashMap::<Symbol, Variable<'a>>::new();

    Self {
      builder: builder,
      context: context,
      module: module,
      scope_sym_table: scope_sym_table
    }
  }



  // pub fn binary_expr(&self, op: Operator, lhs: Node, rhs: Node) -> (AnyValueEnum<'a>, AnyTypeEnum<'a>) {
  //   match op {
  //     Operator::Plus => match lhs {
  //       AnyValueEnum::IntValue(x) => match rhs {
  //         AnyValueEnum::IntValue(y) => {
  //           (AnyValueEnum::IntValue(self.builder.build_int_add(x, y, "__add_temp").unwrap()),
  //            AnyTypeEnum::IntType(self.context.i64_type()))
  //         },
  //         _ => panic!("Unsupported operator combination")
  //       },
  //       _ => panic!("Unsupported primitive type in binary")
  //     }
  //   }
  // }

  // pub fn build_alloca(&mut self, val_type: AnyTypeEnum<'a>, string: &str) -> PointerValue<'a> {
  //   let ptr_val = match val_type {
  //     AnyTypeEnum::FloatType(x) => self.builder.build_alloca(x, string),
  //     AnyTypeEnum::IntType(x) => self.builder.build_alloca(x, string),
  //     _ => panic!("Bad type in compiler build_alloca")
  //   };
  //   ptr_val.unwrap()
  // }

  pub fn build_default_return(&self) -> () {
    let int = self.context.i32_type().const_int(0, false);
    let _ = self.builder.build_return(Some(&int));
  }

  pub fn build_format_string(&mut self, string: &str, name: &str) -> () {
    let c_string = CString::new(string).unwrap();
    let global_string = self.context.const_string(c_string.as_bytes_with_nul(), true);
    let string_type = global_string.get_type();
    let global_var = self.module.add_global(string_type, None, name);
    global_var.set_initializer(&global_string);
    global_var.set_constant(true);
  }

  pub fn build_load(&mut self, sym: &Symbol, var: Variable<'a>) -> BasicValueEnum<'a> {
    self.builder.build_load(*var.get_pointer(), sym.name()).unwrap()
  }

  pub fn build_pointer(&mut self, sym: &Symbol, val_type: AnyTypeEnum<'a>) -> PointerValue<'a> {
    match val_type {
      AnyTypeEnum::FloatType(t) => self.builder.build_alloca(t, sym.name()).unwrap(),
      AnyTypeEnum::IntType(t) => self.builder.build_alloca(t, sym.name()).unwrap(),
      _ => panic!("wtf")
    }
  }

  pub fn build_primitive(&mut self, sym: Symbol, val: Primitive) -> Variable<'a> {
    let (val_type, val): (AnyTypeEnum, AnyValueEnum) = match val {
      Primitive::Char(x) => {
        let val_type = self.context.i8_type();
        let val = val_type.const_int((x as u8).into(), false);
        (val_type.try_into().unwrap(), val.try_into().unwrap())
      },
      Primitive::Float(x) => {
        let val_type = self.context.f64_type();
        let val = val_type.const_float(x);
        (val_type.try_into().unwrap(), val.try_into().unwrap())
      },
      Primitive::Int(x) => {
        let val_type = self.context.i64_type();
        let val = val_type.const_int(x as u64, true);
        (val_type.try_into().unwrap(), val.try_into().unwrap())
      }
    };

    let ptr = self.build_pointer(&sym, val_type);
    let _ = self.build_store(ptr, val);
    let var = Variable::new(val_type, ptr, val);
    // self.scope_sym_table.insert(sym, var.clone());
    var
  }

  pub fn build_store(&mut self, ptr: PointerValue<'a>, val: AnyValueEnum) -> () {
    let _ = match val {
      AnyValueEnum::FloatValue(x) => self.builder.build_store::<FloatValue>(ptr, x),
      AnyValueEnum::IntValue(x) => self.builder.build_store::<IntValue>(ptr, x),
      _ => panic!("Not supported store.")
    };
  }

  pub fn clear_scope(&mut self) -> () {
    self.scope_sym_table.clear();
  }

  pub fn dump_ir(&self) -> String {
    self.module.print_to_string().to_string()
  }

  pub fn evaluate_expr(&mut self, sym: Symbol, expr: Node) -> Variable<'a> {
    println!("Expr = {:?}", expr);
    match expr {
      // Node::BinaryExpr { op, lhs, rhs } => farnese_binary_operator(
      //   self.context, &self.builder, op, 
      //   self.evaluate_expr(*lhs).0,
      //   self.evaluate_expr(*rhs).0
      // ),
      Node::BinaryExpr { op, lhs, rhs } => {
        let lhs_sym = Symbol::new("__lhstmp");
        let rhs_sym = Symbol::new("__rhstmp");
        let lhs_expr = self.evaluate_expr(lhs_sym.clone(), *lhs);
        let rhs_expr = self.evaluate_expr(rhs_sym.clone(), *rhs);
        // let lhs_expr = self.scope_sym_table.get(&lhs_sym).unwrap();
        // let rhs_expr = self.scope_sym_table.get(&rhs_sym).unwrap();
        let binary_expr = farnese_binary_operator(self.context, &self.builder, op, *lhs_expr.get_value(), *rhs_expr.get_value());
        // let ptr = self.builder.build_alloca(binary_expr.0, sym.name()).unwrap();
        let ptr = self.build_pointer(&sym, binary_expr.0);
        let _ = self.build_store(ptr, binary_expr.1);
        let var = Variable::new(binary_expr.0, ptr, binary_expr.1);
        // let _ = self.build_load(&sym, var.clone());
        // self.scope_sym_table.insert(sym, var);
        // Variable { binary_expr:, }
        // match binary_expr {
        //   AnyTypeEnum(x) => self.scope_sym_table.insert(sym, x),
        //   _ => panic!("wtf in this match")
        // }
        var
      },
      Node::ParenthesesExpr { expr } => self.evaluate_expr(sym, *expr),
      Node::Primitive(x) => self.build_primitive(sym, x),
      Node::Symbol(x) => self.scope_sym_table.get(&x).unwrap().clone(),
      _ => panic!("Invalid expression type sym = {}, expr = {:?}", sym, expr)
    }
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
        println!("Adding variable {} = {:?}", identifier, expr);
        self.scope_sym_table.insert(identifier, expr);
      }
      // Node::BinaryExpr { op, lhs, rhs } => {
      //   println!("Binaryexpr");
      // }
      Node::Empty => (),
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
                  let printf = self.module.get_function("printf").unwrap();
                  let zero = self.context.i32_type().const_int(0, false);

                  // get the format string based on type
                  let fmt_str = match loaded_value {
                    Ok(BasicValueEnum::FloatValue(_x)) => self.module.get_global("__format_f64").unwrap(),
                    Ok(BasicValueEnum::IntValue(_x)) => self.module.get_global("__format_i64").unwrap(),
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
      // Node::Symbol(x) => {
      //   let var_value = self.scope_sym_table.get(&x).expect("Failed to find variable in scope sym table");
      //   let var = self.builder.build_alloca(var_value.get_value_type(), x.name()).unwrap();
      // },
      _ => panic!("Not supported {:?}", ast)
      // _ => panic!("NO")
    }
  }
}

// pub fn farnese_expr<'a>(
//   context: &'a Context, builder: &Builder<'a>, val: Node
// ) -> (AnyValueEnum<'a>, AnyTypeEnum<'a>) {
//   match val {
//     Node::BinaryExpr { op, lhs, rhs } => farnese_binary_operator(
//       context, builder, op, 
//       farnese_expr(context, builder, *lhs).0, 
//       farnese_expr(context, builder, *rhs).0
//     ),
//     Node::Primitive(x) => farnese_primitive(context, x),
//     // Node::Symbol(x) => panic!("have a symbol"),
//     // Node::Symbol(x) => builder.build_load(x.name(), "__loaded_value").unwrap(),
//     Node::UnaryExpr { op, child } => farnese_unary_operator(
//       context, builder, op,
//       farnese_expr(context, builder, *child).0
//     ),
//     _ => panic!("unsupported type in expression {:?}", val)
//   }
// }

pub fn farnese_binary_operator<'a>(
  context: &'a Context, builder: &Builder<'a>, 
  op: Operator, lhs: AnyValueEnum<'a>, rhs: AnyValueEnum<'a>
) -> (AnyTypeEnum<'a>, AnyValueEnum<'a>) {
  match op {
    // todo need to check if things are equal or not

    // Operator::Divide => (),
    // Operator::Minus => (),
    // Operator::Multiply => (),
    // Operator::Plus => builder.build_int_add(lhs, rhs, "__addtmp").unwrap(),
    Operator::Minus => match lhs {
      AnyValueEnum::IntValue(x) => match rhs {
        AnyValueEnum::IntValue(y) => {
          (
            AnyTypeEnum::IntType(context.i64_type()),
            AnyValueEnum::IntValue(builder.build_int_sub(x, y, "__binary_subtemp").unwrap()),
          )
          // let val_type = context.i64_type();
          // let val = val_type.const_int(x as u64, true);
          // let ptr = self.builder.build_alloca(val_type, sym.name()).unwrap();
          // let _ = self.builder.build_store::<IntValue>(ptr, val);
          // Variable { val_type: AnyTypeEnum::IntType(val_type), ptr: ptr, val: AnyValueEnum::IntValue(val) }
        }
        _ => panic!("Unsupported operator combination")
      },
      _ => panic!("Unsupported primitive type in binary operation")
    },
    Operator::Plus => match lhs {
      AnyValueEnum::IntValue(x) => match rhs {
        AnyValueEnum::IntValue(y) => {
          (
            AnyTypeEnum::IntType(context.i64_type()),
            AnyValueEnum::IntValue(builder.build_int_add(x, y, "__binary_addtmp").unwrap()),
          )
          // let val_type = context.i64_type();
          // let val = val_type.const_int(x as u64, true);
          // let ptr = self.builder.build_alloca(val_type, sym.name()).unwrap();
          // let _ = self.builder.build_store::<IntValue>(ptr, val);
          // Variable { val_type: AnyTypeEnum::IntType(val_type), ptr: ptr, val: AnyValueEnum::IntValue(val) }
        }
        _ => panic!("Unsupported operator combination")
      },
      _ => panic!("Unsupported primitive type in binary operation")
    },
    _ => panic!("Not supported operator")
  }
}

// pub fn farnese_unary_operator<'a>(
//   context: &'a Context, builder: &Builder<'a>, 
//   op: Operator, child: AnyValueEnum<'a>
// ) -> (AnyValueEnum<'a>, AnyTypeEnum<'a>) {
//   match op {
//     Operator::Minus => panic!("minus"),
//     Operator::Plus => panic!("plus"),
//     _ => panic!("unsupported unary operator")
//   }
// }

// pub fn farnese_primitive<'a>(context: &'a Context, val: Primitive) -> (AnyValueEnum<'a>, AnyTypeEnum<'a>) {
//   println!("val = {:?}", val);
//   match val {
//     Primitive::Char(x) => (farnese_char(context, x.try_into().unwrap()), AnyTypeEnum::IntType(context.i8_type())),
//     Primitive::Float(x) => (farnese_float(context, x.try_into().unwrap()), AnyTypeEnum::FloatType(context.f64_type())),
//     Primitive::Int(x) => (farnese_int(context, x.try_into().unwrap()), AnyTypeEnum::IntType(context.i64_type())),
//     _ => panic!("cant call this method a non-primitive ast node! {:?}", val)
//   }
// }

// // different primitive types
// pub fn farnese_char<'a>(context: &'a Context, val: char) -> AnyValueEnum<'a> {
//   let val = context.i8_type().const_int((val as u8).into(), false);
//   AnyValueEnum::IntValue(val)
// }

// pub fn farnese_float<'a>(context: &'a Context, val: f64) -> AnyValueEnum<'a> {
//   let val = context.f64_type().const_float(val);
//   AnyValueEnum::FloatValue(val)
// }

// pub fn farnese_int<'a>(context: &'a Context, val: i64) -> AnyValueEnum<'a> {
//   let val = context.i64_type().const_int(val as u64, true);
//   AnyValueEnum::IntValue(val)
// }
