use std::ops::Deref;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Expr, ExprLit, Lit, LitInt};
use syn_rsx::{Node, NodeElement};

#[derive(Clone)]
enum Constraint {
    Min,
    Max,
    Percentage,
    Length,
}

#[derive(Clone)]
enum ViewType {
    Row(Vec<View>),
    Column(Vec<View>),
    Element {
        name: Ident,
        props: Option<proc_macro2::TokenStream>,
    },
}

#[derive(Clone)]
struct View {
    view_type: ViewType,
    constraint: Constraint,
    constraint_val: Expr,
}

impl View {
    fn get_view_constraint(&self) -> proc_macro2::TokenStream {
        let constraint_val = &self.constraint_val;
        match self.constraint {
            Constraint::Min => quote! { Constraint::Min(#constraint_val) },
            Constraint::Max => quote! { Constraint::Min(#constraint_val) },
            Constraint::Percentage => quote! { Constraint::Percentage(#constraint_val) },
            Constraint::Length => quote! { Constraint::Length(#constraint_val) },
        }
    }

    fn get_layout_tokens(
        &self,
        direction: proc_macro2::TokenStream,
        children: &[View],
        i: Option<usize>,
    ) -> proc_macro2::TokenStream {
        let constraints: Vec<_> = children.iter().map(|c| c.get_view_constraint()).collect();
        let child_tokens: Vec<_> = children
            .iter()
            .enumerate()
            .map(|(i, v)| v.view_to_tokens(Some(i)))
            .collect();

        let layout_tokens = quote! {
            |f: &mut Frame<_>, rect: Rect| {
                let layout = Layout::default().direction(#direction);
                let chunks = layout
                    .constraints([#(#constraints),*])
                    .split(rect);
                #(#child_tokens)*
            }
        };
        if let Some(i) = i {
            quote!((#layout_tokens)(f, chunks[#i]);)
        } else {
            layout_tokens
        }
    }

    fn view_to_tokens(&self, i: Option<usize>) -> proc_macro2::TokenStream {
        match &self.view_type {
            ViewType::Row(children) => {
                self.get_layout_tokens(quote! {Direction::Horizontal}, children, i)
            }
            ViewType::Column(children) => {
                self.get_layout_tokens(quote! {Direction::Vertical}, children, i)
            }
            ViewType::Element { name, props } => {
                if let Some(props) = props {
                    quote! { #name(f, chunks[#i], #props); }
                } else {
                    quote! { #name(f, chunks[#i]); }
                }
            }
        }
    }
}

impl ToTokens for View {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(self.view_to_tokens(None));
    }
}

struct NodeAttributes {
    constraint: Constraint,
    expr: Expr,
    props: Option<proc_macro2::TokenStream>,
}

impl NodeAttributes {
    fn from_nodes(tag_name: Option<&str>, nodes: &[Node]) -> Self {
        let mut attrs = Self {
            constraint: Constraint::Min,
            expr: Expr::Lit(ExprLit {
                lit: Lit::Int(LitInt::new("0", Span::call_site())),
                attrs: vec![],
            }),
            props: None,
        };

        for node in nodes {
            if let Node::Attribute(attribute) = node {
                match attribute.key.to_string().as_str() {
                    "min" => {
                        attrs.constraint = Constraint::Min;
                        attrs.expr = attribute.value.as_deref().unwrap().clone();
                    }
                    "max" => {
                        attrs.constraint = Constraint::Max;
                        attrs.expr = attribute.value.as_deref().unwrap().clone();
                    }
                    "percentage" => {
                        attrs.constraint = Constraint::Percentage;
                        attrs.expr = attribute.value.as_deref().unwrap().clone();
                    }
                    "length" => {
                        attrs.constraint = Constraint::Length;
                        attrs.expr = attribute.value.as_deref().unwrap().clone();
                    }
                    name => {
                        if let Some(tag_name) = tag_name {
                            let func_name = Ident::new(name, Span::call_site());
                            if let Some(val) = &attribute.value {
                                let val = val.deref();
                                if let Some(props) = attrs.props {
                                    attrs.props = Some(quote! {
                                        #props.#func_name(#val)
                                    });
                                } else {
                                    let object =
                                        tag_name[0..1].to_uppercase() + &tag_name[1..] + "Props";
                                    let ident = Ident::new(&object, Span::call_site());
                                    attrs.props = Some(quote! {
                                        #ident::default().#func_name(#val)
                                    });
                                }
                            }
                        }
                    }
                }
            } else {
            }
        }

        attrs
    }
}

#[proc_macro]
#[proc_macro_error]
pub fn rsx(tokens: TokenStream) -> TokenStream {
    match syn_rsx::parse(tokens) {
        Ok(nodes) => {
            let view = parse_root_nodes(nodes);
            quote! { #view }
        }
        Err(e) => e.to_compile_error(),
    }
    .into()
}

fn parse_root_nodes(nodes: Vec<Node>) -> View {
    if let [node] = &nodes[..] {
        parse_root_node(node)
    } else {
        abort_call_site!("RSX should contain a single root node");
    }
}

fn parse_root_node(node: &Node) -> View {
    if let Node::Element(element) = node {
        parse_element(element)
    } else {
        abort_call_site!("RSX root node shoule be a named element");
    }
}

fn parse_elements(nodes: &[Node]) -> Vec<View> {
    let mut views = vec![];
    for node in nodes {
        if let Node::Element(element) = node {
            views.push(parse_element(element));
        } else {
            abort_call_site!("RSX root node shoule be a named element");
        }
    }
    views
}

fn parse_element(element: &NodeElement) -> View {
    match element.name.to_string().as_str() {
        "Row" => {
            let children = parse_elements(&element.children);
            let attrs = NodeAttributes::from_nodes(None, &element.attributes);
            View {
                view_type: ViewType::Row(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
            }
        }
        "Column" => {
            let children = parse_elements(&element.children);
            let attrs = NodeAttributes::from_nodes(None, &element.attributes);
            View {
                view_type: ViewType::Column(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
            }
        }
        name => {
            let attrs = NodeAttributes::from_nodes(Some(name), &element.attributes);
            View {
                view_type: ViewType::Element {
                    name: Ident::new(name, Span::call_site()),
                    props: attrs.props,
                },
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
            }
        }
    }
}
