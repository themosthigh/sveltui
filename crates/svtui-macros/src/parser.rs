use crate::ast::{
    Attribute, AwaitNode, EachNode, Element, IfBranch, IfCondition, IfNode, Node, RenderNode,
    SnippetNode, ViewTree,
};
use proc_macro2::TokenStream;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{Expr, Ident, LitStr, Pat, Result, Token};

pub fn parse_attr_value(input: ParseStream) -> Result<Expr> {
    if input.peek(syn::token::Brace) {
        let content;
        syn::braced!(content in input);
        let expr: Expr = content.parse()?;
        Ok(expr)
    } else {
        let mut tokens = TokenStream::new();
        while !input.is_empty()
            && !input.peek(Token![>])
            && !input.peek(Token![/])
            && !(input.peek(Ident) && input.peek2(Token![=]))
        {
            let tt: proc_macro2::TokenTree = input.parse()?;
            tokens.extend(std::iter::once(tt));
        }
        syn::parse2::<Expr>(tokens)
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let value = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            parse_attr_value(input)?
        } else {
            syn::parse_quote_spanned!(name.span() => true)
        };
        Ok(Attribute { name, value })
    }
}

fn is_closing_element_tag(input: ParseStream) -> bool {
    let fork = input.fork();
    fork.parse::<Token![<]>().is_ok() && fork.parse::<Token![/]>().is_ok()
}

fn is_control_boundary(input: ParseStream) -> bool {
    if input.peek(syn::token::Brace) {
        let fork = input.fork();
        let Ok(group) = fork.parse::<proc_macro2::Group>() else {
            return false;
        };
        let mut stream = group.stream().into_iter();
        match stream.next() {
            Some(proc_macro2::TokenTree::Punct(p)) => p.as_char() == ':' || p.as_char() == '/',
            _ => false,
        }
    } else {
        false
    }
}

fn parse_if_condition(content: ParseStream) -> Result<IfCondition> {
    if content.peek(Token![let]) {
        content.parse::<Token![let]>()?;
        let mut pat_tokens = TokenStream::new();
        while !content.peek(Token![=]) && !content.is_empty() {
            let tt: proc_macro2::TokenTree = content.parse()?;
            pat_tokens.extend(std::iter::once(tt));
        }
        content.parse::<Token![=]>()?;
        let pat = Pat::parse_single.parse2(pat_tokens)?;
        let expr: Expr = content.parse()?;
        Ok(IfCondition::Let { pat, expr })
    } else {
        let expr: Expr = content.parse()?;
        Ok(IfCondition::Expr(expr))
    }
}

fn parse_nodes_until(
    input: ParseStream,
    stop_condition: impl Fn(ParseStream) -> bool,
) -> Result<Vec<Node>> {
    let mut nodes = Vec::new();
    while !input.is_empty() && !stop_condition(input) {
        if input.peek(Token![<]) {
            if input.peek2(Token![/]) {
                break;
            }
            nodes.push(Node::Element(input.parse::<Element>()?));
        } else if input.peek(LitStr) {
            nodes.push(Node::Text(input.parse::<LitStr>()?));
        } else if input.peek(syn::token::Brace) {
            if is_control_boundary(input) {
                break;
            }
            nodes.push(parse_brace_node(input)?);
        } else {
            return Err(input.error("Expected element, string literal, or block expression"));
        }
    }
    Ok(nodes)
}

