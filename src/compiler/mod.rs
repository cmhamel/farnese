pub mod compiler;
pub mod interpreter;

use crate::ast::Node;
use crate::parser;
pub use compiler::Compiler;

pub trait Compile {
  type Output;

  fn from_ast(&mut self, ast: Vec<Node>) -> Self::Output;

  fn from_ast_inner(&mut self, ast: Node) -> () {}

  // fn from_source(source: &str) -> Self::Output {
  fn from_source(&mut self, source: &str) -> Self::Output {
    println!("Compiling the source:\n\n{}", source);
    println!("Begging parsing.");
    let ast: Vec<_> = parser::parse(source).unwrap();
    // Self::from_ast(ast);
    self.from_ast(ast)
  }
}
