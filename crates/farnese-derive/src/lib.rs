use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input,
    ItemFn,
};


#[proc_macro_attribute]
pub fn farnese_component_test(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let body = &input.block;
    let expanded = quote! {
        #[test]
        fn #fn_name() {
            let context = inkwell::context::Context::create();
            let mut helper = TestHelper::new(&context, stringify!(#fn_name));
            helper.start();
            {
                let codegen = &mut helper.codegen;
                #body
            }
            helper.end();
            let output = helper.run();
            println!("Generated output: {}", output);
        }
    };

    expanded.into()
}
