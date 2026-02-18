use crate::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn hash_symbol(sym: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    sym.hash(&mut hasher);
    let hash: u64 = hasher.finish() as u64;
    hash
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Symbol {
    hash: u64,
    pub name: String
}

impl Symbol {
    pub fn new(string: &str) -> Self {
        Self {
            hash: hash_symbol(string),
            name: string.to_string()
        }
    }

    pub fn len(&self) -> usize {
        self.name.len()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<'a, 'b> Alloca<'a, 'b> for Symbol {}

impl<'a, 'b> LLVMType<'a, 'b> for Symbol {
    fn llvm_type(&self, _builder: &'b Builder<'a>, module: &Module<'a>) -> BasicTypeEnum<'a> {
        module.get_struct_type("Symbol")
            .unwrap().try_into().unwrap()
    }
}

impl<'a, 'b> LLVMValue<'a, 'b> for Symbol {
    fn llvm_value(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> BasicValueEnum<'a> {
        let context = module.get_context();
        let datatype = module.get_struct_type("Symbol").unwrap();
        let prim_val = context.i64_type().const_int(self.hash, false);
        datatype.const_named_struct(&[
            prim_val.into(),
            self.name().llvm_value(builder, module).into()
        ]).try_into().unwrap()
    }
}

impl<'a, 'b> OpaqueType<'a, 'b> for Symbol {
    fn set_opaque_type_init(_builder: &'b Builder<'a>, module: &Module<'a>) {
        let context = module.get_context();
        let opaque_type = module.get_struct_type("Symbol")
            .expect("Failed to load Symbol opaque type");
        opaque_type.set_body(
            &[
                context.i64_type().into(),
                context.ptr_type(AddressSpace::default()).into()
            ],
            false
        );
    }
}

impl<'a, 'b> Printf<'a, 'b> for Symbol {
    fn printf(
        &self, 
        builder: &'b Builder<'a>, 
        module: &Module<'a>,
        ptr: PointerValue<'a>
    ) {
        let context = module.get_context();
        let datatype = module.get_struct_type("Symbol").unwrap();
        let gep_ptr = builder.build_struct_gep(datatype, ptr, 1, "").unwrap();
        let str_ptr = builder.build_load(
            context.ptr_type(AddressSpace::default()),
            gep_ptr,
            ""
        ).unwrap();
        let str_ptr = match str_ptr {
            BasicValueEnum::PointerValue(x) => x,
            _ => panic!("Shouldn't ever happen")
        };
        self.name().printf(builder, module, str_ptr)
    }
}

impl<'a, 'b> Store<'a, 'b> for Symbol {}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    use super::*;

    #[test]
    fn test_symbol() {
        let context = Context::create();
        let test = TestHelper::new(&context, "test_symbol");

        test.start();

        let string = "Some long string";
        let sym = Symbol::new(string);
        let ptr = sym.alloca(&test.builder, &test.module);
        let _ = sym.store(&test.builder, &test.module, ptr);
        let _ = sym.printf(&test.builder, &test.module, ptr);

        test.end();
        let results = test.run();
        assert_eq!("Some long string", results);
    }
}