use crate::core::{DataType, Symbol};
use crate::lexer::ast::Node;
use crate::lexer::lexer;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use std::collections::HashMap;

pub struct Compiler<'a, 'b> {
  builder: &'b Builder<'a>,
  context: &'a Context,
  modules: HashMap<Symbol, Module<'a>>
}

impl<'a, 'b> Compiler<'a, 'b> {
  pub fn new(context: &'a Context, builder: &'b Builder<'a>) -> Self {
    let modules = HashMap::<Symbol, Module<'a>>::new();
    Self {
      builder: builder,
      context: context,
      modules: modules
    }
  }

  pub fn include(&mut self, file_name: &str) -> () {
    let ast: Vec<_> = lexer::parse(file_name).unwrap();
    for node in ast {
      match node {
        Node::Module { name, exprs } => {
          self.compile_module(name, exprs)
        },
        _ => println!("Got a simple source file")
      }
    }
  }

  fn compile_expr(&mut self, expr: Node, module: &Module<'a>) -> () {
    match expr {
      Node::AbstractType { name, params, supertype } => {
        let new_type = DataType::new(name, supertype, module);
      },
      Node::Comment => {},
      Node::PrimitiveType { name, supertype, bits } => {
        let new_type = DataType::new(name, supertype, module);
      },
      Node::StructType { name, generics, supertype, fields } => {
        let new_type = DataType::new(name, supertype, module);
      },
      _ => todo!("Unsupported type {:?}", expr)
    }
  }

  fn compile_module(&mut self, name: Symbol, exprs: Vec<Box<Node>>) -> () {
    // println!("Ast = {:?}", exprs)
    let module = self.context.create_module(name.name());
    for ast in exprs {
      println!("ast = {:?}", ast);
      self.compile_expr(*ast, &module);
    }
    // module.print_to_stderr();
    let _ = module.print_to_file(format!("{}.ll", name.name()));
  }

  // method mainly used to include core...
  pub fn insert_module(&mut self, name: Symbol, module: Module<'a>) -> () {
    self.modules.insert(name, module);
  }

  pub fn modules(&self) -> &HashMap<Symbol, Module<'a>> { &self.modules }
}
