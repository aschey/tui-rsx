use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::proc_macro_error;
use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

mod caller_id;
mod component;
mod component_children;
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
    view::view(tokens.into(), true).into()
}

#[proc_macro]
#[proc_macro_error]
pub fn mount(tokens: TokenStream) -> TokenStream {
    view::view(tokens.into(), false).into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn component(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    parse_macro_input!(tokens as component::Model)
        .into_token_stream()
        .into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn caller_id(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    caller_id::parse(input).into()
}

#[proc_macro_derive(ComponentChildren, attributes(children))]
#[proc_macro_error]
pub fn component_children(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    component_children::parse(input).into()
}

fn get_import() -> proc_macro2::TokenStream {
    let found_crate = crate_name("tui-rsx").expect("tui-rsx not found");

    match found_crate {
        FoundCrate::Itself => quote::quote!(crate),
        FoundCrate::Name(name) => {
            let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
            quote::quote!(#ident)
        }
    }
}
