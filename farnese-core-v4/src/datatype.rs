use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

pub type Symbol = u64;

// TODO can we pre-allocate the default hasher?
pub fn hash_symbol(sym: &str) -> Symbol {
    let mut hasher = DefaultHasher::new();
    sym.hash(&mut hasher);
    hasher.finish()
}

pub type TypeId = usize;
pub type TypeVarId = usize;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeExpr {
    Apply(TypeId, Vec<TypeExpr>),        // e.g. Vector{Int, Float64}
    Concrete(TypeId),                    // e.g. Int64
    Tuple(Vec<TypeExpr>),
    Union(Vec<TypeExpr>),                // e.g. Union{Int, Float64}
    Var(TypeVarId),                      // e.g. T
    // VarArg(TypeExpr)
}

impl TypeExpr {
    pub fn from_datatype(datatype: &DataType) -> Self {
        match datatype.parameters.len() {
            0 => TypeExpr::Concrete(datatype.id),
            _ => {
                let param_ids = datatype.parameters.iter()
                    .enumerate()
                    .map(|(n, _)| TypeExpr::Var(n))
                    .collect();
                TypeExpr::Apply(datatype.id, param_ids)
            }
        }
    }

    pub fn get_type_id(&self) -> TypeId {
        match &self {
            TypeExpr::Apply(id, _) => *id,
            TypeExpr::Concrete(id) => *id,
            _ => panic!("Not supported for type {:#?}", &self)
        }
    }

    pub fn substitute_parameters(&self, args: &[TypeExpr]) -> Self {
        match &self {
            TypeExpr::Apply(f, inner_args) => {
                TypeExpr::Apply(
                    *f,
                    inner_args.iter()
                        .map(|t| t.substitute_parameters(args))
                        .collect()
                )
            }
            TypeExpr::Concrete(id) => {
                TypeExpr::Concrete(*id)
            }
            TypeExpr::Tuple(types) => {
                TypeExpr::Tuple(
                    types.iter()
                        .map(|t| t.substitute_parameters(args))
                        .collect()
                )
            }
            TypeExpr::Union(types) => {
                TypeExpr::Union(
                    types.iter()
                        .map(|t| t.substitute_parameters(args))
                        .collect()
                )
            }
            TypeExpr::Var(i) => {
                args[*i].clone()
            }
        }
    }
}

#[derive(Debug)]
pub enum TypeKind {
    Abstract,
    Primitive { 
        size_in_bits: u32 
    },
    Struct { 
        is_mutable: bool 
    }
}

#[derive(Clone, Debug)]
pub struct TypeVar {
    pub name: Symbol,
    pub lower_bound: TypeExpr,
    pub upper_bound: TypeExpr
}

impl TypeVar {
    pub fn new(name: Symbol) -> Self {
        Self {
            name: name,
            lower_bound: TypeExpr::Union(vec![]),
            upper_bound: TypeExpr::Concrete(0)
        }
    }

    pub fn new_with_upper_bound(name: Symbol, upper_bound: TypeExpr) -> Self {
        Self {
            name: name,
            lower_bound: TypeExpr::Union(vec![]),
            upper_bound: upper_bound
        }
    }
}

#[derive(Debug)]
pub struct DataType {
    pub id: TypeId,
    pub name: Symbol,
    pub parameters: Vec<TypeVar>,
    pub supertype: Option<TypeExpr>,
    pub kind: TypeKind,
    pub field_names: Vec<Symbol>,
    pub field_types: Vec<TypeExpr>
}

// TODO maybe eventually make the hash_to_str only a debug
// option
pub struct TypeContext {
    pub hash_to_str: HashMap<Symbol, String>,
    pub name_to_type: HashMap<String, usize>,
    pub types: Vec<DataType>
}

pub const ANY_ID: TypeExpr = TypeExpr::Concrete(0);
pub const TUPLE_ID: TypeExpr = TypeExpr::Concrete(1);

