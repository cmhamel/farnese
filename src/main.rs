use clap::{Parser, Subcommand};
use farnese::compiler::Compiler;
use farnese_core::Module;
use farnese_lexer::lexer;
use inkwell::context::Context;
use std::fs;

#[derive(Clone, Debug, Parser)]
#[command(arg_required_else_help = true, version)]
struct CLIArgs {
  #[command(subcommand)]
  command: Option<Commands>
}

#[derive(Clone, Debug, Subcommand)]
enum Commands {
  #[command(about = "ast")]
  Ast {
    #[arg(long, short, value_name = "FARNESE FILE")]
    input: String
  },
  #[command(about = "Compiler")]
  Compiler {
    #[arg(long, short, value_name = "FARNESE FILE")]
    input: String,
    #[arg(long, short, value_name = "IR FILE")]
    output: String,
    #[arg(long, value_name = "OPTIMIZE")]
    optimize: bool,
  }
}

fn main() {
  let args = CLIArgs::parse();
  match args.command {
    Some(Commands::Ast { input }) => {
      let source = fs::read_to_string(input)
        .expect("Failed reading file");
      let asts: Vec<_> = lexer::parse(&source).unwrap();
      println!("Dumping ASTs\n\n");
      for ast in asts.iter() {
        println!("{:?}", ast);
      }
    },
    Some(Commands::Compiler { input, output, optimize: _ }) => {
      let source = fs::read_to_string(input)
        .expect("Failed reading file");
      let context = Context::create();

      let mut compiler = Compiler::new(&context);

      let mut main_module = Module::new(&context, "Main");
      compiler.insert_module("Main", main_module.clone());
      compiler.include(&mut main_module, &source);

      let _ = main_module.print_to_file(&output);
    },
    _ => println!("Wtf")
  }
}
