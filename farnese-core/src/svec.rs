use super::{
    DataType,
    FarneseInternal,
    LLVMGlobal,
    Module,
    Symbol
};
use inkwell::AddressSpace;
use inkwell::values::GlobalValue;


#[derive(Clone, Debug)]
pub struct SimpleVector {
    values: Vec<Symbol>
}

impl<'a> SimpleVector {
    pub fn new(values: Vec<Symbol>) -> Self {
        Self {
            values: values
        }
    }

    pub fn from_strs(values: Vec<String>) -> Self {
        let values = values
            .iter()
            .map(|x| Symbol::new(x))
            .collect::<Vec<_>>();
        Self {
            values: values
        }
    }

    pub fn values(&self) -> &Vec<Symbol> {
        &self.values
    }
}

impl<'a, 'b> FarneseInternal<'a> for SimpleVector {
    fn create_datatype() -> DataType {
        let sym = Symbol::new("SimpleVector");
        let field_names = SimpleVector::new(vec![]);
        let field_parameters = SimpleVector::new(vec![]);
        let field_types = SimpleVector::new(vec![]);
        DataType::new(
            sym,
            Symbol::new("Any"),
            false,
            false,
            false,
            field_names,
            field_parameters,
            field_types
        )
    }

    fn create_new_method(_module: &Module<'a>) {

    }

    fn create_opaque_type(module: &Module<'a>) {
        let context = module.get_context();
        let _ = context.opaque_struct_type("SimpleVector");
    }

    fn set_opaque_type_initializer(module: &Module<'a>) {
        let context = module.get_context();
        let svec_opaque_type = context.opaque_struct_type("SimpleVector");
        // let datatype_type = module.get_struct_type("DataType");
        svec_opaque_type.set_body(
            &[
                context.i64_type().into(),
                // datatype_type.ptr_type(AddressSpace::default())
                //     .ptr_type(AddressSpace::default())
                context.ptr_type(AddressSpace::default())//.ptr_type(AddressSpace::default())
                    .into()
            ],
            false
        );
    }
}

impl<'a> LLVMGlobal<'a> for SimpleVector {
    fn emit_ir_global(&self, module: &Module<'a>) -> GlobalValue<'a> {
        let context = module.get_context();
        // let datatype_type = module.get_struct_type("DataType");
        let svec_type = module.get_struct_type("SimpleVector");
        let field_type_ptrs = self.values()
            .iter()
            .map(|x| module.get_global(x.name()).as_pointer_value())
            .collect::<Vec<_>>();
        let field_type_ptrs = module.get_context()
            .ptr_type(AddressSpace::default())
            .const_array(&field_type_ptrs);
        let svec_glob = module.add_global(svec_type, None, "");

        let svec_values = svec_type.const_named_struct(&[
            context.i64_type().const_int(
                self.values().len().try_into().unwrap(), 
                false
            ).into(),
            field_type_ptrs.into()
        ]);
        svec_glob.set_initializer(&svec_values);
        svec_glob
    }
}