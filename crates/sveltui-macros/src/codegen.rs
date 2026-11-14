use crate::ast::{
    AwaitNode, EachNode, Element, IfCondition, IfNode, KeyNode, Node, RenderNode, SnippetNode,
};
use quote::{ToTokens, quote, quote_spanned};
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
        Node::Key(key_node) => codegen_key(key_node),
        Node::Const(const_node) => syn::Error::new(
            const_node.pat.span(),
            "`{@const}` must sit directly inside an element, component or block.",
        )
        .to_compile_error(),
    }
}

/// Lowers `{#key expr}`. The body runs in a scope that goes by the value,
/// so when the value changes the old scope is dropped and the body starts
/// afresh, with new state for every component in it.
fn codegen_key(key_node: &KeyNode) -> proc_macro2::TokenStream {
    let root = element_root();
    let expr = &key_node.expr;
    let body_ts = codegen_nodes(&key_node.body);
    quote! {
        #root::IntoNode::into_node({
            let __key = &(#expr);
            ::sveltui::reactive::ComponentScope::run_keyed("key", __key, || #body_ts)
        })
    }
}

pub fn codegen_nodes(nodes: &[Node]) -> proc_macro2::TokenStream {
    codegen_block(nodes.iter())
}

pub fn codegen_node_refs(nodes: &[&Node]) -> proc_macro2::TokenStream {
    codegen_block(nodes.iter().copied())
}

