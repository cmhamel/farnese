// TODO things to implement
// abstract type
// binary operators
// methods 
// modules
// primite types
// variables

// struct AbstractType<T...> {}
// struct PrimitiveType<T...> {}
// struct AbstractType {
//   name: String
//   arguments: Vec<T>
// }

#[derive(Debug)]
pub struct Symbol {
  data: String
}

impl Symbol {
  pub fn new(data: String) -> Self {
    Symbol {data: data}
  }
}

#[derive(Debug)]
pub struct AbstractType {
  name: Symbol,
  arguments: Vec<Symbol>,
  parent: AbstractType
}

#[derive(Debug)]
pub struct PrimitiveType {
  name: Symbol
}