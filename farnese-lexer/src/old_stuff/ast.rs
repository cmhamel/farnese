use farnese_core_v3::primitives::Primitive;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operator {
    // Assignment,
    Divide,
    Equal,
    EqualEqual,
    Minus,
    Multiply,
    Plus,
    SubType,
}

// #[derive(Clone, Debug, PartialEq)]
// pub enum Primitive {
//     Char(char),
//     Float32(f32),
//     Float64(f64),
//     Int16(i16),
//     Int32(i32),
//     Int64(i64),
//     String(String),
//     UInt16(u16),
//     UInt32(u32),
//     UInt64(u64),
// }

pub struct BinaryExpr {
    pub lhs: Primitive,
    pub op: Operator,
    pub rhs: Primitive
}

// impl From<char> for Primitive {
//     fn from(value: char) -> Self {
//         Primitive::Char(value)
//     }
// }

// impl From<f32> for Primitive {
//     fn from(value: f32) -> Self {
//         Primitive::Float32(value)
//     }
// }

// impl From<f64> for Primitive {
//     fn from(value: f64) -> Self {
//         Primitive::Float64(value)
//     }
// }

// impl From<i16> for Primitive {
//     fn from(value: i16) -> Self {
//         Primitive::Int16(value)
//     }
// }

// impl From<i32> for Primitive {
//     fn from(value: i32) -> Self {
//         Primitive::Int32(value)
//     }
// }

// impl From<i64> for Primitive {
//     fn from(value: i64) -> Self {
//         Primitive::Int64(value)
//     }
// }

// note this is not the Symbol in core
// but has the same name since it will be
// converted to core::Symbol downstream
pub type Symbol = String;

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    AbstractType {
        name: Symbol,
        // params: Box<Node>,
        supertype: Symbol,
    },
    AssignmentExpr {
        identifier: Symbol,
        value: Box<Node>,
    },
    BinaryExpr {
        op: Operator,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    ConstExpr {
        expr: Box<Node>,
    },
    Empty,
    Exports {
        symbols: Box<Vec<Node>>,
    },
    Function {
        name: Symbol,
        // arg_names: Vec<Symbol>,
        args: Box<Vec<Node>>,
        return_type: Symbol,
        body: Box<Vec<Node>>,
    },
    FunctionArg {
        name: Symbol,
        // arg_type: Box<Node>
        arg_type: Symbol,
    },
    FunctionArgs {
        names: Vec<Symbol>,
        types: Vec<Symbol>,
    },
    IfExpr {
        condition: Box<Node>,
        if_block: Box<Vec<Node>>,
        // TODO elseif
        else_block: Box<Vec<Node>>,
    },
    Macro {
        name: Symbol,
        args: Box<Vec<Node>>,
        body: Box<Vec<Node>>,
    },
    MethodCall {
        name: Symbol,
        args: Box<Vec<Node>>,
    },
    Module {
        name: Symbol,
        exprs: Box<Vec<Node>>,
    },
    Operator(Operator),
    ParenthesesExpr {
        expr: Box<Node>,
    },
    Primitive(Primitive),
    PrimitiveType {
        name: Symbol,
        supertype: Symbol,
        bits: u32,
    },
    ReturnExpr {
        name: Symbol
    },
    StructField {
        name: Symbol,
        field_type: Symbol,
    },
    StructType {
        name: Symbol,
        // generics: Vec<Symbol>,
        supertype: Symbol,
        field_names: Vec<Symbol>,
        field_types: Vec<Symbol>,
    },
    SuperType(Symbol),
    Symbol(Symbol),
    UnaryExpr {
        op: Operator,
        child: Box<Node>,
    },
}
