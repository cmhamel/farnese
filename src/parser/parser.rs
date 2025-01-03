use crate::core::Symbol;
use super::ast::{Node, Operator, Primitive};
use pest::{self, Parser};

#[derive(pest_derive::Parser)]
#[grammar = "./parser/grammar.pest"]
struct FarneseParser;

pub fn parse(source: &str) -> std::result::Result<Vec<Node>, pest::error::Error<Rule>> {
  let mut ast = vec![];
  let pairs = FarneseParser::parse(Rule::Program, source)?;
  
  for pair in pairs {
    let ast_temp = create_ast(pair);
    ast.push(ast_temp);
  }
  Ok(ast)
}

fn create_ast(pair: pest::iterators::Pair<Rule>) -> Node {
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
      let terms: Vec<_> = pair.into_inner().collect();
      let name = Symbol::new(terms[0].as_str());
      let val = create_ast(terms[1].clone());
      Node::AssignmentExpr { identifier: name, value: Box::new(val) }
    },
    Rule::BinaryExpr => {
      let terms: Vec<_> = pair.into_inner().collect();
      let lhs = create_ast(terms[0].clone());
      let op = match terms[1].as_str() {
        "/" => Operator::Divide,
        "-" => Operator::Minus,
        "*" => Operator::Multiply,
        "+" => Operator::Plus,
        _ => panic!("wtf {:?}", terms)
      };
      let rhs = create_ast(terms[2].clone());
      Node::BinaryExpr { op: op, lhs: Box::new(lhs), rhs: Box::new(rhs) }
    },
    Rule::Comment => Node::Empty,
    Rule::EOI => Node::Empty,
    Rule::Expr => create_ast(pair.into_inner().next().unwrap()),
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
    Rule::ImportExpr => {
      let module = pair.clone().into_inner().next().unwrap().as_str();
      let element = pair.clone().into_inner().skip(1).next().unwrap().as_str();
      Node::ImportExpr { module: Symbol::new(module), element: Symbol::new(element) }
    },
    Rule::MethodCall => {
      let params: Vec<_> = pair.into_inner().collect();
      let name = Symbol::new(params[0].as_str());
      let args: Vec<_> = params[1].clone().into_inner().collect();
      let args: Vec<_> = args
        .into_iter()
        .map(|x| Box::new(create_ast(x)))
        .collect();
      Node::MethodCall { name: name, args: args }
    },
    Rule::Module => {
      // let exprs: Vec<_> = pair.into_inner().collect();
      let name = Symbol::new(pair.clone().into_inner().next().unwrap().as_str());
      let exprs: Vec<_> = pair.into_inner().skip(1).collect();
      // println!("Module = {}", name);
      let mut asts = Vec::<Box<Node>>::new();
      for expr in exprs {
        let ast = create_ast(expr);
        asts.push(Box::new(ast))
      }
      // panic!("pair = {:?}", pair)
      // panic!()
      Node::Module { name: name, exprs: asts }
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
      let params: Vec<_> = pair.into_inner().collect();
      Node::ParenthesesExpr { expr: Box::new(create_ast(params[0].clone())) }
    }
    Rule::Primitive => create_primitive_ast(pair),
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
      let terms: Vec<_> = pair.into_inner().collect();
      let op = match terms[0].as_str() {
        "-" => Operator::Minus,
        "+" => Operator::Plus,
        _ => panic!("unsupported op deteceted in unaryexpr")
      };
      let val = create_ast(terms[1].clone());
      Node::UnaryExpr { op: op, child: Box::new(val) }
    },
    Rule::UsingExpr => {
      println!("Here for using, time to link");
      
      Node::Empty
    }
    _ => todo!("todo {:?}", pair.as_rule())
  };
  ast
}

fn create_primitive_ast(pair: pest::iterators::Pair<Rule>) -> Node {
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
