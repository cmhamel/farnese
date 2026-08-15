#[derive(Clone, Debug, PartialEq)]
pub struct DataType {
    bits: i32,
    is_abstract: bool,
    is_mutable: bool,
    is_primitive: bool,
    pub name: String,
    // supertype: Box<DataType>
}

impl DataType {
    pub fn new_any() -> Self {
        Self {
            bits: 16,
            is_abstract: false,
            is_mutable: false,
            is_primitive: false,
            name: "Any".to_string()
        }
    }

    pub fn new_primitive(name: &str, bits: i32) -> Self {
        Self {
            bits: bits,
            is_abstract: false,
            is_mutable: false,
            is_primitive: true,
            name: name.to_string()
        }
    }
}

// impl fmt::Display for DataType {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // Use the write! macro to send formatted data to the buffer
//         write!(f, "{}", self.name)
//     }
// }
