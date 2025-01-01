use crate::ast::{Node, Operator};
use crate::compiler::Compile;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FloatValue, IntValue};

use std::any::Any;
use std::collections::HashMap;

pub struct Compiler<'a> {
  builder: Builder<'a>,
  context: &'a Context,
  module: Module<'a>
}

impl<'a> Compiler<'a> {
  pub fn new(context: &'a Context) -> Self {
    let module = context.create_module("farnese");
    let builder = context.create_builder();

    // setting up basic stuff for now
    // TODO figure out the minimum set we need
    // Create the main function
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);
    let function = module.add_function("main", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    Self {
      builder: builder,
      context: context,
      module: module
    }
  }

  pub fn build_default_return(&self) -> () {
    // let int = farnese_int(self, Node::Int(0));
    let int = self.context.i32_type().const_int(0, false);
    self.builder.build_return(Some(&int));
  }

  pub fn dump_ir(&self) -> String {
    // self.builder.build_return()
    self.module.print_to_string().to_string()
  }
}

impl<'a> Compile for Compiler<'a> {
  type Output = anyhow::Result<i32>; 

  fn from_ast(&mut self, ast: Vec<Node>) -> Self::Output {
    let mut ret = 0i32;
    for node in ast {
      let _ = self.from_ast_inner(node);
    }
    Ok(ret)
  }

  fn from_ast_inner(&mut self, ast: Node) -> () {
    println!("Node = {:?}", ast);
    // self.evaluator.eval(&node);
    match ast {
      Node::AssignmentExpr { identifier, value } => {
        println!("Assignment");
        // self.from_ast_inner(*value)
        let value = match *value {
          Node::Int(x) => {
            let val = self.context.i64_type().const_int(x.try_into().unwrap(), false);
            let var = self.builder.build_alloca(self.context.i64_type(), identifier.name());
            self.builder.build_store(var.expect("Something wrong here"), val);
          },
          _ => panic!("Unsupported value in assignment {:?}", value)
        };
      },
      Node::Eoi => (),
      // Node::Int(x) => {
      //   // farnese_int(self, ast);
      //   self.context.i64_type().const_int(x.try_into().unwrap(), false);
      // }
      // Node::BinaryExpr { op, lhs, rhs } => {
      //   // need to check all the different types
      //   if lhs.type_id() != rhs.type_id() {
      //     panic!("Currently not supporting mixed types");
      //   }
      //   let left = self.from_ast_inner(*lhs);
      //   let right = self.from_ast_inner(*rhs);
      //   let op_name = match op {
      //     Operator::Minus => {
      //       // match lhs {
      //       //   Node::Int(x) => self.builder.build_int_sub(left, right, "subtmp"),
      //       //   Node::Float(x) => self.builder.build_float_sub(left, right, "subtmp"),
      //       //   _ => panic!("Not supported")
      //       // }
      //       // self.builder.
      //     },
      //     // Operator::Plus => "addtmp",
      //     _ => todo!("Operator {:?} not implemented", op)
      //   };
      // },
      // primitives
      // Node::Float(x) => farnese_float(&self, ast),
      // Node::Int(x) => farnese_int(&self, ast),
      _ => panic!("Not supported {:?}", ast)
      // _ => panic!("NO")
    }
  }
}

pub fn farnese_float<'a>(compiler: &'a Compiler<'a>, node: Node) -> FloatValue<'a> {
  match node {
    Node::Float(x) => {
      compiler.context.f64_type().const_float(x.try_into().unwrap())
    },
    _ => panic!("Shouldn't be called with this {:?}", node)
  }
}

pub fn farnese_int<'a>(compiler: &'a Compiler<'a>, node: Node) -> IntValue<'a> {
  match node {
    Node::Int(x) => {
      compiler.context.i64_type().const_int(x.try_into().unwrap(), false)
    },
    _ => panic!("Shouldn't be called with this {:?}", node)
  }
}
