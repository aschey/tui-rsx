use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort_call_site;
use quote::{quote, ToTokens, TokenStreamExt};
use rstml::node::KeyedAttribute;
use rstml::node::{Node, NodeAttribute, NodeElement};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use syn::{Block, Expr, ExprLit, Lit, LitInt};

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug)]
enum Constraint {
    Min,
    Max,
    Percentage,
    Length,
}

#[derive(Clone, Debug)]
enum ViewType {
    Row(Vec<View>),
    Column(Vec<View>),
    Overlay(Vec<View>),
    Element {
        name: Ident,
        fn_name: Ident,
        props: Option<TokenStream>,
        state: Option<TokenStream>,
    },
    Block {
        fn_name: Ident,
        tokens: TokenStream,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct View {
    view_type: ViewType,
    constraint: Constraint,
    constraint_val: Expr,
    layout_props: Option<TokenStream>,
    create_dummy_parent: bool,
}

impl View {
    fn get_view_constraint(&self) -> TokenStream {
        let constraint_val = &self.constraint_val;

        match self.constraint {
            Constraint::Min => quote! { Constraint::Min(#constraint_val) },
            Constraint::Max => quote! { Constraint::Max(#constraint_val) },
            Constraint::Percentage => quote! { Constraint::Percentage(#constraint_val) },
            Constraint::Length => quote! { Constraint::Length(#constraint_val) },
        }
    }

    fn get_overlay_tokens(&self, children: &[View], is_child: bool) -> TokenStream {
        let fn_clones = self.generate_fn_clones();
        let child_tokens: Vec<_> = children
            .iter()
            .enumerate()
            .map(|(i, v)| v.view_to_tokens(Some(i), true))
            .collect();
        let layout_tokens = quote! {
            move |f: &mut Frame<_>, rect: Rect| {
                #fn_clones
                #(#child_tokens)*
            }
        };

        if is_child {
            quote!((#layout_tokens).view(f, rect);)
        } else {
            layout_tokens
        }
    }

    fn get_layout_tokens(
        &self,
        direction: TokenStream,
        children: &[View],
        child_index: Option<usize>,
        parent_is_overlay: bool,
    ) -> TokenStream {
        let constraints: Vec<_> = children.iter().map(|c| c.get_view_constraint()).collect();

        let child_tokens: Vec<_> = children
            .iter()
            .enumerate()
            .map(|(i, v)| v.view_to_tokens(Some(i), false))
            .collect();
        let layout_props = self.layout_props.clone();
        let fn_clones = self.generate_fn_clones();

        let layout_tokens = quote! {
            move |f: &mut Frame<_>, rect: Rect| {
                #fn_clones
                let layout = Layout::default().direction(#direction);
                let chunks = layout
                    .constraints([#(#constraints),*])
                    #layout_props
                    .split(rect);
                #(#child_tokens)*
            }
        };

        if let Some(child_index) = child_index {
            if parent_is_overlay {
                quote!((#layout_tokens).view(f, rect);)
            } else {
                quote!((#layout_tokens).view(f, chunks[#child_index]);)
            }
        } else {
            layout_tokens
        }
    }

    fn generate_fn_clones(&self) -> TokenStream {
        match &self.view_type {
            ViewType::Row(children) | ViewType::Column(children) | ViewType::Overlay(children) => {
                let child_fns: Vec<_> = children.iter().map(|c| c.generate_fn_clones()).collect();
                quote! { #(#child_fns)* }
            }
            ViewType::Block { fn_name, .. } => {
                quote! {
                    let mut #fn_name = #fn_name.clone();
                }
            }
            ViewType::Element { fn_name, .. } => {
                quote! {
                    let mut #fn_name = #fn_name.clone();
                }
            }
        }
    }

    fn generate_fns(&self) -> TokenStream {
        match &self.view_type {
            ViewType::Row(children) | ViewType::Column(children) | ViewType::Overlay(children) => {
                let child_fns: Vec<_> = children.iter().map(|c| c.generate_fns()).collect();
                quote! { #(#child_fns)* }
            }
            ViewType::Block { fn_name, tokens } => {
                quote! {
                    let mut #fn_name = ::std::rc::Rc::new(::std::cell::RefCell::new(
                        move |f: &mut Frame<_>, chunks: Rect| #tokens.view(f, chunks)));
                }
            }
            ViewType::Element {
                name,
                fn_name,
                props,
                state,
            } => match (props, state) {
                (Some(props), Some(state)) => {
                    quote! { let mut #fn_name = ::std::rc::Rc::new(::std::cell::RefCell::new(#name(#props, #state))); }
                }
                (Some(props), None) => {
                    quote! { let mut #fn_name = ::std::rc::Rc::new(::std::cell::RefCell::new(#name(#props))); }
                }
                (_, _) => {
                    quote! { let mut #fn_name = ::std::rc::Rc::new(::std::cell::RefCell::new(#name())); }
                }
            },
        }
    }

    fn view_to_tokens(&self, child_index: Option<usize>, parent_is_overlay: bool) -> TokenStream {
        match &self.view_type {
            ViewType::Row(children) => self.get_layout_tokens(
                quote! {Direction::Horizontal},
                children,
                child_index,
                parent_is_overlay,
            ),
            ViewType::Column(children) => self.get_layout_tokens(
                quote! {Direction::Vertical},
                children,
                child_index,
                parent_is_overlay,
            ),
            ViewType::Overlay(children) => self.get_overlay_tokens(children, child_index.is_some()),
            ViewType::Block { fn_name, .. } | ViewType::Element { fn_name, .. } => {
                if let Some(child_index) = child_index {
                    if parent_is_overlay {
                        quote! { (#fn_name).view(f, rect); }
                    } else {
                        quote! { (#fn_name).view(f, chunks[#child_index]); }
                    }
                } else {
                    quote! { (#fn_name) }
                }
            }
        }
    }
}

impl ToTokens for View {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fns = self.generate_fns();
        let view = self.view_to_tokens(None, false);
        let dummy_parent = if self.create_dummy_parent {
            quote!(let __parent_id = 0;)
        } else {
            quote!()
        };
        tokens.append_all(quote! {
            {
                #dummy_parent
                #fns
                #view
            }
        });
    }
}

struct NodeAttributes {
    constraint: Constraint,
    expr: Expr,
    props: Option<TokenStream>,
    state: Option<TokenStream>,
    key: Option<Expr>,
}

impl NodeAttributes {
    fn from_custom(
        cx_name: Option<&TokenStream>,
        element: &NodeElement,
        children: TokenStream,
        object_suffix: &str,
        include_parent_id: bool,
    ) -> Self {
        Self::from_nodes(
            cx_name,
            Some(&snake_case_to_pascal_case(&element.name().to_string())),
            element.attributes(),
            if children.is_empty() {
                None
            } else {
                Some(children)
            },
            object_suffix,
            include_parent_id,
        )
    }

    fn parse_standard_attrs(&mut self, attribute: &KeyedAttribute) -> bool {
        match attribute.key.to_string().as_str() {
            "min" => {
                self.constraint = Constraint::Min;
                self.expr = attribute.value().unwrap().clone();
                true
            }
            "max" => {
                self.constraint = Constraint::Max;
                self.expr = attribute.value().unwrap().clone();
                true
            }
            "percentage" => {
                self.constraint = Constraint::Percentage;
                self.expr = attribute.value().unwrap().clone();
                true
            }
            "length" => {
                self.constraint = Constraint::Length;
                self.expr = attribute.value().unwrap().clone();
                true
            }
            "state" => {
                if let Some(val) = &attribute.value() {
                    self.state = Some(val.to_token_stream());
                }
                true
            }
            "key" => {
                self.key = Some(attribute.value().unwrap().clone());
                true
            }
            _ => false,
        }
    }

    fn from_nodes(
        cx_name: Option<&TokenStream>,
        tag_name: Option<&str>,
        nodes: &[NodeAttribute],
        args: Option<TokenStream>,
        object_suffix: &str,
        include_parent_id: bool,
    ) -> Self {
        let mut attrs = Self {
            constraint: Constraint::Min,
            expr: get_default_constraint(),
            props: None,
            state: None,
            key: None,
        };

        let custom_attrs: Vec<_> = nodes
            .iter()
            .filter_map(|node| {
                if let NodeAttribute::Attribute(attribute) = node {
                    if !attrs.parse_standard_attrs(attribute) {
                        return Some(attribute);
                    }
                }
                None
            })
            .collect();

        for attribute in &custom_attrs {
            let func_name = Ident::new(&attribute.key.to_string(), Span::call_site());
            if let Some(tag_name) = tag_name {
                let val = if let Some(val) = &attribute.value() {
                    quote!(#val)
                } else {
                    quote!()
                };

                if let Some(props) = attrs.props {
                    attrs.props = Some(quote! {
                        #props.#func_name(#val)
                    });
                } else {
                    let props = build_struct(
                        tag_name,
                        &args,
                        object_suffix,
                        include_parent_id,
                        attrs.key.clone(),
                    );
                    if let Some(cx_name) = cx_name {
                        attrs.props = Some(quote! { #cx_name.clone(), #props.#func_name(#val) });
                    } else {
                        attrs.props = Some(quote! { #props.#func_name(#val) });
                    }
                }
            }
        }

        if let Some(props) = &attrs.props {
            attrs.props = Some(quote! { #props.build() });
        }

        if let Some(tag_name) = tag_name {
            if custom_attrs.is_empty() {
                let props = build_struct(
                    tag_name,
                    &args,
                    object_suffix,
                    include_parent_id,
                    attrs.key.clone(),
                );
                if let Some(cx_name) = cx_name {
                    attrs.props = Some(quote! { #cx_name.clone(), #props.build() });
                } else {
                    attrs.props = Some(quote! { #props.build() });
                }
            }
        }

        attrs
    }

    fn from_layout_nodes(nodes: &[NodeAttribute]) -> Self {
        let mut attrs = Self {
            constraint: Constraint::Min,
            expr: get_default_constraint(),
            props: None,
            state: None,
            key: None,
        };

        // let mut attribute_parsed = false;
        for node in nodes {
            if let NodeAttribute::Attribute(attribute) = node {
                if !attrs.parse_standard_attrs(attribute) {
                    let func_name = Ident::new(&attribute.key.to_string(), Span::call_site());
                    if let Some(val) = &attribute.value() {
                        if let Some(props) = attrs.props {
                            attrs.props = Some(quote! {
                                #props.#func_name(#val)
                            });
                        } else {
                            attrs.props = Some(quote! {.#func_name(#val)});
                        }
                    }
                }
            }
        }

        attrs
    }
}

fn build_struct(
    tag_name: &str,
    args: &Option<TokenStream>,
    object_suffix: &str,
    include_parent_id: bool,
    key: Option<Expr>,
) -> TokenStream {
    let object = capitalize(tag_name) + object_suffix;
    let ident = Ident::new(&object, Span::call_site());
    let caller_id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let key_clause = key.map(|k| quote!(+ &#k.to_string()));
    let caller_id_args = if include_parent_id {
        quote!((__parent_id.to_string() + &#caller_id.to_string() #key_clause).parse().expect("invalid integer"))
    } else if key_clause.is_some() {
        quote!((#caller_id.to_string() #key_clause).parse().expect("invalid integer"))
    } else {
        quote!(#caller_id)
    };
    if let Some(args) = args.as_ref() {
        quote! {
            #ident::new(#args).__caller_id(#caller_id_args)
        }
    } else {
        quote! {
            #ident::builder().__caller_id(#caller_id_args)
        }
    }
}

pub(crate) fn view(tokens: TokenStream, include_parent_id: bool) -> TokenStream {
    let mut tokens = tokens.into_iter().peekable();
    let cx_token = if tokens.peek().unwrap().to_string() != "<" {
        let token = tokens.next().unwrap().to_token_stream();
        let _comma = tokens.next().unwrap();
        token
    } else {
        quote! { () }
    };

    match rstml::parse2(tokens.collect()) {
        Ok(nodes) => {
            let mut view = parse_root_nodes(&cx_token, nodes, include_parent_id);
            view.create_dummy_parent = !include_parent_id;
            view.to_token_stream()
        }
        Err(e) => e.to_compile_error(),
    }
}

fn parse_root_nodes(cx_name: &TokenStream, nodes: Vec<Node>, include_parent_id: bool) -> View {
    if let [node] = &nodes[..] {
        parse_root_node(cx_name, node, include_parent_id)
    } else {
        abort_call_site!(format!("RSX should contain a single root node"));
    }
}

fn parse_root_node(cx_name: &TokenStream, node: &Node, include_parent_id: bool) -> View {
    if let Node::Element(element) = node {
        parse_element(cx_name, element, include_parent_id)
    } else {
        abort_call_site!("RSX root node should be a named element");
    }
}

fn parse_elements(cx_name: &TokenStream, nodes: &[Node], include_parent_id: bool) -> Vec<View> {
    let mut views = vec![];
    for node in nodes {
        match node {
            Node::Element(element) => {
                views.push(parse_element(cx_name, element, include_parent_id));
            }
            Node::Block(block) => {
                if let Some(block) = block.try_block() {
                    let content = get_block_contents(block);
                    views.push(View {
                        view_type: ViewType::Block {
                            tokens: content,
                            fn_name: Ident::new(
                                &format!("__fn{}", NEXT_ID.fetch_add(1, Ordering::SeqCst)),
                                Span::call_site(),
                            ),
                        },
                        constraint: Constraint::Min,
                        constraint_val: get_default_constraint(),
                        create_dummy_parent: false,
                        layout_props: None,
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

pub(crate) fn parse_named_element_children(nodes: &[Node], include_parent_id: bool) -> TokenStream {
    let mut tokens = vec![];
    let mut force_vec = false;
    for node in nodes {
        match node {
            Node::Element(element) => {
                let children = parse_named_element_children(&element.children, include_parent_id);
                let attrs =
                    NodeAttributes::from_custom(None, element, children, "", include_parent_id);

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
                    let content: TokenStream =
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
                let children = parse_named_element_children(&fragment.children, include_parent_id);
                tokens.push(children);
                force_vec = true;
            }
            _ => {}
        }
    }
    if tokens.is_empty() {
        TokenStream::default()
    } else if tokens.len() == 1 && !force_vec {
        tokens[0].clone()
    } else {
        quote! { vec![#(#tokens),*] }
    }
}

fn parse_element(cx_name: &TokenStream, element: &NodeElement, include_parent_id: bool) -> View {
    match element.name().to_string().as_str() {
        "row" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes());
            let children = parse_elements(cx_name, &element.children, include_parent_id);

            View {
                view_type: ViewType::Row(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                create_dummy_parent: false,
                layout_props: attrs.props,
            }
        }
        "column" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes());
            let children = parse_elements(cx_name, &element.children, include_parent_id);

            View {
                view_type: ViewType::Column(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                create_dummy_parent: false,
                layout_props: attrs.props,
            }
        }
        "overlay" => {
            let attrs = NodeAttributes::from_layout_nodes(element.attributes());
            let children = parse_elements(cx_name, &element.children, include_parent_id);

            View {
                view_type: ViewType::Overlay(children),
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                create_dummy_parent: false,
                layout_props: attrs.props,
            }
        }
        name => {
            let children = parse_named_element_children(&element.children, include_parent_id);
            let attrs = NodeAttributes::from_custom(
                Some(cx_name),
                element,
                children,
                "Props",
                include_parent_id,
            );
            View {
                view_type: ViewType::Element {
                    name: Ident::new(name, Span::call_site()),
                    fn_name: Ident::new(
                        &format!("__fn{}", NEXT_ID.fetch_add(1, Ordering::SeqCst)),
                        Span::call_site(),
                    ),
                    props: attrs.props,
                    state: attrs.state,
                },
                constraint: attrs.constraint,
                constraint_val: attrs.expr,
                create_dummy_parent: false,
                layout_props: None,
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

fn get_block_contents(block: &Block) -> TokenStream {
    block.stmts.iter().map(|s| s.to_token_stream()).collect()
}

fn get_default_constraint() -> Expr {
    Expr::Lit(ExprLit {
        lit: Lit::Int(LitInt::new("0", Span::call_site())),
        attrs: vec![],
    })
}
