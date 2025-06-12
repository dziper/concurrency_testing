use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Error, Expr, ExprCall, FnArg, Ident, ItemFn, Token};
use syn::parse::{Parse, ParseStream};
use syn::{punctuated::Punctuated, ImplItem, ItemImpl};

/// Mark a Label in a [`testable!`] function, that the `MainController` can [`run_to!`].
///
/// ## Usage
/// ```rust,ignore
/// // code
/// label!("label 1");
/// // code
/// label!("label 2");
/// // code
/// ```
///
/// ## Expansion
/// ```rust,ignore
/// label!("label 1");
/// // Expands to
/// tokitest_thread_controller.label("label 1").await;
/// tokitest_thread_controller.label("label 1 block").await;
/// ```
#[proc_macro]
pub fn label(input: TokenStream) -> TokenStream {
    let label = syn::parse_macro_input!(input as syn::LitStr);
    let label_str = label.value();
    let block_label = format!("{} block", label_str);

    let expanded = quote! {
        #[cfg(feature = "tokitest")] // Label expands to nothing when not in test mode
        {
            tokitest_thread_controller.label(#label).await;
            tokitest_thread_controller.label(#block_label).await;
        }
    };
    TokenStream::from(expanded)
}

/// Mark a function as `testable` to allow it to contain [`label!`], [`call!`], [`Networkcall!`]
///
/// ## Usage
/// ```rust,ignore
/// #[testable]
/// fn my_function (arg: i32, ...){
///     label!("label 1");
///     call!(my_other_testable_function_with_labels())
///     label!("label 2");
///     ...
/// }
/// ```
///
/// ## Expanded
/// ```rust,ignore
/// #[testable]
/// fn my_function (arg: i32, ...) {
///     ...
/// }
/// // Expands to
/// fn my_function (tokitest_thread_controller &std::sync::Arc<::tokitest::controller::ThreadController>, arg: i32, ...) {
///     ...
/// }
/// ```
#[proc_macro_attribute]
pub fn testable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);

    // Create the first argument: controller: &Arc<::tokitest::controller::ThreadController>
    let controller_arg: FnArg = syn::parse_quote! {
        tokitest_thread_controller: std::sync::Arc<::tokitest::controller::ThreadController>
    };

    let insert_pos = match input_fn.sig.inputs.first() {
        Some(syn::FnArg::Receiver(_)) => 1, // after &self or self
        _ => 0,                             // normal free function
    };

    let unchanged_input_fn = input_fn.clone();

    input_fn.sig.inputs.insert(insert_pos, controller_arg);

    // Return modified function
    TokenStream::from(quote! {
        #[cfg(feature = "tokitest")]
        #input_fn
        
        #[cfg(not(feature = "tokitest"))]
        #unchanged_input_fn
    })
}

/// Makes all functions in an impl block `testable`
///
/// ## Usage
/// ```rust,ignore
/// struct MyStruct {}
///
/// #[testable_struct]
/// impl MyStruct {
///     fn my_func() {
///         label!("label 1");
///         // ...
///     }
///     fn my_func2() {
///         label!("label 2");
///         // ...
///     }
/// }
/// ```
// #[proc_macro_attribute]
// pub fn testable_struct(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     let mut impl_block = syn::parse_macro_input!(item as syn::ItemImpl);

//     for impl_item in impl_block.items.iter_mut() {
//         if let syn::ImplItem::Fn(method) = impl_item {
//             // Build the controller argument
//             let controller_arg: syn::FnArg = syn::parse_quote! {
//                 tokitest_thread_controller: std::sync::Arc<::tokitest::controller::ThreadController>
//             };

//             // Where to insert: after `self` if present
//             let insert_pos = match method.sig.inputs.first() {
//                 Some(syn::FnArg::Receiver(_)) => 1,
//                 _ => 0,
//             };

//             method.sig.inputs.insert(insert_pos, controller_arg);
//         }
//     }

