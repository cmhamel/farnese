use crate::ast::{Node, Operator, Parameter};
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
        let ast_temp = create_ast(extras[0].clone());
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
    Rule::EOI => {
      Node::Eoi
    },
    Rule::Expr => {
      create_ast(pair.into_inner().next().unwrap())
    },
    // Rule::ExprElem => {
    //   let expr: Vec<_> = pair.into_inner().collect();
    //   create_ast(expr[0].clone())
    //   // Node::Eoi
    // }
    Rule::Float => {
      let float = pair.as_str().parse::<f64>().unwrap();
      Node::Float(float)
    },
    Rule::Function => {
      let params: Vec<_> = pair.into_inner().collect();
      let name = Symbol::new(params[0].as_str());
      let mut args = Vec::<Symbol>::new();
      let mut body = Vec::<Node>::new();

      // read function argument names
      match params.get(1) {
        Some(x) => {  
          for param in params[1].clone().into_inner() {
            match param.as_rule() {
              Rule::FunctionArg => {
                let name: Vec<_> = param.into_inner().collect();
                let name = Symbol::new(name[0].as_str());
                args.push(name);
              },
              _ => ()
            };
          }
        },
        None => ()
      }
      
      // read function body
      match params.get(2) {
        Some(x) => {
          for line in params[2].clone().into_inner() {
            let temp_ast = create_ast(line);
            body.push(temp_ast);
          }
        }
        None => ()
      }

      Node::Function { name: name, args: args, body: Box::new(body) }
      // Node::Eoi
    },
    Rule::FunctionArg => {
      let name: Vec<_> = pair.into_inner().collect();
      let name = Symbol::new(name[0].as_str());
      Node::Symbol(name)
    },
    Rule::FunctionExpr => {
      let temp: Vec<_> = pair.into_inner().collect();
      create_ast(temp[0].clone())
    }
    Rule::Generics => {
      let params = pair.into_inner();
      let mut new_params = Vec::<Node>::new();
      for param in params {
        let temp = create_ast(param);
        new_params.push(temp);
      }
      Node::Generics { params: new_params }
    },
    Rule::Identifier => {
      let name = Symbol::new(pair.as_str());
      Node::Symbol(name)
    },
    Rule::Int => {
      let int: i64 = pair.as_str().parse::<i64>().unwrap();
      Node::Int(int)
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
    }
    Rule::Parameter => {
      let params = pair.into_inner();
      let mut name = String::new();
      // let mut subtype = "Any".to_string();
      let mut subtype: Node = Node::Eoi;
      for param in params {
        match param.as_rule() {
          Rule::Identifier => name = param.as_str().to_string(),
          // Rule::SubType => subtype = param.as_str().to_string(),
          Rule::SubType => subtype = create_ast(param),
          _ => ()
        };
      };
      let name = Symbol::new(&name);
      // let subtype = Symbol::new(&subtype);
      let ret_type = Parameter { name: name, supertype: Some(Box::new(Parameter { name: Symbol::new("Any"), supertype: None }))};
      // Node::Parameter { name: name, subtype: subtype }
      println!("Ret type = {:?}", ret_type);
      Node::Parameter(ret_type)
    },
    Rule::Primitive => {
      let val: Vec<_> = pair.into_inner().collect();
      create_ast(val[0].clone())
    }
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
    // Rule::SingleLineComment => Node::Eoi,
    Rule::SubType => {
      println!("Subtype = {:?}", pair);
      let subtype = pair.into_inner();
      let mut name = String::new();
      let mut generics = Node::Generics { params: Vec::<Node>::new() };
      for param in subtype {
        println!("param = {:?}", param);
        match param.as_rule() {
          Rule::Generics => generics = create_ast(param),
          Rule::Identifier => name = param.as_str().to_string(),
          _ => ()
        };
      }
      println!("name = {:?}", name);
      // Node::Eoi
      let name = Symbol::new(&name);
      let ret_type = Parameter { 
        name: name, 
        supertype: Some(Box::new(
          Parameter { 
            name: Symbol::new("Any"), 
            supertype: None 
          }
        ))};
      Node::Parameter(ret_type)
    },
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

