// use crate::{Compile, Node, Operator, Result};
use crate::ast::Node;
use crate::base::{DataType, Symbol};
use crate::compiler::Compile;
// use crate::parser::Rule;
use anyhow;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Interpreter {
  evaluator: Eval
}

impl Interpreter {
  pub fn new() -> Self {
    Self {
      evaluator: Eval::new()
    }
  }
}

impl Compile for Interpreter {
  type Output = anyhow::Result<i32>; 

  fn from_ast(&mut self, ast: Vec<Node>) -> () {
    for node in ast {
      self.evaluator.eval(&node);
    }
  }
}

struct Eval {
  datatypes: HashMap<Symbol, DataType>,
  symbols: HashMap<Symbol, Symbol>,
  values: HashMap<Symbol, Node>
}

impl Eval {
  pub fn new() -> Self {
    let mut datatypes = HashMap::<Symbol, DataType>::new();
    let mut symbols = HashMap::<Symbol, Symbol>::new();
    let mut values = HashMap::<Symbol, Node>::new();
    let any_sym = Symbol::new("Any");
    let any_type = DataType {
      name: any_sym.clone().into(),
      fields: None,
      parameters: None,
      supertype: None,
      is_abstract: true,
      is_mutable: false,
      is_primitive: false
    };
    datatypes.insert(any_sym.clone(), any_type);
    symbols.insert(any_sym.clone(), any_sym.clone());
    Self {
      datatypes: datatypes,
      symbols: symbols,
      values: values
    }
  }
  pub fn eval(&mut self, node: &Node) -> () {
    match node {
      Node::AbstractType { name, params } => {
        println!("Adding AbstractType = {} {}", name, params);
        
        if self.datatypes.contains_key(name) ||
           self.symbols.contains_key(name) {
          panic!("type {} already in type table", name)
        }

        println!("Params = {:?}", params);

        let new_type = DataType {
          name: Arc::new(name.clone()),
          fields: None,
          parameters: None,
          // supertype: self.datatypes.get(**supertype),
          supertype: None,
          is_abstract: true,
          is_mutable: false,
          is_primitive: false
        };
        self.datatypes.insert(name.clone(), new_type);
        self.symbols.insert(name.clone(), name.clone());
        // for param in params {
        //   self.symbols.push(param);
        // }
        println!("Currently available types = ");
        for (sym, datatype) in self.datatypes.clone() {
          println!("{} => {}", sym, datatype);
        }
      },
      Node::AssignmentExpr { identifier, value } => {
        println!("Assignment = {} := {}", identifier, value);
        self.symbols.insert(identifier.clone(), identifier.clone());
        self.values.insert(identifier.clone(), *value.clone());
      }
      Node::BinaryExpr { op, lhs, rhs } => {
        println!("BinaryExpr = {} {} {}", op, lhs, rhs);
      }
      Node::Float(val) => {
        println!("Float value = {}", val);
      },
      Node::Int(val) => {
        println!("Int value = {}", val);
      },
      Node::PrimitiveType { name, supertype, bits } => { 
        println!("Adding PrimitiveType = {} <: {} {}", name.to_ir(), supertype.to_ir(), bits);
        let new_type = DataType {
          name: Arc::new(name.clone()),
          fields: None,
          parameters: None,
          supertype: None,
          is_abstract: false,
          is_mutable: false,
          is_primitive: true
        };
        self.datatypes.insert(name.clone(), new_type);
        self.symbols.insert(name.clone(), name.clone());
      },
      Node::Symbol(name) => println!("Symbol = {}", name),
      Node::UnaryExpr { op, child } => println!("UnaryExpr = {} {}", op, child),
      _ => println!("Here in interpreter {}", node)
//       Node::Int(n) => *n,
//       Node::UnaryExpr { op, child } => {
//         let child = self.eval(child);
//         match op {
//           Operator::Plus => child,
//           Operator::Minus => -child,
//         }
//       }
//       Node::BinaryExpr { op, lhs, rhs } => {
//         let lhs_ret = self.eval(lhs);
//         let rhs_ret = self.eval(rhs);

//         match op {
//           Operator::Plus => lhs_ret + rhs_ret,
//           Operator::Minus => lhs_ret - rhs_ret,
//         }
//       }
    }
  }
}
