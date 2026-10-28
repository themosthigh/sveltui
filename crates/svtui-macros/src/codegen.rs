use crate::ast::{
    AwaitNode, EachNode, Element, IfCondition, IfNode, Node, RenderNode, SnippetNode,
};
use quote::{quote, quote_spanned};
use syn::Expr;
use syn::spanned::Spanned;

fn element_root() -> proc_macro2::TokenStream {
    match std::env::var("CARGO_PKG_NAME").as_deref() {
        Ok("svtui-components") => quote! { ::svtui_element },
        Ok("svtui-element") => quote! { crate },
        _ => quote! { ::svtui::element },
    }
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
            let __state = ::svtui::reactive::use_async_state(#future);
            match __state {
                ::svtui::reactive::AsyncState::Loading => #root::IntoNode::into_node(#loading_ts),
                ::svtui::reactive::AsyncState::Resolved(__val) => {
                    #root::IntoNode::into_node({
                        let __val = Some(__val);
                        let __err: Option<String> = None;
                        #then_body
                    })
                }
                ::svtui::reactive::AsyncState::Rejected(__err) => {
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
    let tag_str = tag.to_string();

    match tag_str.as_str() {
        "Box" => {
            let mut setter_calls = Vec::new();
            for attr in &el.attributes {
                let attr_name = &attr.name;
                let attr_val = &attr.value;
                setter_calls.push(quote! {
                    .#attr_name(#attr_val)
                });
            }

            let mut child_calls = Vec::new();
            for child in &el.children {
                let child_stream = codegen_node(child);
                child_calls.push(quote! {
                    .child(#child_stream)
                });
            }

            quote! {
                #root::BoxElement::new()
                    #(#setter_calls)*
                    #(#child_calls)*
                    .into_node()
            }
        }
        "Text" => {
            let mut setter_calls = Vec::new();
            for attr in &el.attributes {
                let attr_name = &attr.name;
                let attr_val = &attr.value;
                setter_calls.push(quote! {
                    .#attr_name(#attr_val)
                });
            }

            let mut text_arg = quote! { "" };
            let mut extra_children = Vec::new();

            for (i, child) in el.children.iter().enumerate() {
                if i == 0 {
                    match child {
                        Node::Text(lit) => {
                            text_arg = quote! { #lit };
                        }
                        Node::Block(expr) => {
                            text_arg = quote! { #expr };
                        }
                        _ => {
                            let child_stream = codegen_node(child);
                            extra_children.push(quote! {
                                .child(#child_stream)
                            });
                        }
                    }
                } else {
                    let child_stream = codegen_node(child);
                    extra_children.push(quote! {
                        .child(#child_stream)
                    });
                }
            }

            quote! {
                #root::TextElement::new(#text_arg)
                    #(#setter_calls)*
                    #(#extra_children)*
                    .into_node()
            }
        }
        "Overlay" => {
            let mut setter_calls = Vec::new();
            for attr in &el.attributes {
                let attr_name = &attr.name;
                let attr_val = &attr.value;
                setter_calls.push(quote! {
                    .#attr_name(#attr_val)
                });
            }

            let mut child_calls = Vec::new();
            for child in &el.children {
                let child_stream = codegen_node(child);
                child_calls.push(quote! {
                    .child(#child_stream)
                });
            }

            quote! {
                #root::OverlayElement::new()
                    #(#setter_calls)*
                    #(#child_calls)*
                    .into_node()
            }
        }
        "Provider" => {
            let mut val_expr = quote! { () };
            for attr in &el.attributes {
                if attr.name == "value" {
                    let v = &attr.value;
                    val_expr = quote! { #v };
                }
            }

            let children_ts = codegen_nodes(&el.children);
            quote! {
                #root::Provider::new(#val_expr)
                    .snippet(#root::Snippet::new(move |_| {
                        #children_ts
                    }))
                    .into_node()
            }
        }
        "ErrorBoundary" => {
            let mut fallback_ts = None;
            let mut regular_children = Vec::new();

            for child in &el.children {
                if let Node::Snippet(s) = child
                    && s.name == "fallback"
                {
                    let snippet_ts = codegen_snippet(s);
                    fallback_ts = Some(quote! {
                        .fallback(#snippet_ts)
                    });
                    continue;
                }
                regular_children.push(child);
            }

            let children_ts = if regular_children.is_empty() {
                quote! { #root::VNode::Empty }
            } else if regular_children.len() == 1 {
                codegen_node(regular_children[0])
            } else {
                let streams = regular_children.iter().map(|c| codegen_node(c));
                quote! {
                    #root::VNode::Fragment(vec![
                        #(#streams),*
                    ])
                }
            };

            quote! {
                #root::ErrorBoundary::new()
                    #fallback_ts
                    .render_child(move || {
                        #children_ts
                    })
                    .into_node()
            }
        }
        "Button" | "TextInput" | "TextArea" | "Checkbox" | "Switch" | "Badge" | "Divider"
        | "ProgressBar" | "DropdownMenu" | "DropdownItem" | "DropdownSubmenu" | "Popover" => {
            let mut setter_calls = Vec::new();
            for attr in &el.attributes {
                let attr_name = &attr.name;
                let attr_val = &attr.value;
                setter_calls.push(quote! {
                    .#attr_name(#attr_val)
                });
            }

            let mut child_calls = Vec::new();
            for child in &el.children {
                match child {
                    Node::Text(lit) => {
                        child_calls.push(quote! {
                            .label(#lit)
                        });
                    }
                    _ => {
                        let child_stream = codegen_node(child);
                        child_calls.push(quote! {
                            .child(#child_stream)
                        });
                    }
                }
            }

            quote! {
                #root::IntoNode::into_node(
                    #tag::default()
                        #(#setter_calls)*
                        #(#child_calls)*
                )
            }
        }
        "Dialog" => {
            let has_snippets = el.children.iter().any(|c| matches!(c, Node::Snippet(_)));
            if has_snippets {
                // Component Dialog from APPLICATION_SYNTAX.md
                codegen_custom_component(el, tag, &root)
            } else {
                // Built-in Dialog widget
                let mut setter_calls = Vec::new();
                for attr in &el.attributes {
                    let attr_name = &attr.name;
                    let attr_val = &attr.value;
                    setter_calls.push(quote! {
                        .#attr_name(#attr_val)
                    });
                }

                let mut child_calls = Vec::new();
                for child in &el.children {
                    let child_stream = codegen_node(child);
                    child_calls.push(quote! {
                        .child(#child_stream)
                    });
                }

                quote! {
                    #root::IntoNode::into_node(
                        #tag::default()
                            #(#setter_calls)*
                            #(#child_calls)*
                    )
                }
            }
        }
        _ => codegen_custom_component(el, tag, &root),
    }
}

fn codegen_custom_component(
    el: &Element,
    tag: &syn::Ident,
    root: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let mut prop_setters = Vec::new();
    for attr in &el.attributes {
        let attr_name = &attr.name;
        let attr_val = &attr.value;
        prop_setters.push(quote! {
            .#attr_name(#attr_val)
        });
    }

    let mut snippet_setters = Vec::new();
    let mut regular_children = Vec::new();

    for child in &el.children {
        if let Node::Snippet(s) = child {
            let s_name = &s.name;
            let snippet_ts = codegen_snippet(s);
            snippet_setters.push(quote! {
                .#s_name(#snippet_ts)
            });
        } else {
            regular_children.push(child);
        }
    }

    let has_props = !prop_setters.is_empty() || !snippet_setters.is_empty();
    let has_regular_children = !regular_children.is_empty();

    let children_snippet = if has_regular_children {
        let body_ts = if regular_children.len() == 1 {
            codegen_node(regular_children[0])
        } else {
            let streams = regular_children.iter().map(|c| codegen_node(c));
            quote! {
                #root::VNode::Fragment(vec![
                    #(#streams),*
                ])
            }
        };
        Some(quote! {
            #root::Snippet::new(move |_| {
                #body_ts
            })
        })
    } else {
        None
    };

    let tag_props = syn::Ident::new(&format!("{}Props", tag), tag.span());

    if has_props && has_regular_children {
        let cs = children_snippet.unwrap();
        quote! {
            #root::IntoNode::into_node(
                #tag(
                    #tag_props::builder()
                        #(#prop_setters)*
                        #(#snippet_setters)*
                        .build(),
                    #cs
                )
            )
        }
    } else if has_props {
        quote! {
            #root::IntoNode::into_node(
                #tag(
                    #tag_props::builder()
                        #(#prop_setters)*
                        #(#snippet_setters)*
                        .build()
                )
            )
        }
    } else if has_regular_children {
        let cs = children_snippet.unwrap();
        quote! {
            #root::IntoNode::into_node(
                #tag(#cs)
            )
        }
    } else {
        quote! {
            #root::IntoNode::into_node(
                #tag()
            )
        }
    }
}
