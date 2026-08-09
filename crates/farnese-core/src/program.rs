use crate::codegen::CodeGen;
use crate::traits::*;
use farnese_parser::*;
// use farnese_parser::ast::*;
use inkwell::values::BasicValueEnum;

impl<'ctx> Compile<'ctx> for Expr {
    type Output = Option<BasicValueEnum<'ctx>>;
    fn compile(&self, codegen: &mut CodeGen<'ctx>) -> Self::Output {
        match self {
            // Expr::BinaryExpr { op, lhs, rhs} => {
            //     let lhs = lhs.compile(codegen);
            //     let rhs = rhs.compile(codegen);
            // },
            Expr::Primitive(x) => {
                // this one is a weird case where just have a dangling int.
                let val = x.compile(codegen);
                Some(val)
            },
            _ => todo!("Expression {:?} not implemented yet.", self)
        }
    }
}

impl<'ctx> Compile<'ctx> for Program {
    type Output = ();
    fn compile(&self, codegen: &mut CodeGen<'ctx>) -> Self::Output {
        for item in &self.items {
            match item {
                Item::Statement(x) => match x {
                    Statement::Assignment { name, value } => {
                        let value = value.compile(codegen).unwrap();
                        let ptr = if codegen.variables.contains_key(name) {
                            *codegen.variables.get(name).unwrap()
                        } else {
                            let ptr = codegen.builder.build_alloca(value.get_type(), "").unwrap();
                            codegen.variables.insert(name.to_string(), ptr);
                            ptr
                        };
                        let _ = codegen.builder.build_store(ptr, value);
                    }
                    Statement::Expr(y) => {
                        let _ = y.compile(codegen);
                    },
                    _ => todo!("Not yet implemented {:?}", x)
                },
                _ => todo!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::TestHelper;
    use super::*;
    use farnese_derive::*;

    #[farnese_component_test]
    fn test_simple_program() {
        let p = parse_program("a = 1.0\na = 2.0").unwrap();
        println!("Program = {:?}", p);
        p.compile(codegen);
        codegen.module.print_to_stderr()
    }
}