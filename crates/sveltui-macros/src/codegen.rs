use crate::ast::{
    AwaitNode, EachNode, Element, IfCondition, IfNode, Node, RenderNode, SnippetNode,
};
use quote::{quote, quote_spanned};
use syn::Expr;
use syn::spanned::Spanned;

fn element_root() -> proc_macro2::TokenStream {
    quote! { ::sveltui::element }
}

pub fn codegen_node(node: &Node) -> proc_macro2::TokenStream {
    let root = element_root();
    match node {
        Node::Element(el) => codegen_element(el),
        Node::Text(lit) => {
            quote! {
                #root::IntoNode::into_node(#lit)
            }
        }
        Node::Block(expr) => {
            quote_spanned! { expr.span() =>
                #root::IntoNode::into_node(#expr)
            }
        }
        Node::If(if_node) => codegen_if(if_node),
        Node::Each(each_node) => codegen_each(each_node),
        Node::Await(await_node) => codegen_await(await_node),
        Node::Snippet(snippet_node) => codegen_snippet(snippet_node),
        Node::Render(render_node) => codegen_render(render_node),
    }
}

pub fn codegen_nodes(nodes: &[Node]) -> proc_macro2::TokenStream {
    let root = element_root();
    if nodes.is_empty() {
        quote! { #root::VNode::Empty }
    } else if nodes.len() == 1 {
        codegen_node(&nodes[0])
    } else {
        let streams = nodes.iter().map(codegen_node);
        quote! {
            #root::VNode::Fragment(vec![
                #(#streams),*
            ])
        }
    }
}

pub fn codegen_node_refs(nodes: &[&Node]) -> proc_macro2::TokenStream {
    let root = element_root();
    if nodes.is_empty() {
        quote! { #root::VNode::Empty }
    } else if nodes.len() == 1 {
        codegen_node(nodes[0])
    } else {
        let streams = nodes.iter().map(|n| codegen_node(n));
        quote! {
            #root::VNode::Fragment(vec![
                #(#streams),*
            ])
        }
    }
}