fn parse_if_node(input: ParseStream, first_content: ParseStream) -> Result<IfNode> {
    let initial_cond = parse_if_condition(first_content)?;
    let initial_body = parse_nodes_until(input, is_control_boundary)?;

    let initial = IfBranch {
        condition: initial_cond,
        body: initial_body,
    };

    let mut else_ifs = Vec::new();
    let mut else_branch = None;

    loop {
        if input.is_empty() {
            return Err(input.error("Unclosed `{#if}` block, expected `{/if}`"));
        }

        if input.peek(syn::token::Brace) {
            let fork = input.fork();
            let content;
            syn::braced!(content in fork);

            if content.peek(Token![/]) {
                // Closing tag {/if}
                let real_content;
                syn::braced!(real_content in input);
                real_content.parse::<Token![/]>()?;
                let tag = real_content.call(Ident::parse_any)?;
                if tag != "if" {
                    return Err(syn::Error::new(
                        tag.span(),
                        format!("Expected `{{/if}}`, found `{{/{}}}`", tag),
                    ));
                }
                break;
            } else if content.peek(Token![:]) {
                let real_content;
                syn::braced!(real_content in input);
                real_content.parse::<Token![:]>()?;
                let tag = real_content.call(Ident::parse_any)?;

                if tag == "else" {
                    if real_content.peek(Token![if]) {
                        real_content.parse::<Token![if]>()?;
                        let cond = parse_if_condition(&real_content)?;
                        let body = parse_nodes_until(input, is_control_boundary)?;
                        else_ifs.push(IfBranch {
                            condition: cond,
                            body,
                        });
                    } else {
                        let body = parse_nodes_until(input, is_control_boundary)?;
                        else_branch = Some(body);
                    }
                } else {
                    return Err(syn::Error::new(
                        tag.span(),
                        format!("Expected `else` or `else if`, found `{}`", tag),
                    ));
                }
            } else {
                return Err(input.error("Expected `{:else}` or `{/if}`"));
            }
        } else {
            return Err(input.error("Expected control tag `{/if}` or `{:else}`"));
        }
    }

    Ok(IfNode {
        initial,
        else_ifs,
        else_branch,
    })
}

fn parse_each_node(input: ParseStream, first_content: ParseStream) -> Result<EachNode> {
    let mut expr_tokens = TokenStream::new();
    while !first_content.peek(Token![as]) && !first_content.is_empty() {
        let tt: proc_macro2::TokenTree = first_content.parse()?;
        expr_tokens.extend(std::iter::once(tt));
    }
    first_content.parse::<Token![as]>()?;
    let expr: Expr = syn::parse2(expr_tokens)?;

    let mut pat_tokens = TokenStream::new();
    while !first_content.peek(syn::token::Paren) && !first_content.is_empty() {
        let tt: proc_macro2::TokenTree = first_content.parse()?;
        pat_tokens.extend(std::iter::once(tt));
    }
    let pat = Pat::parse_single.parse2(pat_tokens)?;

    let key = if first_content.peek(syn::token::Paren) {
        let key_content;
        syn::parenthesized!(key_content in first_content);
        Some(key_content.parse::<Expr>()?)
    } else {
        None
    };

    let body = parse_nodes_until(input, is_control_boundary)?;
    let mut else_branch = None;

    loop {
        if input.is_empty() {
            return Err(input.error("Unclosed `{#each}` block, expected `{/each}`"));
        }

        let real_content;
        syn::braced!(real_content in input);

        if real_content.peek(Token![/]) {
            real_content.parse::<Token![/]>()?;
            let tag = real_content.call(Ident::parse_any)?;
            if tag != "each" {
                return Err(syn::Error::new(
                    tag.span(),
                    format!("Expected `{{/each}}`, found `{{/{}}}`", tag),
                ));
            }
            break;
        } else if real_content.peek(Token![:]) {
            real_content.parse::<Token![:]>()?;
            let tag = real_content.call(Ident::parse_any)?;
            if tag != "else" {
                return Err(syn::Error::new(
                    tag.span(),
                    format!("Expected `else`, found `{}`", tag),
                ));
            }
            else_branch = Some(parse_nodes_until(input, is_control_boundary)?);
        } else {
            return Err(input.error("Expected `{:else}` or `{/each}`"));
        }
    }

    Ok(EachNode {
        expr,
        pat,
        key,
        body,
        else_branch,
    })
}

