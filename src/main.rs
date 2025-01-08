use clap::{Parser, Subcommand};
use farnese::compiler::Compiler;
use farnese::core::{Core, Symbol};
use farnese::core::main::create_main;
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
  #[command(about = "Compiler")]
  Compiler {
    #[arg(long, short, value_name = "FARNESE FILE")]
    input: String,
    #[arg(long, short, value_name = "IR FILE")]
    output: String,
    #[arg(long, value_name = "OPTIMIZE")]
    optimize: bool,
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
    Some(Commands::Compiler { input, output: _, optimize: _ }) => {
      let source = fs::read_to_string(input)
        .expect("Failed reading file");
      let context = Context::create();
      let builder = context.create_builder();

      // let ast = lexer::parse(&source);
      let mut core = Core::new(&context, &builder);
      let _ = core.bootstrap();
      let _ = core.module().print_to_file("Core.ll");
      
      let mut compiler = Compiler::new(&context, &builder);
      compiler.insert_module(Symbol::new("Core"), core.module());
      // let core = core.create_core(&context, &builder);

      // try to include this file
      compiler.include(&source);

      // now lets creat a main module
      let main_module = context.create_module("Main");

      main_module.link_in_module(core.module())
        .expect("Failed to link Core into Main");
      println!("Linked core into main");

      let _ = create_main(&context, &builder, &main_module, &compiler.modules());

      for (name, module) in compiler.modules() {
        if name.name() == "Core" {
          continue
        } else {
          main_module.link_in_module(module.clone())
            .expect(format!("Failed to link {} into Main", name.name()).as_str())
        }
      }

      let _ = main_module.print_to_file("Main.ll");
    },
    _ => println!("Wtf")
  }
}
