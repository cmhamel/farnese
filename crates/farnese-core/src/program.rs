use crate::codegen::CodeGen;
use crate::datatype::DataType;
use crate::traits::*;
use crate::value::Value;
use farnese_parser::*;
use inkwell::values::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum TypedExpr {
    Assignment {
        name: String,
        value: Box<TypedExpr>
    },
    BinaryExpr {
        op: BinOp,
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>
    },
    Call {
        name: String,
        args: Box<Vec<TypedExpr>>
    },
    Identifier {
        name: String,
        datatype: DataType
    },
    Primitive {
        val: Primitive,
        datatype: DataType
    },
    UnaryExpr {
        op: UnaryOp,
        expr: Box<TypedExpr>
    }
}

impl TypedExpr {
    pub fn fmt_str<'ctx>(&self, codegen: &mut CodeGen<'ctx>) -> PointerValue<'ctx> {
        match self.get_datatype().name.as_str() {
            "Float64" => {
                codegen.builder.build_global_string_ptr("%.4f\0", "")
                .unwrap().as_pointer_value()
            },
            "Int64" => {
                codegen.builder.build_global_string_ptr("%ld\0", "")
                .unwrap().as_pointer_value()
            },
            _ => todo!()
        }
    }
    pub fn get_datatype(&self) -> &DataType {
        match self {
            TypedExpr::BinaryExpr { op: _, lhs, rhs } => {
                if lhs.get_datatype() == rhs.get_datatype() {
                    lhs.get_datatype()
                } else {
                    todo!("Need to implement conversion methods!")
                }
            },
            TypedExpr::Identifier { name: _, datatype } => {
                datatype
            }
            TypedExpr::Primitive { val: _, datatype } => {
                datatype
            },
            _ => todo!("get_datatype needs to be implemented for {:?}", self)
        }
    }
}

// impl<'ctx> LLVMAlloca<'ctx> for TypedExpr {
//     fn emit_ir_alloca(&self, codegen: &mut CodeGen<'ctx>, value: &BasicValueEnum<'ctx>) -> PointerValue<'ctx> {
//         // codegen.builder.build_alloca(self.get_type(), "").unwrap()
//         // let value = self.compile(codegen);
//         codegen.builder.build_alloca(value.get_type(), "").unwrap()
//     }
// }

impl<'ctx> Compile<'ctx> for TypedExpr {
    type Output = BasicValueEnum<'ctx>;
    fn compile(&self, codegen: &mut CodeGen<'ctx>) -> Self::Output {
        match self {
            TypedExpr::Assignment { name, value } => {
                let datatype = value.get_datatype();
                let ptr = if codegen.variables.contains_key(name) {
                    let val = codegen.variables.get(name).unwrap();
                    val.value
                } else {
                    let value = value.compile(codegen);
                    let ptr = codegen.builder.build_alloca(value.get_type(), "").unwrap();
                    let val = Value::new(ptr.try_into().unwrap(), datatype.clone());
                    codegen.variables.insert(name.to_string(), val.clone());
                    let _ = codegen.builder.build_store(ptr, value);

                    ptr.try_into().unwrap()
                };
                ptr.into()
            },
            TypedExpr::BinaryExpr { op, lhs, rhs } => {
                let (lhs_val, rhs_val) = if lhs.get_datatype() == rhs.get_datatype() {
                    let lhs_val = lhs.compile(codegen);
                    let rhs_val = rhs.compile(codegen);
                    (lhs_val, rhs_val)
                } else {
                    todo!("Need to implement conversion methods!");
                };
                match op {
                    BinOp::Add => {
                        match (lhs_val, rhs_val) {
                            (BasicValueEnum::IntValue(x), BasicValueEnum::IntValue(y)) => {
                                codegen.builder.build_int_add(x, y, "").unwrap()
                            },
                            _ => todo!("Need to implement for pair ({:?}, {:?})", lhs_val, rhs_val)
                        }
                    },
                    _ => todo!()
                }.try_into().unwrap()
            },
            TypedExpr::Call { name, args } => {
                println!("Args = {:?}", args);
                let fmt_str = args[0].fmt_str(codegen);
                let func = codegen.module.get_function(name).unwrap();
                let args = args.iter()
                    .map(|x| x.compile(codegen))
                    .collect::<Vec<_>>();
                let mut args: Vec<BasicMetadataValueEnum> =
                    args.into_iter().map(Into::into).collect();
                if name == "printf" {
                    args.insert(0, fmt_str.try_into().unwrap());
                } else {
                    todo!()
                }
                let call_out = codegen.builder.build_call(func, &args, "").unwrap()
                    .try_as_basic_value();
                match call_out {
                    ValueKind::Basic(x) => {
                        x.into()    
                    },
                    _ => panic!()
                }
            },
            TypedExpr::Identifier { name, datatype } => {
                let val = codegen.variables.get(name).unwrap();
                let ty = match val.datatype.name.as_str() {
                    "Int64" => codegen.context.i64_type(),
                    _ => todo!("Loading datatype {:?} not supported yet", datatype.name)
                };
                let ptr = codegen.builder.build_load(ty, val.value.try_into().unwrap(), "").unwrap();
                ptr.into()
            },
            TypedExpr::Primitive { val, .. } => {
                val.compile(codegen).into()
            },
            _ => todo!("Need to implement compile for {:?}", self)
        }
    }
}

