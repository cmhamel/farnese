use core::cmp::Ordering;
use crate::constants;

// TODO probably want to make a parse trait for these tokens
// and do something with enum_dispatch

#[derive(Debug)]
pub enum Token {
  BinaryOperator,
  Keyword,
  Name,
  Number,
  OtherOperator
}

fn parse_keyword(line: &String, i_in: i32) -> (String, i32) {

  let mut i = i_in;
  let mut token_str = String::new();

  for (n, c) in line.chars().enumerate() {
    // ignore the white space
    if n < i.try_into().unwrap() {
      continue;
    }

    // first char should be alphabeta
    if n == i.try_into().unwrap() {
      if c.is_alphabetic() {
        token_str.push(c);
      }
    }

    // allow for none alphabetic
    if n > i.try_into().unwrap() {
      if constants::OPERATORS.contains(&c) {
        i = n.try_into().expect("Index convert");
        break;
      } else {
        i = i + 1;
        if c.is_alphanumeric() {
          token_str.push(c);
        }
      }
    }

    if c.is_whitespace() {
      i = n.try_into().expect("Index convert");
      break;
    }
  }
  
  return (token_str, i);
}

// parses a single number into a temp string
fn parse_number(line: &String, i_in: i32) -> (String, i32) {

  let mut i = i_in;
  let mut token_str = String::new();

  for (n, c) in line.chars().enumerate() {
    match n.cmp(&mut i.try_into().unwrap()) {
      Ordering::Less => continue,
      Ordering::Equal => token_str.push(c),
      Ordering::Greater => {
        if c.is_digit(10) || c == '.' {
          token_str.push(c);
        }
  
        if c.is_whitespace() {
          i = n.try_into().expect("Index convert");
          break;
        }
      }
    }
  }

  return (token_str, i);
}

fn parse_token(
  line: &String, start_i: i32
) -> (String, i32) {
  let mut last_char = ' ';
  let mut token_str = String::new();
  let mut i = 0;

  if start_i != 0 {
    i = start_i;
  }

  // do an initial pass to get rid of whitespace
  for (n, c) in line.chars().enumerate() {
    if n < i.try_into().unwrap() {
      continue;
    }

    if c.is_whitespace() {
      i = i + 1;
      continue;
    } else {
      last_char = c;
      break;
    }
  }

  // now loop over the next set of chars
  // to find the keyword

  if last_char.is_alphabetic() {
    (token_str, i) = parse_keyword(line, i);
  } else if last_char.is_digit(10) {
    (token_str, i) = parse_number(line, i);    
  } else {
    if constants::OPERATORS.contains(&last_char) {
      token_str.push(last_char);
      i = i + 1;
    } else {
      i = i + 1;
    }
  }

  return (token_str, i)
}

pub fn parse_tokens(chunk: &String) -> Vec<String> {
  let mut n = 0;
  let mut token_str = String::new();
  let mut tokens: Vec<String> = Vec::new();
  while n < chunk.len().try_into().unwrap() {
    (token_str, n) = parse_token(chunk, n);
    if !token_str.is_empty() {
      tokens.push(token_str)
    }
  }

  return tokens;
}

pub fn create_tokens(tokens: Vec<String>) -> Vec<Token> {
  let mut token_types: Vec<Token> = Vec::new();

  for token in tokens.iter() {
    // check for keywors first
    if constants::KEYWORDS.contains(&token.as_str()) {
      token_types.push(Token::Keyword);
      continue;
    }

    // now check for operations
    if token.chars().any(|c| constants::BINARY_OPERATORS.contains(&c)) {
      token_types.push(Token::BinaryOperator);
      continue;
    }

    if token.chars().any(|c| constants::OTHER_OPERATORS.contains(&c)) {
      token_types.push(Token::OtherOperator);
      continue;
    }

    // check if we have numbers
    match token.parse::<f64>() {
      Ok(_) => {
        token_types.push(Token::Number);
        continue;
      },
      Err(_)  => {
        // do nothing and continue on
      }
    }

    // if we made it here, this is a name of some kind
    token_types.push(Token::Name);
  }

  token_types
}