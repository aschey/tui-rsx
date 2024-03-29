use crate::get_import;
use attribute_derive::Attribute as AttributeDerive;
use convert_case::{
    Case::{Pascal, Snake},
    Casing,
};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{
    parse::Parse, parse_quote, spanned::Spanned, AngleBracketedGenericArguments, Attribute, FnArg,
    GenericArgument, Item, ItemFn, LitStr, Meta, Pat, PatIdent, Path, PathArguments, ReturnType,
    Stmt, Type, TypeParamBound, TypePath, Visibility,
};

pub struct Model {
    is_transparent: bool,
    docs: Docs,
    vis: Visibility,
    name: Ident,
    scope_name: PatIdent,
    scope_type: Type,
    props: Vec<Prop>,
    body: ItemFn,
    ret: ReturnType,
    view_type: Type,
}

impl Parse for Model {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut item = ItemFn::parse(input)?;
        let docs = Docs::new(&item.attrs);

        let props = item
            .sig
            .inputs
            .clone()
            .into_iter()
            .map(Prop::new)
            .collect::<Vec<_>>();
        let children_count = props.iter().filter(|p| p.prop_opts.children).count();
        if children_count > 1 {
            abort!(
                item.sig.inputs,
                "only one parameter can be used as children"
            );
        }

        let (scope_name, scope_type) = if props.is_empty() {
            abort!(
                item.sig,
                "this method requires a `Scope` parameter";
                help = "try `fn {}(cx: Scope, /* ... */)`", item.sig.ident
            );
        } else {
            (props[0].name.clone(), props[0].ty.clone())
        };

        // else if !is_valid_scope_type(&props[0].ty) {
        //     abort!(
        //         item.sig.inputs,
        //         "this method requires a `Scope` parameter";
        //         help = "try `fn {}(cx: Scope, /* ... */ */)`", item.sig.ident
        //     );
        // }

        // We need to remove the `#[doc = ""]` and `#[builder(_)]`
        // attrs from the function signature
        drain_filter(&mut item.attrs, |attr| match &attr.meta {
            Meta::NameValue(attr) => attr.path == parse_quote!(doc),
            Meta::List(attr) => attr.path == parse_quote!(prop),
            _ => false,
        });
        item.sig.inputs.iter_mut().for_each(|arg| {
            if let FnArg::Typed(ty) = arg {
                drain_filter(&mut ty.attrs, |attr| match &attr.meta {
                    Meta::NameValue(attr) => attr.path == parse_quote!(doc),
                    Meta::List(attr) => attr.path == parse_quote!(prop),
                    _ => false,
                });
            }
        });

        let view_type = get_view_generics(&item.sig.output);
        // Make sure return type is correct
        // if !is_valid_into_view_return_type(&item.sig.output) {
        //     abort!(
        //         item.sig,
        //         "return type is incorrect";
        //         help = "return signature must be `-> impl IntoView`"
        //     );
        // }

        Ok(Self {
            is_transparent: false,
            docs,
            vis: item.vis.clone(),
            name: convert_from_snake_case(&item.sig.ident),
            scope_name,
            scope_type,
            props,
            ret: item.sig.output.clone(),
            body: item,
            view_type,
        })
    }
}

fn get_view_generics(return_type: &ReturnType) -> Type {
    if let ReturnType::Type(_, return_type) = &return_type {
        if let Type::ImplTrait(impl_trait) = return_type.as_ref() {
            let bound = impl_trait.bounds.first().unwrap();
            if let TypeParamBound::Trait(bound_trait) = bound {
                if let PathArguments::AngleBracketed(args) = &bound_trait.path.segments[0].arguments
                {
                    if let GenericArgument::Type(generic_type) = &args.args.first().unwrap() {
                        return generic_type.clone();
                    }
                }
            }
        }
    };
    abort!(return_type,"return type is incorrect"; help = "return signature must be `-> impl View<B>`");
}

// implemented manually because Vec::drain_filter is nightly only
// follows std recommended parallel
pub fn drain_filter<T>(vec: &mut Vec<T>, mut some_predicate: impl FnMut(&mut T) -> bool) {
    let mut i = 0;
    while i < vec.len() {
        if some_predicate(&mut vec[i]) {
            _ = vec.remove(i);
        } else {
            i += 1;
        }
    }
}

pub fn convert_from_snake_case(name: &Ident) -> Ident {
    let name_str = name.to_string();
    if !name_str.is_case(Snake) {
        name.clone()
    } else {
        Ident::new(&name_str.to_case(Pascal), name.span())
    }
}

