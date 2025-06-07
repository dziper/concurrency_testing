use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, ExprCall, FnArg, ItemFn, LitStr, Token};
use syn::parse::{Parse, ParseStream};
use syn::{punctuated::Punctuated};


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
// #[proc_macro]
// pub fn Call(input: TokenStream) -> TokenStream {
//     use syn::{parse_macro_input, ExprCall};

//     let expr = parse_macro_input!(input as ExprCall);
//     let func = &expr.func;
//     let args = &expr.args;

//     let expanded = quote! {
//         #func(tokitest_thread_controller.clone(), #args)
//     };

//     TokenStream::from(expanded)
// }
#[proc_macro]
pub fn Call(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    match expr {
        Expr::Call(ExprCall { func, args, .. }) => {
            let expanded = quote! {
                #func(tokitest_thread_controller.clone(), #args)
            };
            TokenStream::from(expanded)
        }
        Expr::MethodCall(mut method_call) => {
            let method = &method_call.method;
            let receiver = &method_call.receiver;
            let args = &method_call.args;

            let expanded = quote! {
                #receiver.#method(tokitest_thread_controller.clone(), #args)
            };
            TokenStream::from(expanded)
        }
        other => syn::Error::new_spanned(other, "Call! macro supports only function or method calls")
            .to_compile_error()
            .into(),
    }
}


/*
Call!(some_func(x, y));            // Expands to: some_func(tokitest_thread_controller.clone(), x, y)

Call!(obj.method(a, b));           // Expands to: obj.method(tokitest_thread_controller.clone(), a, b)

Call!(obj1.obj2.func(z));          // Expands to: obj1.obj2.func(tokitest_thread_controller.clone(), z)
*/


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
        let tokitest_main_controller = Arc::new(MainController::new());
        let tokitest_thread_controller = tokitest_main_controller.nest("").await;
    };
    TokenStream::from(expanded)
}

struct RunToArgs {
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for RunToArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RunToArgs {
            args: Punctuated::parse_terminated(input)?,
        })
    }
}


#[proc_macro]
pub fn RunTo(input: TokenStream) -> TokenStream {
    let RunToArgs { args } = syn::parse_macro_input!(input as RunToArgs);

    // Expect exactly two arguments
    // TODO: may want to allow non string literal?
    if args.len() != 2 {
        return syn::Error::new_spanned(args, "RunTo! requires exactly two arguments: a string literal and an expression")
            .to_compile_error()
            .into();
    }

    let mut args_iter = args.into_iter();
    let label_expr = args_iter.next().unwrap();
    let second_expr = args_iter.next().unwrap();

    // First argument must be a LitStr
    let label_lit = match &label_expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => lit_str,
        _ => {
            return syn::Error::new_spanned(label_expr, "First argument to RunTo! must be a string literal")
                .to_compile_error()
                .into();
        }
    };

    // Now check whether second argument is also a LitStr
    let expanded = match &second_expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(_),
            ..
        }) => {
            // Both are string literals
            quote! {
                tokitest_main_controller.run_to(#label_lit, #second_expr)
            }
        }
        _ => {
            // Second argument is a general expression, assume is a LabelTrait
            quote! {
                tokitest_main_controller.run_to_label(#label_lit, #second_expr)
            }
        }
    };

    TokenStream::from(expanded)
}
