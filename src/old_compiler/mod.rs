pub mod compiler;
// pub mod interpreter;

use crate::lexer::ast::Node;
use crate::lexer;
pub use compiler::Compiler;

pub trait Compile {
  type Output;

  fn from_ast(&mut self, ast: Vec<Node>) -> ();

  fn from_ast_inner(&mut self, _ast: Node) -> () {}

  // fn from_source(source: &str) -> Self::Output {
  fn from_source(&mut self, source: &str) -> () {
    let ast: Vec<_> = lexer::lexer::parse(source).unwrap();
    self.from_ast(ast)
  }
}
