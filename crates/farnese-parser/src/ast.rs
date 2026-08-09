#[derive(Debug, Clone)]
pub enum Expr {
    BinaryExpr {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Identifier(String),
    Primitive(Primitive),
    UnaryExpr {
        op: UnaryOp,
        expr: Box<Expr>,
    }
}

#[derive(Clone, Debug)]
pub enum Primitive {
    Float64(f64),
    Int(i64),
    String(String),
    Symbol(String)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, // -x
    Not, // !x
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
}

/// A statement is either a variable assignment (`x = expr`) or a bare
/// expression evaluated for its value/side effects.
#[derive(Debug, Clone)]
pub enum Statement {
    Assignment { name: String, value: Expr },
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(FunctionDef),
    Statement(Statement),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}
