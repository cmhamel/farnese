use crate::ast::{Node, Operator, Primitive};
use crate::base::Symbol;
use pest::{self, Parser};

#[derive(pest_derive::Parser)]
#[grammar = "grammar_v2.pest"]
struct FarneseParser;

pub fn parse(source: &str) -> std::result::Result<Vec<Node>, pest::error::Error<Rule>> {
  let mut ast = vec![];
  let pairs = FarneseParser::parse(Rule::Program, source)?;
  
  for pair in pairs {
    let ast_temp = create_ast(pair);
    // println!("ast = {:?}", ast_temp);
    ast.push(ast_temp);
  }
  Ok(ast)
}

fn create_ast(pair: pest::iterators::Pair<Rule>) -> Node {
  // println!("pair = {:?}", pair);
  let ast: Node = match pair.as_rule() {
    Rule::AbstractType => {
      let name = pair.clone()
        .into_inner()
        .next()
        .unwrap();
      let extras: Vec<_> = pair
        .into_inner()
        .skip(1)
        .collect();
      let mut generics = Node::Generics { params: Vec::<Node>::new() };
      // let mut supertype = Node::SubType { name: Symbol::new("Any") };
      // let mut supertype = Node::Parameter { name: }
      if !extras.is_empty() {
        let _ = create_ast(extras[0].clone());
        for extra in extras {
          match extra.as_rule() {
            Rule::Generics => generics = create_ast(extra),
            // Rule::SubType => println!("subtype"),
            // Rule::SubType => supertype = create_ast(extra),
            _ => ()
          };
        }
      }

      Node::AbstractType {
        name: Symbol::new(name.as_str()),
        params: Box::new(generics),
        // supertype: Box::new(supertype)
      }
    },
    Rule::AssignmentExpr => {
      // println!("Assignment = {:?}", pair);
      let terms: Vec<_> = pair.into_inner().collect();
      // let name = create_ast(terms[0].clone());
      let name = Symbol::new(terms[0].as_str());
      let val = create_ast(terms[1].clone());

      // Node::Eoi
      Node::AssignmentExpr { identifier: name, value: Box::new(val) }
    }
    Rule::BinaryExpr => {
      // println!("WTF I SHOULD BE HERE {:?}", pair);
      let terms: Vec<_> = pair.into_inner().collect();
      let lhs = create_ast(terms[0].clone());
      let op = match terms[1].as_str() {
        // "=" => Operator::Assignment,
        "/" => Operator::Divide,
        "-" => Operator::Minus,
        "*" => Operator::Multiply,
        "+" => Operator::Plus,
        _ => panic!("wtf {:?}", terms)
      };
      let rhs = create_ast(terms[2].clone());
      Node::BinaryExpr { op: op, lhs: Box::new(lhs), rhs: Box::new(rhs) }
    },
    // Rule::Char => {
    //   // let c = pair.into_inner().collect()::<Vec<_>>()[0].as_str();
    //   println!("Pair = {:?}", pair);
    //   // let c: Vec<_> = pair.into_inner().collect();
    //   // let c = c[0].as_str();
    //   let c = pair.as_str().replace("'", "");
    //   let c = c.parse::<char>().unwrap();
    //   println!("C = {:?}", c);
    //   // let c = match c.chars().next() {
    //   //   Some(x) => x,
    //   //   None => panic!("Bad char encountered")
    //   // };
    //   Node::Char(c)
    // }
    Rule::Comment => Node::Empty,
    Rule::EOI => Node::Empty,
    Rule::Expr => {
      create_ast(pair.into_inner().next().unwrap())
    },
    // Rule::ExprElem => {
    //   let expr: Vec<_> = pair.into_inner().collect();
    //   create_ast(expr[0].clone())
    //   // Node::Eoi
    // }
    // Rule::Float => {
    //   let float = pair.as_str().parse::<f64>().unwrap();
    //   Node::Float(float)
    // },
    // Rule::Function => {
    //   let params: Vec<_> = pair.into_inner().collect();
    //   let name = Symbol::new(params[0].as_str());
    //   let mut args = Vec::<Symbol>::new();
    //   let mut body = Vec::<Node>::new();

    //   // read function argument names
    //   match params.get(1) {
    //     Some(x) => {  
    //       for param in params[1].clone().into_inner() {
    //         match param.as_rule() {
    //           Rule::FunctionArg => {
    //             let name: Vec<_> = param.into_inner().collect();
    //             let name = Symbol::new(name[0].as_str());
    //             args.push(name);
    //           },
    //           _ => ()
    //         };
    //       }
    //     },
    //     None => ()
    //   }
      
    //   // read function body
    //   match params.get(2) {
    //     Some(x) => {
    //       for line in params[2].clone().into_inner() {
    //         let temp_ast = create_ast(line);
    //         body.push(temp_ast);
    //       }
    //     }
    //     None => ()
    //   }

    //   Node::Function { name: name, args: args, body: Box::new(body) }
    //   // Node::Eoi
    // },
    Rule::FunctionArg => {
      let name: Vec<_> = pair.into_inner().collect();
      let name = Symbol::new(name[0].as_str());
      Node::Symbol(name)
    },
    Rule::FunctionExpr => {
      let temp: Vec<_> = pair.into_inner().collect();
      create_ast(temp[0].clone())
    }
    // todo fix this later
    // Rule::Generics => {
    //   let params = pair.into_inner();
    //   let mut new_params = Vec::<Node>::new();
    //   for param in params {
    //     let temp = create_ast(param);
    //     new_params.push(temp);
    //   }
    //   Node::Generics { params: new_params }
    // },
    Rule::Identifier => {
      let name = Symbol::new(pair.as_str());
      Node::Symbol(name)
    },
    // Rule::Int => {
    //   let int: i64 = pair.as_str().parse::<i64>().unwrap();
    //   Node::Int(int)
    // },
    Rule::MethodCall => {
      let params: Vec<_> = pair.into_inner().collect();
      let name = Symbol::new(params[0].as_str());
      let args: Vec<_> = params[1].clone().into_inner().collect();
      let args: Vec<_> = args
        .into_iter()
        .map(|x| Box::new(create_ast(x)))
        .collect();
      Node::MethodCall { name: name, args: args }
    }
    // Rule::Parameter => {
    //   let params = pair.into_inner();
    //   let mut name = String::new();
    //   // let mut subtype = "Any".to_string();
    //   let mut subtype: Node = Node::Empty;
    //   for param in params {
    //     match param.as_rule() {
    //       Rule::Identifier => name = param.as_str().to_string(),
    //       // Rule::SubType => subtype = param.as_str().to_string(),
    //       Rule::SubType => subtype = create_ast(param),
    //       _ => ()
    //     };
    //   };
    //   let name = Symbol::new(&name);
    //   // let subtype = Symbol::new(&subtype);
    //   let ret_type = Parameter { name: name, supertype: Some(Box::new(Parameter { name: Symbol::new("Any"), supertype: None }))};
    //   // Node::Parameter { name: name, subtype: subtype }
    //   println!("Ret type = {:?}", ret_type);
    //   Node::Parameter(ret_type)
    // },
    Rule::ParenthesesExpr => {
      println!("parans = {:?}", pair);
      let params: Vec<_> = pair.into_inner().collect();
      Node::ParenthesesExpr { expr: Box::new(create_ast(params[0].clone())) }
      // let 
      // Node::Empty
    }
    Rule::Primitive => create_primitive_ast(pair),
    // Rule::Primitive => {
    //   // let val: Vec<_> = pair.into_inner().collect();
    //   // create_ast(val[0].clone())
    //   panic!("Got here")
    // }
    Rule::PrimitiveType => {
      let name = pair.clone()
        .into_inner()
        .next()
        .unwrap();
      let name = Symbol::new(name.as_str());
      let extras: Vec<_> = pair
        .into_inner()
        .skip(1)
        .collect();
      let mut supertype = Symbol::new("Any");
      let mut bits: u32 = 0;

      for extra in extras.iter() {
        match extra.as_rule() {
          Rule::PrimitiveBits => bits = extra.as_str().parse::<u32>().unwrap(),
          Rule::PrimitiveSubType => supertype = Symbol::new(extra.clone().into_inner().as_str()),
          _ => ()
        };
      }
      let prim_type = Node::PrimitiveType{ name: name, supertype: supertype, bits: bits };
      prim_type
    },
    // Rule::SubType => {
    //   println!("Subtype = {:?}", pair);
    //   let subtype = pair.into_inner();
    //   let mut name = String::new();
    //   let mut generics = Node::Generics { params: Vec::<Node>::new() };
    //   for param in subtype {
    //     println!("param = {:?}", param);
    //     match param.as_rule() {
    //       Rule::Generics => generics = create_ast(param),
    //       Rule::Identifier => name = param.as_str().to_string(),
    //       _ => ()
    //     };
    //   }
    //   println!("name = {:?}", name);
    //   // Node::Eoi
    //   let name = Symbol::new(&name);
    //   let ret_type = Parameter { 
    //     name: name, 
    //     supertype: Some(Box::new(
    //       Parameter { 
    //         name: Symbol::new("Any"), 
    //         supertype: None 
    //       }
    //     ))};
    //   Node::Parameter(ret_type)
    // },
    Rule::UnaryExpr => {
      println!("pair = {:?}", pair);
      let terms: Vec<_> = pair.into_inner().collect();
      // let val = farnese_expr()
      // let op = create_ast(terms[0].clone());
      let op = match terms[0].as_str() {
        "-" => Operator::Minus,
        "+" => Operator::Plus,
        _ => panic!("unsupported op deteceted in unaryexpr")
      };
      let val = create_ast(terms[1].clone());
      // panic!("here");
      // Node::Empty
      Node::UnaryExpr { op: op, child: Box::new(val) }
    }
    // Rule::UnaryExpr => {
    //   println!("Here = {:?}", pair);
    //   let terms: Vec<_> = pair.into_inner().collect();
    //   let val = create_ast(terms[1].clone());
    //   let op = match terms[0].as_str() {
    //     "-" => Operator::Minus,
    //     "+" => Operator::Plus,
    //     _ => panic!("wtf")
    //   };
    //   Node::UnaryExpr { op: op, child: Box::new(val) }
    // },
    _ => todo!("todo {:?}", pair.as_rule())
  };
  ast
}

fn create_primitive_ast(pair: pest::iterators::Pair<Rule>) -> Node {
  // let prim = match pair.as_rule() {
  //   Rule::Primitive
  //   Rule::Float => Primitive::Float(pair.as_str().parse::<f64>().unwrap()),
  //   Rule::Int => Primitive::Int(pair.as_str().parse::<i64>().unwrap()),
  //   _ => panic!("Unsupported primitive encountered in ast {:?}", pair)
  // };
  let prim: Vec<_> = pair.clone().into_inner().collect();
  let prim = match prim[0].as_rule() {
    Rule::Char => {
      let c = prim[0].as_str().replace("'", "");
      let c = c.parse::<char>().unwrap();
      Primitive::Char(c)
    },
    Rule::Float => Primitive::Float(pair.as_str().parse::<f64>().unwrap()),
    Rule::Int => Primitive::Int(pair.as_str().parse::<i64>().unwrap()),
    _ => panic!("Unsupported primitive encountered in ast {:?}", pair)
  };
  Node::Primitive(prim)
}
