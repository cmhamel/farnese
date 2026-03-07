use crate::datatype::{hash_symbol, Symbol, TypeContext, TypeExpr};
use crate::primitives::Primitive;
use std::cmp::Ordering;
use std::collections::HashMap;

// // to test A.signature <: B.signature
// fn type_is_more_specific(ctx: &TypeContext, a: &TypeExpr, b: &TypeExpr) -> bool {
//     ctx.is_subtype(a, b) && !ctx.is_subtype(b, a)
// }

#[derive(Debug)]
pub enum BodyExpr {
    Assignment(Symbol, Box<BodyExpr>),
    Call(Symbol, Vec<BodyExpr>),
    Primitive(Primitive)
}

impl BodyExpr {
    pub fn infer(&self, state: &InferenceState) -> () {
        match &self {
            BodyExpr::Assignment(sym, expr) => {
                // let rhs = expr.infer(state);
                // state.locals.insert(*sym, rhs);

                // should return type of rhs
            }
            BodyExpr::Call(sym, args) => {
                // should return return type of call

                // first infer the actual types of the args
                // let arg_types: Vec<_> = args
                //     .iter()
                //     .map(|a| a.infer(state))
                //     .collect();
                // let tuple_ty = TypeExpr::Tuple(arg_types);

                // now need to check if such a method actually exists
                // if let Some(method) = self.lookup_method(name, &tuple_ty) {
                //     self.infer_return_type(method, &arg_types)
                // } else {
                //     TypeExpr::Concrete(ANY_ID)
                // }
            }
            BodyExpr::Primitive(val) => {
                // should return type of primitive
                // here's where things getting annoying
                // we have to have primitives already registered
                // in the type context before this can even work
                // otherwise we'll always return Any
            }
            // _ => ANY_ID
        }
    }
}

#[derive(Debug)]
pub struct Method {
    pub body: Vec<BodyExpr>, // TODO
    // pub return_type: Option<TypeId>,
    pub signature: TypeExpr
}

impl Method {
    pub fn new(signature: TypeExpr, body: Vec<BodyExpr>) -> Self {
        Self {
            body: body,
            signature: signature
        }
    } 

    pub fn infer_return_type(&self, ctx: &TypeContext) {
        let mut state = InferenceState::new(ctx);
        for expr in &self.body {
            // match expr {
            //     BodyExpr::Assignment(sym, expr) => {
            //         // let expr = self.infer_return_type()
            //     }
            //     BodyExpr::Call(sym, args) => {

            //     }
            //     BodyExpr::Primitive(val) => {

            //     }
            // }
            expr.infer(&state)
        }
    }
}

fn compare_specificity(
    ctx: &TypeContext,
    m1: &Method,
    m2: &Method,
) -> Ordering {
    let s1_sub_s2 = ctx.is_subtype(&m1.signature, &m2.signature);
    let s2_sub_s1 = ctx.is_subtype(&m2.signature, &m1.signature);

    match (s1_sub_s2, s2_sub_s1) {
        (true, false) => Ordering::Less,      // m1 more specific
        (false, true) => Ordering::Greater,   // m2 more specific
        (true, true) => Ordering::Equal,      // identical or ambiguous
        (false, false) => Ordering::Equal,    // incomparable (ambiguous)
    }
}

#[derive(Debug)]
pub struct Function {
    pub methods: Vec<Method>,
    pub name: Symbol
}

impl Function {
    pub fn new(name: &str, methods: Vec<Method>) -> Self {
        Self {
            methods: methods,
            name: hash_symbol(name)
        }
    }

    fn lookup_method(
        &self,
        ctx: &TypeContext,
        arg_tuple: &TypeExpr,
    ) -> Option<&Method> {
        // TODO ensure that TypeExpr is actually a Tuple
        let mut applicable: Vec<&Method> = self.methods
            .iter()
            .filter(|m| ctx.is_subtype(arg_tuple, &m.signature))
            .collect();
        if applicable.is_empty() {
            return None;
        }
    
        applicable.sort_by(|m1, m2| compare_specificity(ctx, m1, m2));
    
        let best = applicable[0];
    
        // check ambiguity with second best
        if applicable.len() > 1 {
            let second = applicable[1];
    
            let best_sub_second =
                ctx.is_subtype(&best.signature, &second.signature);
            let second_sub_best =
                ctx.is_subtype(&second.signature, &best.signature);
    
            if !best_sub_second || second_sub_best {
                panic!("Method ambiguity detected");
            }
        }
    
        Some(best)
    }
}


pub struct InferenceState<'a> {
    pub type_context: &'a TypeContext,
    pub locals: HashMap<Symbol, TypeExpr>
}

impl<'a> InferenceState<'a> {
    pub fn new(type_context: &'a TypeContext) -> Self {
        Self {
            type_context: type_context,
            locals: HashMap::<Symbol, TypeExpr>::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_specificity() {
        let ctx = TypeContext::new();
        // ctx.add_primitive_type("Int", vec![], ANY_ID, 32);
        let t1 = TypeExpr::Tuple(vec![TypeExpr::Concrete(0), TypeExpr::Concrete(0)]);
        let t2 = TypeExpr::Tuple(vec![TypeExpr::Concrete(2), TypeExpr::Concrete(2)]);
        let m1 = Method::new(t1.clone(), vec![]);
        let m2 = Method::new(t2.clone(), vec![]);
        let f1 = Function::new("add", vec![m1, m2]);
        println!("M1 = {:#?}", f1.lookup_method(&ctx, &t1));
        println!("M2 = {:#?}", f1.lookup_method(&ctx, &t2));
    }

    #[test]
    fn test_simple_add() {
        let ctx = TypeContext::new();
        // ctx.add_primitive_type("Int", vec![], ANY_ID, 32);
        let t1 = TypeExpr::Tuple(vec![TypeExpr::Concrete(2), TypeExpr::Concrete(2)]);
        // let f1 = Function::new(
        //     "add", vec![
        //         Method::new(t1.clone(), vec![BodyExpr::Call(hash_symbol("__addi32plusi32"), t1.clone())])
        // ]);
        // // println!("Function = {:#?}", f1);
        // f1.methods[0].infer_return_type(&ctx);
    }
}
