use farnese_core::{
    Core, DataType, LLVMAlloca, LLVMPrintf, LLVMValue, MethodHelper, Module, Primitive, Symbol,
};
use farnese_lexer::ast::{Node, Operator};
use farnese_lexer::lexer;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::BasicMetadataValueEnum;
use std::collections::HashMap;

/// a table of modules
type Modules<'a> = HashMap<Symbol, Module<'a>>;
/// scope
type Scope<'a> = HashMap<Symbol, (Value<'a>, DataType)>;
/// a stack of LLVM values
type Stack<'a> = Vec<(Value<'a>, DataType)>;
/// basic value type
type Value<'a> = BasicMetadataValueEnum<'a>;

pub struct Compiler<'a> {
    modules: Modules<'a>,
    pub scope: Scope<'a>,
    pub stack: Stack<'a>,
}

impl<'a> Compiler<'a> {
    pub fn new(context: &'a Context) -> Self {
        let mut modules = Modules::<'a>::new();
        let scope = Scope::<'a>::new();
        let stack = Stack::<'a>::new();

        // setup core
        let mut core = Core::new(&context);
        let core_module = core.bootstrap();
        let _ = core_module.print_to_file("Core.ll");
        modules.insert(Symbol::new("Core"), core_module);

        Self {
            modules: modules,
            scope: scope,
            stack: stack,
        }
    }

