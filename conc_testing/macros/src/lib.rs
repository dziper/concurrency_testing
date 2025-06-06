use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, ItemFn, FnArg};

/*
from
    Label!("label 1");

to
    tc.label("label 1").await;
    tc.label("label 1 block").await;
*/
#[proc_macro]
pub fn Label(input: TokenStream) -> TokenStream {
    let label = parse_macro_input!(input as LitStr);
    let label_str = label.value();
    let block_label = format!("{label_str} block");

    let expanded = quote! {
        tokitestThreadController.label(#label_str).await;
        tokitestThreadController.label(#block_label).await;
    };

    TokenStream::from(expanded)
}

// implement testable
/*
from
    #[testable]
    fn <fn_name> (args){
    }
to
    fn <fn_name> (tokitestThreadController &std::sync::Arc<ThreadController>, args){
    }
*/
#[proc_macro_attribute]
pub fn testable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);

    // Create the first argument: controller: &Arc<ThreadController>
    let controller_arg: FnArg = syn::parse_quote! {
        tokitestThreadController: &std::sync::Arc<ThreadController>
    };

    // Insert as first parameter
    input_fn.sig.inputs.insert(0, controller_arg);

    // Return modified function
    TokenStream::from(quote! {
        #input_fn
    })
}

// #[macro_export]
// macro_rules! Call {
//     ($func:ident $(, $arg:expr)* $(,)?) => {
//         $func(tokitestThreadController $(, $arg)*)
//     };
// }

