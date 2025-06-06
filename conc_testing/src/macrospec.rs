use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

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
        tc.label(#label_str).await;
        tc.label(#block_label).await;
    };

    TokenStream::from(expanded)
}