//     // Return modified impl block
//     TokenStream::from(quote! {
//         #impl_block
//     })
// }

#[proc_macro_attribute]
pub fn testable_struct(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(item as ItemImpl);

    // Split each method into two: test and non-test versions
    let mut new_items = Vec::new();

    for item in input_impl.items {
        if let ImplItem::Fn(method) = item {
            let mut test_method = method.clone();
            let non_test_method = method.clone();

            // Insert controller param for test version
            let insert_pos = match test_method.sig.inputs.first() {
                Some(FnArg::Receiver(_)) => 1,
                _ => 0,
            };

            let controller_arg: FnArg = syn::parse_quote! {
                tokitest_thread_controller: std::sync::Arc<::tokitest::controller::ThreadController>
            };

            test_method.sig.inputs.insert(insert_pos, controller_arg);

            // Wrap each in cfg
            new_items.push(ImplItem::Verbatim(quote! {
                #[cfg(feature = "tokitest")]
                #test_method
            }));

            new_items.push(ImplItem::Verbatim(quote! {
                #[cfg(not(feature = "tokitest"))]
                #non_test_method
            }));
        } else {
            // Non-method items stay as-is
            new_items.push(item);
        }
    }

    // Recreate the impl block with duplicated methods
    let output_impl = ItemImpl {
        items: new_items,
        ..input_impl
    };

    TokenStream::from(quote! {
        #output_impl
    })
}


/*
from
    #[testable]
    fn <fn_name1> (args1){
        call!(fn_name2(args2))
    }

    #[testable]
    fn <fn_name2> (args){
    }
to
    fn <fn_name1> (tc &std::sync::Arc<::tokitest::controller::ThreadController>, args1) {
        fn_name2(tc, args2)
    }

    fn <fn_name2> (tc &std::sync::Arc<::tokitest::controller::ThreadController>, args2) {

    }
*/

