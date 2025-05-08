use crate::core::{DataType, Symbol};
use crate::lexer::ast::{Node, Operator, Primitive};
use crate::lexer::lexer;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, PointerValue};
use inkwell::AddressSpace;
use std::collections::HashMap;

pub struct Compiler<'a, 'b> {
  builder: &'b Builder<'a>,
  context: &'a Context,
  modules: HashMap<Symbol, Module<'a>>,
  pub scope_variables: HashMap<Symbol, PointerValue<'a>>,
  value_stack: Vec<PointerValue<'a>>
}

impl<'a, 'b> Compiler<'a, 'b> {
  pub fn new(context: &'a Context, builder: &'b Builder<'a>) -> Self {
    let modules = HashMap::<Symbol, Module<'a>>::new();
    let scope_variables = HashMap::<Symbol, PointerValue<'a>>::new();
    let value_stack = Vec::<PointerValue<'a>>::new();
    Self {
      builder: builder,
      context: context,
      modules: modules,
      scope_variables: scope_variables,
      value_stack: value_stack,
    }
  }

  pub fn include(&mut self, file_name: &str, module: &'b Module<'a>) -> () {
    let ast: Vec<_> = lexer::parse(file_name).unwrap();
    for node in ast {
      match node {
        Node::Module { name, exprs } => {
          self.compile_module(name, exprs)
        },
        _ => self.compile_expr(node, module)
      }
    }

    // println!("Value stack = {:?}", self.value_stack);
    // println!("Scope variables = {:?}", self.scope_variables);
  }

  // fn any_type_to_basic_type(&self, in_type: AnyTypeEnum<'a>) -> BasicTypeEnum<'a> {
  //   match in_type {
  //     AnyTypeEnum::FloatType(x) => BasicTypeEnum::FloatType(x),
  //     AnyTypeEnum::IntType(x) => BasicTypeEnum::IntType(x),
  //     _ => todo!("Not a basic type {:?}", in_type)
  //   }
  // }

  fn compiler_binary_expr(
    &mut self, op: Operator, 
    lhs: Node, rhs: Node,
    module: &Module<'a>
  ) -> () {
    let i64_type = self.context.i64_type();
    // println!("lhs top = {:?}", lhs);
    self.compile_expr(lhs.clone(), module);
    self.compile_expr(rhs, module);
    let rhs_ptr = self.value_stack.pop().unwrap();
    let lhs_ptr = self.value_stack.pop().unwrap();
    // println!("lhs = mid = {:?}", lhs);
    // let val = self.compiler_binary_expr(op, lhs, rhs);
    let lhs_val = self.builder.build_load(lhs_ptr, "__load").unwrap();
    let rhs_val = self.builder.build_load(rhs_ptr, "__load").unwrap();

    // println!("rhs_val = {:?}", rhs_val);

    let result = if lhs_val.get_type() == rhs_val.get_type() {
      // println!("lhs type = {:?}", lhs.get_type());
      match op {
        // Operator::Minus => {
          
        // },
        Operator::Plus => {
          match lhs_val {
            // BasicValueEnum::FloatValue(x) => println!("float value = {:?}", x),
            BasicValueEnum::IntValue(x) => {
              self.builder.build_int_add(x, rhs_val.into_int_value(), "addtmp")
            },
            _ => todo!("wtf {:?}", lhs_val)
          }
        },
        _ => todo!("op not yet implement {:?}", op)
      }
    } else {
      panic!("Different type binary op not supported yet {:?} {:?}", op, lhs_val)
    };

    // alloc and store result
    let result_ptr = self.builder.build_alloca(i64_type, "__binary_res").unwrap();
    let _ = self.builder.build_store(result_ptr, result.unwrap());

    self.value_stack.push(result_ptr);
  }

  fn compile_unary_expr(
    &mut self, op: Operator,
    child: Node,
    module: &Module<'a>
  ) -> () {
    let i64_type = self.context.i64_type();

    self.compile_expr(child, module);
    let child_ptr = self.value_stack.pop().unwrap();
    let child_val = self.builder.build_load(child_ptr, "__load").unwrap();
    let result: BasicValueEnum = match op {
      Operator::Minus => {
        match child_val {
          BasicValueEnum::IntValue(x) => {
            self.builder.build_int_neg(x, "__neg").unwrap().into()
          },
          _ => todo!("Value type no implemented yet")
        }
      },
      Operator::Plus => {
        child_val
      },
      _ => panic!("This op {:?} should never be called as unary", op)
    };

    let result_ptr = self.builder.build_alloca(i64_type, "__unary_res").unwrap();
    let _ = self.builder.build_store(result_ptr, result);

    self.value_stack.push(result_ptr);
  }

