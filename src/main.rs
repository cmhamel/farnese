use clap::{Parser, Subcommand};
use farnese_compiler::Compiler;
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
      //   .replace('\u{240A}', "\n") // visual linefeed
      //   .replace('\u{240D}', "\n") // visual carriage return
      //   .replace("\r\n", "\n")
      //   .replace('\r', "\n")
      //   .chars()
      //   .filter(|&c| !(('\u{2400}'..='\u{2426}').contains(&c))) // strip U+2400–U+2426
      //   .collect::<String>();

      // for (i, c) in source.chars().enumerate() {
      //   if c == '\u{240A}' {
      //     println!("Found visible LF symbol (U+240A) at char index {}", i);
      //   } else if !c.is_ascii() {
      //     println!("Non-ASCII char at {}: U+{:04X} ({:?})", i, c as u32, c);
      //   }
      // }
      // panic!();
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
