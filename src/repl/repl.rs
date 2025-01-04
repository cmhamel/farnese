// use crate::ast;
// use crate::ast::Expression;
// use crate::parser::lexer;
use crate::compiler::Compile;
use crate::compiler::interpreter::Interpreter;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use termion::color;

pub fn repl() -> Result<()> {
  let mut interp = Interpreter::new();
  let mut rl = DefaultEditor::new()?;
  // let mut prev_line = "".to_string();
  loop {
    let farnese = format!("{}farnese> {}", color::Fg(color::Green), color::Fg(color::White));
    let readline = rl.readline(&farnese);
    match readline {
      Ok(line) => {
        let mut parsed = line.clone();
        if line.contains("struct") {
          'inner: loop {
            let subline = rl.readline("             ");
            match subline {
              Ok(subline) => {
                if subline.contains("end") {
                  parsed = format!("{} {}", parsed, subline);
                  break;
                } else {
                  parsed = format!("{}{}", parsed, subline);
                }
              }
              Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
              },
              Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
              },
              Err(err) => {
                println!("Error: {:?}", err);
                break;
              }
            }
          }
        }
        println!("parsed = {:?}", parsed);
        // let ast = parse(&parsed);
        // let ast = Interpreter::from_source(&parsed);
        let ast = interp.from_source(&parsed);
        println!("ast = {:?}", ast);
      },
      Err(ReadlineError::Interrupted) => {
        println!("CTRL-C");
        break;
      },
      Err(ReadlineError::Eof) => {
        println!("CTRL-D");
        break;
      },
      Err(err) => {
        println!("Error: {:?}", err);
        break;
      }
    }
  }
  Ok(())
}
