use std::ops::Deref;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenTree};
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::{quote, ToTokens, TokenStreamExt};
use rstml::node::{Node, NodeAttribute, NodeElement};
use syn::{Block, Expr, ExprLit, Lit, LitInt};

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
        state: Option<proc_macro2::TokenStream>,
    },
    Block(proc_macro2::TokenStream),
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
            ViewType::Block(tokens) => {
                if let Some(i) = i {
                    quote! { (#tokens)(f, chunks[#i]) }
                } else {
                    tokens.to_owned()
                }
            }
            ViewType::Element { name, props, state } => match (props, state, i) {
                (Some(props), Some(state), Some(i)) => {
                    quote! { #name(f, chunks[#i], #props, #state); }
                }
                (Some(props), None, Some(i)) => {
                    quote! { #name(f, chunks[#i], #props); }
                }
                (Some(props), Some(state), None) => {
                    quote! {
                        |f: &mut Frame<_>, rect: Rect| {
                            #name(f, rect, #props, #state);
                        }
                    }
                }
                (Some(props), None, None) => {
                    quote! {
                        |f: &mut Frame<_>, rect: Rect| {
                            #name(f, rect, #props);
                        }
                    }
                }
                (None, _, Some(i)) => {
                    quote! { #name(f, chunks[#i]); }
                }
                (None, _, None) => {
                    quote! { #name }
                }
            },
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
    state: Option<proc_macro2::TokenStream>,
}

impl NodeAttributes {
    fn from_custom(
        element: &NodeElement,
        children: proc_macro2::TokenStream,
        object_suffix: &str,
    ) -> Self {
        Self::from_nodes(
            Some(&snake_case_to_pascal_case(&element.name().to_string())),
            element.attributes(),
            if children.is_empty() {
                None
            } else {
                Some(children)
            },
            object_suffix,
        )
    }
    fn from_nodes(
        tag_name: Option<&str>,
        nodes: &[NodeAttribute],
        args: Option<proc_macro2::TokenStream>,
        object_suffix: &str,
    ) -> Self {
        let mut attrs = Self {
            constraint: Constraint::Min,
            expr: get_default_constraint(),
            props: None,
            state: None,
        };

        let mut attribute_parsed = false;
        for node in nodes {
            if let NodeAttribute::Attribute(attribute) = node {
                match attribute.key.to_string().as_str() {
                    "min" => {
                        attrs.constraint = Constraint::Min;
                        attrs.expr = attribute.value().unwrap().clone();
                    }
                    "max" => {
                        attrs.constraint = Constraint::Max;
                        attrs.expr = attribute.value().unwrap().clone();
                    }
                    "percentage" => {
                        attrs.constraint = Constraint::Percentage;
                        attrs.expr = attribute.value().unwrap().clone();
                    }
                    "length" => {
                        attrs.constraint = Constraint::Length;
                        attrs.expr = attribute.value().unwrap().clone();
                    }
                    "state" => {
                        if let Some(val) = &attribute.value() {
                            let val = val.deref();
                            attrs.state = Some(val.to_token_stream());
                        }
                    }
                    name => {
                        attribute_parsed = true;
                        let func_name = Ident::new(name, Span::call_site());
                        if let Some(tag_name) = tag_name {
                            if let Some(val) = &attribute.value() {
                                let val = val.deref();
                                if let Some(props) = attrs.props {
                                    attrs.props = Some(quote! {
                                        #props.#func_name(#val)
                                    });
                                } else {
                                    let props = build_struct(tag_name, &args, object_suffix);
                                    attrs.props = Some(quote! { #props.#func_name(#val) });
                                }
                            } else if name == "default" {
                                attrs.props = Some(build_struct(tag_name, &args, object_suffix));
                            }
                        }
                    }
                }
            }
        }

        if let Some(tag_name) = tag_name {
            let should_add_props = !attribute_parsed && (args.is_some() || attrs.state.is_some());
            if should_add_props {
                attrs.props = Some(build_struct(tag_name, &args, object_suffix));
            }
        }

        attrs
    }
}

fn build_struct(
    tag_name: &str,
    args: &Option<proc_macro2::TokenStream>,
    object_suffix: &str,
) -> proc_macro2::TokenStream {
    let object = capitalize(tag_name) + object_suffix;
    let ident = Ident::new(&object, Span::call_site());
    if let Some(args) = args.as_ref() {
        quote! {
            #ident::new(#args)
        }
    } else {
        quote! {
            #ident::default()
        }
    }
}

#[proc_macro]
#[proc_macro_error]
pub fn prop(tokens: TokenStream) -> TokenStream {
    match rstml::parse(tokens) {
        Ok(nodes) => parse_named_element_children(&nodes),
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro]
#[proc_macro_error]
pub fn view(tokens: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = tokens.into();
    let mut tokens = tokens.into_iter().peekable();
    let mut set_move = false;
    if let Some(TokenTree::Ident(ident)) = tokens.peek() {
        if *ident == "move" {
            set_move = true;
            tokens.next();
        }
    }
    match rstml::parse2(tokens.collect()) {
        Ok(nodes) => {
            let view = parse_root_nodes(nodes);
            if set_move {
                quote! { move #view }
            } else {
                quote! { #view }
            }
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
        match node {
            Node::Element(element) => {
                views.push(parse_element(element));
            }
            Node::Block(block) => {
                if let Some(block) = block.try_block() {
                    let content = get_block_contents(block);
                    views.push(View {
                        view_type: ViewType::Block(content),
                        constraint: Constraint::Min,
                        constraint_val: get_default_constraint(),
                    })
                }
            }
            node => {
                abort_call_site!(format!("Invalid RSX node: {node:?}"));
            }
        }
    }
    views
}

fn parse_named_element_children(nodes: &[Node]) -> proc_macro2::TokenStream {
    let mut tokens = vec![];
    let mut force_vec = false;
    for node in nodes {
        match node {
            Node::Element(element) => {
                let children = parse_named_element_children(&element.children);
                let attrs = NodeAttributes::from_custom(element, children, "");

                if let Some(props) = attrs.props {
                    tokens.push(quote! { #props });
                }
            }
            Node::Text(text) => {
                tokens.push(text.value.to_token_stream());
            }
            Node::Block(block) => {
                if let Some(block) = block.try_block() {
                    // Get content without braces
                    let content: proc_macro2::TokenStream =
                        block.stmts.iter().map(|s| s.to_token_stream()).collect();

                    tokens.push(quote! { #content });
                }
            }
            Node::Doctype(_) => {
                abort_call_site!("Doctype invalid at this location");
            }
            // Node::Attribute(_) => {
            //     abort_call_site!("Attribute invalid at this location");
            // }
            Node::Fragment(fragment) => {
                let children = parse_named_element_children(&fragment.children);
                tokens.push(children);
                force_vec = true;
            }
            _ => {}
        }
    }
    if tokens.is_empty() {
        proc_macro2::TokenStream::default()
    } else if tokens.len() == 1 && !force_vec {
        tokens[0].clone()
    } else {
        quote! { vec![#(#tokens),*] }
    }
}

fn parse_element(element: &NodeElement) -> View {
    match element.name().to_string().as_str() {
        "Row" | "row" => {
            let children = parse_elements(&element.children);
            let attrs = NodeAttributes::from_nodes(None, element.attributes(), None, "Props");
            View {
                view_type: ViewType::Row(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
            }
        }
        "Column" | "column" => {
            let children = parse_elements(&element.children);
            let attrs = NodeAttributes::from_nodes(None, element.attributes(), None, "Props");
            View {
                view_type: ViewType::Column(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
            }
        }
        name => {
            let children = parse_named_element_children(&element.children);
            let attrs = NodeAttributes::from_custom(element, children, "Props");
            View {
                view_type: ViewType::Element {
                    name: Ident::new(name, Span::call_site()),
                    props: attrs.props,
                    state: attrs.state,
                },
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
            }
        }
    }
}

fn capitalize(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
}

fn snake_case_to_pascal_case(s: &str) -> String {
    s.split('_').map(capitalize).collect::<Vec<_>>().join("")
}

fn get_block_contents(block: &Block) -> proc_macro2::TokenStream {
    block.stmts.iter().map(|s| s.to_token_stream()).collect()
}

fn get_default_constraint() -> Expr {
    Expr::Lit(ExprLit {
        lit: Lit::Int(LitInt::new("0", Span::call_site())),
        attrs: vec![],
    })
}
