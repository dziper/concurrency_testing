use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, ItemFn, FnArg, Expr, Token};
use syn::parse::{Parse, ParseStream};


/*
from
    Label!("label 1");

to
    tokitest_thread_controller.label("label 1").await;
    tokitest_thread_controller.label("label 1 block").await;
*/
#[proc_macro]
pub fn Label(input: TokenStream) -> TokenStream {
    let label = syn::parse_macro_input!(input as syn::LitStr);
    let label_str = label.value();
    let block_label = format!("{} block", label_str);

    let expanded = quote! {
        tokitest_thread_controller.label(#label).await;
        tokitest_thread_controller.label(#block_label).await;
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
    fn <fn_name> (tokitest_thread_controller &std::sync::Arc<ThreadController>, args){
    }
*/
#[proc_macro_attribute]
pub fn testable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);

    // Create the first argument: controller: &Arc<ThreadController>
    let controller_arg: FnArg = syn::parse_quote! {
        tokitest_thread_controller: std::sync::Arc<ThreadController>
    };

    let insert_pos = match input_fn.sig.inputs.first() {
        Some(syn::FnArg::Receiver(_)) => 1, // after &self or self
        _ => 0,                             // normal free function
    };

    input_fn.sig.inputs.insert(insert_pos, controller_arg);

    // Return modified function
    TokenStream::from(quote! {
        #input_fn
    })
}

#[proc_macro_attribute]
pub fn Testable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut impl_block = syn::parse_macro_input!(item as syn::ItemImpl);

    for impl_item in impl_block.items.iter_mut() {
        if let syn::ImplItem::Fn(method) = impl_item {
            // Build the controller argument
            let controller_arg: syn::FnArg = syn::parse_quote! {
                tokitest_thread_controller: std::sync::Arc<ThreadController>
            };

            // Where to insert: after `self` if present
            let insert_pos = match method.sig.inputs.first() {
                Some(syn::FnArg::Receiver(_)) => 1,
                _ => 0,
            };

            method.sig.inputs.insert(insert_pos, controller_arg);
        }
    }

    // Return modified impl block
    TokenStream::from(quote! {
        #impl_block
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
        #func(tokitest_thread_controller.clone(), #args)
    };

    TokenStream::from(expanded)
}


/*
from
    Spawn!("child thread", async {
        // some async code
    });

to
    let tcNew = tokitest_thread_controller.nest("child thread").await;
    tokio::spawn(async move {
        tcNew.label("INIT").await;
        let tokitest_thread_controller = tcNew.clone();
        let result = { 
            // some async code
        }.await;
        tcNew.label("END").await;
        result
    })
*/
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
            let tcNew = tokitest_thread_controller.nest(#label).await;
            tokio::spawn(async move {
                tcNew.label("INIT").await;
                let tokitest_thread_controller = tcNew.clone();
                let result = { #body }.await;
                tcNew.label("END").await;
                result
            })
        }
    };

    TokenStream::from(expanded)
}

/*
from
    NetworkCall!(client.get("/api/data").send().await);

to
    async {
        if tokitest_thread_controller.networkDead() {
            return Err("Network is dead");
        }
        client.get("/api/data").send().await
    }
*/
#[proc_macro]
pub fn NetworkCall(input: TokenStream) -> TokenStream {
    let call = syn::parse_macro_input!(input as syn::Expr);

    let expanded = quote! {
        async {
            if tokitest_thread_controller.networkDead() {
                return Err("Network is dead");
            }
            #call
        }
    };

    TokenStream::from(expanded)
}

/*
from 
CreateMainController!()
to



*/
#[proc_macro]
pub fn CreateMainController(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        let tokitest_thread_controller = MainController::new();
    };
    TokenStream::from(expanded)
}