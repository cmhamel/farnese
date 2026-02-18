use crate::datatype::{ANY_ID, hash_symbol, Symbol, TypeContext, TypeExpr};
use std::cmp::Ordering;
use std::collections::HashMap;

// // to test A.signature <: B.signature
// fn type_is_more_specific(ctx: &TypeContext, a: &TypeExpr, b: &TypeExpr) -> bool {
//     ctx.is_subtype(a, b) && !ctx.is_subtype(b, a)
// }

enum BodyExpr {
    Assignment(Symbol, Box<BodyExpr>),
    Call(Symbol, TypeExpr),
    Literal(TypeExpr)
}

#[derive(Debug)]
pub struct Method {
    // pub body: BodyExpr, // TODO
    // pub return_type: Option<TypeId>,
    pub signature: TypeExpr
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

    fn infer_return_type(&self) -> () {
        
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

struct InferenceState<'a> {
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

mod tests {
    use super::*;

    #[test]
    fn test_method_specificity() {
        let mut ctx = TypeContext::new();
        ctx.add_primitive_type("Int", vec![], ANY_ID, 32);
        let t1 = TypeExpr::Tuple(vec![TypeExpr::Concrete(0), TypeExpr::Concrete(0)]);
        let t2 = TypeExpr::Tuple(vec![TypeExpr::Concrete(2), TypeExpr::Concrete(2)]);
        let m1 = Method { signature: t1.clone() };
        let m2 = Method { signature: t2.clone() };
        let f1 = Function::new("add", vec![m1, m2]);
        // let m3 = f1.lookup_method(&ctx, &t2);
        println!("M1 = {:#?}", f1.lookup_method(&ctx, &t1));
        println!("M2 = {:#?}", f1.lookup_method(&ctx, &t2));
    }

    #[test]
    fn test_simple_add() {
        let mut ctx = TypeContext::new();
        ctx.add_primitive_type("Int", vec![], ANY_ID, 32);
        let t1 = TypeExpr::Tuple(vec![TypeExpr::Concrete(2), TypeExpr::Concrete(2)]);
        let f1 = Function::new("add", vec![Method { signature: t1.clone() }]);
        println!("Function = {:#?}", f1);
    }
}