use crate::ast::Expr;
use crate::expr_parser::parse_expr;
use pest::{
    self,
    iterators::Pair,
    Parser
};
use std::fs;

#[derive(pest_derive::Parser)]
#[grammar = ".//grammar.pest"]
pub struct FarneseParser;

impl FarneseParser {
    pub fn from_source(source: &str) -> pest::iterators::Pairs<'_, Rule> {
        FarneseParser::parse(Rule::Program, source)
            .expect(format!("Failed parsing file {}", source).as_str())
    }
}

pub fn parse_file(file_name: &str) -> std::result::Result<Vec<Expr>, pest::error::Error<Rule>> {
    let source = fs::read_to_string(file_name)
        .expect(format!("Failed reading file: {}", file_name).as_str());

    let mut ast = Vec::new();
    match FarneseParser::parse(Rule::Program, &source) {
        Ok(mut pairs) => {
            for pair in pairs.next().unwrap().into_inner() {
                match pair.as_rule() {
                    Rule::EOI => {
                        // Expr::Empty
                    }
                    Rule::Statement => {
                        let expr = parse_statement(pair);
                        // println!("Expr = {:#?}", expr);
                        // ast.push(parse_statement(pair));
                        ast.push(expr);
                    }
                    _ => todo!("Pair = {:#?}", pair)
                }
            }
        }
        Err(e) => {
            println!("Parse failed: {:?}", e);
        }
    }
    Ok(ast)
}

pub fn parse_statement(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::AbstractType => {
            let mut parts = pair.into_inner();
            // let sym = Expr::Symbol(parts.next().unwrap().as_str().to_string());
            let sym = parts.next().unwrap().as_str().to_string();
            let mut supertype = Expr::Symbol("Any".to_string());
            while let Some(val) = parts.next() {
                supertype = Expr::Symbol(val.as_str().to_string());
            }
            Expr::AbstractType {
                name: sym,
                supertype: Box::new(supertype)
            }
        }
        Rule::Assignment => {
            let mut parts = pair.into_inner();
            let sym = Expr::Symbol(parts.next().unwrap().as_str().to_string());
            let expr = parse_expr(parts.next().unwrap().as_str());
            Expr::Assignment {
                symbol: Box::new(sym),
                expr: Box::new(expr)
            }
        },
        Rule::Call => {
            let mut parts = pair.into_inner();
            let sym = parts.next().unwrap().as_str().to_string();
            let mut args = Vec::new();
            while let Some(val) = parts.next() {
                args.push(parse_statement(val))
            }
            Expr::Call {
                func: sym,
                args: Box::new(args)
            }
        }
        Rule::ComplexFuncDef |
        Rule::SimpleFuncDef => {
            let mut parts = pair.clone().into_inner();
            let func = parts.next().unwrap().as_str().to_string();
            let mut args = Vec::new();
            let mut body = Vec::new();
            while let Some(val) = parts.next() {
                match val.as_rule() {
                    Rule::Identifier |
                    Rule::TypedArg => {
                        let mut parts_in = val.into_inner();
                        let sym = Expr::Symbol(parts_in.next().unwrap().as_str().to_string());
                        // let ty = Expr::Type { name: match parts_in.next() {
                        let ty = Expr::Type(match parts_in.next() {
                            Some(s) => s.as_str().to_string(),
                            None    => "Any".to_string()
                        });
                        args.push(Expr::Val {
                            ty: Box::new(ty),
                            val: Box::new(sym)
                        })
                    },
                    Rule::Statement => {
                        body.push(parse_statement(val))
                    },
                    _ => todo!()
                }
            }
            Expr::Method {
                func: func,
                args: Box::new(args),
                body: Box::new(body),
                ret: Box::new(Expr::Symbol("Any".to_string()))
            }
        }
        Rule::PrimitiveType => {
            let mut parts = pair.into_inner();
            let sym = parts.next().unwrap().as_str().to_string();
            let mut extras = Vec::new();
            while let Some(val) = parts.next() {
                extras.push(val)
            }

            let (supertype, bits) = match extras.len() {
                1 => (Expr::Symbol("Any".to_string()), extras[0].as_str().parse::<i32>().unwrap()),
                2 => (Expr::Symbol(extras[0].as_str().to_string()), extras[1].as_str().parse::<i32>().unwrap()),
                _ => todo!()
            };

            Expr::PrimitiveType {
                name: sym,
                supertype: Box::new(supertype),
                bits: bits
            }
        }
        Rule::Expr => {
            parse_expr(pair.as_str())
        }
        Rule::Identifier => {
            Expr::Symbol(pair.as_str().to_string())
        }
        Rule::SingleLineComment => {
            Expr::Empty
        }
        Rule::Statement => {
            if pair.clone().into_inner().len() > 1 {
                panic!()
            }
            let mut parts = pair.into_inner();
            parse_statement(parts.next().unwrap())
        }
        _ => todo!("Pair = {:#?}", pair)
    }
}

pub fn parse_typed_arg(pair: Pair<Rule>) -> (Expr, Expr) {
    match pair.as_rule() {
        Rule::Identifier => {
            let name = parse_statement(pair);
            let type_name = Expr::Symbol("Any".to_string());
            (name, type_name)
        }
        _ => todo!()
    }
}

impl Expr {
    pub fn from_string(string: &str) -> Expr {
        match FarneseParser::parse(Rule::Program, &string) {
            Ok(mut pairs) => {
                // if pairs.len() > 1 {
                //     panic!()
                // }
                let pair = pairs.next()
                    .unwrap().into_inner()
                    .collect::<Vec<_>>()[0].clone();
                match pair.as_rule() {
                    Rule::EOI => Expr::Empty,
                    Rule::Statement => parse_statement(pair),
                    _ => todo!()
                }
            }
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
                panic!();
            }
        }
    }
}
