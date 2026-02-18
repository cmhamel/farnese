use std::collections::HashMap;

// primitives
type Int = i64;
type Float64 = f64;
type Symbol = String;
type Type = Symbol;

#[derive(Clone, Debug)]
pub enum BinaryOperator {
    Add,
    Divide,
    Modulo,
    Multiply,
    Power,
    Subtract
}

#[derive(Clone, Debug)]
pub enum UnaryOperator {
    Minus,
    Plus
}

#[derive(Clone, Debug)]
pub enum Expr {
    AbstractType {
        name: Symbol,
        supertype: Box<Expr>
    },
    Assignment {
        symbol: Box<Expr>,
        expr: Box<Expr>
    },
    Call {
        func: Symbol,
        args: Box<Vec<Expr>>
    },
    Function {
        func: Symbol,
        methods: Box<HashMap<Vec<Expr>, (Expr, Expr)>>
    },
    Method {
        func: Symbol,
        args: Box<Vec<Expr>>,
        body: Box<Vec<Expr>>,
        ret: Box<Expr>
    },
    PrimitiveType {
        name: Symbol,
        supertype: Box<Expr>,
        bits: i32
    },
    // Type {
    //     name: Symbol
    // },
    Type(Type),
    Val {
        ty: Box<Expr>,
        val: Box<Expr>
    },
    // primitives below
    Float64(Float64),
    Int(Int),
    String(String),
    Symbol(Symbol),
    //
    Bottom,
    // for comments, EOI, etc.
    Empty
}
