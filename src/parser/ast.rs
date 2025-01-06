use crate::core;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operator {
  // Assignment,
  Divide,
  Minus,
  Multiply,
  Plus
}

impl fmt::Display for Operator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match &self {
      // Operator::Assignment => write!(f, "="),
      Operator::Divide => write!(f, "/"),
      Operator::Minus => write!(f, "-"),
      Operator::Multiply => write!(f, "*"),
      Operator::Plus => write!(f, "+"),
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
  pub name: core::Symbol,
  pub supertype: Option<Box<Parameter>>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Identifier {
  Parameter(Parameter),
  Symbol(core::Symbol)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
  Char(char),
  Float(f64),
  Int(i64),
}
// pub type Generics = Vec<Parameter>;

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
  // basics
  AbstractType {
    name: core::Symbol,
    params: Box<Node>, 
    supertype: core::Symbol,
  },
  AssignmentExpr {
    identifier: core::Symbol,
    value: Box<Node>
  },
  BinaryExpr {
    op: Operator,
    lhs: Box<Node>,
    rhs: Box<Node>,
  },
  // Char(char),
  Empty,
  // Eoi,
  // Float(f64),
  Function {
    name: core::Symbol,
    args: Vec<core::Symbol>,
    body: Box<Vec<Node>>
  },
  Generics {
    params: Vec<Node>
  },
  ImportExpr {
    module: core::Symbol,
    element: core::Symbol,
  },
  // Parameter {
  //   name: core::Symbol,
  //   subtype: core::Symbol
  // },
  MethodCall {
    name: core::Symbol,
    args: Vec<Box<Node>>
  },
  Module {
    name: core::Symbol,
    exprs: Vec<Box<Node>>
  },
  Operator(Operator),
  Parameter(Parameter),
  ParenthesesExpr {
    expr: Box<Node>
  },
  Primitive(Primitive),
  PrimitiveType {
    name: core::Symbol,
    supertype: core::Symbol,
    bits: u32
  },
  SuperType {
    expr: Box<Node>
  },
  Symbol(core::Symbol),
  UnaryExpr {
    op: Operator,
    child: Box<Node>,
  },
  UsingExpr(core::Symbol),
}

impl fmt::Display for Node {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match &self {
      Node::AbstractType { name, params, supertype } => write!(f, "{} {} {}", name, params, supertype),
      Node::AssignmentExpr { identifier, value } => write!(f, "{} = {}", identifier, value),
      // Node::AbstractType { name, params, subtype } => write!(f, "{} {:?} {}", name, params, subtype),
      Node::BinaryExpr { op, lhs, rhs } => write!(f, "{} {} {}", lhs, op, rhs),
      // Node::Char(c) => write!(f, "{}", c),
      Node::Empty {} => write!(f, ""),
      // Node::Eoi {} => write!(f, ""),
      // Node::Float(val) => write!(f, "{}", val),
      Node::Function { name, args, body } => write!(f, "function {}({:?}) {:?} end", name, args, body),
      Node::Generics { params } => {
        for param in params.iter() {
          write!(f, " {},", param).expect("wtf");
        }
        Ok(())
      },
      Node::ImportExpr { module, element } => write!(f, "{}.{}", module, element),
      // Node::Int(int) => write!(f, "{}", int),
      Node::MethodCall { name, args } => write!(f, "call {}({:?})", name, args),
      Node::Module { name, exprs } => {
        write!(f, "{}", name).expect("wtf");
        for expr in exprs.iter() {
          write!(f, " {},", expr).expect("wtf");
        }
        Ok(())
      },
      Node::Operator(op) => write!(f, "{:?}", op),
      Node::Parameter(param) => write!(f, ":{} <: :{:?}", param.name.name(), param.supertype),
      Node::ParenthesesExpr { expr } => write!(f, "{:?}", expr),
      Node::Primitive(_p) => write!(f, ""),
      Node::PrimitiveType { name, supertype, bits } => write!(f, "{} <: {} {}", name.to_ir(), supertype, bits),
      Node::SuperType { expr } => write!(f, "{}", expr),
      Node::Symbol(name) => write!(f, "{}", name),
      Node::UnaryExpr { op, child } => write!(f, "{}{}", op, child),
      Node::UsingExpr(name) => write!(f, "{}", name),
    }
  }
}
