use crate::parser::lexer::{self, Token};
use std::fmt::{self, Display, Formatter};

pub trait Expression {
  fn expression(&self) -> String;
}

// Keyword and interfaces
pub struct Keyword {
  name: String
}

impl Keyword {
  pub fn new(keyword: &str) -> Keyword {
    Keyword {
      name: String::from(keyword)
    }
  }
}

impl Display for Keyword {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "Keyword:\n  {}\n", self.name)
  }
}

impl Expression for Keyword {
  fn expression(&self) -> String {
    let expr = String::from(&self.name);
    expr
  }
}

// Number and interfaces
pub struct Number {
  pub value: f64
}

impl Number {
  pub fn new(value: &str) -> Number {
    let f: f64 = match value.parse() {
      Ok(v)  => v,
      Err(_) => panic!("This is not a Float!") // or whatever error handling
    };
    Number {
      value: f
    }
  }
}

impl Display for Number {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "Number:\n  {}\n", self.value)
  }
}

impl Expression for Number {
  fn expression(&self) -> String {
    let expr = self.value.to_string();
    expr
  }
}

// Variable
pub struct Variable {
  name: String
}

impl Variable {
  pub fn new(value: &str) -> Variable {
    Variable {
      name: String::from(value)
    }
  }
}

impl Expression for Variable {
  fn expression(&self) -> String {
    let expr = String::from(&self.name);
    expr
  }
}

// Binary expression
pub struct BinaryExpression<L, R> {
  op: String,
  lhs: L,
  rhs: R
}

impl<L, R> BinaryExpression<L, R> {
  pub fn new(lhs: L, op: &str, rhs: R) -> BinaryExpression<L, R> {
    BinaryExpression {
      op: String::from(op),
      lhs: lhs,
      rhs: rhs
    }
  }
}

impl<L, R> Expression for BinaryExpression<L, R> {
  fn expression(&self) -> String {
    // let mut expr = String::from(&self.lhs);
    // expr = expr + " " self.op + " " + self.rhs;
    // expr
    let expr = String::from(&self.op.to_string());
    expr
  }
}

// setup methods
pub fn create_ast(
  tokens: Vec<String>,
  token_types: Vec<lexer::Token>
) {

  let mut asts: Vec<Box<dyn Expression>> = Vec::new();

  // for (token_type, token) in token_types.iter().zip(tokens.iter()) {
  //   let ast_temp = match token_type {
  //     Token::BinaryOperator => {

  //     }
  //     Token::Keyword => Box::new(Keyword::new(token)) as Box<dyn Expression>,
  //     Token::Number  => Box::new(Number::new(token)),
  //     _              => panic!("weird bug")
  //   };
  //   asts.push(ast_temp);
  // }
  // let iter = asts.iter()
  let mut iter = token_types.iter().zip(tokens.iter());
  let mut prev = iter.next();
  let mut iter = token_types.iter().zip(tokens.iter());

  loop {
    let curr = iter.next();
    let temp = match prev {
      Some(x) => x,
      None    => panic!("Somethings wrong!")
    };
    match curr {
      Some(x) => {
        // println!("{:?}", x.0);
        // println!("{:?}", x.1);
        let ast_temp = match x.0 {
          // this one is a mess and needs to be broken down into methods
          Token::BinaryOperator => {
            // matching on type of lhs
            // println!("Found a binary expression");
            let lhs = match temp.0 {
              Token::Name   => Box::new(Variable::new(temp.1))as Box<dyn Expression>,
              Token::Number => Box::new(Number::new(temp.1)),
              _ => panic!("Issue in binary expression")
            };
            let rhs_curr = iter.next();
            let rhs = match rhs_curr {
              Some(y) => {
                // println!("{:?}", y.0);
                // println!("{:?}", y.1);
                let rhs = match y.0 {
                  Token::Name   => Box::new(Variable::new(y.1))as Box<dyn Expression>,
                  Token::Number => Box::new(Number::new(y.1)),
                  _ => panic!("Issue in binary expression")
                };
                rhs
              },
              None => panic!("Error here")
            };
            
            Box::new(BinaryExpression::new(lhs, x.1, rhs)) as Box<dyn Expression>
          }
          Token::Keyword => Box::new(Keyword::new(x.1)),
          Token::Name    => {
            // println!("Name here");
            // println!("{:?}, {:?}", temp.0, temp.1);
            match temp.0 {
              Token::Keyword => {
                println!("Method name");
                println!("you need to finish this");
                panic!("need to finish this")
              }
              _              => Box::new(Variable::new(x.1))
            }
          }
          Token::Number  => Box::new(Number::new(x.1)),
          _              => panic!("weird bug")
        };
        asts.push(ast_temp);
      },
      None => break
    }
    prev = curr;
  }

  for ast in asts.iter() {
    println!("{:?}", ast.expression());
  }
  // println!("{:?}", asts)
}