use crate::ast::{Element, Node};
use quote::{quote, quote_spanned};
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
                        _ => {}
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
        _ => {
            // Any custom element or component (Button, Switch, TextInput, Checkbox, Divider, ProgressBar, Badge, etc.)
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
    }
}
