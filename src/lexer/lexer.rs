use crate::core::Symbol;
use crate::parser::{FarneseParser, Rule};
use super::ast::{Node, Operator, Primitive};

pub fn parse(source: &str) -> std::result::Result<Vec<Node>, pest::error::Error<Rule>> {
  let mut ast = vec![];
  let pairs = FarneseParser::from_source(source);
  
  for pair in pairs {
    let ast_temp = create_ast(pair);
    ast.push(ast_temp);
  }
  Ok(ast)
}

fn create_ast(pair: pest::iterators::Pair<Rule>) -> Node {
  let ast: Node = match pair.as_rule() {
    Rule::AbstractType => {
      let exprs: Vec<_> = pair.into_inner().collect();
      let mut name = Symbol::new("HOWTFDIDTHISHAPPEN");
      let mut generics = Node::Generics { params: Vec::<Node>::new() };
      let mut supertype = Node::Symbol(Symbol::new("Any"));
      for expr in exprs {
        let ast = create_ast(expr.clone());
        match ast {
          Node::Generics { params: _ } => generics = ast,
          Node::SuperType { expr: x } => supertype = *x,
          Node::Symbol(x) => name = x,
          _ => todo!("unsupported {:?}", ast)
        }
      }

      let supertype_sym = match supertype {
        Node::Symbol(ref x) => x,
        _ => todo!("x = {:?}", supertype),
      };

      Node::AbstractType {
        name: name,
        params: Box::new(generics),
        supertype: supertype_sym.clone()
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
    Rule::Comment => {
      Node::Comment
    },
    Rule::EOI => {
      Node::Eoi
    },
    Rule::Expr => {
      create_ast(pair.into_inner().next().unwrap())
    },
    // Rule::Function => {
    //   println!("Pair = {:?}", pair);
    //   let mut terms = pair.into_inner();
    //   let name = create_ast(terms.next().unwrap());
    //   println!("Func name {:?}", name);
    //   Node::Empty
    // }
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

    Rule::Function => {
      println!("Node = {:?}", pair);
      // let terms: Vec<_> = pair.clone().into_inner().collect();
      // let name = Symbol::new(terms[0].as_str());
      // let mut generics = Vec::<Node>::new();
      let mut args = Vec::<Symbol>::new();
      // let exprs = Vec::<Node>::new();
      for x in pair.into_inner().skip(1) {
        match x.as_rule() {
          Rule::FunctionArgs => {
            for arg in x.into_inner() {
              match arg.as_rule() {
                Rule::FunctionArg => {
                  args.push(Symbol::new(arg.into_inner().next().unwrap().as_str()));
                },
                _ => panic!("Shouldn't happen")
              }
            } 
          },
          Rule::FunctionExprs => {
            // let ast = create_ast(x);
            for expr in x.into_inner() {
              // exprs.push(create_ast(expr));
              match expr.as_rule() {
                Rule::FunctionExpr => {
                  // exprs.push(create_ast(expr))
                },
                _ => panic!("Shouldn't happen")
              }
            }
          },
          _ => todo!("Unsupported function stuff {:?}", x)
        }
      }
      println!("Args = {:?}", args);
      Node::Eoi
    },
    Rule::FunctionArg => {
      let name: Vec<_> = pair.into_inner().collect();
      let name = Symbol::new(name[0].as_str());
      Node::Symbol(name)
    },
    // Rule::FunctionArgs => {
    //   for arg in pair.into_inner() {
    //     match arg.as_rule() {
    //       Rule::FunctionArg => create_ast(arg),
    //       _ => panic!("Unsupported function arg")
    //     }
    //   }
    // },
    // Rule::FunctionExpr => {
    //   let temp: Vec<_> = pair.into_inner().collect();
    //   if temp.len() > 0 {
    //     create_ast(temp[0].clone())
    //   } else {
    //     Node::Empty
    //   }
    // }
    // todo fix this later
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
    Rule::ImportExpr => {
      let module = pair.clone().into_inner().next().unwrap().as_str();
      let element = pair.clone().into_inner().skip(1).next().unwrap().as_str();
      Node::ImportExpr { module: Symbol::new(module), element: Symbol::new(element) }
    },
    Rule::MainFunction => {
      let exprs = pair.into_inner().next().unwrap();
      let mut asts = Vec::<Box<Node>>::new();

      for expr in exprs.into_inner() {
        let ast = create_ast(expr);
        asts.push(Box::new(ast));
      }
      Node::MainFunction { exprs: asts }
    }
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
      let name = Symbol::new(pair.clone().into_inner().next().unwrap().as_str());
      let exprs: Vec<_> = pair.into_inner().skip(1).collect();
      let mut asts = Vec::<Box<Node>>::new();
      for expr in exprs {
        let ast = create_ast(expr);
        asts.push(Box::new(ast))
      }
      Node::Module { name: name, exprs: asts }
    },
    Rule::Parameter => {
      let exprs: Vec<_> = pair.into_inner().collect();
      assert!(exprs.len() == 1, "Bad parameter encountered");
      Node::Symbol(Symbol::new(exprs[0].as_str()))
    },
    Rule::ParenthesesExpr => {
      let params: Vec<_> = pair.into_inner().collect();
      Node::ParenthesesExpr { expr: Box::new(create_ast(params[0].clone())) }
    },
    Rule::Primitive => {
      create_primitive_ast(pair)
    },
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
          Rule::PrimitiveSuperType => supertype = Symbol::new(extra.clone().into_inner().as_str()),
          _ => ()
        };
      }
      let prim_type = Node::PrimitiveType{ name: name, supertype: supertype, bits: bits };
      prim_type
    },
    Rule::StructField => {
      println!("Struct field = {:?}", pair);
      let exprs: Vec<_> = pair.into_inner().collect();
      let name = Symbol::new(exprs[0].as_str());
      // todo, currently generics don't do anything
      let generics = Vec::<Box<Node>>::new();
      let supertype = Box::new(Node::Symbol(Symbol::new("Any")));
      Node::FieldType { 
        name: name, 
        generics: generics,
        supertype: supertype
      }
      // panic!()
    },
    Rule::StructType => {
      let exprs: Vec<_> = pair.into_inner().collect();
      let mut name = Symbol::new("HOWTFDIDTHISHAPPEN");
      let generics = Vec::<Box<Node>>::new();
      let supertype = Symbol::new("Any");
      let mut fields = Vec::<Box<Node>>::new();

      // }
      for expr in exprs {
        let ast = create_ast(expr);
        match ast {
          Node::FieldType { .. } => fields.push(Box::new(ast)),
          Node::Symbol(x) => name = x,
          _ => todo!("Not supported yet {:?}", ast)
        }
      }
      Node::StructType { 
        name: name, 
        generics: generics, 
        supertype: supertype, 
        fields: fields 
      }
    }
    Rule::SuperType => {
      let exprs: Vec<_> = pair.into_inner().collect();
      Node::SuperType { expr: Box::new(create_ast(exprs[0].clone())) }
    }
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
    _ => todo!("todo {:?} {:?}", pair.as_rule(), pair)
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