    fn compile_binary_expr<'b>(
        &mut self,
        builder: &'b Builder<'a>,
        module: &mut Module<'a>,
        op: Operator,
        lhs: Node,
        rhs: Node,
    ) {
        self.compile_expr(builder, module, lhs);
        self.compile_expr(builder, module, rhs);
        let (rhs, _rhs_type) = self.stack.pop().unwrap();
        let (lhs, lhs_type) = self.stack.pop().unwrap();

        // let result: BasicValueEnum<'a> = match lhs {
        let result: (Value<'a>, DataType) = match lhs {
            Value::IntValue(x) => match rhs {
                Value::IntValue(y) => match op {
                    Operator::Minus => (
                        Value::IntValue(builder.build_int_sub(x, y, "").unwrap()),
                        lhs_type,
                    ),
                    Operator::Plus => (
                        Value::IntValue(builder.build_int_add(x, y, "").unwrap()),
                        lhs_type,
                    ),
                    _ => todo!("Unsupported op {:?}", op),
                },
                _ => todo!("Types don't match"),
            },
            _ => todo!("Not supported yet"),
        };
        self.stack.push(result);
    }

    pub fn compile_expr<'b>(
        &mut self,
        builder: &'b Builder<'a>,
        module: &mut Module<'a>,
        expr: Node,
    ) {
        match expr {
            Node::AbstractType { name, supertype } => {
                let datatype = DataType::new_abstract_type(&name, &supertype);
                module.insert_type(datatype);
            }
            Node::AssignmentExpr { identifier, value } => {
                self.compile_expr(&builder, module, *value);
                let prev_val_ptr = self.stack.pop().unwrap();

                self.scope.insert(Symbol::new(&identifier), prev_val_ptr);
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                self.compile_binary_expr(&builder, module, op, *lhs, *rhs)
            }
            Node::Empty => {
                // do nothing
            }
            Node::Exports { symbols } => {
                for export in symbols.iter() {
                    // module.push_export(export);
                    match export {
                        Node::Symbol(x) => module.push_export(Symbol::new(x)),
                        _ => todo!(),
                    }
                }
            }
            Node::Function {
                name,
                args,
                return_type,
                body,
            } => self.compile_function(module, &name, &args, &return_type, &body),
            Node::MethodCall { name, args } => {
                self.compile_method_call(&builder, module, &name, args)
            }
            Node::ParenthesesExpr { expr } => {
                self.compile_expr(&builder, module, *expr);
            }
            Node::Primitive(x) => {
                let x: Primitive = x.into();
                // let val = x.emit_ir_value(module);
                let val = match x {
                    Primitive::String(_) => x.emit_ir_alloca(&builder, module).into(),
                    _ => x.emit_ir_value(module),
                };
                let datatype = x.get_datatype();
                self.stack.push((val.into(), datatype));
            }
            Node::PrimitiveType {
                name,
                supertype,
                bits,
            } => {
                let datatype = DataType::new_primitive_type(&name, &supertype, bits);
                module.insert_type(datatype);
            }
            Node::StructType {
                name,
                supertype,
                field_names,
                field_types,
            } => {
                // hack for now TODO refactor this
                let field_names = field_names
                    .iter()
                    .map(|x| Symbol::new(x))
                    .collect::<Vec<_>>();
                let field_types = Box::new(
                    field_types
                        .iter()
                        .map(|x| module.get_type(x).clone())
                        .collect(),
                );
                let datatype = DataType::new(
                    Symbol::new(&name),
                    Symbol::new(&supertype),
                    false,
                    false,
                    false,
                    field_names,
                    field_types,
                );
                module.insert_type(datatype);
            }
            Node::Symbol(x) => {
                // let val = self
                //     .scope
                //     .get(&Symbol::new(&x))
                //     .expect(format!("Symbol not found: {}", x).as_str());
                // self.stack.push(val.clone());
                let x = Symbol::new(&x);
                let val = x.emit_ir_alloca(&builder, module);
                let datatype = self.modules.get(&Symbol::new("Core"))
                    .unwrap()
                    .get_type("Symbol");
                self.stack.push((val.into(), datatype.clone()));
            }
            _ => todo!("Unsupported type {:?}", expr),
        }
    }

    fn compile_function<'b>(
        &mut self,
        module: &mut Module<'a>,
        name: &str,
        args: &Vec<Node>,
        return_type: &str,
        body: &Box<Vec<Node>>,
    ) {
        let context = module.get_context();
        let builder = context.create_builder();

        // handle main as a special case
        if name == "main" {
            let return_type = context.i32_type();
            let func = return_type.fn_type(&[], false);
            let func = module.add_function(&name, func, None);
            let entry = context.append_basic_block(func, "entry");
            builder.position_at_end(entry);
            for expr in (*body).iter() {
                self.compile_expr(&builder, module, expr.clone())
            }
            let return_val = return_type.const_int(0, false);
            let _ = builder.build_return(Some(&return_val));
            return;
        }

        let mut arg_names = Vec::<Symbol>::new();
        let mut arg_types = Vec::<DataType>::new();
        let _ = args
            .iter()
            .map(|arg| match arg {
                Node::FunctionArg { name, arg_type } => {
                    arg_names.push(Symbol::new(name));
                    arg_types.push(module.get_type(arg_type).clone());
                }
                _ => panic!("Shouldn't happen"),
            })
            .collect::<Vec<_>>();
        let method_name = name.to_owned()
            + "_"
            + &arg_types
                .iter()
                .map(|x| x.name().name())
                .collect::<Vec<_>>()
                .join("_");

        // TODO need to figure out how to infer return type
        let return_type = module.get_type(&return_type);

        // setup field types and return type
        let arg_types: Vec<_> = arg_types
            .iter()
            .map(|x| x.get_ir_value_type(module).try_into().unwrap())
            .collect();

        // TODO infer type based on last IR value
        let return_type = return_type.get_ir_value_type(module);
        let func = match return_type {
            BasicMetadataTypeEnum::FloatType(x) => x.fn_type(&arg_types, false),
            BasicMetadataTypeEnum::IntType(x) => x.fn_type(&arg_types, false),
            _ => todo!("Need to support blah..."),
        };
        // let func = module.add_function(&name, func, None);
        let func = module.add_function(&method_name, func, None);
        let entry = context.append_basic_block(func, "entry");
        builder.position_at_end(entry);

        // need to first load up arguments and store in scope
        // TODO need to get actual DataType
        let arg_datatype = DataType::new_primitive_type("Int64", "Integer", 64);

        let _ = arg_names
            .clone()
            .into_iter()
            .enumerate()
            .map(|(n, name)| {
                self.scope.insert(
                    name,
                    (
                        func.get_nth_method_input(n.try_into().unwrap()),
                        arg_datatype.clone(),
                    ),
                )
            })
            .collect::<Vec<_>>();

        for expr in (*body).iter() {
            self.compile_expr(&builder, module, expr.clone())
        }

        // using last value on stack as return value
        let result = self.stack.pop().unwrap();
        let _ = match result.0 {
            Value::FloatValue(x) => builder.build_return(Some(&x)),
            Value::IntValue(x) => builder.build_return(Some(&x)),
            _ => todo!("Not supported return type yet"),
        };

        // pop input argument values
        let _ = arg_names
            .iter()
            .map(|name| self.scope.remove(name))
            .collect::<Vec<_>>();
    }

    fn compile_method_call<'b>(
        &mut self,
        builder: &'b Builder<'a>,
        module: &mut Module<'a>,
        name: &str,
        args: Box<Vec<Node>>,
    ) {
        // get arguments values from scope
        let arg_vals = args
            .iter()
            .map(|x| match &*x {
                // TODO currently not using the arg type here
                Node::FunctionArg { name, arg_type: _ } => Symbol::new(&name),
                _ => todo!("Not supported"),
            })
            .map(|x| self.scope.get(&x).unwrap().clone())
            // .map(|x| x.0) // only need first val
            .collect::<Vec<_>>();

        // handle printf specially for now.. eventually use a trait
        let func_result = if name == "printf" {
            assert!(arg_vals.len() == 1, "printf needs to have one input");
            arg_vals[0]
                .emit_ir_printf(builder, module)
                .try_as_basic_value()
                .unwrap_left()
        // }
        } else {
            let arg_vals = arg_vals.iter().map(|x| x.0).collect::<Vec<_>>();
            builder
                .build_call(
                    module.get_function(&name),
                    &arg_vals,
                    format!("__call__{}", name).as_str(),
                )
                .unwrap()
                .try_as_basic_value()
                .unwrap_left()
        };
        // TODO big hack for now
        // let func_result_type = context.i64_type();
        let func_result_type = DataType::new_primitive_type("Int64", "Signed", 64);

        // TODO need the return DataType so we can allocate on the stack
        // self.stack.push(func_result.into());
        self.stack.push((func_result.into(), func_result_type));
    }

    fn compile_module(&mut self, name: Symbol, exprs: Box<Vec<Node>>, context: &'a Context) {
        let mut module = Module::new(context, name.name());
        module.link(self.get_module("Core"));
        let builder = context.create_builder();
        for ast in exprs.iter() {
            self.compile_expr(&builder, &mut module, ast.clone());
        }
        self.modules.insert(name.clone(), module.clone());
        let _ = module.print_to_file(format!("{}.ll", name.name()).as_str());
    }

    pub fn get_module(&self, name: &str) -> &Module<'a> {
        self.modules.get(&Symbol::new(name)).unwrap()
    }

    // method mainly used to include core...
    pub fn insert_module(&mut self, name: &str, module: Module<'a>) {
        self.modules.insert(Symbol::new(name), module);
    }

    pub fn include(&mut self, module: &mut Module<'a>, file_name: &str) {
        let ast: Vec<_> = lexer::parse_file(file_name).unwrap();
        let context = module.get_context();
        let builder = context.create_builder();
        for node in ast {
            match node {
                Node::Module { name, exprs } => {
                    self.compile_module(Symbol::new(&name), exprs, &context)
                }
                _ => self.compile_expr(&builder, module, node),
            }
        }
    }

    pub fn modules(&self) -> &HashMap<Symbol, Module<'a>> {
        &self.modules
    }
}
