use super::{
    DataType, 
    FarneseInternal, 
    LLVMAlloca,
    MethodHelper,
    Module, 
    Primitive,
    SimpleVector,
    StructHelper,
    Symbol
};
use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::values::PointerValue;

#[derive(Clone, Debug)]
pub enum AnyType {
    // DataType(DataType),
    Primitive(Primitive),
    // Symbol(Symbol)
}

impl From<Primitive> for AnyType {
    fn from(val: Primitive) -> Self {
        AnyType::Primitive(val)
    }
}

impl<'a, 'b> AnyType {
    fn emit_ir_bitcast_to_i64_ptr(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a> {
        let context = module.get_context();
        let i64_ptr_type = context.ptr_type(AddressSpace::default());
        match self {
            AnyType::Primitive(x) => {
                match x {
                    Primitive::Float32(_) |
                    Primitive::Float64(_) |
                    Primitive::Int16(_) | 
                    Primitive::Int32(_) | 
                    Primitive::Int64(_) => {
                        let alloca = x.emit_ir_alloca(&builder, module);
                        builder.build_bit_cast(alloca, i64_ptr_type, "")
                    },  
                    _ => todo!()
                }
            }
        }.unwrap().into_pointer_value()
    }
}

#[derive(Clone, Debug)]
pub struct Any {
    datatype: DataType,
    value: AnyType
}

impl Any {
    pub fn new(datatype: DataType, value: AnyType) -> Self {
        Self {  
            datatype: datatype,
            value: value
        }
    }
}

impl<'a> FarneseInternal<'a> for Any {
    fn create_datatype() -> DataType {
        DataType::new(
            Symbol::new("Any"),
            Symbol::new("Any"),
            false,
            false,
            false,
            SimpleVector::new(vec![]),
            SimpleVector::new(vec![]),
            SimpleVector::new(vec![])
        )
    }
    
    fn create_new_method(module: &Module<'a>) {
        let any_type = module.get_struct_type("Any");
        // let datatype = module.get_struct_type("DataType");
        let context = module.get_context();
        let builder = context.create_builder();
        let i64_ptr_type = context.ptr_type(AddressSpace::default());
        let func = module.get_context().ptr_type(AddressSpace::default()).fn_type(&[
            i64_ptr_type.into(),
            context.ptr_type(AddressSpace::default()).into()
        ], false);
        let func = module.add_function("Any_new", func, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        let val_ptr = func.get_nth_method_input(0);
        let datatype_ptr = func.get_nth_method_input(1);
        let struct_ptr = builder.build_alloca(any_type, "").unwrap();
        let _ = struct_ptr.set_nth_field(&builder, 0, val_ptr);
        let _ = struct_ptr.set_nth_field(&builder, 1, datatype_ptr);
        let _ = builder.build_return(Some(&struct_ptr));
    }

    fn create_opaque_type(module: &Module<'a>) {
        let context = module.get_context();
        let _ = context.opaque_struct_type("Any");
    }

    fn set_opaque_type_initializer(module: &Module<'a>) {
        let context = module.get_context();
        let opaque_type = module.get_struct_type("Any");
        opaque_type.set_body(
            &[
                context.ptr_type(AddressSpace::default()).into(),
                context.ptr_type(AddressSpace::default()).into()
            ],
            false
        );
    }
}

impl<'a, 'b> LLVMAlloca<'a, 'b> for Any {
    fn emit_ir_alloca(&self, builder: &'b Builder<'a>, module: &Module<'a>) -> PointerValue<'a> {
        let val_ptr = self.value
            .emit_ir_bitcast_to_i64_ptr(builder, module);
        // let datatype_ptr = self.datatype
        //     .emit_ir_alloca(builder, module);
        let datatype_ptr = module
            .get_global(self.datatype.name().name())
            .as_pointer_value();
        let result = builder
            .build_call(module.get_function("Any"), &[
                val_ptr.try_into().unwrap(), 
                // val_ptr.into(),
                datatype_ptr.try_into().unwrap()
            ], "")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap();
        let result_alloca = builder.build_alloca(result.get_type(), "").unwrap();
        builder.build_store(result_alloca, result).unwrap();
        result_alloca
    }
}