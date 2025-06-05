use clap::{Parser, Subcommand};
use farnese::compiler::Compiler;
use farnese::core::Module;
use farnese_lexer::lexer::lexer;
use inkwell::context::Context;
use inkwell::passes::PassManager;
use inkwell::targets::{CodeModel, RelocMode, Target, TargetTriple};
use inkwell::targets::InitializationConfig;
use inkwell::OptimizationLevel;
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

pub fn optimize_ir<'a>(module: &Module<'a>) -> () {
  // let module = self.modules.get(&Symbol::new("Main")).unwrap();
  // module.print_to_stderr();
  let fpm = PassManager::create(());
  // Add some optimization passes
  fpm.add_instruction_combining_pass();
  fpm.add_reassociate_pass();
  fpm.add_gvn_pass();
  fpm.add_cfg_simplification_pass();
  fpm.add_basic_alias_analysis_pass();
  // run optimization passes
  fpm.run_on(&module.module);
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
        println!("{}", ast);
      }
    },
    Some(Commands::Compiler { input, output, optimize }) => {
      let source = fs::read_to_string(input)
        .expect("Failed reading file");
      let context = Context::create();

      let mut compiler = Compiler::new(&context);

      let mut main_module = Module::new(&context, "Main");
      compiler.insert_module("Main", main_module.clone());
      // compiler.main_func_begin(&main_module);
      compiler.include(&mut main_module, &source);
      // compiler.main_func_end(&main_module);

      // TODO move elsewhere
      if optimize {
        optimize_ir(&main_module);

        // Initialize target and target machine
        // This is necessary to apply machine-specific optimizations (optional)
        // Target::initialize_x86(&InitializationConfig::default());
        // let target = Target::from_name("x86_64-unknown-linux-gnu").unwrap();
        // let target_machine = target.create_target_machine(
        //     &main_module.get_triple(),
        //     &main_module.get_data_layout(),
        //     OptimizationLevel::Aggressive,
        //     false,
        //     false,
        // ).unwrap();
        // let file_path = "optimized_output.ll";
        // target_machine.write_to_file(&main_module, &file_path.into()).expect("Failed to write optimized IR");

        // todo hook up command line option
        let reloc = RelocMode::Default;
        let model = CodeModel::Default;
        let opt = OptimizationLevel::Aggressive;

        Target::initialize_x86(&InitializationConfig::default());

        let triple = TargetTriple::create("x86_64-pc-linux-gnu");
        let target = Target::from_triple(&triple).unwrap();
        let _target_machine = target.create_target_machine(
          &TargetTriple::create("x86_64-pc-linux-gnu"),
          "x86-64",
          "",
          opt,
          reloc,
          model
        ).unwrap();
        // let options = TargetMachineOptions::default()
        //     .set_cpu("x86-64")
        //     // .set_features("+avx2")
        //     // .set_abi("sysv")
        //     .set_level(OptimizationLevel::Aggressive);

        // let target_machine = target.create_target_machine_from_options(&triple, options).unwrap();

      }

      // let _ = main_module.print_to_file("Main.ll");
      let _ = main_module.print_to_file(&output);
    },
    _ => println!("Wtf")
  }
}
