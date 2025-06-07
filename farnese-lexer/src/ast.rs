#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operator {
  // Assignment,
  Divide,
  Minus,
  Multiply,
  Plus
}

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
  Char(char),
  Float32(f32),
  Float64(f64),
  Int16(i16),
  Int32(i32),
  Int64(i64),
  String(String),
  UInt16(u16),
  UInt32(u32),
  UInt64(u64)
}

// note this is not the Symbol in core
// but has the same name since it will be
// converted to core::Symbol downstream
pub type Symbol = String;

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
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
  MethodCall {
    name: Symbol,
    args: Box<Vec<Node>>
  },
  Module {
    name: Symbol,
    exprs: Box<Vec<Node>>
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
  Symbol(Symbol),
  UnaryExpr {
    op: Operator,
    child: Box<Node>,
  },
}
