use std::fmt::{self, Formatter};

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

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
  Char(char),
  Float64(f64),
  Int64(i64)
}

// note this is not the Symbol in core
// but has the same name since it will be
// converted to core::Symbol downstream
pub type Symbol = String;

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
  // basics
  AbstractType {
    name: Symbol,
    // params: Box<Node>, 
    supertype: Symbol,
  },
  AssignmentExpr {
    identifier: Symbol,
    value: Box<Node>
  },
  BinaryExpr {
    op: Operator,
    lhs: Box<Node>,
    rhs: Box<Node>,
  },
  Empty,
  Function {
    name: Symbol,
    // arg_names: Vec<Symbol>,
    args: Box<Vec<Node>>,
    return_type: Symbol,
    body: Box<Vec<Node>>
  },
  FunctionArg {
    name: Symbol,
    // arg_type: Box<Node>
    arg_type: Symbol
  },
  FunctionArgs {
    names: Vec<Symbol>,
    types: Vec<Symbol>
  },
  // MainFunction {
  //   exprs: Box<Vec<Node>>
  // },
  MethodCall {
    name: Symbol,
    args: Vec<Box<Node>>
  },
  Module {
    name: Symbol,
    exprs: Vec<Box<Node>>
  },
  ParenthesesExpr {
    expr: Box<Node>
  },
  Primitive(Primitive),
  PrimitiveType {
    name: Symbol,
    supertype: Symbol,
    bits: u32
  },
  StructField {
    name: Symbol,
    field_type: Symbol
  },
  StructType {
    name: Symbol,
    // generics: Vec<Symbol>,
    supertype: Symbol,
    field_names: Vec<Symbol>,
    field_types: Vec<Symbol>
  },
  SuperType(Symbol),
  Symbol(String),
  UnaryExpr {
    op: Operator,
    child: Box<Node>,
  },
}

// Eventually we can remove derive(Debug) if we finish this
impl fmt::Display for Node {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Node::AbstractType{ name, supertype } => {
        let params = "";
        let string = format!(
          "(abstract_type ({} ({}) {}))", name, params, supertype
        );
        write!(f, "{}", string)
      },
      Node::Empty => {
        write!(f, "")
      },
      Node::FunctionArg { name, arg_type } => {
        writeln!(f, "arg name = {} arg type = {}", name, arg_type)
      }
      // Node::Function { name, arg_names, body } => {
      //   writeln!(f, "\nFunction {}", name).unwrap();
      //   writeln!(f, "Arguments:").unwrap();
      //   for arg in args {
      //     write!(f, "{}, ", arg).unwrap();
      //   }
      //   writeln!(f, "Body:").unwrap();
      //   writeln!(f, "{:?}", body).unwrap();
      //   // for expr in (*body).iter() {
      //   //   write!(f, "{}", expr).unwrap();
      //   // }
      //   Ok(())
      // }
      Node::Module { name, exprs } => {
        writeln!(f, "Dumping AST for exprs in module {}", name).unwrap();
        for expr in exprs.iter() {
          writeln!(f, "{}", expr).unwrap();
        }
        Ok(())
      },
      Node::PrimitiveType { name, supertype, bits: _ } => {
        write!(f, "(primitive_type false ({}) {})", name, supertype)
      },
      Node::StructType { name, supertype, field_names: _, field_types: _ } => {
        let string = format!(
          "(struct_type false ({} (", name
        );
        let generic_str = "";
        // let generic_str = generics
        //   .iter()
        //   .map(|x| format!("{}", x))
        //   .collect::<Vec<String>>()
        //   .join(", ");
        // let fields_str = fields
        //   .iter()
        //   .map(|x| match &**x {
        //     Node::StructField { name, field_type } => {
        //       format!(":: {} {}", name, field_type)
        //     },
        //     _ => panic!("Should never happend {:?}", fields)
        //   })
        //   .collect::<Vec<String>>()
        //   .join(", ");
        let fields_str = "";
        write!(f, "{}{}) {} (block ({}))))", string, generic_str, supertype, fields_str)
      },
      Node::SuperType(expr) => {
        write!(f, "{}", expr)
      },
      Node::Symbol(x) => {
        write!(f, "{}", x)
      },
      _ => panic!("ast fmt not supported for {:?} yet", self)
    }
  }
}
