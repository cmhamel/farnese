use pest::{self, Parser};

#[derive(pest_derive::Parser)]
#[grammar = ".//grammar.pest"]
pub struct FarneseParser;

impl FarneseParser {
  pub fn from_source(source: &str) -> pest::iterators::Pairs<'_, Rule> {
    FarneseParser::parse(Rule::Program, source)
      .expect(format!("Failed parsing file {}", source).as_str())
  }
}