fn codegen_if(if_node: &IfNode) -> proc_macro2::TokenStream {
    let root = element_root();
    let initial_cond = match &if_node.initial.condition {
        IfCondition::Expr(expr) => quote! { #expr },
        IfCondition::Let { pat, expr } => quote! { let #pat = #expr },
    };
    let initial_body = codegen_nodes(&if_node.initial.body);

    let mut else_ifs = Vec::new();
    for branch in &if_node.else_ifs {
        let cond = match &branch.condition {
            IfCondition::Expr(expr) => quote! { #expr },
            IfCondition::Let { pat, expr } => quote! { let #pat = #expr },
        };
        let body = codegen_nodes(&branch.body);
        else_ifs.push(quote! {
            else if #cond {
                #body
            }
        });
    }

    let else_branch = if let Some(body) = &if_node.else_branch {
        let body_ts = codegen_nodes(body);
        quote! {
            else {
                #body_ts
            }
        }
    } else {
        quote! {
            else {
                #root::VNode::Empty
            }
        }
    };

    quote! {
        #root::IntoNode::into_node(
            if #initial_cond {
                #initial_body
            } #(#else_ifs)* #else_branch
        )
    }
}

fn codegen_each(each_node: &EachNode) -> proc_macro2::TokenStream {
    let root = element_root();
    let expr = &each_node.expr;
    let pat = &each_node.pat;
    let _key = &each_node.key;
    let body_ts = codegen_nodes(&each_node.body);

    let else_ts = if let Some(else_branch) = &each_node.else_branch {
        codegen_nodes(else_branch)
    } else {
        quote! { #root::VNode::Empty }
    };

    quote! {
        {
            let mut __items = Vec::new();
            for #pat in (#expr).into_iter() {
                __items.push(#body_ts);
            }
            if __items.is_empty() {
                #root::IntoNode::into_node(#else_ts)
            } else {
                #root::VNode::Fragment(__items)
            }
        }
    }
}

fn codegen_render(render_node: &RenderNode) -> proc_macro2::TokenStream {
    let root = element_root();
    let target = &render_node.target;
    match target {
        Expr::Call(call) => {
            let func = &call.func;
            let args = &call.args;
            quote! {
                #root::IntoNode::into_node((#func).render(#args))
            }
        }
        Expr::MethodCall(m) => {
            let receiver = &m.receiver;
            let method = &m.method;
            let args = &m.args;
            quote! {
                #root::IntoNode::into_node(((#receiver).#method).render(#args))
            }
        }
        _ => {
            quote! {
                #root::IntoNode::into_node((#target).render(()))
            }
        }
    }
}

fn codegen_snippet(snippet_node: &SnippetNode) -> proc_macro2::TokenStream {
    let root = element_root();
    let body_ts = codegen_nodes(&snippet_node.body);
    let args = &snippet_node.args;
    let param = if args.is_empty() {
        quote! { () }
    } else if args.len() == 1 {
        let first = &args[0];
        quote! { #first }
    } else {
        quote! { (#(#args),*) }
    };

    quote! {
        #root::Snippet::new(move |#param| {
            #body_ts
        })
    }
}

fn codegen_await(await_node: &AwaitNode) -> proc_macro2::TokenStream {
    let root = element_root();
    let future = &await_node.future;
    let loading_ts = codegen_nodes(&await_node.loading);

    let then_body = if let Some((pat, body)) = &await_node.then_branch {
        let body_ts = codegen_nodes(body);
        quote! {
            if let Some(#pat) = __val {
                #body_ts
            } else {
                #root::VNode::Empty
            }
        }
    } else {
        quote! { #root::VNode::Empty }
    };

    let catch_body = if let Some((pat, body)) = &await_node.catch_branch {
        let body_ts = codegen_nodes(body);
        quote! {
            if let Some(#pat) = __err {
                #body_ts
            } else {
                #root::VNode::Empty
            }
        }
    } else {
        quote! { #root::VNode::Empty }
    };

    quote! {
        {
            let __state = ::sveltui::reactive::use_async_state(#future);
            match __state {
                ::sveltui::reactive::AsyncState::Loading => #root::IntoNode::into_node(#loading_ts),
                ::sveltui::reactive::AsyncState::Resolved(__val) => {
                    #root::IntoNode::into_node({
                        let __val = Some(__val);
                        let __err: Option<String> = None;
                        #then_body
                    })
                }
                ::sveltui::reactive::AsyncState::Rejected(__err) => {
                    #root::IntoNode::into_node({
                        let __val: Option<()> = None;
                        let __err = Some(__err);
                        #catch_body
                    })
                }
            }
        }
    }
}

pub fn codegen_element(el: &Element) -> proc_macro2::TokenStream {
    let root = element_root();
    let tag = &el.name;
    codegen_custom_component(el, tag, &root)
}

fn codegen_custom_component(
    el: &Element,
    tag_path: &syn::Path,
    root: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let tag = quote!(#tag_path).to_string();
    let mut prop_setters = Vec::new();
    let mut attr_names = std::collections::HashSet::with_capacity(el.attributes.len());

    for attr in &el.attributes {
        let attr_name = &attr.name;
        if attr_name == "children" {
            return syn::Error::new(
                attr_name.span(),
                format!(
                    "Cannot pass 'children' as an attribute prop on `<{}>`. Primary children must be placed directly inside the element body: `<{}>...</{}>`",
                    tag, tag, tag
                ),
            )
            .to_compile_error();
        }
        attr_names.insert(attr_name);
        let attr_val = &attr.value;
        prop_setters.push(quote! {
            .#attr_name(#attr_val)
        });
    }

    let mut snippet_setters = Vec::new();
    let mut meaningful_children = Vec::new();
    let mut seen_snippets = std::collections::HashSet::new();

    for child in &el.children {
        match child {
            Node::Snippet(s) => {
                let s_name = &s.name;
                if s_name == "children" {
                    return syn::Error::new(
                        s_name.span(),
                        format!(
                            "Cannot declare a named snippet for 'children' on `<{}>`. Primary children must be placed directly inside the element body: `<{}>...</{}>`",
                            tag, tag, tag
                        ),
                    )
                    .to_compile_error();
                }

                if attr_names.contains(s_name) {
                    return syn::Error::new(
                        s_name.span(),
                        format!(
                            "Slot '{}' is defined both as an attribute prop and as a `{{#snippet {}}}` block on `<{}>`.",
                            s_name, s_name, tag
                        ),
                    )
                    .to_compile_error();
                }

                if !seen_snippets.insert(s_name) {
                    return syn::Error::new(
                        s_name.span(),
                        format!("Duplicate snippet slot '{}' on `<{}>`.", s_name, tag),
                    )
                    .to_compile_error();
                }

                let slot_ident = quote::format_ident!("slot_{}", s_name);
                let snippet_ts = codegen_snippet(s);
                snippet_setters.push(quote! {
                    .#slot_ident(#snippet_ts)
                });
            }
            Node::Text(lit) if lit.value().trim().is_empty() => {
                // Ignore empty or whitespace-only text literals
            }
            other => {
                meaningful_children.push(other);
            }
        }
    }

    let children_setter = if !meaningful_children.is_empty() {
        let body_ts = codegen_node_refs(&meaningful_children);
        if tag == "ErrorBoundary" || tag == "ContextProvider" || tag_path.segments.len() > 1 {
            Some(quote! {
                .slot_children(#root::Snippet::new(move |_| { #body_ts }))
            })
        } else {
            Some(quote! {
                .slot_children(#body_ts)
            })
        }
    } else {
        None
    };

    let mut tag_props = tag_path.clone();
    let segment = tag_props.segments.last_mut().expect("component path");
    segment.ident = quote::format_ident!("{}Props", segment.ident);

    quote! {
        #root::IntoNode::into_node(
            #tag_path(
                #tag_props::builder()
                    #(#prop_setters)*
                    #(#snippet_setters)*
                    #children_setter
                    .build()
            )
        )
    }
}
