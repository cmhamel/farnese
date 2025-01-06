use super::Symbol;
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::types::StructType;
use inkwell::values::GlobalValue;
use inkwell::AddressSpace;


pub fn create_any_type<'a>(context: &'a Context, module: Module<'a>, type_tag: StructType<'a>) -> Module<'a> {
  // let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default()).into()
  // let type_tag = create_type_tag(context);
  let any_sym = Symbol::new("Any");
  let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::default());
  let any_type = module.add_global(type_tag, None, any_sym.name());
  any_type.set_initializer(&type_tag.const_named_struct(&[
    // context.i64_type().const_int(any_sym.hash(), false).as_pointer_value(),
    // any_id.as_pointer_value().into(),
    i8_ptr_type.const_null().into(),
    any_type.as_pointer_value().into(), // Parent: points to itself
  ]));
  module.clone()
}

#[derive(Debug)]
pub struct AnyValue {
  // data: PointerValue<'a>
}

impl<'a> AnyValue {
  pub fn new_on_stack(
    context: &'a Context, builder: &'a Builder, 
    type_tag: StructType<'a>, value_type: StructType<'a>,
    any_type: GlobalValue<'a>
  ) -> Self {
    // TODO make this read in
    let value_in_val = context.i32_type().const_int(42, false);
    let value_in = builder.build_alloca(value_in_val.get_type(), "__valuein").unwrap();
    let _ = builder.build_store(value_in, value_in_val);
    let value_in_ptr = builder.build_alloca(context.i8_type().ptr_type(AddressSpace::default()), "__ref__valuein").unwrap();
    let casted_ptr = builder.build_bit_cast(value_in, context.i8_type().ptr_type(AddressSpace::default()), "casted_ptr").unwrap();
    let _ = builder.build_store(value_in_ptr, casted_ptr);

    // works below
    // let value_type = create_value_type(context, type_tag);
    let value = builder.build_alloca(value_type, "__refany").unwrap();
    let value_index = context.i32_type().const_int(0, false);
    let value_ptr = unsafe {
      builder.build_gep(value, &[value_index, value_index], "__valueptr")
    }.unwrap();
    // builder.build_store(value_ptr, value_in_ptr);
    let ptr_index = context.i32_type().const_int(1, false);
    let type_ptr = unsafe {
      builder.build_gep(value, &[ptr_index, ptr_index], "__typeptr")
    }.unwrap();
    let _ = builder.build_store(type_ptr, any_type);

    // // now build a load
    // builder.build_load(value, "__anyvalue");
    Self {}
  }
}


// #[derive(Debug)]
// enum TypeTag {
//   Int,
//   Float,
//   Char,
// }

// #[derive(Debug)]
// struct RefCountedAny<'a> {
//   ref_count: Arc<AtomicUsize>, // Reference count to manage memory
//   data: PointerValue<'a>,       // Pointer to actual data
//   type_tag: u8,                 // Type tag for the data (e.g., Int, Float)
//   supertype_ptr: Option<PointerValue<'a>>, // Supertype pointer (if any)
// }

// impl<'a> RefCountedAny<'a> {
//   fn new(
//     context: &'a Context,
//     builder: &Builder<'a>,
//     tag: TypeTag,
//     data: PointerValue<'a>,
//     supertype: Option<PointerValue<'a>>,
//   ) -> PointerValue<'a> {
//     // Create reference-counted Any type
//     let ref_count = Arc::new(AtomicUsize::new(1)); // Initialize reference count to 1

//     // Define the 'Any' struct type: [ref_count, type_tag, data_ptr, supertype_ptr]
//     let type_tag_type = context.i8_type();
//     let ref_counted_type = context.struct_type(
//       &[
//         type_tag_type.ptr_type(AddressSpace::default()).into(),  // ref_count (pointer)
//         type_tag_type.into(),                                   // type_tag
//         context.i8_type().ptr_type(AddressSpace::default()).into(), // data pointer
//         context.i8_type().ptr_type(AddressSpace::default()).into(), // supertype pointer
//       ],
//       false,
//     );

