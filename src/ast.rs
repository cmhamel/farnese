use crate::base;
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
  pub name: base::Symbol,
  pub supertype: Option<Box<Parameter>>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Identifier {
  Parameter(Parameter),
  Symbol(base::Symbol)
}

// pub type Generics = Vec<Parameter>;

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
  // basics
  AbstractType {
    name: base::Symbol,
    params: Box<Node>, 
    // supertype: Box<Node>
    // subtype: Box<Node>
  },
  AssignmentExpr {
    identifier: base::Symbol,
    value: Box<Node>
  },
  BinaryExpr {
    op: Operator,
    lhs: Box<Node>,
    rhs: Box<Node>,
  },
  Eoi,
  Float(f64),
  Function {
    name: base::Symbol,
    args: Vec<base::Symbol>,
    body: Box<Vec<Node>>
  },
  Generics {
    params: Vec<Node>
  },
  Int(i64),
  // Parameter {
  //   name: base::Symbol,
  //   subtype: base::Symbol
  // },
  MethodCall {
    name: base::Symbol,
    args: Vec<Box<Node>>
  },
  Parameter(Parameter),
  PrimitiveType {
    name: base::Symbol,
    supertype: base::Symbol,
    bits: u32
  },
  // SubType {
  //   name: base::Symbol
  // },
  // Symbol {
  //   name: base::Symbol
  // },
  Symbol(base::Symbol),
  UnaryExpr {
    op: Operator,
    child: Box<Node>,
  },
}

impl fmt::Display for Node {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match &self {
      Node::Int(n) => write!(f, "{}", n),
      Node::AbstractType { name, params } => write!(f, "{} {}", name, params),
      Node::AssignmentExpr { identifier, value } => write!(f, "{} = {}", identifier, value),
      // Node::AbstractType { name, params, subtype } => write!(f, "{} {:?} {}", name, params, subtype),
      Node::BinaryExpr { op, lhs, rhs } => write!(f, "{} {} {}", lhs, op, rhs),
      Node::Eoi {} => write!(f, ""),
      Node::Float(val) => write!(f, "{}", val),
      Node::Function { name, args, body } => write!(f, "function {}({:?}) {:?} end", name, args, body),
      Node::Generics { params } => {
        for param in params.iter() {
          write!(f, " {},", param);
        }
        Ok(())
      },
      Node::Int(int) => write!(f, "{}", int),
      Node::MethodCall { name, args } => write!(f, "call {}({:?})", name, args),
      Node::Parameter(param) => write!(f, ":{} <: :{:?}", param.name.name(), param.supertype),
      Node::PrimitiveType { name, supertype, bits } => write!(f, "{} <: {} {}", name.to_ir(), supertype, bits),
      Node::Symbol(name) => write!(f, "{}", name),
      Node::UnaryExpr { op, child } => write!(f, "{}{}", op, child),
    }
  }
}
