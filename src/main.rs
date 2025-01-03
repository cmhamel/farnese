// move to a lib.rs
pub mod ast;
pub mod base;
pub mod compiler;
pub mod parser;
// pub mod repl;

use crate::compiler::{Compile, Compiler};
// use crate::compiler::interpreter::Interpreter;

use clap::{Parser, Subcommand};
use inkwell::context::Context;
// use inkwell::types::AnyTypeEnum;
use std::fs;
use std::fs::File;
use std::io::Write;


#[derive(Clone, Debug, Parser)]
#[command(arg_required_else_help = true, version)]
struct CLIArgs {
  #[command(subcommand)]
  command: Option<Commands>
}

#[derive(Clone, Debug, Subcommand)]
enum Commands {
  #[command(about = "Compiler")]
  Compiler {
    #[arg(long, short, value_name = "FARNESE FILE")]
    input: String,
    #[arg(long, short, value_name = "IR FILE")]
    output: String,
  },
  #[command(about = "Read script")]
  Farnese {
    #[arg(long, short, value_name = "FILE")]
    input: String
  },
  #[command(about = "Start the repl")]
  Repl
}

fn main() {
  let args = CLIArgs::parse();
  match args.command {
    Some(Commands::Compiler {
      input, output
    }) => {
      let text = fs::read_to_string(input);
      let context = Context::create();
      let mut compiler = Compiler::new(&context);
      // let _ = match text {
      //   Ok(x) => compiler.from_source(&x),
      //   Err(_) => panic!("Bad file.")
      // };

      // add format strings for different types
      compiler.build_format_string("%f ", "__format_f64");
      compiler.build_format_string("%d ", "__format_i64");

      // compiler.build_alloca(AnyTypeEnum::IntType(context.i32_type()), "tag");

      let _ = match text {
        Ok(x) => compiler.from_source(&x),
        Err(_) => panic!("Bad file.")
      };
      compiler.build_default_return();

      let ir = compiler.dump_ir();
      let mut file = File::create(output).expect("Unable to create file");
      file.write_all(ir.as_bytes()).expect("Unable to write data");
      // compiler.build();
    },
    // Some(Commands::Farnese {
    //   input
    // }) => {
    //   println!("Reading script");
    //   let text = fs::read_to_string(input);
    //   let mut interp = Interpreter::new();
    //   println!("Beginning AST construction");
    //   let _ = match text {
    //     Ok(x) => interp.from_source(&x),
    //     Err(_) => panic!("Bad file.")
    //   };
    // },
    // Some(Commands::Repl) => {
    //   let _ = repl::repl();
    // },
    _ => println!("Wtf")
  }
}
