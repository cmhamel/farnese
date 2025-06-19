use crate::{Core, Module};
use core::ffi::c_char;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use libc;
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::Read;
use std::ptr;
use std::sync::Mutex;

static STDOUT_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub struct TestHelper<'a, 'b> {
    pub builder: &'b Builder<'a>,
    pub module: Module<'a>,
}

impl<'a, 'b> TestHelper<'a, 'b> {
    pub fn new(name: &str, builder: &'b Builder<'a>, context: &'a Context) -> Self {
        let core = Core::new(&context).bootstrap();
        let mut module = Module::new(&context, name);
        module.link(&core);
        Self {
            builder: &builder,
            module: module,
        }
    }

    pub fn end(&self) {
        let zero = self.module.i32_type().const_int(0, false);
        let _ = self.builder.build_return(Some(&zero));
    }

    pub fn run(&self) -> String {
        let _lock = STDOUT_LOCK.lock().unwrap();

        let engine = self
            .module
            .module()
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        unsafe {
            // === Redirect stdout ===
            // Save current stdout
            let stdout_fd = libc::dup(libc::STDOUT_FILENO);
            let tmpfile = tempfile::NamedTempFile::new().unwrap();
            let tmpfile_path = tmpfile.path().to_owned();
            let tmp_fd = libc::open(
                tmpfile_path.to_str().unwrap().as_ptr() as *const c_char,
                libc::O_WRONLY,
            );

            // Redirect stdout to tmp file
            libc::dup2(tmp_fd, libc::STDOUT_FILENO);

            // Run JITted `main`
            let addr = engine.get_function_address("main").unwrap();
            let main_fn = std::mem::transmute::<usize, extern "C" fn() -> i32>(addr as usize);
            let _ret_code = main_fn();

            // Flush stdout
            libc::fflush(ptr::null_mut());

            // Restore original stdout
            libc::dup2(stdout_fd, libc::STDOUT_FILENO);
            libc::close(tmp_fd);

            // Read output from temp file
            let mut file = File::open(tmpfile_path).unwrap();
            let mut output = String::new();
            file.read_to_string(&mut output).unwrap();

            // assert_eq!(output.trim(), "Hello, LLVM!");
            output.trim().to_string()
        }
    }

    pub fn start(&self) {
        let context = self.module.get_context();
        let fn_type = context.i32_type().fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let entry = context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
    }
}
