use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::{quote, ToTokens};
use syn::parse_macro_input;

mod component;
mod view;

#[proc_macro]
#[proc_macro_error]
pub fn prop(tokens: TokenStream) -> TokenStream {
    match rstml::parse(tokens) {
        Ok(nodes) => view::parse_named_element_children(&nodes),
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro]
#[proc_macro_error]
pub fn view(tokens: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = tokens.into();
    let mut tokens = tokens.into_iter().peekable();
    let cx_token = if tokens.peek().unwrap().to_string() != "<" {
        let token = tokens.next().unwrap().to_token_stream();
        let _comma = tokens.next().unwrap();
        token
    } else {
        quote! { () }
    };
    // let cx_token = tokens.next().unwrap() else {
    //     abort_call_site!("Missing context parameter");
    // };

    match rstml::parse2(tokens.collect()) {
        Ok(nodes) => {
            let view = view::parse_root_nodes(&cx_token, nodes);
            // abort_call_site!(format!("{}", view.to_token_stream()));
            quote! { #view }
        }
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn component(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    parse_macro_input!(tokens as component::Model)
        .into_token_stream()
        .into()
}
