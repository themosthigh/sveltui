use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, Ident, LitStr, Result, Token};

enum Node {
    Element(Element),
    Text(LitStr),
    Block(Expr),
}

struct Attribute {
    name: Ident,
    value: Expr,
}

struct Element {
    name: Ident,
    attributes: Vec<Attribute>,
    children: Vec<Node>,
}

fn parse_attr_value(input: ParseStream) -> Result<Expr> {
    if input.peek(syn::token::Brace) {
        let content;
        syn::braced!(content in input);
        let expr: Expr = content.parse()?;
        Ok(expr)
    } else if input.peek(syn::Lit) {
        let lit: syn::Lit = input.parse()?;
        Ok(Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit,
        }))
    } else if input.peek(Ident) || input.peek(Token![::]) {
        let path: syn::Path = input.parse()?;
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let args = content.parse_terminated(Expr::parse, Token![,])?;
            Ok(Expr::Call(syn::ExprCall {
                attrs: Vec::new(),
                func: Box::new(Expr::Path(syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                })),
                paren_token: Default::default(),
                args,
            }))
        } else {
            Ok(Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path,
            }))
        }
    } else {
        input.parse::<Expr>()
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let value = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            parse_attr_value(input)?
        } else {
            // Boolean attribute without value: <Box border /> => border = true
            syn::parse_quote_spanned!(name.span() => true)
        };
        Ok(Attribute { name, value })
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![<]>()?;
        let name: Ident = input.parse()?;

        let mut attributes = Vec::new();
        while !input.peek(Token![>]) && !input.peek(Token![/]) {
            attributes.push(input.parse::<Attribute>()?);
        }

        if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            return Ok(Element {
                name,
                attributes,
                children: Vec::new(),
            });
        }

        input.parse::<Token![>]>()?;

        let mut children = Vec::new();
        while !is_closing_tag(input, &name) {
            if input.is_empty() {
                return Err(syn::Error::new(
                    name.span(),
                    format!("Unclosed tag `<{}>`", name),
                ));
            }

            if input.peek(Token![<]) {
                if input.peek2(Token![/]) {
                    break;
                }
                children.push(Node::Element(input.parse::<Element>()?));
            } else if input.peek(LitStr) {
                children.push(Node::Text(input.parse::<LitStr>()?));
            } else if input.peek(syn::token::Brace) {
                let content;
                syn::braced!(content in input);
                let expr = content.parse::<Expr>()?;
                children.push(Node::Block(expr));
            } else {
                return Err(input.error("Expected element, string literal, or block expression"));
            }
        }

        // Parse closing tag </Name>
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let close_name: Ident = input.parse()?;
        input.parse::<Token![>]>()?;

        if name != close_name {
            return Err(syn::Error::new(
                close_name.span(),
                format!("Expected closing tag </{}>, found </{}>", name, close_name),
            ));
        }

        Ok(Element {
            name,
            attributes,
            children,
        })
    }
}

fn is_closing_tag(input: ParseStream, _tag_name: &Ident) -> bool {
    let fork = input.fork();
    if fork.parse::<Token![<]>().is_ok() && fork.parse::<Token![/]>().is_ok() {
        return true;
    }
    false
}

struct ViewTree {
    root: Node,
}

impl Parse for ViewTree {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![<]) {
            let el = input.parse::<Element>()?;
            Ok(ViewTree {
                root: Node::Element(el),
            })
        } else if input.peek(LitStr) {
            let lit = input.parse::<LitStr>()?;
            Ok(ViewTree {
                root: Node::Text(lit),
            })
        } else if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);
            let expr = content.parse::<Expr>()?;
            Ok(ViewTree {
                root: Node::Block(expr),
            })
        } else {
            Err(input.error("Expected JSX element, string literal, or block expression"))
        }
    }
}

fn codegen_node(node: &Node) -> proc_macro2::TokenStream {
    match node {
        Node::Element(el) => codegen_element(el),
        Node::Text(lit) => {
            quote! {
                ::svtui::element::IntoNode::into_node(#lit)
            }
        }
        Node::Block(expr) => {
            quote_spanned! { expr.span() =>
                ::svtui::element::IntoNode::into_node(#expr)
            }
        }
    }
}

fn codegen_element(el: &Element) -> proc_macro2::TokenStream {
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
                ::svtui::element::BoxElement::new()
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
                ::svtui::element::TextElement::new(#text_arg)
                    #(#setter_calls)*
                    #(#extra_children)*
                    .into_node()
            }
        }
        _ => {
            let mut prop_assignments = Vec::new();
            for attr in &el.attributes {
                let attr_name = &attr.name;
                let attr_val = &attr.value;
                prop_assignments.push(quote! {
                    #attr_name: #attr_val.into(),
                });
            }

            let child_nodes: Vec<_> = el.children.iter().map(codegen_node).collect();

            if el.attributes.is_empty() && el.children.is_empty() {
                quote! {
                    ::svtui::element::IntoNode::into_node(#tag())
                }
            } else {
                quote! {
                    ::svtui::element::IntoNode::into_node(
                        #tag(
                            #(#prop_assignments)*
                            vec![ #(#child_nodes),* ]
                        )
                    )
                }
            }
        }
    }
}

#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let tree = parse_macro_input!(input as ViewTree);
    let expanded = codegen_node(&tree.root);
    TokenStream::from(expanded)
}
