use crate::ast::Expr;
use pest::{
    self,
    iterators::{Pair, Pairs},
    Parser,
    pratt_parser::PrattParser
};
use std::sync::LazyLock;

#[derive(pest_derive::Parser)]
#[grammar = ".//expr_grammar.pest"]
pub struct ExprParser;

impl ExprParser {
    pub fn from_source(source: &str) -> pest::iterators::Pairs<'_, Rule> {
        ExprParser::parse(Rule::Expr, source)
            .expect(format!("Failed parsing file {}", source).as_str())
    }
}

pub static PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    use pest::pratt_parser::{Assoc::*, Op};
    use Rule::*;

    // Precedence is defined lowest to highest
    PrattParser::new()
        .op(Op::infix(Rule::Add, Left)      | Op::infix(Rule::Subtract, Left))
        .op(Op::infix(Rule::Multiply, Left) | Op::infix(Rule::Divide, Left) | Op::infix(Rule::Modulo, Left))
        .op(Op::infix(Rule::Power, Right))
        .op(Op::prefix(Minus))
});

pub fn parse_expr(source: &str) -> Expr {
    match ExprParser::parse(Rule::ExprAll, &source) {
        Ok(mut pairs) => {
            parse_expr_part(pairs.next().unwrap().into_inner())
        }
        Err(e) => {
            eprintln!("Parse failed: {:#?}", e);
            panic!()
        }
    }
}

pub fn parse_expr_part(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::Expr       => parse_expr_part(primary.into_inner()),
            Rule::Identifier => Expr::Symbol(primary.as_str().to_string()),
            Rule::Primitive  => parse_primitive(primary),
            rule => unreachable!("Expr::parse expected atom, found {:?} {:#?}", rule, primary),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                // Rule::Add      => BinaryOperator::Add,
                // Rule::Divide   => BinaryOperator::Divide,
                // Rule::Modulo   => BinaryOperator::Modulo,
                // Rule::Multiply => BinaryOperator::Multiply,
                // Rule::Power    => BinaryOperator::Power,
                // Rule::Subtract => BinaryOperator::Subtract,
                Rule::Add      => "+",
                Rule::Divide   => "/",
                Rule::Modulo   => "%",
                Rule::Multiply => "*",
                Rule::Power    => "^",
                Rule::Subtract => "-",
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::Call {
                func: op.to_string(),
                args: Box::new(vec![lhs, rhs])
            }
        })
        .map_prefix(|op, rhs| {
            let op = match op.as_rule() {
                // Rule::Minus => UnaryOperator::Minus,
                Rule::Minus => "-",
                rule => unreachable!("Expr::parse expected prefix operation, found {:?}", rule),
            };
            Expr::Call {
                func: op.to_string(),
                args: Box::new(vec![rhs])
            }
        })
        .parse(pairs)
}

fn parse_primitive(pair: Pair<Rule>) -> Expr {
    let parts = pair.into_inner();
    if parts.len() > 1 {
        panic!()
    }
    let mut ty_ret = "";
    let mut val_ret = Expr::Empty;
    for part in parts {
        ty_ret = match part.as_rule() {
            Rule::Float64 => "Float64",
            Rule::Int     => "Int64",
            Rule::String  => "String",
            _ => todo!("Part = {:#?}", part)
        };
        val_ret = match part.as_rule() {
            Rule::Float64 => Expr::Float64(part.as_str().trim().parse::<f64>().unwrap()),
            Rule::Int     => Expr::Int(part.as_str().trim().parse::<i64>().unwrap()),
            Rule::String  => Expr::String(part.to_string()),
            _ => todo!("Part = {:#?}", part)
        };
    };
    Expr::Val {
        ty: Box::new(Expr::Symbol(ty_ret.to_string())),
        val: Box::new(val_ret)
    }
}
