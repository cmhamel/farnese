use crate::ast::{Node, Operator, Primitive, Symbol};
use crate::parser::{FarneseParser, Rule};

pub fn parse(source: &str) -> std::result::Result<Vec<Node>, pest::error::Error<Rule>> {
  let mut ast = vec![];
  let pairs = FarneseParser::from_source(source);
  
  for pair in pairs {
    let ast_temp = create_ast(&pair);
    ast.push(ast_temp);
  }
  Ok(ast)
}

fn create_ast(pair: &pest::iterators::Pair<Rule>) -> Node {
  let ast: Node = match pair.as_rule() {
    Rule::AbstractType => {
      let parts: Vec<_> = pair.clone().into_inner().collect();
      let name = parts
        .iter()
        .filter(|p| matches!(p.as_rule(), Rule::Identifier))
        .next()
        .unwrap()
        .as_str().to_string();
      let supertype = parts
        .iter()
        .filter(|p| matches!(p.as_rule(), Rule::SuperType))
        .next();
      let supertype = match supertype {
        Some(x) => x.as_str().to_string(),
        None => "Any".to_string()
      };
      Node::AbstractType {
        name: name,
        supertype: supertype
      }
    },
    Rule::AssignmentExpr => {
      let terms: Vec<_> = pair.clone().into_inner().collect();
      let name = terms[0].as_str().to_string();
      let val = create_ast(&terms[1]);
      Node::AssignmentExpr { identifier: name, value: Box::new(val) }
    },
    Rule::BinaryExpr => {
      let terms: Vec<_> = pair.clone().into_inner().collect();
      let lhs = create_ast(&terms[0]);
      let op = match terms[1].as_str() {
        "/" => Operator::Divide,
        "==" => Operator::Equal,
        "===" => Operator::EqualEqual,
        "-" => Operator::Minus,
        "*" => Operator::Multiply,
        "+" => Operator::Plus,
        _ => panic!("wtf {:?}", terms)
      };
      let rhs = create_ast(&terms[2]);
      Node::BinaryExpr { op: op, lhs: Box::new(lhs), rhs: Box::new(rhs) }
    },
    Rule::BinaryOperator => {
      let op = match pair.as_str() {
        "/" => Operator::Divide,
        "==" => Operator::Equal,
        "===" => Operator::EqualEqual,
        "-" => Operator::Minus,
        "*" => Operator::Multiply,
        "+" => Operator::Plus,
        "<:" => Operator::SubType,
        _ => todo!()
      };
      Node::Operator(op)
    }
    Rule::Comment => {
      Node::Empty
    },
    Rule::ConstExpr => {
      let parts: Vec<_> = pair.clone().into_inner().collect();
      // for part in parts {
      //   println!("part = {:?}", part)
      // }
      let ast = &parts
        .iter()
        .map(|p| create_ast(&p))
        .collect::<Vec<_>>()[0];
      Node::ConstExpr { expr: Box::new(ast.clone()) }
    }
    Rule::EndLineComment |
    Rule::EOI => {
      Node::Empty
    },
    Rule::ExportExpr => {
      let mut exports = Vec::<Node>::new();
      for export in pair.clone().into_inner() {
        match export.as_rule() {
          Rule::ExportLine => {
            let line_exports = export
              .into_inner()
              .map(|p| create_ast(&p))
              .collect::<Vec<_>>();
            exports.extend(line_exports);
          },
          _ => panic!()
        }
      }
      let exports = exports
        .into_iter()
        .filter(|e| match e {
          Node::Symbol(_) => true,
          _ => false
        })
        .collect::<Vec<_>>();
      Node::Exports { symbols: Box::new(exports) }
    }
    Rule::Expr => {
      create_ast(&pair.clone().into_inner().next().unwrap())
    },
    Rule::Function => {
      let parts = pair
        .clone()
        .into_inner()
        .collect::<Vec<_>>();
      let name = create_ast(&parts[0]);
      let name = match name {
        Node::Symbol(x) => x,
        _ => panic!("This shouldn't happend")
      };

      let extras = pair
        .clone()
        .into_inner()
        .skip(1)
        .collect::<Vec<_>>();
      let mut args = Vec::<Node>::new();
      let mut body = Vec::<Node>::new();
      let mut return_type = "Any".to_string();
      for extra in extras {
        match extra.as_rule() {
          Rule::FunctionArgs => {
            let _ = extra
              .into_inner()
              // .map(|x| args.push(create_ast(x).to_string()))
              .map(|x| args.push(create_ast(&x)))
              .collect::<Vec<_>>();
          },
          Rule::FunctionExprs => {
            let _ = extra
              .into_inner()
              .into_iter()
              .map(|x| body.push(create_ast(&x)))
              .collect::<Vec<_>>();
          },
          Rule::FunctionReturnType => {
            return_type = extra
              .into_inner()
              .collect::<Vec<_>>()[0]
              .as_str()
              .to_string();
          },
          _ => todo!("{}", extra)
        }
      }
      let args = Box::new(args);
      let body = Box::new(body);
      Node::Function { name, args, return_type, body }
    },
    Rule::Function2 => {
      let parts = pair
        .clone()
        .into_inner()
        .collect::<Vec<_>>();
      let name = create_ast(&parts[0]);
      let name = match name {
        Node::Symbol(x) => x,
        _ => panic!("This shouldn't happend")
      };
      for part in &parts {
        println!("\n\npart = {:?}", part);
      }
      let args = parts
        .iter()
        .filter(|p| matches!(p.as_rule(), Rule::FunctionArgs))
        .map(|p| {
          p.clone()
            .into_inner()
            .collect::<Vec<_>>()
            .iter()
            .filter(|p| matches!(p.as_rule(), Rule::FunctionArg))
            .map(|a| create_ast(&a))
            .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
      let body = parts
        .iter()
        .filter(|p| matches!(p.as_rule(),
          Rule::Expr
        ))
        .map(|p| create_ast(&p))
        .collect::<Vec<_>>();
      println!("name = {:?}", name);
      println!("args = {:?}", args);
      println!("expr = {:?}", body);

      // TODO TODO TODO TODO
      let args = Box::new(args);
      let return_type = "Any".to_string();
      let body = Box::new(body);
      Node::Function { name, args, return_type, body }
    }
    Rule::FunctionArg => {
      let parts: Vec<_> = pair.clone().into_inner().collect();
      let name = parts[0].as_str().to_string();

      let arg_type = if parts.len() > 1 {
        // Node::Symbol(parts[1].as_str().to_string())
        parts[1].as_str().to_string()
      } else {
        "Any".to_string()
      };
      Node::FunctionArg { name, arg_type: arg_type }
    },
    Rule::FunctionExpr => {
      // Node::Empty
      let expr = pair
        .clone()
        .into_inner()
        .collect::<Vec<_>>();
      if expr.len() > 0 {
        create_ast(&expr[0])
      } else {
        Node::Empty
      }
    }
    Rule::Identifier => {
      let name = pair.as_str().to_string();
      Node::Symbol(name)
    },
    Rule::IfExpr => {
      let parts: Vec<_> = pair.clone().into_inner().collect();
      let if_block = parts
        .iter()
        .filter(|p| matches!(p.as_rule(), Rule::IfBlock))
        .collect::<Vec<_>>();

      assert!(if_block.len() == 1);
      let (condition, if_block) = match if_block[0].as_rule() {
        Rule::IfBlock => {
          let block = if_block[0].clone().into_inner().collect::<Vec<_>>();
          let condition = block
            .iter()
            .filter(|p| matches!(p.as_rule(), Rule::ConditionExpr))
            .map(|p| {
              let parts = p.clone().into_inner().collect::<Vec<_>>();
              assert!(parts.len() == 1);
              create_ast(&parts[0])
            })
            .collect::<Vec<_>>();
          let asts = block
            .iter()
            .filter(|p| matches!(p.as_rule(), Rule::Expr))
            .map(|p| create_ast(&p))
            .collect::<Vec<_>>();
          (condition, asts)
        },
        _ => panic!()
      };
      assert!(condition.len() == 1);
      // TODO else if block

      let else_block = parts
        .iter()
        .filter(|p| matches!(p.as_rule(), Rule::ElseBlock))
        .collect::<Vec<_>>();
      let else_block = match else_block[0].as_rule() {
        Rule::ElseBlock => {
          let block = else_block[0].clone().into_inner().collect::<Vec<_>>();
          block
            .iter()
            .filter(|p| matches!(p.as_rule(), Rule::Expr))
            .map(|p| create_ast(&p))
            .collect::<Vec<_>>()
        },
        _ => panic!()
      };

      let condition = Box::new(condition[0].clone());
      let if_block = Box::new(if_block);
      let else_block = Box::new(else_block);
      Node::IfExpr { condition, if_block, else_block }
    },
    Rule::MacroExpr => {
      let parts = pair.clone().into_inner().collect::<Vec<_>>();
      let name = parts
        .iter()
        .filter(|p| matches!(p.as_rule(), Rule::Identifier))
        .map(|p| create_ast(&p))
        .next()
        .unwrap();
      let name = match name {
        Node::Symbol(x) => x,
        _ => panic!()
      };
      let args = parts
        .iter()
        .filter(|p| matches!(p.as_rule(), Rule::MacroArgs))
        .map(|p| {
          p
            .clone()
            .into_inner()
            .collect::<Vec<_>>()
            .iter()
            .map(|a| match a.as_rule() {
              Rule::MacroArg => {
                let parts = a.clone().into_inner().collect::<Vec<_>>();
                assert!(parts.len() == 1);
                create_ast(&parts[0])
              },
              _ => panic!()
            })
            .collect::<Vec<_>>()
        })
        .next();
      let args = match args {
        Some(x) => x,
        None => Vec::<Node>::new()
      };
      let body = parts
        .iter()
        .filter(|p| matches!(p.as_rule(), Rule::Expr))
        .map(|p| create_ast(&p))
        .collect::<Vec<_>>();
      let args = Box::new(args);
      let body = Box::new(body);
      Node::Macro { name, args, body }
    },
    Rule::MethodCall => {
      let params: Vec<_> = pair.clone().into_inner().collect();
      let name = params[0].as_str().to_string();
      let args: Vec<_> = params[1].clone().into_inner().collect();
      let args: Vec<_> = args
        .into_iter()
        .map(|x| create_ast(&x))
        .collect();
      let args = Box::new(args);
      Node::MethodCall { name: name, args: args }
    },
    Rule::Module => {
      let name = pair.clone().into_inner().next().unwrap().as_str().to_string();
      let exprs: Vec<_> = pair.clone().into_inner().skip(1).collect();
      let mut asts = Vec::<Node>::new();
      for expr in exprs {
        let ast = create_ast(&expr);
        asts.push(ast)
      }
      let asts = Box::new(asts);
      Node::Module { name: name, exprs: asts }
    },
    Rule::ParenthesesExpr => {
      let params: Vec<_> = pair.clone().into_inner().collect();
      Node::ParenthesesExpr { expr: Box::new(create_ast(&params[0])) }
    },
    Rule::Primitive => {
      create_primitive_ast(pair)
    },
    Rule::PrimitiveType => {
      let parts = pair.clone().into_inner();
      let name = parts.clone().collect::<Vec<_>>()[0].as_str().to_string();
      let extras: Vec<_> = parts.skip(1).collect();
      let mut supertype = "Any".to_string();
      let mut bits: u32 = 0;
      for extra in extras.iter() {
        match extra.as_rule() {
          Rule::PrimitiveBits => bits = extra.as_str().parse::<u32>().unwrap(),
          Rule::PrimitiveSuperType => supertype = extra.clone().into_inner().collect::<Vec<_>>()[0].as_str().to_string(),
          _ => todo!("Shouldn't happen")
        }
      }
      Node::PrimitiveType { name: name, supertype: supertype, bits: bits}
    },
    Rule::StructField => {
      let exprs: Vec<_> = pair.clone().into_inner().collect();
      let name = exprs[0].as_str().to_string();
      let field_type = match exprs.len() {
        1 => "Any".to_string(),
        2 => exprs[1].as_str().to_string(),
        _ => panic!("Unsupported thing in StructField with len {}", exprs.len())
      };
      Node::StructField {
        name: name,
        field_type: field_type
      }
    },
    Rule::StructType => {
      let exprs: Vec<_> = pair.clone().into_inner().collect();
      let mut name = "HOWTFDIDTHISHAPPEN".to_string();
      let mut supertype = "Any".to_string();
      let mut field_names = Vec::<Symbol>::new();
      let mut field_types = Vec::<Symbol>::new();

      // }
      for expr in exprs {
        let ast = create_ast(&expr);
        match ast {
          // Node::FieldType { .. } => fields.push(Box::new(ast)),
          // Node::Generics { .. } => generics.push(ast),
          Node::StructField { name, field_type } => {
            field_names.push(name);
            field_types.push(field_type);
          },
          // Node::SuperType { .. } => supertype = ast,
          Node::SuperType(x) => supertype = x,
          Node::Symbol(x) => name = x,
          _ => todo!("Not supported yet {:?}", ast)
        }
      }
      Node::StructType { 
        name: name, 
        // generics: generics, 
        supertype: supertype, 
        field_names: field_names,
        field_types: field_types
      }
    },
    Rule::SuperType => {
      let exprs: Vec<_> = pair.clone().into_inner().collect();
      Node::SuperType(exprs[0].as_str().to_string())
    }
    Rule::UnaryExpr => {
      let terms: Vec<_> = pair.clone().into_inner().collect();
      let op = match terms[0].as_str() {
        "-" => Operator::Minus,
        "+" => Operator::Plus,
        _ => panic!("unsupported op deteceted in unaryexpr")
      };
      let val = create_ast(&terms[1]);
      Node::UnaryExpr { op: op, child: Box::new(val) }
    },
    _ => todo!("todo {:?} {:?}", pair.as_rule(), pair)
  };
  ast
}

fn create_primitive_ast(pair: &pest::iterators::Pair<Rule>) -> Node {
  let prim: Vec<_> = pair.clone().into_inner().collect();
  let prim = match prim[0].as_rule() {
    Rule::Char => {
      let c = prim[0].as_str().replace("'", "");
      let c = c.parse::<char>().unwrap();
      Primitive::Char(c)
    },
    Rule::Float => Primitive::Float64(pair.as_str().parse::<f64>().unwrap()),
    Rule::Int => Primitive::Int64(pair.as_str().parse::<i64>().unwrap()),
    Rule::String => Primitive::String(pair.as_str().to_string()
      .replace("\"", "").replace("\"", "")),
    _ => panic!("Unsupported primitive encountered in ast {:?}", pair)
  };
  Node::Primitive(prim)
}