// impl<'ctx> Compile<'ctx> for Expr {
//     type Output = Option<BasicValueEnum<'ctx>>;
//     // type Output = ();
//     fn compile(&self, codegen: &mut CodeGen<'ctx>) -> Self::Output {
//         match self {
//             Expr::BinaryExpr { op, lhs, rhs} => {
//                 let lhs = lhs.compile(codegen);
//                 let rhs = rhs.compile(codegen);
//                 println!("lhs = {:?}", lhs);
//                 println!("rhs = {:?}", rhs);
//                 panic!()
//             },
//             Expr::Identifier(x) => {
//                 Some(BasicValueEnum::PointerValue(*codegen.variables.get(x).unwrap()))
//             }
//             Expr::Primitive(x) => {
//                 // this one is a weird case where just have a dangling int.
//                 let val = x.compile(codegen);
//                 Some(val)
//             },
//             _ => todo!("Expression {:?} not implemented yet.", self)
//         }
//     }
// }

// impl<'ctx> Compile<'ctx> for Program {
//     type Output = ();
//     fn compile(&self, codegen: &mut CodeGen<'ctx>, typed_exprs: Vec<TypedExpr>) -> Self::Output {
//         for item in &self.items {
//             match item {
//                 Item::Statement(x) => match x {
//                     Statement::Assignment { name, value } => {
//                         let value = value.compile(codegen).unwrap();
//                         let ptr = if codegen.variables.contains_key(name) {
//                             *codegen.variables.get(name).unwrap()
//                         } else {
//                             let ptr = codegen.builder.build_alloca(value.get_type(), "").unwrap();
//                             codegen.variables.insert(name.to_string(), ptr);
//                             ptr
//                         };
//                         let _ = codegen.builder.build_store(ptr, value);
//                     }
//                     Statement::Expr(y) => {
//                         let _ = y.compile(codegen);
//                     },
//                     _ => todo!("Not yet implemented {:?}", x)
//                 },
//                 _ => todo!()
//             }
//         }
//     }
// }

pub struct InferenceHelper {
    variables: HashMap::<String, TypedExpr>
}

impl InferenceHelper {
    pub fn new() -> Self {
        InferenceHelper {
            variables: HashMap::<String, TypedExpr>::new()
        }
    }
}

impl Inference for Expr {
    type Output = TypedExpr;
    fn inference(&self, helper: &mut InferenceHelper) -> Self::Output {
        match self {
            Expr::BinaryExpr { op, lhs, rhs } => {
                let lhs = lhs.inference(helper);
                let rhs = rhs.inference(helper);
                TypedExpr::BinaryExpr { op: *op, lhs: Box::new(lhs), rhs: Box::new(rhs) }
            }
            Expr::Identifier(name) => {
                let datatype = helper.variables.get(name).unwrap().get_datatype();
                TypedExpr::Identifier { name: name.to_string(), datatype: datatype.clone() }
            }
            Expr::Primitive(x) => {
                x.inference(helper)
            },
            Expr::UnaryExpr { op, expr } => {
                let expr = expr.inference(helper);
                TypedExpr::UnaryExpr { op: *op, expr: Box::new(expr) }
            },
            _ => todo!("Need to implement {:?}", self)
        }
    }
}

impl Inference for Program {
    type Output = Vec<TypedExpr>;
    fn inference(&self, helper: &mut InferenceHelper) -> Self::Output {
        let mut typed_exprs = vec![];
        for item in &self.items {
            let typed_expr = match item {
                Item::Statement(x) => match x {
                    Statement::Assignment { name, value }  => {
                        let val = value.inference(&mut *helper);
                        println!("Val = {:?}", val);
                        helper.variables.insert(name.to_string(), val.clone());     
                        TypedExpr::Assignment { name: name.to_string(), value: Box::new(val) }
                    },
                    Statement::Expr(y) => match y {
                        Expr::Call { name, args } => {
                            let args = args.iter()
                                .map(|x| x.inference(helper))
                                .collect::<Vec<_>>();
                            TypedExpr::Call { name: name.to_string(), args: Box::new(args) }
                        },
                        _ => {
                            todo!("Need to implement expr inference {:?}", self)
                        }
                    }
                },
                _ => todo!()
            };
            typed_exprs.push(typed_expr);
        }
        typed_exprs
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::TestHelper;
    use super::*;
    use farnese_derive::*;

    // #[farnese_component_test]
    // fn test_simple_program() {
    //     let p = parse_program("a = 1.0\na = 2.0").unwrap();
    //     // println!("Program = {:?}", p);
    //     p.compile(codegen);
    //     codegen.module.print_to_stderr()
    // }

    #[farnese_component_test]
    fn test_type_inference() {
        let p = parse_program("a = 69\nb = 420\nc = a + b\nprintf(c)").unwrap();
        let mut helper = InferenceHelper::new();
        let infs = p.inference(&mut helper);
        println!("Infs = {:#?}", infs);
        for inf in infs {
            inf.compile(codegen);
        }
        codegen.module.print_to_stderr()
    }
}