impl ToTokens for Model {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            is_transparent,
            docs,
            vis,
            name,
            scope_name,
            scope_type,
            props,
            body,
            ret,
            view_type,
        } = self;

        let no_props = false; //props.len() == 1;

        let mut body = body.to_owned();
        let mut props = props.to_owned();

        // check for components that end ;
        if !is_transparent {
            let ends_semi = body.block.stmts.iter().last().and_then(|stmt| match stmt {
                Stmt::Item(Item::Macro(mac)) => mac.semi_token.as_ref(),
                _ => None,
            });
            if let Some(semi) = ends_semi {
                proc_macro_error::emit_error!(
                    semi.span(),
                    "A component that ends with a `view!` macro followed by a \
                     semicolon will return (), an empty view. This is usually \
                     an accident, not intentional, so we prevent it. If you'd \
                     like to return (), you can do it it explicitly by \
                     returning () as the last item from the component."
                );
            }
        }

        body.sig.ident = format_ident!("__{}", body.sig.ident);
        body.sig.inputs.push(syn::parse_quote!(__parent_id: u32));
        body.sig.output = syn::parse_quote!(-> impl LazyView<#view_type>);
        #[allow(clippy::redundant_clone)] // false positive
        let body_name = body.sig.ident.clone();

        let (impl_generics, generics, where_clause) = body.sig.generics.split_for_impl();
        let generics_tokens: Vec<_> = body
            .sig
            .generics
            .type_params()
            .map(|p| p.ident.to_token_stream())
            .collect();

        if !body.sig.generics.params.is_empty() {
            props.push(Prop {
                docs: Docs::new(&[]),
                prop_opts: PropOpt {
                    default: Some(syn::parse_quote!(Default::default())),
                    ..Default::default()
                },
                name: PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None,
                    ident: Ident::new("_phantom", Span::call_site()),
                    subpat: None,
                },
                ty: Type::Path(TypePath {
                    qself: None,
                    path: syn::parse_quote!(::std::marker::PhantomData<(#(#generics_tokens),*)>),
                }),
            });
        }
        let lifetimes = body.sig.generics.lifetimes();

        let props_name = format_ident!("{name}Props");

        let prop_builder_fields = prop_builder_fields(vis, &props);

        let prop_names = prop_names(&props);
        let used_prop_names = prop_names_for_component(&props);
        let builder_name_doc =
            LitStr::new(&format!("Props for the [`{name}`] component."), name.span());

        let component_fn_prop_docs = generate_component_fn_prop_docs(&props);

        let crate_import = get_import();
        let component = quote! {
            #crate_import::LazyViewWrapper::new(#body_name(#scope_name, #used_prop_names __caller_id))
        };

        let props_arg = if no_props {
            quote! {}
        } else {
            quote! {
                props: #props_name #generics
            }
        };

        let destructure_props = if no_props {
            quote! {}
        } else {
            quote! {
                let #props_name {
                    #prop_names
                } = props;
            }
        };

        let cache_name = format_ident!("{}_CACHE", body.sig.ident.to_string().to_uppercase());
        let widget_cache_decl = quote! {
            thread_local! {
                static #cache_name: ::std::cell::RefCell<#crate_import::once_cell::sync::Lazy<#crate_import::typemap::TypeMap>> =
                    ::std::cell::RefCell::new(#crate_import::once_cell::sync::Lazy::new(#crate_import::typemap::TypeMap::new));
            }
        };

        let widget_cache_impl = quote! {
            #cache_name.with(|c| {
                let mut cache_mut = c.borrow_mut();
                if let Some(map) = cache_mut.get_mut::<#crate_import::KeyWrapper<#view_type>>() {
                    if let Some(cache) = map.get(&__caller_id) {
                        cache.clone()
                    } else {
                        let res = ::std::rc::Rc::new(::std::cell::RefCell::new(#component));
                        map.insert(__caller_id, res.clone());
                        res
                    }
                } else {
                    let mut map = ::std::collections::HashMap::<u32, ::std::rc::Rc<::std::cell::RefCell<dyn View<#view_type>>>>::new();
                    let res = ::std::rc::Rc::new(::std::cell::RefCell::new(#component));
                    map.insert(__caller_id, res.clone());
                    cache_mut.insert::<#crate_import::KeyWrapper<#view_type>>(map);
                    res
                }
            })
        };

        let output = quote! {
            #[doc = #builder_name_doc]
            #[doc = ""]
            #docs
            #component_fn_prop_docs
            #[caller_id]
            #[derive(#crate_import::typed_builder::TypedBuilder, #crate_import::ComponentChildren)]
            #[builder(doc)]
            #vis struct #props_name #impl_generics #where_clause {
                #prop_builder_fields
            }

            #widget_cache_decl

            #docs
            #component_fn_prop_docs
            #[allow(non_snake_case, clippy::too_many_arguments, unused_mut)]
            // #tracing_instrument_attr
            #vis fn #name #impl_generics (
                #[allow(unused_variables)]
                #scope_name: #scope_type,
                #props_arg
            ) #ret #(+ #lifetimes)*
            #where_clause
            {
                #body

                #destructure_props

                #widget_cache_impl
            }
        };

        tokens.append_all(output)
    }
}