impl TypeContext {
    pub fn new() -> Self {
        let mut context = TypeContext {
            hash_to_str: HashMap::<Symbol, String>::new(),
            name_to_type: HashMap::<String, usize>::new(),
            types: vec![]
        };
        // creating any type as a special standalone guy
        let any_type = DataType {
            id: 0,
            name: hash_symbol("Any"),
            parameters: vec![],
            supertype: None,
            kind: TypeKind::Abstract,
            field_names: vec![],
            field_types: vec![]
        };
        context.types.push(any_type);
        // also creating special case tuple type for Tuple{Args...} <: Tuple case in type system
        context.add_abstract_type("Tuple", vec![], ANY_ID);

        // for now we're going to register the minimal type system as well

        // basic abstract types
        context.add_abstract_type("Number", vec![], ANY_ID);
        context.add_abstract_type("Real", vec![], TypeExpr::Concrete(2));
        context.add_abstract_type("Integer", vec![], TypeExpr::Concrete(3));
        context.add_abstract_type("Signed", vec![], TypeExpr::Concrete(4));

        // basic primitive types
        context.add_primitive_type("Int", vec![], TypeExpr::Concrete(5), 32);

        context
    }

    pub fn add_abstract_type(
        &mut self, 
        name: &str, parameters: Vec<TypeVar>, supertype: TypeExpr
    ) {
        self.add_type(name, parameters, supertype, TypeKind::Abstract, vec![], vec![]);
    }

    pub fn add_primitive_type(
        &mut self, 
        name: &str, parameters: Vec<TypeVar>, supertype: TypeExpr,
        size_in_bits: u32
    ) {
        self.add_type(name, parameters, supertype, TypeKind::Primitive { size_in_bits }, vec![], vec![]);
    }

    pub fn add_struct_type(
        &mut self, 
        name: &str, parameters: Vec<TypeVar>, supertype: TypeExpr,
        field_names: Vec<String>, field_types: Vec<TypeExpr>,
        is_mutable: bool
    ) {
        self.add_type(name, parameters, supertype, TypeKind::Struct { is_mutable }, field_names, field_types);
    }

    pub fn add_type(
        &mut self, 
        name: &str, parameters: Vec<TypeVar>, supertype: TypeExpr,
        kind: TypeKind,
        field_names: Vec<String>, field_types: Vec<TypeExpr>
    ) {
        let name_hash = hash_symbol(name);
        let field_name_hashes = field_names.iter()
            .map(|x| hash_symbol(x))
            .collect();
        let datatype = DataType {
            id: self.types.len(),
            name: name_hash,
            parameters: parameters,
            supertype: Some(supertype),
            kind: kind,
            field_names: field_name_hashes,
            field_types: field_types
        };
        // TODO make this optional below
        self.add_string_hash(name);
        self.add_string_hashes(&field_names);
        // TODO make this optional above
        self.types.push(datatype);
        self.name_to_type.insert(name.to_string(), self.types.len() - 1);
    }

    pub fn add_string_hash(&mut self, string: &str) {
        self.hash_to_str.insert(hash_symbol(string), string.to_string());
    }

    pub fn add_string_hashes(&mut self, string: &Vec<String>) {
        string.iter()
            .for_each(|x| self.add_string_hash(&*x));
    }

    pub fn get_datatype_type_expr(&self, id: TypeId) -> TypeExpr {
        let datatype = &self.types[id];
        match datatype.parameters.len() {
            0 => TypeExpr::Concrete(id),
            _ => panic!("Implement parameteric stuff")
        }
    }

    pub fn get_supertype_from_str(&self, name: &str) -> &DataType {
        let datatype = self.get_type_from_str(name);
        match &datatype.supertype {
            Some(x) => &self.types[x.get_type_id()],
            None => &self.types[ANY_ID.get_type_id()]
        }
    }

    pub fn get_type_from_str(&self, name: &str) -> &DataType {
        let type_index = self.get_type_index_from_str(name);
        &self.types[type_index]
    }

