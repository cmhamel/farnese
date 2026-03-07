#[cfg(test)]
use crate::modules::Module;
#[cfg(test)]
use core::ffi::c_char;
#[cfg(test)]
use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    OptimizationLevel
};
#[cfg(test)]
use std::{
    fs::File,
    io::Read,
    ptr
};

#[cfg(test)]
pub struct TestHelper<'a> {
    pub builder: Builder<'a>,
    pub module: Module<'a>
}

#[cfg(test)]
impl<'a> TestHelper<'a> {
    pub fn new(context: &'a Context, name: &str) -> Self {
        let ptr_type = context.ptr_type(AddressSpace::default());
        let builder = context.create_builder();
        // let module = context.create_module(name);
        let module = Module::new(context, name);
        
        // TODO eventually move a lot of below
        // to the module type once we figured out
        // what we need and where it should be
        let printf_type = context.i32_type()
            .fn_type(&[ptr_type.into()], true);
        module.add_function("printf", printf_type, None);

        let ptr_type = context.ptr_type(AddressSpace::default());
        let memcpy_ty = context.void_type().fn_type(
            &[
                ptr_type.into(), // dest
                ptr_type.into(), // src
                context.i64_type().into(),    // size
                context.bool_type().into(), // is_volatile
            ],
            false,
        );
        // TODO figure out what the characters mean
        module.add_function("llvm.memcpy.p0.p0.i64", memcpy_ty, None);

        // Symbol::create_opaque_type(&builder, &module, "Symbol");
        // Symbol::set_opaque_type_init(&builder, &module);

        // DataType::create_opaque_type(&builder, &module, "DataType");
        // DataType::set_opaque_type_init(&builder, &module);

        // Value::create_opaque_type(&builder, &module, "Value");
        // Value::set_opaque_type_init(&builder, &module);


        Self {
            builder: builder,
            module: module
        }
    }

    pub fn start(&self) {
        let context = self.module.get_context();
        let fn_type = context.i32_type().fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let entry = context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
    }

    pub fn end(&self) {
        let zero = self.module.get_context().i32_type().const_int(0, false);
        let _ = self.builder.build_return(Some(&zero));
    }

    pub fn run(&self) -> String {
        let engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .expect("Failed to create execution engine");
        let string = unsafe {
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
        };
        string
    }

    pub fn show_ir(&self) {
        println!("Current IR:");
        self.module.print_to_stderr();
        println!("End of current IR\n\n\n")
    }
}