  fn compile_expr(&mut self, expr: Node, module: &Module<'a>) -> () {
    // let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
    let f64_type = self.context.f64_type();
    let i64_type = self.context.i64_type();
    // let f64_ptr_type = f64_type.ptr_type(AddressSpace::default());
    // let i64_ptr_type = i64_type.ptr_type(AddressSpace::default());

    match expr {
      Node::AbstractType { name, params: _, supertype } => {
        let _ = DataType::new(name, supertype, module);
        // datatype.create_new_type(module);
        // datatype.create_new_val(module);
      },
      Node::AssignmentExpr { identifier, value } => {
        self.compile_expr(*value, module);
        let prev_val_ptr = self.value_stack.pop().unwrap();
        // let prev_val = self.builder.build_load(prev_val_ptr, "__load__assignment");
        self.scope_variables.insert(identifier, prev_val_ptr);
      },
      Node::BinaryExpr { op, lhs, rhs } => {
        self.compiler_binary_expr(op, *lhs, *rhs, module)
      },
      Node::Comment => {},
      Node::Eoi => {},
      Node::MethodCall { name, args } => {
        // lots of things to do here
        // need to check if methods exist
        // pass by reference vs. pass by value
        // ensure method exists in vtable (which we haven't even thought of yet)

        let mut syms = Vec::<Symbol>::new();
        for arg in args {
          match *arg {
            Node::Symbol(x) => syms.push(x),
            _ => panic!("not supported currently")
          }
        }
        let args: Vec<_> = syms
          .iter()
          .map(|x| {
            let var = self.scope_variables.get(x).unwrap();
            self.builder.build_load(*var, "load_method_arg").unwrap()
          })
          .collect();
        let mut converted_args: Vec<BasicMetadataValueEnum> = args
          .into_iter()
          .map(|ptr| ptr.into())  // Convert each PointerValue to BasicMetadataValueEnum
          .collect();
        
        // special case just to help us boot strap things
        // maybe move this to core somehow
        // todo
        if name.name() == "printf" {
          println!("Got printf");
          // adding a format string in front
          let format_string = self.context.const_string(b"%d\n\0", false);
          let ptr = self.builder.build_alloca(format_string.get_type(), "__str").unwrap();
          let _ = self.builder.build_store(ptr, format_string);
          // do I need the one below TODO?
          let _ptr_i8 = self.builder.build_alloca(
            self.context.i8_type().ptr_type(AddressSpace::default()), 
            "__str_i8_cast"
          ).unwrap();
          let gep_ptr = unsafe { self.builder.build_in_bounds_gep(ptr, &[
            self.context.i32_type().const_zero(),
            self.context.i32_type().const_zero(),
          ], "__gep_ptr").unwrap() };
          converted_args.insert(0, gep_ptr.into());
        }

        // let func = module.get_function(name.name()).unwrap();
        let _ = self.builder.build_call(
          module.get_function(name.name()).unwrap(),
          &converted_args,
          format!("__call__{}", name.name()).as_str()
        );
      },
      Node::ParenthesesExpr { expr } => {
        self.compile_expr(*expr, module);
      }
      Node::Primitive(x) => {
        match x {
          Primitive::Char(_y) => {
            panic!("Need to implement char")
          },
          Primitive::Float(y) => {
            let val = f64_type.const_float(y.try_into().unwrap());
            let ptr = self.builder.build_alloca(f64_type, "__float64_ptr").unwrap();
            let _ = self.builder.build_store(ptr, val);
            self.value_stack.push(ptr);
            // let ptr_i8 = self.builder.build_alloca(i8_ptr_type, "__float64_i8_cast_ptr").unwrap();
            // let casted_ptr = self.builder.build_bit_cast(ptr_i8, i8_ptr_type, "__casted_ptr").unwrap();
            // let _ = self.builder.build_store(ptr_i8, casted_ptr);
          },
          Primitive::Int(y) => {
            let val = i64_type.const_int(y.try_into().unwrap(), true);
            let ptr = self.builder.build_alloca(i64_type, "__int_ptr").unwrap();
            let _ = self.builder.build_store(ptr, val);
            self.value_stack.push(ptr);
            // let ptr_i8 = self.builder.build_alloca(i8_ptr_type, "__int_i8_cast_ptr").unwrap();
            // let casted_ptr = self.builder.build_bit_cast(ptr_i8, i8_ptr_type, "__casted_ptr").unwrap();
            // let _ = self.builder.build_store(ptr_i8, casted_ptr);
          }
        }
      },
      Node::PrimitiveType { name, supertype, bits: _ } => {
        let _ = DataType::new(name, supertype, module);
      },
      Node::StructType { name, generics: _, supertype, fields: _ } => {
        let _ = DataType::new(name, supertype, module);
      },
      Node::Symbol(x) => {
        let val = self.scope_variables.get(&x)
          .expect("Symbol not found");
        self.value_stack.push(*val);
      },
      Node::UnaryExpr { op, child } => {
        self.compile_unary_expr(op, *child, module);
      },
      _ => todo!("Unsupported type {:?}", expr)
    }
  }

  fn compile_module(&mut self, name: Symbol, exprs: Vec<Box<Node>>) -> () {
    // println!("Ast = {:?}", exprs)
    let module = self.context.create_module(name.name());
    // link in core
    // module.link_in_module(core.module())
    module.link_in_module(self.modules.get(&Symbol::new("Core")).unwrap().clone())
      .expect("Failed to link Core into Main");
    for ast in exprs {
      // println!("ast = {:?}", ast);
      self.compile_expr(*ast, &module);
    }
    self.modules.insert(name.clone(), module.clone());
    // module.print_to_stderr();
    let _ = module.print_to_file(format!("{}.ll", name.name()));
  }

  // method mainly used to include core...
  pub fn insert_module(&mut self, name: Symbol, module: Module<'a>) -> () {
    self.modules.insert(name, module);
  }

  pub fn modules(&self) -> &HashMap<Symbol, Module<'a>> { &self.modules }
}
