use clap::{Parser, Subcommand};
use farnese_compiler::Compiler;
use farnese_core::Module;
use farnese_lexer::lexer;
#[cfg(feature = "repl")]
use farnese_repl::Repl;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::passes::{PassManager, PassManagerSubType};
use inkwell::targets::{
    CodeModel, 
    FileType, 
    InitializationConfig,
    RelocMode, 
    Target,
    TargetMachine
};
use std::path::Path;

#[derive(Clone, Debug, Parser)]
#[command(arg_required_else_help = true, version)]
struct CLIArgs {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Debug, Subcommand)]
enum Commands {
    #[command(about = "ast")]
    Ast {
        #[arg(long, short, value_name = "FARNESE FILE")]
        input: String,
    },
    #[command(about = "Compiler")]
    Compiler {
        #[arg(long, short, value_name = "FARNESE FILE")]
        input: String,
        #[arg(long, short, value_name = "LLVM ASSEMBLY FILE")]
        assembly_file: Option<String>,
        #[arg(long, short, value_name = "LLVM BITCODE FILE")]
        bitcode_file: Option<String>,
        #[arg(long, short, value_name = "LLVM IR FILE")]
        llvm_ir_file: Option<String>,
        #[arg(long, short, value_name = "LLVM OBJECT FILE")]
        object_file: Option<String>,
        #[arg(long, value_name = "OPTIMIZE")]
        optimization_level: Option<i32>,
    },
    #[cfg(feature = "repl")]
    #[command(about = "Repl")]
    Repl {},
}

fn setup_pass_manager<T: PassManagerSubType<Input = ()>>() -> PassManager<T> {
    let pm = inkwell::passes::PassManager::create(());
    pm.add_instruction_combining_pass();
    pm.add_reassociate_pass();
    pm.add_gvn_pass();
    pm.add_cfg_simplification_pass();
    pm.add_basic_alias_analysis_pass();
    pm.add_promote_memory_to_register_pass();
    pm.add_dead_store_elimination_pass();
    pm.add_scalar_repl_aggregates_pass();

    // pm.add_function_inlining_pass();
    // pm.add_aggressive_dce_pass();
    pm
}

fn setup_target_machine(optimization_level: OptimizationLevel) -> TargetMachine {
    Target::initialize_all(&InitializationConfig::default());

    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();

    target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            // inkwell::OptimizationLevel::Default,
            optimization_level,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or("Unable to create target machine")
        .unwrap()
}

fn main() {
    let args = CLIArgs::parse();
    match args.command {
        Some(Commands::Ast { input }) => {
            let asts: Vec<_> = lexer::parse_file(&input).unwrap();
            println!("Dumping ASTs\n\n");
            for ast in asts.iter() {
                println!("{:?}", ast);
            }
        }
        Some(Commands::Compiler {
            input,
            assembly_file,
            bitcode_file,
            llvm_ir_file,
            object_file,
            optimization_level
        }) => {
            let context = Context::create();

            let mut compiler = Compiler::new(&context);
            // let mut base_module = Module::new(&context, "Base");
            // compiler.insert_module("Base", base_module.clone());
            // compiler.include(&mut base_module, "src/base/base.jl");
            let mut main_module = Module::new(&context, "Main");
            compiler.insert_module("Main", main_module.clone());
            compiler.include(&mut main_module, &input);

            // match optimization_level
            let optimization_level = match optimization_level {
                Some(x) => {
                    match x {
                        0 => OptimizationLevel::None,
                        1 => OptimizationLevel::Less,
                        2 => OptimizationLevel::Default,
                        3 => OptimizationLevel::Aggressive,
                        _ => panic!("Non-existent optimization level should be 0, 1, 2, or 3")
                    }
                },
                None => OptimizationLevel::None
            };

            // target
            let pass_manager = setup_pass_manager();
            let target_machine = setup_target_machine(optimization_level);

            match assembly_file {
                Some(x) => {
                    target_machine.write_to_file(&main_module.module(), FileType::Assembly, x.as_ref())
                        .unwrap();
                },
                _ => {}
            }

            match bitcode_file {
                Some(x) => {
                    pass_manager.run_on(main_module.module());
                    main_module.module().write_bitcode_to_path(&x)
                },
                _ => false
            };

            match llvm_ir_file {
                Some(x) => {
                    pass_manager.run_on(main_module.module());
                    main_module.print_to_file(&x)
                },
                _ => {}
            }

            match object_file {
                Some(x) => {
                    for (name, module) in compiler.modules() {
                        let file_name_str = format!("{}.o", name.name());
                        let file_name = Path::new(&file_name_str);
                        target_machine.write_to_file(&module.module(), FileType::Object, file_name)
                            .unwrap();
                    }   
                    target_machine.write_to_file(&main_module.module(), FileType::Object, x.as_ref())
                        .unwrap();
                },
                _ => {}
            }
        }
        #[cfg(feature = "repl")]
        Some(Commands::Repl {}) => {
            let context = Context::create();
            let mut repl = Repl::new(&context);
            repl.start();
        }
        _ => println!("Wtf"),
    }
}