/// All [`testable`] functions must be called with this macro for [`label!`] to work.
///
/// [`call!`] can only be used from within a [`testable`] function or a tokitest.
///
/// ## Usage
/// ```rust,ignore
/// #[testable]
/// fn my_function (arg: i32, ...) {
///     label!("label 1");
/// }
///
/// call!(my_function(123));
/// ```
#[proc_macro]
pub fn call(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    match expr {
        Expr::Call(ExprCall { func, args, .. }) => {
            let args_test = quote! { tokitest_thread_controller.clone(), #args };
            let args_normal = quote! { #args };
            let expanded = quote! {
                {
                    #[cfg(feature = "tokitest")]
                    {
                        #func(#args_test)
                    }
                    #[cfg(not(feature = "tokitest"))]
                    {
                        #func(#args_normal)
                    }
                }
            };
            TokenStream::from(expanded)
        }
        Expr::MethodCall(method_call) => {
            let method = &method_call.method;
            let receiver = &method_call.receiver;
            let args = &method_call.args;
            let args_test = quote! { tokitest_thread_controller.clone(), #args };
            let args_normal = quote! { #args };

            let expanded = quote! {
                {
                    #[cfg(feature = "tokitest")]
                    {
                        #receiver.#method(#args_test)
                    }
                    #[cfg(not(feature = "tokitest"))]
                    {
                        #receiver.#method(#args_normal)
                    }
                }
            };
            TokenStream::from(expanded)
        }
        other => syn::Error::new_spanned(other, "call! macro supports only function or method calls")
            .to_compile_error()
            .into(),
    }
}


/// Use the `spawn!` macro to spawn a testable thread with the new thread ID, using `tokio::spawn`.
/// 
/// The Spawned thread inherits it's parent's thread ID and adds the new ID as a prefix.
/// Threads are blocked until [`run_to!`] is called on their threadID
/// 
/// ## Usage
/// 
/// ```
/// // Spawns a thread with threadID `thread1`
/// spawn!("thread1", async {
///     label!("label 1");
/// 
///     // Spawns a thread with threadID `thread1.child`
///     spawn!("child", async {
///         label!("label 2");
///     });
///     
///     label!("label 3");
/// });
/// 
/// // thread1 is blocked. thread1.child has not been spawned yet.
/// run_to!("thread1", "label 3").await; // Run thread 1 until label 3, this spawns thread1.child
/// // thread1.child spawns and is immediately blocked.
/// run_to!("thread1.child", "label 2").await; // Run thread1.child until label 2
/// ```
/// 
/// ## Expansion
/// ```rust
/// // from
/// spawn!("child thread", async {
///     // some async code
/// });
/// 
/// // to
/// 
/// let tcNew = tokitest_thread_controller.nest("child thread").await;
/// tokio::spawn(async move {
///     tcNew.label("INIT").await;
///     let tokitest_thread_controller = tcNew.clone();
///     let result = {
///      // some async code
///     }.await;
///     tcNew.label("END").await;
///     result
/// })
/// ```
#[proc_macro]
pub fn spawn(input: TokenStream) -> TokenStream {
    struct SpawnInput {
        label: Expr,
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
            #[cfg(feature = "tokitest")]
            {
                // let tcNew = tokitest_thread_controller.nest(#label).await;
                let tcNew = tokitest_thread_controller.nest().with_id(#label).build().await;
                tokio::spawn(async move {
                    tcNew.label("INIT").await;
                    let tokitest_thread_controller = tcNew.clone();
                    let result = { #body }.await;
                    tcNew.label("END").await;
                    result
                })
            }

            #[cfg(not(feature = "tokitest"))]
            {
                tokio::spawn(async move {
                    #body.await
                })
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn spawn_join_set(item: TokenStream) -> TokenStream {
    struct SpawnJoinSetInput {
        label_expr: Expr,
        _comma1: Token![,],
        joinset_var: Ident,
        _comma2: Token![,],
        body: Expr,
    }

    impl Parse for SpawnJoinSetInput {
        fn parse(input: ParseStream) -> Result<Self, Error> {
            Ok(SpawnJoinSetInput {
                label_expr: input.parse()?,
                _comma1: input.parse()?,
                joinset_var: input.parse()?,
                _comma2: input.parse()?,
                body: input.parse()?,
            })
        }
    }

    let SpawnJoinSetInput {
        label_expr,
        _comma1,
        joinset_var,
        _comma2,
        body,
    } = parse_macro_input!(item as SpawnJoinSetInput);

    let expanded = quote! {

        #[cfg(feature = "tokitest")]
        {
            {
            // let tcNew = tokitest_thread_controller.nest(#label_expr).await;
            let tcNew = tokitest_thread_controller.nest().with_id(#label_expr).build().await;
            #joinset_var.spawn(async move {
                tcNew.label("INIT").await;
                let tokitest_thread_controller = tcNew.clone();
                let result = { #body }.await;
                tcNew.label("END").await;
                result
            })
            }
        }
        #[cfg(not(feature = "tokitest"))]
        {
            {
                #joinset_var.spawn(async move {
                    #body.await
                })
            }
        }
    };
    TokenStream::from(expanded)
}


/*
from
    Networkcall!(client.get("/api/data").send(), callback_on_error());

to
    async {
        if tokitest_thread_controller.networkDead() {
            return callback_on_error();
        } else {
            return client.get("/api/data").send();
        }
    }
*/
#[proc_macro]
pub fn network_call(input: TokenStream) -> TokenStream {
    struct NetworkCallInput {
        network_call: Expr,
        _comma: Token![,],
        error_callback: Expr,
    }

    impl Parse for NetworkCallInput {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            Ok(NetworkCallInput {
                network_call: input.parse()?,
                _comma: input.parse()?,
                error_callback: input.parse()?,
            })
        }
    }

    let NetworkCallInput { network_call, _comma, error_callback } = parse_macro_input!(input as NetworkCallInput);

    let expanded = quote! {
        {
            #[cfg(feature = "tokitest")]
            {
                if tokitest_thread_controller.is_isolated().await {
                    // Box::pin(#error_callback) as std::pin::Pin<Box<dyn std::future::Future<Output = _> + Send>>
                    ::futures::future::Either::Left(#error_callback)
                } else {
                    // Box::pin(#network_call) as std::pin::Pin<Box<dyn std::future::Future<Output = _> + Send>>
                    ::futures::future::Either::Right(#network_call)
                }
            }
            #[cfg(not(feature = "tokitest"))]
            {
                #network_call
            }
        }
    };

    TokenStream::from(expanded)
}

/**
from
    Isolate!("thrad-id")

to
    async {
        tokitest_main_controller.isolate("thread_id").await
    }
*/
#[proc_macro]
pub fn isolate(input: TokenStream) -> TokenStream {
    let thread_id = syn::parse_macro_input!(input as syn::LitStr);

    let expanded = quote! {
        tokitest_main_controller.isolate(#thread_id)
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn heal(input: TokenStream) -> TokenStream {
    let thread_id = syn::parse_macro_input!(input as syn::LitStr);

    let expanded = quote! {
        tokitest_main_controller.heal(#thread_id)
    };

    TokenStream::from(expanded)
}

// /*
// from
// start_tokitest!()
// to

// */
// #[proc_macro]
// pub fn start_tokitest(_input: TokenStream) -> TokenStream {
//     let expanded = quote! {
//         let tokitest_main_controller = Arc::new(::tokitest::controller::MainController::new());
//         let tokitest_thread_controller = tokitest_main_controller.nest("").await;
//     };
//     TokenStream::from(expanded)
// }

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


/// Thingy
#[proc_macro]
pub fn run_to(input: TokenStream) -> TokenStream {
    let RunToArgs { args } = syn::parse_macro_input!(input as RunToArgs);

    // Expect exactly two arguments
    // TODO: may want to allow non string literal?
    if args.len() != 2 {
        return syn::Error::new_spanned(args, "run_to! requires exactly two arguments: a string literal and an expression")
            .to_compile_error()
            .into();
    }

    let mut args_iter = args.into_iter();
    let thread_id = args_iter.next().unwrap();
    let label = args_iter.next().unwrap();

    // First argument must be a LitStr
    let label_lit = match &thread_id {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => lit_str,
        _ => {
            return syn::Error::new_spanned(thread_id, "First argument to run_to! must be a string literal")
                .to_compile_error()
                .into();
        }
    };

    // Now check whether second argument is also a LitStr
    let expanded = match &label {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(_),
            ..
        }) => {
            // Both are string literals
            quote! {
                tokitest_main_controller.run_to(#label_lit, #label)
            }
        }
        _ => {
            // Second argument is a general expression, assume is a LabelTrait
            quote! {
                tokitest_main_controller.run_to_label(#label_lit, #label)
            }
        }
    };

    TokenStream::from(expanded)
}

/**
from
    complete!("thread-id").await

to
    async {
        tokitest_main_controller.run_to_end("thread_id").await
    }
*/
#[proc_macro]
pub fn complete(input: TokenStream) -> TokenStream {
    let thread_id = syn::parse_macro_input!(input as syn::LitStr);

    let expanded = quote! {
        tokitest_main_controller.run_to_end(#thread_id)
    };

    TokenStream::from(expanded)
}


/// Mark tests with #[tokiotest::test] to use the testing framework.
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);

    // Extract the original function body
    let original_body = &input_fn.block;

    // Generate new body with tokitest setup + original code
    let new_body = quote! {
        {
            let tokitest_main_controller = std::sync::Arc::new(::tokitest::controller::MainController::new());
            let tokitest_thread_controller = tokitest_main_controller.nest().build().await;

            #original_body
        }
    };

    // Replace the function body
    input_fn.block = syn::parse2(new_body).unwrap();

    // Add tokio::test attribute and return the modified function
    quote! {
        #[tokio::test]
        #input_fn
    }.into()  // <-- Add this!
}