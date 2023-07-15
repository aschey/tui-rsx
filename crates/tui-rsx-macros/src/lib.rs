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
        Ok(nodes) => view::parse_named_element_children(&nodes, false),
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro]
#[proc_macro_error]
pub fn view(tokens: TokenStream) -> TokenStream {
    view::view(tokens, true)
}

#[proc_macro]
#[proc_macro_error]
pub fn mount(tokens: TokenStream) -> TokenStream {
    view::view(tokens, false)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn component(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    parse_macro_input!(tokens as component::Model)
        .into_token_stream()
        .into()
}