    pub fn get_type_id_from_str(&self, name: &str) -> u64 {
        self.get_type_index_from_str(name) as u64
    }

    pub fn get_type_index_from_str(&self, name: &str) -> usize {
        let index = self.name_to_type.get(name);
        match index {
            Some(x) => *x,
            None => panic!("Type {} not found in TypeContext", name)
        }
    }

    pub fn is_concrete_subtype(&self, mut a: TypeId, b: TypeId) -> bool {
        loop {
            if a == b {
                return true;
            }
    
            match &self.types[a].supertype {
                Some(super_id) => a = super_id.get_type_id(),
                None => return false,
            }
        }
    }

    pub fn is_subtype(&self, a: &TypeExpr, b: &TypeExpr) -> bool {
        if a == b {
            return true;
        }

        match (a, b) {
            // TODO this one still needs work
            (TypeExpr::Apply(f1, args1), TypeExpr::Apply(f2, args2)) => {
                // check if they are concrete subtypes from symbol
                let name_check = self.is_concrete_subtype(*f1, *f2);
                // now check arg signatures are compatables
                let arg_check = args1.iter()
                    .zip(args2.iter())
                    .all(|(x, y)| self.is_subtype(x, y));
                arg_check && name_check
            } 
            // Concrete types
            (TypeExpr::Concrete(a_id), TypeExpr::Concrete(b_id)) => {
                self.is_concrete_subtype(*a_id, *b_id)
            }
            (TypeExpr::Tuple(_), TypeExpr::Concrete(id)) => {
                return match *id {
                    val if val == ANY_ID.get_type_id() => true,
                    val if val == TUPLE_ID.get_type_id() => true,
                    _ => false
                };
            }
            (TypeExpr::Tuple(a_types), TypeExpr::Tuple(b_types)) => {
                if a_types.len() != b_types.len() {
                    return false;
                }
            
                for (ai, bi) in a_types.iter().zip(b_types.iter()) {
                    if !self.is_subtype(ai, bi) {
                        return false;
                    }
                }
            
                true
            }
            // TODO
            // Union on RHS
            (_, TypeExpr::Union(types)) => {
                types.iter().any(|t| self.is_subtype(a, t))
            }
            // Union on LHS
            (TypeExpr::Union(types), _) => {
                types.iter().all(|t| self.is_subtype(t, b))
            }
            _ => panic!("Not implemented a = {:#?} b = {:#?}", a, b)
        }
    }
    
    pub fn instantiate_supertype(&self, expr: &TypeExpr, args: &[TypeExpr]) -> TypeExpr {
        let datatype = &self.types[expr.get_type_id()];
        let supertype_id = match &datatype.supertype {
            Some(x) => x.get_type_id(),
            None => ANY_ID.get_type_id()
        };
        let supertype = TypeExpr::from_datatype(&self.types[supertype_id]);
        supertype.substitute_parameters(args);
        supertype
    }

    pub fn join(&self, a: &TypeExpr, b: &TypeExpr) -> TypeExpr {
        if self.is_subtype(a, b) {
            return b.clone();
        }
    
        if self.is_subtype(b, a) {
            return a.clone();
        }
    
        match (a, b) {
            (TypeExpr::Union(a_types), TypeExpr::Union(b_types)) => {
                let mut all = a_types.clone();
                all.extend(b_types.clone());
                TypeExpr::Union(all)
            }
    
            (TypeExpr::Union(a_types), _) => {
                let mut all = a_types.clone();
                all.push(b.clone());
                TypeExpr::Union(all)
            }
    
            (_, TypeExpr::Union(b_types)) => {
                let mut all = b_types.clone();
                all.push(a.clone());
                TypeExpr::Union(all)
            }
    
            _ => {
                TypeExpr::Union(vec![a.clone(), b.clone()])
            }
        }
    }    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_type_system() {
        let mut types = TypeContext::new();
        // types.add_abstract_type("Number", vec![], ANY_ID);
        // types.add_abstract_type("Real", vec![], TypeExpr::Concrete(2));
        // types.add_abstract_type("Integer", vec![], TypeExpr::Concrete(3));
        // types.add_abstract_type("Signed", vec![], TypeExpr::Concrete(4));
        // types.add_primitive_type("Int", vec![], TypeExpr::Concrete(5), 32);
        types.add_struct_type("Nothing", vec![], ANY_ID, vec![], vec![], false);
        types.add_struct_type(
            "Pair", vec![], ANY_ID, 
            vec!["a".to_string(), "b".to_string()], 
            vec![TypeExpr::Concrete(4), TypeExpr::Concrete(4)],
            false
        );
        assert_eq!(types.is_subtype(
            &TypeExpr::Concrete(types.get_type_id_from_str("Pair").try_into().unwrap()),
            &ANY_ID
        ), true)
    }

