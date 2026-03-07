use crate::datatype::{hash_symbol, TypeContext, TypeExpr};
use crate::functions::Function;
use crate::modules::Module;
use farnese_lexer::ast::Expr;
use farnese_lexer::parser::parse_string;
use inkwell::{
    context::Context,
    values::PointerValue
};
use std::fmt::Debug;
use pest::{
    iterators::Pair,
    Parser
};
use std::collections::HashMap;
use std::hash::Hash;

// #[derive(Clone, Copy, Debug)]
struct Compiler<'a> {
    functions: Vec<Function>,
    module: Module<'a>,
    scope: HashMap<u64, PointerValue<'a>>,
    type_context: TypeContext
}

impl<'a> Compiler<'a> {
    pub fn new(module: Module<'a>) -> Self {
        Self {
            functions: Vec::new(),
            module: module,
            scope: HashMap::<u64, PointerValue<'a>>::new(),
            type_context: TypeContext::new()
        }
    }

    pub fn compile_call(&self, name: &str, args: &Vec<Expr>) {
        let name = hash_symbol(name);
        // here's where we need to infer arg types

        // then need to fetch the actual args from e.g.
        // a scope/stack
        let arg_ptrs = args.iter()
            // .map(|x| self.scope.get(&hash_symbol(x)))
            .map(|x| match x {
                Expr::Symbol(s) => self.scope.get(&hash_symbol(s))
                    .expect(format!("Failed to find variable \"{}\" in scope", s).as_str()),
                _ => todo!()
            })
            .collect::<Vec<_>>();

        println!("Args = {:#?}", args);
    } 

    pub fn compile_expr(&self, expr: &Expr) {
        match expr {
            Expr::Call { func, args } => {
                self.compile_call(func, args);
            }
            Expr::Method { func, args, body, .. } => {
                self.compile_method(func, args, body);
            }
            _ => todo!("Expr = {:#?}", expr)
        }
    }

    pub fn compile_from_str(&self, my_code: &str) {
        let exprs_read = parse_string(my_code);
        match exprs_read {
            Ok(exprs) => {
                for expr in exprs {
                    self.compile_expr(&expr)
                }
            }
            Err(e) => {
                eprintln!("{}", e)
            }
        }
        
    }

    pub fn compile_method(&self, name: &str, args: &Vec<Expr>, body: &Vec<Expr>) {
        let name = hash_symbol(name);
        let arg_syms = args.iter()
            .map(|x| match x {
                Expr::Val {ty, val} => val,
                _ => todo!("{:#?}", x)
            })
            .collect::<Vec<_>>();
        let arg_type_ids = args.iter()
            .map(|x| match x {
                Expr::Val {ty, val} => match &**ty {
                    Expr::Type(t) => self.type_context.get_type_index_from_str(t),
                    _ => todo!("{:#?}", ty)
                }
                _ => unreachable!()
            })
            .collect::<Vec<_>>();
        let arg_type_exprs = TypeExpr::Tuple(arg_type_ids.iter()
            .map(|x| self.type_context.get_datatype_type_expr(*x))
            .collect::<Vec<_>>());
        println!("Args = {:#?}", arg_type_exprs);
        
        // need to create ptrs for inputs and add them to scope

        body.iter()
            .map(|x| self.compile_expr(x))
            .collect::<Vec<_>>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use pest::parser::Parser;

    #[test]
    fn test_parse() {
        let my_code = r#"
            function +(a::Int, b::Int)
                a + b
            end
        "#;
        let context = Context::create();
        let module = Module::new(&context, "my_module");
        let compiler = Compiler::new(module);
        compiler.compile_from_str(my_code);
    }
}