//     // Allocate memory for the 'Any' object
//     let ref_counted_value = builder.build_alloca(ref_counted_type, "__ref_counted_any").unwrap();

//     // Set the reference count to 1
//     let ref_count_ptr = unsafe {
//       builder.build_struct_gep(ref_counted_value, 0, "ref_count_ptr").unwrap()
//     };
//     builder.build_store(ref_count_ptr, builder.build_int_add(
//       context.i64_type().const_int(1, false),
//       context.i64_type().const_int(0, false),
//       "inc_ref_count",
//     ).unwrap()).unwrap();

//     // Set the type tag (could be 0 for int, 1 for float, etc.)
//     let tag_value = match tag {
//       TypeTag::Int => context.i8_type().const_int(0, false),
//       TypeTag::Float => context.i8_type().const_int(1, false),
//       TypeTag::Char => context.i8_type().const_int(2, false),
//     };

//     // Store the type tag in the 'Any' object
//     let tag_ptr = unsafe { builder.build_struct_gep(ref_counted_value, 1, "type_tag_ptr").unwrap() };
//     builder.build_store(tag_ptr, tag_value);

//     // Store the pointer to the actual data
//     let data_ptr = unsafe { builder.build_struct_gep(ref_counted_value, 2, "data_ptr").unwrap() };
//     builder.build_store(data_ptr, data);

//     // Store the supertype pointer (if present)
//     let supertype_ptr_value = unsafe {
//       builder.build_struct_gep(ref_counted_value, 3, "supertype_ptr").unwrap()
//     };
//     match supertype {
//       Some(ptr) => builder.build_store(supertype_ptr_value, ptr),
//       None => builder.build_store(supertype_ptr_value, builder.build_null(ref_counted_type.ptr_type(AddressSpace::default()))),
//     }

//     ref_counted_value
//   }

//   // Decrements reference count and cleans up if necessary
//   fn release<'b>(
//     builder: &Builder<'b>,
//     ref_counted_value: PointerValue<'b>,
//     ref_count: Arc<AtomicUsize>,
//   ) {
//     let ref_count_ptr = unsafe {
//       builder.build_struct_gep(ref_counted_value, 0, "ref_count_ptr").unwrap()
//     };

//     // Decrement the reference count
//     let decremented = builder.build_int_sub(
//       builder.build_load(ref_count_ptr, "ref_count").into_int_value(),
//       builder.build_int(1, "decrement_ref_count"),
//       "new_ref_count",
//     );

//     // Check if reference count reaches 0 (deallocate if so)
//     let is_zero = builder.build_int_compare(
//       inkwell::IntPredicate::EQ,
//       decremented,
//       builder.build_int(0, "zero", false).into_int_value(),
//       "__is_zero",
//     );

//     // If ref count is 0, clean up memory (deallocate)
//     // builder.build_conditional_branch(is_zero, /* deallocate */, /* continue */);
//   }
// }

// // fn main() {
// //     let context = Context::create();
// //     let builder = context.create_builder();

// //     // Example: Store an integer value
// //     let int_type = context.i64_type();
// //     let int_value = int_type.const_int(42, false);
// //     let int_ptr = builder.build_alloca(int_type, "int_value");
// //     builder.build_store(int_ptr, int_value);
    
// //     // Create 'Any' object that stores the integer
// //     let any_int = RefCountedAny::new(&context, &builder, TypeTag::Int, int_ptr, None);

// //     // Example: Store a float value
// //     let float_type = context.f64_type();
// //     let float_value = float_type.const_float(3.14);
// //     let float_ptr = builder.build_alloca(float_type, "float_value");
// //     builder.build_store(float_ptr, float_value);

// //     // Create 'Any' object that stores the float with the previous 'Any' object as supertype
// //     let any_float = RefCountedAny::new(&context, &builder, TypeTag::Float, float_ptr, Some(any_int));

// //     println!("{:?}", any_int); // For debugging, you can print the pointer value
// //     println!("{:?}", any_float); // For debugging, you can print the pointer value
// // }