    #[test]
    fn test_add_abstract_type_with_parameters() {
        let mut types = TypeContext::new();
        types.add_abstract_type("MyType", vec![TypeVar::new(hash_symbol("T"))], TypeExpr::Concrete(0));
        // println!("My abstract type with params = {:#?}", types.get_type_from_str("MyType"));
        types.add_abstract_type("MyType2", vec![TypeVar::new(hash_symbol("T"))], TypeExpr::Concrete(1));
        // println!("My abstract type with params = {:#?}", types.get_type_from_str("MyType2"));

        let type_expr1 = TypeExpr::from_datatype(types.get_type_from_str("MyType"));
        // println!("Any type expr = {:#?}", TypeExpr::from_datatype(types.get_type_from_str("Any")));
        // println!("Type expr = {:#?}", type_expr1);
        // println!("Supertype = {:#?}", types.instantiate_supertype(
        //     &TypeExpr::from_datatype(types.get_type_from_str("MyType2")), 
        //     &[TypeExpr::Var(0)])
        // )
        // assert_eq!(types.is_subtype(
        //     &TypeExpr::Apply(
        //         types.get_type_id_from_str("MyType2").try_into().unwrap(),
        //         vec![TypeExpr::Var(0)]
        //     ),
        //     &TypeExpr::Apply(
        //         types.get_type_id_from_str("MyType").try_into().unwrap(),
        //         vec![TypeExpr::Var(0)]
        //     )
        // ), true)
    }

    #[test]
    fn test_tuples() {
        let mut types = TypeContext::new();
        // types.add_abstract_type("Number", vec![], ANY_ID);
        // types.add_abstract_type("Real", vec![], TypeExpr::Concrete(2));
        // types.add_abstract_type("Integer", vec![], TypeExpr::Concrete(3));
        // types.add_abstract_type("Signed", vec![], TypeExpr::Concrete(4));
        // types.add_primitive_type("Int", vec![], TypeExpr::Concrete(5), 32);
        let a = TypeExpr::Tuple(vec![TypeExpr::Concrete(types.get_type_id_from_str("Int").try_into().unwrap())]);
        let b = TypeExpr::Tuple(vec![TypeExpr::Concrete(types.get_type_id_from_str("Real").try_into().unwrap())]);
        assert_eq!(types.is_subtype(&a, &ANY_ID), true);
        assert_eq!(types.is_subtype(&a, &TUPLE_ID), true);
        assert_eq!(types.is_subtype(&a, &b), true);
    }

    #[test]
    fn test_union_subtyping() {
        let mut types = TypeContext::new();
        // types.add_abstract_type("Number", vec![], ANY_ID);
        // types.add_abstract_type("Real", vec![], TypeExpr::Concrete(2));
        let a = TypeExpr::Union(vec![TypeExpr::Concrete(2), TypeExpr::Concrete(3)]);
        let b = TypeExpr::Union(vec![ANY_ID, ANY_ID]);
        assert_eq!(types.is_subtype(&a, &b), true);
    }
}