// fn build_ast_from_expr(pair: pest::iterators::Pair<Rule>) -> Node {
// fn build_ast_from_expr(pair: pest::iterators::Pair<Rule>) -> () {

//   match pair.as_rule() {
//     Rule::Expr => build_ast_from_expr(pair.into_inner().next().unwrap()),
//     // specialization
//     Rule::AbstractType => build_ast_for_abstract_type(pair.into_inner()),
//     // Rule::Generics => {

//     // },
//     // Rule::Identifier => {

//     // },
//     // Rule::Parameter => {

//     // }
//     // Rule::SubType => {

//     // }
//     // Rule::AbstractType => {
//     //   let mut pair = pair.into_inner();
//     //   // read type name and conver to symbol
//     //   let name = Symbol::new(pair.next().unwrap().as_str());
//     //   // read parameters as symbols, then convert to datatypes later
//     //   let params = parse_generics(&mut pair);
//     //   // read subtype as symbols and convert to data types later
//     //   let subtype = parse_subtype(&mut pair);
//     //   // let subtype = pair.next();
//     //   // let subtype: Option<String> = match subtype {
//     //   //   Some(x) => todo!("need to do this"),
//     //   //   _ => None
//     //   // };
//     //   // let pair_temp = pair.next().unwrap();
//     //   println!("Name = {}", name);
//     //   println!("Params = {:?}", params);
//     //   println!("Subtype = {:?}", subtype);

//     // },
//     // Rule::AbstractTypeExpr => {
//     //   let mut pair = pair.into_inner();

//     // }
//     // Rule::BinaryExpr => {
//     //   let mut pair = pair.into_inner();
//     //   let lhspair = pair.next().unwrap();
//     //   let mut lhs = build_ast_from_term(lhspair);
//     //   let op = pair.next().unwrap();
//     //   let rhspair = pair.next().unwrap();
//     //   let mut rhs = build_ast_from_term(rhspair);
//     //   let mut retval = parse_binary_expr(op, lhs, rhs);
//     //   loop {
//     //     let pair_buf = pair.next();
//     //     if let Some(op) = pair_buf {
//     //       lhs = retval;
//     //       rhs = build_ast_from_term(pair.next().unwrap());
//     //       retval = parse_binary_expr(op, lhs, rhs);
//     //     } else {
//     //       return retval;
//     //     }
//     //   }
//     // },
//     // Rule::UnaryExpr => {
//     //   let mut pair = pair.into_inner();
//     //   let op = pair.next().unwrap();
//     //   let child = pair.next().unwrap();
//     //   let child = build_ast_from_term(child);
//     //   parse_unary_expr(op, child)
//     // },
//     // other
//     // Rule::Identifier => {
//     //   let mut pair = pair.into_inner();
//     //   println!("{:?}", pair);
//     //   pair
//     // }
//     unknown => panic!("Unknown expr: {:?}", unknown),
//   }
// }

// fn collect_ast_leaves<'a>(pair: Pair<'a, Rule>, leaves: &mut Vec<Pair<'a, Rule>>, rule: Rule) {
//   if pair.clone().into_inner().next().is_none() {
//     // If there are no children, it's a leaf
//     // match pair.clone().as_rule() {
//     //   rule => {
//     //     println!("pair = {:?}", pair);
//     //     leaves.push(pair.as_str().to_string());
//     //   },
//     //   _ => ()
//     // }
//     // leaves.push(pair.as_str().to_string());
//   } else {
//     // Recursively process children
//     for inner_pair in pair.clone().into_inner() {
//       match pair.as_rule() {
//         rule => {
//           // println!("found the rule we want {:?}", pair);
//           leaves.push(pair.clone());
//         },
//         _ => ()
//       };
//       collect_ast_leaves(inner_pair, leaves, rule);
//     }
//   }
// }

