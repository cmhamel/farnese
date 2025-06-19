use farnese_compiler::Compiler;
use farnese_core::Module;
use farnese_lexer::lexer::parse_source;
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use termion::color;

pub struct Repl<'a> {
    compiler: Compiler<'a>,
    editor: DefaultEditor,
    main_module: Module<'a>,
}

pub struct ReplError;

impl<'a> Repl<'a> {
    pub fn new(context: &'a Context) -> Self {
        let mut compiler = Compiler::new(context);
        let editor = DefaultEditor::new().unwrap();
        let mut main_module = Module::new(context, "Main");

        compiler.include(&mut main_module, "examples/base.jl");
        main_module.link(compiler.get_module("Core"));
        // main_module.link(compiler.get_module("Base"));

        Self {
            compiler: compiler,
            editor: editor,
            main_module: main_module,
        }
    }

    fn evaluate<'b>(&mut self, line: &str, n: i32) {
        let asts = parse_source(&line).unwrap();
        let context = self.main_module.get_context();
        let builder = context.create_builder();

        // wrap everything in repl method
        let func_name = format!("repl_{}", n);
        let return_type = context.void_type();
        let func_type = return_type.fn_type(&[], false);
        let func = self
            .main_module
            .add_function(func_name.as_str(), func_type, None);

        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        for ast in asts {
            self.compiler
                .compile_expr(&builder, &mut self.main_module, ast);
        }

        let fflush = self.main_module.get_function("fflush");
        let null_ptr = context.i8_type().ptr_type(0.into()).const_null();
        let _ = builder.build_call(fflush, &[null_ptr.into()], "flush");

        let _ = builder.build_return(None);

        self.main_module.module().print_to_stderr();
        let execution_engine = self
            .main_module
            .module()
            .clone()
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();
        let jit_func =
            unsafe { execution_engine.get_function::<unsafe extern "C" fn()>(func_name.as_str()) }
                .expect(format!("Could not find func {}", func_name).as_str());
        unsafe { jit_func.call() }
    }

    fn loop_func(&mut self) {
        let context = self.main_module.get_context();

        let fflush_type = context
            .i32_type()
            .fn_type(&[context.i8_type().ptr_type(0.into()).into()], false);
        let _ = self.main_module.add_function("fflush", fflush_type, None);

        let mut counter = 1;
        loop {
            let line_read = Self::read(&mut self.editor);
            match line_read {
                Ok(line) => {
                    let result = self.evaluate(&line, counter);
                    self.print(result);
                    counter = counter + 1;
                }
                Err(_) => break,
            }
        }
    }

    fn print(&mut self, _result: ()) {
        // println!("last value = {:?}", self.compiler.scope);
        println!("");
        // match result {
        //     () => println!(""),
        //     _ => println!("Here {:?}", result)
        // }
    }

    fn read(editor: &mut DefaultEditor) -> Result<String, ReplError> {
        let farnese = format!(
            "{}farnese> {}",
            color::Fg(color::Green),
            color::Fg(color::White)
        );
        let line_read = editor.readline(&farnese);
        match line_read {
            Ok(line) => Ok(line),
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                Err(ReplError {})
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                Err(ReplError {})
            }
            Err(err) => {
                println!("Error: {:?}", err);
                Err(ReplError {})
            }
        }
    }

    pub fn start(&mut self) {
        self.loop_func()
    }
}