fn parse_await_node(input: ParseStream, first_content: ParseStream) -> Result<AwaitNode> {
    let future: Expr = first_content.parse()?;
    let loading = parse_nodes_until(input, is_control_boundary)?;

    let mut then_branch = None;
    let mut catch_branch = None;

    loop {
        if input.is_empty() {
            return Err(input.error("Unclosed `{#await}` block, expected `{/await}`"));
        }

        let real_content;
        syn::braced!(real_content in input);

        if real_content.peek(Token![/]) {
            real_content.parse::<Token![/]>()?;
            let tag = real_content.call(Ident::parse_any)?;
            if tag != "await" {
                return Err(syn::Error::new(
                    tag.span(),
                    format!("Expected `{{/await}}`, found `{{/{}}}`", tag),
                ));
            }
            break;
        } else if real_content.peek(Token![:]) {
            real_content.parse::<Token![:]>()?;
            let tag = real_content.call(Ident::parse_any)?;
            if tag == "then" {
                let pat = real_content.call(Pat::parse_single)?;
                let body = parse_nodes_until(input, is_control_boundary)?;
                then_branch = Some((pat, body));
            } else if tag == "catch" {
                let pat = real_content.call(Pat::parse_single)?;
                let body = parse_nodes_until(input, is_control_boundary)?;
                catch_branch = Some((pat, body));
            } else {
                return Err(syn::Error::new(
                    tag.span(),
                    format!("Expected `then` or `catch`, found `{}`", tag),
                ));
            }
        } else {
            return Err(input.error("Expected `{:then}`, `{:catch}`, or `{/await}`"));
        }
    }

    Ok(AwaitNode {
        future,
        loading,
        then_branch,
        catch_branch,
    })
}

fn parse_snippet_node(input: ParseStream, first_content: ParseStream) -> Result<SnippetNode> {
    let name: Ident = first_content.parse()?;
    let args = if first_content.peek(syn::token::Paren) {
        let args_content;
        syn::parenthesized!(args_content in first_content);
        let mut list = Vec::new();
        while !args_content.is_empty() {
            let pat = Pat::parse_single(&args_content)?;
            let final_pat = if args_content.peek(Token![:]) {
                args_content.parse::<Token![:]>()?;
                let ty: syn::Type = args_content.parse()?;
                Pat::Type(syn::PatType {
                    attrs: Vec::new(),
                    pat: Box::new(pat),
                    colon_token: Default::default(),
                    ty: Box::new(ty),
                })
            } else {
                pat
            };
            list.push(final_pat);
            if args_content.peek(Token![,]) {
                args_content.parse::<Token![,]>()?;
            } else {
                break;
            }
        }
        list
    } else {
        Vec::new()
    };

    let body = parse_nodes_until(input, is_control_boundary)?;

    let real_content;
    syn::braced!(real_content in input);
    real_content.parse::<Token![/]>()?;
    let tag = real_content.call(Ident::parse_any)?;
    if tag != "snippet" {
        return Err(syn::Error::new(
            tag.span(),
            format!("Expected `{{/snippet}}`, found `{{/{}}}`", tag),
        ));
    }

    Ok(SnippetNode { name, args, body })
}

fn parse_brace_node(input: ParseStream) -> Result<Node> {
    let content;
    syn::braced!(content in input);

    if content.peek(Token![#]) {
        content.parse::<Token![#]>()?;
        let tag = content.call(Ident::parse_any)?;
        if tag == "if" {
            Ok(Node::If(parse_if_node(input, &content)?))
        } else if tag == "each" {
            Ok(Node::Each(parse_each_node(input, &content)?))
        } else if tag == "await" {
            Ok(Node::Await(parse_await_node(input, &content)?))
        } else if tag == "snippet" {
            Ok(Node::Snippet(parse_snippet_node(input, &content)?))
        } else {
            Err(syn::Error::new(
                tag.span(),
                format!("Unknown block directive `#{}`", tag),
            ))
        }
    } else if content.peek(Token![@]) {
        content.parse::<Token![@]>()?;
        let tag = content.call(Ident::parse_any)?;
        if tag == "render" {
            let target: Expr = content.parse()?;
            Ok(Node::Render(RenderNode { target }))
        } else {
            Err(syn::Error::new(
                tag.span(),
                format!("Unknown directive `@{}`", tag),
            ))
        }
    } else {
        let expr = content.parse::<Expr>()?;
        Ok(Node::Block(expr))
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

        let children = parse_nodes_until(input, is_closing_element_tag)?;

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
            let node = parse_brace_node(input)?;
            Ok(ViewTree { root: node })
        } else {
            Err(input.error("Expected JSX element, string literal, or block expression"))
        }
    }
}