/// Lowers the nodes of one block. `{@const}` and local `{#snippet}`
/// declarations become `let` bindings at the top, in source order, so the
/// nodes after them (wherever they sit) can use the names. They render
/// nothing themselves.
fn codegen_block<'a>(nodes: impl Iterator<Item = &'a Node>) -> proc_macro2::TokenStream {
    let root = element_root();
    let mut bindings = Vec::new();
    let mut rendered = Vec::new();
    for node in nodes {
        match node {
            Node::Const(c) => {
                let (pat, expr) = (&c.pat, &c.expr);
                bindings.push(quote! { let #pat = #expr; });
            }
            Node::Snippet(s) => {
                let name = &s.name;
                let snippet = codegen_snippet(s);
                bindings.push(quote! { let #name = #snippet; });
            }
            other => rendered.push(codegen_node(other)),
        }
    }
    let body = match rendered.len() {
        0 => quote! { #root::VNode::Empty },
        1 => rendered.remove(0),
        _ => quote! { #root::VNode::Fragment(vec![ #(#rendered),* ]) },
    };
    if bindings.is_empty() {
        body
    } else {
        quote! {{ #(#bindings)* #body }}
    }
}

fn codegen_if(if_node: &IfNode) -> proc_macro2::TokenStream {
    let root = element_root();
    let initial_cond = match &if_node.initial.condition {
        IfCondition::Expr(expr) => quote! { #expr },
        IfCondition::Let { pat, expr } => quote! { let #pat = #expr },
    };
    let initial_body = branch_scope("if-0", codegen_nodes(&if_node.initial.body));

    let mut else_ifs = Vec::new();
    for (index, branch) in if_node.else_ifs.iter().enumerate() {
        let cond = match &branch.condition {
            IfCondition::Expr(expr) => quote! { #expr },
            IfCondition::Let { pat, expr } => quote! { let #pat = #expr },
        };
        let body = branch_scope(&format!("if-{}", index + 1), codegen_nodes(&branch.body));
        else_ifs.push(quote! {
            else if #cond {
                #body
            }
        });
    }

    let else_branch = if let Some(body) = &if_node.else_branch {
        let body_ts = branch_scope("if-else", codegen_nodes(body));
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

    // Each branch runs in a scope named for it, so flipping the condition
    // destroys the state and cleanups the old branch built instead of
    // handing them to whatever the new branch puts at the same position.
    // The whole chain has a scope too, so it takes one position among its
    // siblings whichever branch is showing.
    quote! {
        ::sveltui::reactive::ComponentScope::run_scoped("if", || {
            #root::IntoNode::into_node(
                if #initial_cond {
                    #initial_body
                } #(#else_ifs)* #else_branch
            )
        })
    }
}

/// Wraps generated code that builds a branch in a scope called `name`.
fn branch_scope(name: &str, body: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote! {
        ::sveltui::reactive::ComponentScope::run_scoped(#name, || #body)
    }
}

fn codegen_each(each_node: &EachNode) -> proc_macro2::TokenStream {
    let root = element_root();
    let expr = &each_node.expr;
    let pat = &each_node.pat;
    let body_ts = codegen_nodes(&each_node.body);
    // `{#each xs as item, i}` counts the rows as we walk them.
    let (binding, rows) = match &each_node.index {
        Some(index) => (
            quote! { (#index, #pat) },
            quote! { (#expr).into_iter().enumerate() },
        ),
        None => (quote! { #pat }, quote! { (#expr).into_iter() }),
    };

    let else_ts = if let Some(else_branch) = &each_node.else_branch {
        codegen_nodes(else_branch)
    } else {
        quote! { #root::VNode::Empty }
    };

    // With a key, each row runs in the scope that goes by that key, so a
    // component in the row keeps its state when rows before it come and go.
    // Without one, rows are matched by position, as before. Either way the
    // whole list gets a scope of its own, so its length never shifts the
    // scopes of the siblings after it.
    let row = if let Some(key) = &each_node.key {
        quote! {
            {
                let __key = &(#key);
                ::sveltui::reactive::ComponentScope::run_keyed("each-row", __key, || #body_ts)
            }
        }
    } else {
        quote! { #body_ts }
    };

    quote! {
        ::sveltui::reactive::ComponentScope::run_scoped("each", || {
            let mut __items = Vec::new();
            for #binding in #rows {
                __items.push(#row);
            }
            if __items.is_empty() {
                #root::IntoNode::into_node(#else_ts)
            } else {
                #root::VNode::Fragment(__items)
            }
        })
    }
}

fn codegen_render(render_node: &RenderNode) -> proc_macro2::TokenStream {
    let root = element_root();
    let target = &render_node.target;
    // The snippet being rendered and the argument we hand it.
    let (snippet, args) = match target {
        Expr::Call(call) => {
            let func = &call.func;
            (quote! { (#func) }, call.args.to_token_stream())
        }
        Expr::MethodCall(m) => {
            let receiver = &m.receiver;
            let method = &m.method;
            (quote! { ((#receiver).#method) }, m.args.to_token_stream())
        }
        _ => (quote! { (#target) }, quote! { () }),
    };
    // A snippet takes one value: none is `()` and several are a tuple, the
    // same shape `{#snippet name(a, b)}` declares.
    let arg_count = match target {
        Expr::Call(call) => call.args.len(),
        Expr::MethodCall(m) => m.args.len(),
        _ => 0,
    };
    let args = match arg_count {
        0 => quote! { () },
        1 => args,
        _ => quote! { (#args) },
    };
    if render_node.optional {
        // Checking first means the arguments aren't evaluated for a snippet
        // that was never passed.
        quote! {
            #root::IntoNode::into_node(
                if #snippet.is_snippet_present() {
                    #root::IntoNode::into_node(#snippet.render(#args))
                } else {
                    #root::VNode::Empty
                }
            )
        }
    } else {
        quote! {
            #root::IntoNode::into_node(#snippet.render(#args))
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
        // Several arguments travel as one tuple. A type written on an
        // argument moves to the tuple's type, with `_` for the others.
        let pats = args.iter().map(|arg| match arg {
            syn::Pat::Type(typed) => &*typed.pat,
            other => other,
        });
        let tys = args.iter().map(|arg| match arg {
            syn::Pat::Type(typed) => {
                let ty = &typed.ty;
                quote! { #ty }
            }
            _ => quote! { _ },
        });
        if args.iter().any(|arg| matches!(arg, syn::Pat::Type(_))) {
            quote! { (#(#pats),*): (#(#tys),*) }
        } else {
            quote! { (#(#pats),*) }
        }
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
    let loading_ts = branch_scope("await-loading", codegen_nodes(&await_node.loading));

    let then_body = if let Some((pat, body)) = &await_node.then_branch {
        let pat = pat.as_ref().map_or_else(|| quote! { _ }, |p| quote! { #p });
        let body_ts = branch_scope("await-then", codegen_nodes(body));
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
        let pat = pat.as_ref().map_or_else(|| quote! { _ }, |p| quote! { #p });
        let body_ts = branch_scope("await-catch", codegen_nodes(body));
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

    // The pending future and the branch showing both live in a scope of
    // their own, so a block that appears or goes takes its state with it.
    quote! {
        ::sveltui::reactive::ComponentScope::run_scoped("await", || {
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
        })
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
        let setter = if attr.bind {
            quote::format_ident!("bind_{}", attr_name, span = attr_name.span())
        } else {
            attr_name.clone()
        };
        prop_setters.push(quote! {
            .#setter(#attr_val)
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
