use crate::ast::*;
use once_cell::sync::Lazy;
use pest::iterators::Pair;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::Parser;
use pest_derive::Parser as PestParser;

#[derive(PestParser)]
#[grammar = "grammar.pest"]
pub struct FarneseParser;

// Precedence table, lowest to highest. This is where operator
// precedence actually lives -- the grammar itself is deliberately flat.
static PRATT: Lazy<PrattParser<Rule>> = Lazy::new(|| {
    use Rule::*;
    PrattParser::new()
        .op(Op::infix(Eq, Assoc::Left) | Op::infix(Neq, Assoc::Left))
        .op(Op::infix(Lt, Assoc::Left) | Op::infix(Gt, Assoc::Left)
            | Op::infix(Le, Assoc::Left) | Op::infix(Ge, Assoc::Left))
        .op(Op::infix(Add, Assoc::Left) | Op::infix(Sub, Assoc::Left))
        .op(Op::infix(Mul, Assoc::Left) | Op::infix(Div, Assoc::Left))
        // Placed last => binds tightest, i.e. -a * b is (-a) * b, not
        // -(a * b). If a power operator were added later with looser-
        // than-unary Julia semantics (-2^2 == -4), it would slot in
        // *after* this line instead of before it.
        .op(Op::prefix(Neg) | Op::prefix(Not))
});

fn parse_assignment(pair: Pair<Rule>) -> Statement {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_expr(inner.next().expect("Assignment always has a value Expr"));
    Statement::Assignment { name, value }
}

fn parse_call(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let args = inner
        .next()
        .map(|args_pair| args_pair.into_inner().map(parse_expr).collect())
        .unwrap_or_default();
    Expr::Call { name, args }
}

fn parse_expr(pair: Pair<Rule>) -> Expr {
    debug_assert_eq!(pair.as_rule(), Rule::Expr);
    PRATT
        .map_primary(parse_primary)
        .map_prefix(|op, rhs| {
            let unary_op = match op.as_rule() {
                Rule::Neg => UnaryOp::Neg,
                Rule::Not => UnaryOp::Not,
                other => unreachable!("unexpected unary operator: {other:?}"),
            };
            Expr::UnaryExpr {
                op: unary_op,
                expr: Box::new(rhs),
            }
        })
        .map_infix(|lhs, op, rhs| {
            let bin_op = match op.as_rule() {
                Rule::Add => BinOp::Add,
                Rule::Sub => BinOp::Sub,
                Rule::Mul => BinOp::Mul,
                Rule::Div => BinOp::Div,
                Rule::Eq => BinOp::Eq,
                Rule::Neq => BinOp::Neq,
                Rule::Lt => BinOp::Lt,
                Rule::Gt => BinOp::Gt,
                Rule::Le => BinOp::Le,
                Rule::Ge => BinOp::Ge,
                other => unreachable!("unexpected binary operator: {other:?}"),
            };
            Expr::BinaryExpr {
                op: bin_op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        })
        .parse(pair.into_inner())
}

fn parse_function_def(pair: Pair<Rule>) -> FunctionDef {
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    let mut next = inner.next().expect("FunctionDef always has a Block");
    let params = if next.as_rule() == Rule::Params {
        let p = next.into_inner().map(|id| id.as_str().to_string()).collect();
        next = inner.next().unwrap();
        p
    } else {
        Vec::new()
    };

    // `next` is now the Block.
    let body = next.into_inner().map(parse_statement).collect();

    FunctionDef { name, params, body }
}

fn parse_primary(pair: Pair<Rule>) -> Expr {
    debug_assert_eq!(pair.as_rule(), Rule::Primary);
    let inner = pair.into_inner().next().expect("Primary always wraps something");
    match inner.as_rule() {
        // Rule::Int => Expr::Int(inner.as_str().parse().expect("Int rule guarantees digits")),
        // Rule::Float => Expr::Float(inner.as_str().parse().expect("Float rule guarantees valid syntax")),
        // Rule::String => Expr::String(parse_string(inner)),
        // Rule::Symbol => Expr::Symbol(inner.as_str()[1..].to_string()), // strip leading ':'
        Rule::Expr => parse_expr(inner), // parenthesized sub-expression
        Rule::FunctionCall => parse_call(inner),
        Rule::Identifier => Expr::Identifier(inner.as_str().to_string()),
        // primitives
        Rule::Float | Rule::Int | Rule::String | Rule::Symbol => {
            parse_primitive(inner)
        },
        other => unreachable!("unexpected primary rule: {other:?}"),
    }
}

fn parse_primitive(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::Int => Expr::Primitive(
            Primitive::Int(pair.as_str().parse().expect("Int rule guarantees digits"))
        ),
        Rule::Float => Expr::Primitive(
            Primitive::Float64(pair.as_str().parse().expect("Float rule guarantees valid syntax"))
        ),
        Rule::String => Expr::Primitive(Primitive::String(parse_string(pair))),
        Rule::Symbol => Expr::Primitive(Primitive::Symbol(pair.as_str()[1..].to_string())), // strip leading ':'
        _ => todo!()
    }
}

pub fn parse_program(source: &str) -> Result<Program, Box<pest::error::Error<Rule>>> {
    let mut pairs = FarneseParser::parse(Rule::Program, source).map_err(Box::new)?;
    let program_pair = pairs.next().expect("Program rule always produces one pair");

    let mut items = Vec::new();
    for pair in program_pair.into_inner() {
        match pair.as_rule() {
            Rule::EOI => {},
            Rule::FunctionDef => items.push(Item::Function(parse_function_def(pair))),
            Rule::Statement => items.push(Item::Statement(parse_statement(pair))),
            other => unreachable!("unexpected top-level rule: {other:?}"),
        }
    }
    Ok(Program { items })
}

fn parse_statement(pair: Pair<Rule>) -> Statement {
    let inner = pair.into_inner().next().expect("Statement wraps Assignment or Expr");
    match inner.as_rule() {
        Rule::Assignment => parse_assignment(inner),
        Rule::Expr => Statement::Expr(parse_expr(inner)),
        other => unreachable!("unexpected statement rule: {other:?}"),
    }
}

fn parse_string(pair: Pair<Rule>) -> String {
    // `pair` is String = "..." ; its one child is StringInner (the
    // content between the quotes, escapes still un-decoded).
    let inner = pair
        .into_inner()
        .next()
        .expect("String always wraps StringInner");
    unescape(inner.as_str())
}

fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('0') => out.push('\0'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            // The grammar's Escape rule only ever admits the above, so
            // this should be unreachable for input that parsed at all.
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_int() {
        let parsed = parse_program("1").unwrap();
        println!("Parsed = {:?}", parsed.items[0]);
    }
}