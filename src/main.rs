use clap::{Parser, Subcommand};
use farnese::compiler::Compiler;
use farnese::core::{Core, Symbol};
use farnese::core::main::create_main;
use inkwell::context::Context;
use inkwell::module::Module;
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

pub fn optimize_ir<'a, 'b>(module: &'b Module<'a>) -> () {
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
  fpm.run_on(module);
}

fn main() {
  let args = CLIArgs::parse();
  match args.command {
    Some(Commands::Compiler { input, output: _, optimize }) => {
      let source = fs::read_to_string(input)
        .expect("Failed reading file");
      let context = Context::create();
      let builder = context.create_builder();

      // let ast = lexer::parse(&source);
      let mut core = Core::new(&context);
      let _ = core.bootstrap();
      let _ = core.module().print_to_file("Core.ll");
      
      let mut compiler = Compiler::new(&context, &builder);
      compiler.insert_module(Symbol::new("Core"), core.module());
      // include base
      // compiler.include("base/base.jl", &main_module);

      // creating main module here for now.
      let main_module = context.create_module("Main");
      // main_module.link_in_module(core.module())
      //   .expect("Failed to link Core into Main");
      // println!("Linked core into main");
      let base_source = fs::read_to_string("src/base/base.jl").unwrap();
      compiler.include(&base_source, &main_module);

      // TODO need to include all modules before main is created
      for (name, module) in compiler.modules() {
        println!("Name = {:?}", name);
        if *name == Symbol::new("Main") {
          println!("Hur")
        }

        if *name == Symbol::new("Core") {
          continue
        }
        let _ = main_module.link_in_module(module.clone())
          .expect(format!("Failed to linke {} into Main", name.name()).as_str());
      }

      let _ = create_main(&context, &builder, &main_module, &compiler.modules());

      // try to include this file
      compiler.include(&source, &main_module);

      

      let int = context.i32_type().const_int(0, false);
      let _ = builder.build_return(Some(&int));


      // for (name, module) in compiler.modules() {
      //   if name.name() == "Core" {
      //     continue
      //   } else {
      //     main_module.link_in_module(module.clone())
      //       .expect(format!("Failed to link {} into Main", name.name()).as_str())
      //   }
      // }
      
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

      let _ = main_module.print_to_file("Main.ll");
    },
    _ => println!("Wtf")
  }
}
