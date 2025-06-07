use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, ItemFn, FnArg, Expr, Token};
use syn::parse::{Parse, ParseStream};


/*
from
    Label!("label 1");

to
    tokitestThreadController.label("label 1").await;
    tokitestThreadController.label("label 1 block").await;
*/
#[proc_macro]
pub fn Label(input: TokenStream) -> TokenStream {
    let label = syn::parse_macro_input!(input as syn::LitStr);
    let label_str = label.value();
    let block_label = format!("{} block", label_str);

    let expanded = quote! {
        tokitestThreadController.label(#label).await;
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
        tokitestThreadController: std::sync::Arc<ThreadController>
    };

    // Insert as first parameter
    input_fn.sig.inputs.insert(0, controller_arg);

    // Return modified function
    TokenStream::from(quote! {
        #input_fn
    })
}

/*
from
    #[testable]
    fn <fn_name1> (args1){
        Call!(fn_name2(args2))
    }

    #[testable]
    fn <fn_name2> (args){
    }
to
    fn <fn_name1> (tc &std::sync::Arc<ThreadController>, args1) {
        fn_name2(tc, args2)
    }

    fn <fn_name2> (tc &std::sync::Arc<ThreadController>, args2) {

    }
*/
#[proc_macro]
pub fn Call(input: TokenStream) -> TokenStream {
    use syn::{parse_macro_input, ExprCall};

    let expr = parse_macro_input!(input as ExprCall);
    let func = &expr.func;
    let args = &expr.args;

    let expanded = quote! {
        #func(tokitestThreadController.clone(), #args)
    };

    TokenStream::from(expanded)
}


#[proc_macro]
pub fn NetworkCall(input: TokenStream) -> TokenStream {
    let call = syn::parse_macro_input!(input as syn::Expr);

    let expanded = quote! {
        async {
            if tokitestThreadController.networkDead() {
                return Err("Network is dead");
            }
            #call
        }
    };

    TokenStream::from(expanded)
}


#[proc_macro]
pub fn Spawn(input: TokenStream) -> TokenStream {
    struct SpawnInput {
        label: LitStr,
        _comma: Token![,],
        body: Expr,
    }

    impl Parse for SpawnInput {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(SpawnInput {
                label: input.parse()?,
                _comma: input.parse()?,
                body: input.parse()?,
            })
        }
    }

    let SpawnInput { label, _comma, body } = parse_macro_input!(input as SpawnInput);

    let expanded = quote! {
        {
            let tcNew = tokitestThreadController.nest(#label).await;
            tokio::spawn(async move {
                tcNew.label("INIT").await;
                let tokitestThreadController = tcNew.clone();
                let result = { #body }.await;
                tcNew.label("END").await;
                result
            })
        }
    };

    TokenStream::from(expanded)
}