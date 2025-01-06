use clap::{Parser, Subcommand};
use farnese::compiler::Compiler;
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
    #[arg(long, short, value_name = "OPTIMIZE")]
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
    Some(Commands::Compiler { input, output, optimize }) => {
      let source = fs::read_to_string(input);
      let context = Context::create();
      let builder = context.create_builder();
      let mut compiler = Compiler::new(&context, &builder);

      // todo could probably move this somewhere else
      // need to compile a few core things first
      compiler.create_core_module();
      compiler.create_main_module();

      let _ = match source {
        Ok(x) => compiler.from_source(&x),
        Err(_) => panic!("Bad file.")
      };
      // // compiler.build_default_return();

      // link everything to main module
      compiler.link();

      if optimize {
        compiler.optimize_ir();
      }
      let _ = compiler.write_ir_to_file(&output);
    },
    _ => println!("Wtf")
  }
}
