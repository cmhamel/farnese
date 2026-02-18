use farnese_lexer::{
    parser::{
        FarneseParser, 
        Rule,
        parse_statement
    }
};
use pest::Parser;
use std::{io::{self, BufRead}};

fn main() -> io::Result<()> {
    for line in io::stdin().lock().lines() {
        match FarneseParser::parse(Rule::Program, &line?) {
            Ok(mut pairs) => {
                let pair = pairs.next().unwrap();
                match pair.as_rule() {
                    Rule::Program => {
                        let parts = pair.into_inner();
                        for part in parts {
                            match part.as_rule() {
                                Rule::EOI => {
                                    None
                                },
                                Rule::Statement => {
                                    let mut exprs = vec![];
                                    for pair in part.into_inner() {
                                        let temp = parse_statement(pair);
                                        println!("Expr = {:#?}", temp);
                                        exprs.push(temp);
                                    }
                                    Some(exprs)
                                },
                                _ => todo!()
                            };
                        }
                    }
                    _ => todo!("Unfinished {:#?}", pair)
                };
            }
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
            }
        }
    }
    Ok(())
}