// fn parse_extras(pair: &mut Pairs<Rule>) -> () {
//   let mut extras = pair.next();
//   // try to parse generics first
//   let params: Option<Vec<Symbol>> = match extras {
//     Some(x) => match x.as_rule() {
//       Rule::Generics => {
//         let mut params_read = Vec::<String>::new();
//         collect_ast_leaves(x, &mut params_read);
//         let mut params_syms = Vec::<Symbol>::new();

//         for param in params_read.iter() {
//           params_syms.push(Symbol::new(param));
//         }
        
//         Some(params_syms)
//       },
//       _ => None
//     },
//     _ => None
//   };

//   if !params.is_none() {
//     extras = pair.next();
//   }
  
//   let subtype: Option<Symbol> = match extras {
//     Some(x) => match x.as_rule() {
//       Rule::SubType => {
//         Some(Symbol::new(x.as_str()))
//       },
//       _ => None
//     },
//     _ => None
//   };
//   // let subtype: Option<Symbol> = match pair {
//   //   Some(x) => match x.unwrap().as_rule() {
//   //     Rule::SubType => {
//   //       Some(Symbol::new(x.as_str()))
//   //     },
//   //     _ => None
//   //   },
//   //   _ => None
//   // };
// }

// fn parse_generics(pair: &mut Pairs<Rule>) -> Option<Vec<Symbol>> {
//   let params = pair.next();
//   let params: Option<Vec<Symbol>> = match params {
//     Some(x) => match x.as_rule() {
//       Rule::Generics => {
//         let mut params_read = Vec::<String>::new();
//         collect_ast_leaves(x, &mut params_read);
//         let mut params_syms = Vec::<Symbol>::new();

//         for param in params_read.iter() {
//           params_syms.push(Symbol::new(param));
//         }
        
//         Some(params_syms)
//       },
//       _ => None
//     },
//     _ => None
//   };
//   params
// }

// fn parse_subtype(pair: &mut Pairs<Rule>) -> Option<Symbol> {
//   let name = pair.next();
//   println!("Name = {:?}", name);
//   let name: Option<Symbol> = match name {
//     Some(x) => match x.as_rule() {
//       Rule::Identifier => {
//         let temp = x.as_str();
//         let temp = Symbol::new(temp);
//         Some(temp)
//       },
//     _ => {
//       println!("here");
//       None
//     }
//     },
//     _ => None
//   };
//   name
// }

// fn build_ast_from_term(pair: pest::iterators::Pair<Rule>) -> Node {
//   match pair.as_rule() {
//     Rule::Int => {
//       let istr = pair.as_str();
//       let (sign, istr) = match &istr[..1] {
//         "-" => (-1, &istr[1..]),
//         _ => (1, istr),
//       };
//       let int: i32 = istr.parse().unwrap();
//       Node::Int(sign * int)
//     }
//     Rule::Expr => build_ast_from_expr(pair),
//     unknown => panic!("Unknown term: {:?}", unknown),
//   }
// }

// fn build_ast_for_abstract_type(pair: pest::iterators::Pair<Rule>) -> Node {
//   match pair.as_rule() {

//   }
// }

// fn parse_unary_expr(pair: pest::iterators::Pair<Rule>, child: Node) -> Node {
//   Node::UnaryExpr {
//     op: match pair.as_str() {
//       "+" => Operator::Plus,
//       "-" => Operator::Minus,
//       _ => unreachable!(),
//     },
//     child: Box::new(child),
//   }
// }

// fn parse_binary_expr(pair: pest::iterators::Pair<Rule>, lhs: Node, rhs: Node) -> Node {
//   Node::BinaryExpr {
//     op: match pair.as_str() {
//       "+" => Operator::Plus,
//       "-" => Operator::Minus,
//       _ => unreachable!(),
//     },
//     lhs: Box::new(lhs),
//     rhs: Box::new(rhs),
//   }
// }

