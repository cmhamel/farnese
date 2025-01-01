use crate::base::Symbol;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DataType {
  pub name: Arc<Symbol>,
  pub fields: Option<Vec<Field>>,
  pub parameters: Option<Vec<Arc<DataType>>>,
  pub supertype: Option<Arc<DataType>>,
  pub is_abstract: bool,
  pub is_mutable: bool,
  pub is_primitive: bool
}

impl DataType {
  pub fn name(&self) -> &str {
    &self.name.name()
  }

  pub fn new_abstract_type(
    name: Arc<Symbol>, 
    parameters: Option<Vec<Arc<DataType>>>,
    supertype: Option<Arc<DataType>> 
  ) -> Self {
    Self {
      name: name,
      fields: Some(Vec::<Field>::new()),
      parameters: parameters,
      supertype: supertype,
      is_abstract: true,
      is_mutable: false,
      is_primitive: false
    }
  }

  pub fn new_primitive_type(
    name: Arc<Symbol>,
    supertype: Option<Arc<DataType>>,
    bits: i32 
  ) -> Self {
    Self {
      name: name,
      fields: Some(Vec::<Field>::new()),
      parameters: Some(Vec::<Arc<DataType>>::new()),
      supertype: supertype,
      is_abstract: false,
      is_mutable: false,
      is_primitive: true
    }
  }
//   pub fn new(name: Arc<Symbol>, super_type: Arc<Symbol>, types: Vec<Arc<Symbol>>) -> Self {
//     Self {
//       name: name,
//       super_type: super_type,
//       types: types
//     }
//   }
}

impl Display for DataType {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let _ = match self.is_primitive {
      true => write!(f, "{}", self.name.name()),
      false => Ok(())
    };

    match self.is_abstract {
      true => {
        write!(f, "{}", self.name.name())
      }
      false => Ok(())
    }
    // write!(f, "{}{{", self.name.name())
    // match self.parameters.len() {
    //   1 => {
    //     write!(f, "{}", self.parameters[0].name())?
    //   },
    //   0_usize.. => {
    //     let args: Vec<_> = self.parameters
    //       .iter()
    //       .take(self.parameters.len() - 1)
    //       .map(|x| write!(f, "{},", x.name()))
    //       .collect();
    //     write!(f, "{}", self.parameters[self.parameters.len() - 1].name())?
    //   }
    // }
    // write!(f, "}} <: {}", self.supertype.name())
  }
}

#[derive(Clone, Debug)]
pub struct Field {
  name: Arc<Symbol>,
  field_type: Arc<DataType>
}