#[derive(Clone, Debug)]
struct Prop {
    docs: Docs,
    prop_opts: PropOpt,
    name: PatIdent,
    ty: Type,
}

impl Prop {
    fn new(arg: FnArg) -> Self {
        let typed = if let FnArg::Typed(ty) = arg {
            ty
        } else {
            abort!(arg, "receiver not allowed in `fn`");
        };

        let prop_opts = PropOpt::from_attributes(&typed.attrs).unwrap_or_else(|e| {
            // TODO: replace with `.unwrap_or_abort()` once https://gitlab.com/CreepySkeleton/proc-macro-error/-/issues/17 is fixed
            abort!(e.span(), e.to_string());
        });

        let name = if let Pat::Ident(i) = *typed.pat {
            i
        } else {
            abort!(
                typed.pat,
                "only `prop: type` style types are allowed within the \
                 `#[component]` macro"
            );
        };

        Self {
            docs: Docs::new(&typed.attrs),
            prop_opts,
            name,
            ty: *typed.ty,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Docs(Vec<(String, Span)>);

impl ToTokens for Docs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let s = self
            .0
            .iter()
            .map(|(doc, span)| quote_spanned!(*span=> #[doc = #doc]))
            .collect::<TokenStream>();

        tokens.append_all(s);
    }
}

impl Docs {
    pub fn new(attrs: &[Attribute]) -> Self {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        enum ViewCodeFenceState {
            Outside,
            Rust,
            Rsx,
        }
        let mut quotes = "```".to_string();
        let mut quote_ws = "".to_string();
        let mut view_code_fence_state = ViewCodeFenceState::Outside;
        const RUST_START: &str =
            "# ::leptos_reactive::create_scope(::leptos_reactive::create_runtime(), |cx| {";
        const RUST_END: &str = "# }).dispose();";
        const RSX_START: &str = "# ::tui_rsx::view! {cx,";
        const RSX_END: &str = "# };}).dispose();";

        // Seperated out of chain to allow rustfmt to work
        let map = |(doc, span): (String, Span)| {
            doc.lines()
                .flat_map(|doc| {
                    let trimmed_doc = doc.trim_start();
                    let leading_ws = &doc[..doc.len() - trimmed_doc.len()];
                    let trimmed_doc = trimmed_doc.trim_end();
                    match view_code_fence_state {
                        ViewCodeFenceState::Outside
                            if trimmed_doc.starts_with("```")
                                && trimmed_doc.trim_start_matches('`').starts_with("view") =>
                        {
                            view_code_fence_state = ViewCodeFenceState::Rust;
                            let view = trimmed_doc.find('v').unwrap();
                            quotes = trimmed_doc[..view].to_owned();
                            quote_ws = leading_ws.to_owned();
                            let rust_options = &trimmed_doc[view + "view".len()..].trim_start();
                            vec![
                                format!("{leading_ws}{quotes}{rust_options}"),
                                format!("{leading_ws}{RUST_START}"),
                            ]
                        }
                        ViewCodeFenceState::Rust if trimmed_doc == quotes => {
                            view_code_fence_state = ViewCodeFenceState::Outside;
                            vec![format!("{leading_ws}{RUST_END}"), doc.to_owned()]
                        }
                        ViewCodeFenceState::Rust if trimmed_doc.starts_with('<') => {
                            view_code_fence_state = ViewCodeFenceState::Rsx;
                            vec![format!("{leading_ws}{RSX_START}"), doc.to_owned()]
                        }
                        ViewCodeFenceState::Rsx if trimmed_doc == quotes => {
                            view_code_fence_state = ViewCodeFenceState::Outside;
                            vec![format!("{leading_ws}{RSX_END}"), doc.to_owned()]
                        }
                        _ => vec![doc.to_string()],
                    }
                })
                .map(|l| (l, span))
                .collect::<Vec<_>>()
        };

        let mut attrs = attrs
            .iter()
            .filter_map(|attr| {
                let Meta::NameValue(attr) = &attr.meta else {
                    return None;
                };
                if !attr.path.is_ident("doc") {
                    return None;
                }

                let Some(val) = value_to_string(&attr.value) else {
                    abort!(
                        attr,
                        "expected string literal in value of doc comment"
                    );
                };

                Some((val, attr.path.span()))
            })
            .flat_map(map)
            .collect::<Vec<_>>();

        if view_code_fence_state != ViewCodeFenceState::Outside {
            if view_code_fence_state == ViewCodeFenceState::Rust {
                attrs.push((format!("{quote_ws}{RUST_END}"), Span::call_site()))
            } else {
                attrs.push((format!("{quote_ws}{RSX_END}"), Span::call_site()))
            }
            attrs.push((format!("{quote_ws}{quotes}"), Span::call_site()))
        }

        Self(attrs)
    }

    pub fn padded(&self) -> TokenStream {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, (doc, span))| {
                let doc = if idx == 0 {
                    format!("    - {doc}")
                } else {
                    format!("      {doc}")
                };

                let doc = LitStr::new(&doc, *span);

                quote! { #[doc = #doc] }
            })
            .collect()
    }

    pub fn typed_builder(&self) -> String {
        let doc_str = self
            .0
            .iter()
            .map(|s| s.0.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        if doc_str.chars().filter(|c| *c != '\n').count() != 0 {
            format!("\n\n{doc_str}")
        } else {
            String::new()
        }
    }
}

#[derive(Clone, Debug, AttributeDerive, Default)]
#[attribute(ident = prop)]
struct PropOpt {
    #[attribute(conflicts = [optional_no_strip, strip_option])]
    optional: bool,
    #[attribute(conflicts = [optional, strip_option])]
    optional_no_strip: bool,
    #[attribute(conflicts = [optional, optional_no_strip])]
    strip_option: bool,
    #[attribute(example = "5 * 10")]
    default: Option<syn::Expr>,
    into: bool,
    children: bool,
}

struct TypedBuilderOpts {
    default: bool,
    default_with_value: Option<syn::Expr>,
    strip_option: bool,
    into: bool,
    children: bool,
}

impl TypedBuilderOpts {
    fn from_opts(opts: &PropOpt, is_ty_option: bool) -> Self {
        Self {
            default: opts.optional || opts.optional_no_strip,
            default_with_value: opts.default.clone(),
            strip_option: opts.strip_option || opts.optional && is_ty_option,
            into: opts.into,
            children: opts.children,
        }
    }
}

impl ToTokens for TypedBuilderOpts {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let default = if let Some(v) = &self.default_with_value {
            let v = v.to_token_stream().to_string();
            quote! { default_code=#v, }
        } else if self.default {
            quote! { default, }
        } else {
            quote! {}
        };

        let strip_option = if self.strip_option {
            quote! { strip_option, }
        } else {
            quote! {}
        };

        let into = if self.into {
            quote! { into, }
        } else {
            quote! {}
        };

        let setter = if !strip_option.is_empty() || !into.is_empty() {
            quote! { setter(#strip_option #into) }
        } else {
            quote! {}
        };

        if self.children {
            tokens.append_all(quote! {#[children]});
        }

        if default.is_empty() && setter.is_empty() {
            return;
        }

        let output = quote! { #[builder(#default #setter)] };

        tokens.append_all(output);
    }
}

fn prop_builder_fields(vis: &Visibility, props: &[Prop]) -> TokenStream {
    props
        .iter()
        .skip(1)
        .map(|prop| {
            let Prop {
                docs,
                name,
                prop_opts,
                ty,
            } = prop;
            let mut name = name.clone();
            name.mutability = None;
            let builder_attrs = TypedBuilderOpts::from_opts(prop_opts, is_option(ty));
            let builder_docs = prop_to_doc(prop, PropDocStyle::Inline);

            // Children won't need documentation in many cases
            let allow_missing_docs = if name.ident == "children" {
                quote!(#[allow(missing_docs)])
            } else {
                quote!()
            };

            quote! {
                #docs
                #builder_docs
                #builder_attrs
                #allow_missing_docs
                #vis #name: #ty,
            }
        })
        .collect()
}

fn prop_names(props: &[Prop]) -> TokenStream {
    let mut props: Vec<_> = props
        .iter()
        .skip(1)
        .map(|Prop { name, .. }| quote! { #name, })
        .collect();
    props.push(quote!(__caller_id));
    props.into_iter().collect()
}

fn prop_names_for_component(props: &[Prop]) -> TokenStream {
    props
        .iter()
        .skip(1)
        .filter(|Prop { name, .. }| {
            let name_str = name.ident.to_string();
            name_str != "_phantom"
        })
        .map(|Prop { name, .. }| {
            let mut name = name.clone();
            name.mutability = None;
            quote! { #name, }
        })
        .collect()
}

fn generate_component_fn_prop_docs(props: &[Prop]) -> TokenStream {
    let required_prop_docs = props
        .iter()
        .filter(|Prop { prop_opts, .. }| !(prop_opts.optional || prop_opts.optional_no_strip))
        .map(|p| prop_to_doc(p, PropDocStyle::List))
        .collect::<TokenStream>();

    let optional_prop_docs = props
        .iter()
        .filter(|Prop { prop_opts, .. }| prop_opts.optional || prop_opts.optional_no_strip)
        .map(|p| prop_to_doc(p, PropDocStyle::List))
        .collect::<TokenStream>();

    let required_prop_docs = if !required_prop_docs.is_empty() {
        quote! {
            #[doc = "# Required Props"]
            #required_prop_docs
        }
    } else {
        quote! {}
    };

    let optional_prop_docs = if !optional_prop_docs.is_empty() {
        quote! {
            #[doc = "# Optional Props"]
            #optional_prop_docs
        }
    } else {
        quote! {}
    };

    quote! {
        #required_prop_docs
        #optional_prop_docs
    }
}

pub fn is_option(ty: &Type) -> bool {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let [first] = &segments.iter().collect::<Vec<_>>()[..] {
            first.ident == "Option"
        } else {
            false
        }
    } else {
        false
    }
}

pub fn unwrap_option(ty: &Type) -> Type {
    const STD_OPTION_MSG: &str =
        "make sure you're not shadowing the `std::option::Option` type that \
         is automatically imported from the standard prelude";

    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let [first] = &segments.iter().collect::<Vec<_>>()[..] {
            if first.ident == "Option" {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = &first.arguments
                {
                    if let [GenericArgument::Type(ty)] = &args.iter().collect::<Vec<_>>()[..] {
                        return ty.clone();
                    }
                }
            }
        }
    }

    abort!(
        ty,
        "`Option` must be `std::option::Option`";
        help = STD_OPTION_MSG
    );
}

#[derive(Clone, Copy)]
enum PropDocStyle {
    List,
    Inline,
}

fn prop_to_doc(
    Prop {
        docs,
        name,
        ty,
        prop_opts,
    }: &Prop,
    style: PropDocStyle,
) -> TokenStream {
    let ty = if (prop_opts.optional || prop_opts.strip_option) && is_option(ty) {
        unwrap_option(ty)
    } else {
        ty.to_owned()
    };

    let type_item: syn::Item = parse_quote! {
        type SomeType = #ty;
    };

    let file = syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![type_item],
    };

    let pretty_ty = prettyplease::unparse(&file);

    let pretty_ty = &pretty_ty[16..&pretty_ty.len() - 2];

    match style {
        PropDocStyle::List => {
            let arg_ty_doc = LitStr::new(
                &if !prop_opts.into {
                    format!("- **{}**: [`{pretty_ty}`]", quote!(#name))
                } else {
                    format!(
                        "- **{}**: [`impl Into<{pretty_ty}>`]({pretty_ty})",
                        quote!(#name),
                    )
                },
                name.ident.span(),
            );

            let arg_user_docs = docs.padded();

            quote! {
                #[doc = #arg_ty_doc]
                #arg_user_docs
            }
        }
        PropDocStyle::Inline => {
            let arg_ty_doc = LitStr::new(
                &if !prop_opts.into {
                    format!(
                        "**{}**: [`{}`]{}",
                        quote!(#name),
                        pretty_ty,
                        docs.typed_builder()
                    )
                } else {
                    format!(
                        "**{}**: `impl`[`Into<{}>`]{}",
                        quote!(#name),
                        pretty_ty,
                        docs.typed_builder()
                    )
                },
                name.ident.span(),
            );

            quote! {
                #[builder(setter(doc = #arg_ty_doc))]
            }
        }
    }
}

fn value_to_string(value: &syn::Expr) -> Option<String> {
    match &value {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => Some(s.value()),
            syn::Lit::Char(c) => Some(c.value().to_string()),
            syn::Lit::Int(i) => Some(i.base10_digits().to_string()),
            syn::Lit::Float(f) => Some(f.base10_digits().to_string()),
            _ => None,
        },
        _ => None,
    }
}
