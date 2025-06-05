use super::{DataType, Symbol};

#[derive(Clone, Debug)]
pub struct Any<T> {
    val: T
}

impl<T> Any<T> {
    pub fn new(val: T) -> Self {
        Self {
            val: val
        }
    }

    pub fn any_type() -> DataType {
        let sym = Symbol::new("Any");
        let super_sym = sym.clone();
        let field_types = Box::new(Vec::<DataType>::new());
        DataType::new(sym, super_sym, false, true, false, field_types)
    }
}
