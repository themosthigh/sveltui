use crate::ast::{
    Attribute, AwaitNode, ConstNode, EachNode, Element, IfBranch, IfCondition, IfNode, KeyNode,
    Node, RenderNode, SnippetNode, ViewTree,
};
use proc_macro2::TokenStream;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Expr, Ident, LitStr, Pat, Result, Token, UnOp};

/// Parses an attribute value. Braces hold any expression. Without braces we
/// accept a literal (`-1` included), a path, a tuple or an array,
/// followed by any run of `.field`, `.method(..)` and `(..)` calls. Every
/// part of that is either one token or a balanced group, so a bare `>` or
/// `/` can never be mistaken for the end of the tag. Operators and closures
/// are not accepted: the tag would end halfway through them.
pub fn parse_attr_value(input: ParseStream) -> Result<Expr> {
    if input.peek(syn::token::Brace) {
        let content;
        syn::braced!(content in input);
        let expr: Expr = content.parse()?;
        return Ok(expr);
    }

    if let Some(format) = parse_format_literal(input)? {
        return Ok(format);
    }

    let start = input.span();
    let mut expr: Expr = if input.peek(syn::Lit) {
        Expr::Lit(input.parse()?)
    } else if input.peek(Token![-]) && input.peek2(syn::Lit) {
        Expr::Unary(syn::ExprUnary {
            attrs: Vec::new(),
            op: UnOp::Neg(input.parse()?),
            expr: Box::new(Expr::Lit(input.parse()?)),
        })
    } else if input.peek(syn::token::Paren) {
        parse_paren_group(input)?
    } else if input.peek(syn::token::Bracket) {
        Expr::Array(input.parse()?)
    } else if input.peek(Ident)
        || input.peek(Token![::])
        || input.peek(Token![self])
        || input.peek(Token![Self])
        || input.peek(Token![crate])
        || input.peek(Token![super])
    {
        Expr::Path(input.parse()?)
    } else {
        return Err(unbraced_error(start));
    };

    loop {
        if input.peek(Token![.]) && (input.peek2(Ident::peek_any) || input.peek2(syn::LitInt)) {
            let dot = input.parse::<Token![.]>()?;
            let member: syn::Member = input.parse()?;
            if let (syn::Member::Named(method), true) = (&member, input.peek(syn::token::Paren)) {
                let args = parse_call_args(input)?;
                expr = Expr::MethodCall(syn::ExprMethodCall {
                    attrs: Vec::new(),
                    receiver: Box::new(expr),
                    dot_token: dot,
                    method: method.clone(),
                    turbofish: None,
                    paren_token: args.0,
                    args: args.1,
                });
            } else {
                expr = Expr::Field(syn::ExprField {
                    attrs: Vec::new(),
                    base: Box::new(expr),
                    dot_token: dot,
                    member,
                });
            }
        } else if input.peek(syn::token::Paren) && matches!(expr, Expr::Path(_) | Expr::Call(_)) {
            let args = parse_call_args(input)?;
            expr = Expr::Call(syn::ExprCall {
                attrs: Vec::new(),
                func: Box::new(expr),
                paren_token: args.0,
                args: args.1,
            });
        } else {
            break;
        }
    }

    // Anything but the end of the tag or the next attribute means the value
    // was a bigger expression than we accept without braces.
    let ends_value = input.is_empty()
        || input.peek(Token![>])
        || (input.peek(Token![/]) && input.peek2(Token![>]))
        || input.peek(Ident)
        || input.peek(syn::token::Brace);
    if !ends_value {
        return Err(unbraced_error(start));
    }
    Ok(expr)
}

/// A `(a)` or `(a, b)` value, which is a parenthesised expression or a tuple.
fn parse_paren_group(input: ParseStream) -> Result<Expr> {
    let (paren_token, elems) = parse_call_args(input)?;
    if elems.len() == 1 && !elems.trailing_punct() {
        Ok(Expr::Paren(syn::ExprParen {
            attrs: Vec::new(),
            paren_token,
            expr: Box::new(elems.into_iter().next().expect("one element")),
        }))
    } else {
        Ok(Expr::Tuple(syn::ExprTuple {
            attrs: Vec::new(),
            paren_token,
            elems,
        }))
    }
}

type CallArgs = (
    syn::token::Paren,
    syn::punctuated::Punctuated<Expr, Token![,]>,
);

fn parse_call_args(input: ParseStream) -> Result<CallArgs> {
    let content;
    let paren = syn::parenthesized!(content in input);
    let args = content.parse_terminated(Expr::parse, Token![,])?;
    Ok((paren, args))
}

fn unbraced_error(span: proc_macro2::Span) -> syn::Error {
    syn::Error::new(
        span,
        "An attribute value without braces must be a single literal or path \
         (like `width=40` or `color=Color::White`). Wrap anything else in \
         braces, like `width={a / 2}` or `disabled={a > b}`.",
    )
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        // `{name}` is shorthand for `name={name}`.
        if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);
            // `{..props}` merges a whole props value in.
            if content.peek(Token![..]) {
                let dots = content.parse::<Token![..]>()?;
                let value: Expr = content.parse()?;
                return Ok(Attribute {
                    name: Ident::new("spread", dots.spans[0]),
                    value,
                    bind: false,
                    spread: true,
                });
            }
            let name: Ident = content.parse().map_err(|e| {
                syn::Error::new(
                    e.span(),
                    "A shorthand attribute is a single name, like `{placeholder}`. \
                     Use `name={expr}` for anything else.",
                )
            })?;
            if !content.is_empty() {
                return Err(content.error(
                    "A shorthand attribute is a single name, like `{placeholder}`. \
                     Use `name={expr}` for anything else.",
                ));
            }
            let value = path_expr(&name);
            return Ok(Attribute {
                name,
                value,
                bind: false,
                spread: false,
            });
        }

        let first: Ident = input.parse()?;
        // `prefix:name` is only valid as `bind:name`. A `::` is a path, not
        // a prefix, so it's left alone.
        let (name, bind) = if input.peek(Token![:]) && !input.peek(Token![::]) {
            input.parse::<Token![:]>()?;
            if first != "bind" {
                return Err(syn::Error::new(
                    first.span(),
                    format!(
                        "Unknown attribute prefix `{first}:`. Only `bind:` is supported, \
                         as in `bind:value={{state}}`."
                    ),
                ));
            }
            (input.parse::<Ident>()?, true)
        } else {
            (first, false)
        };
        let value = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            parse_attr_value(input)?
        } else if bind {
            // `bind:name` is shorthand for `bind:name={name}`.
            path_expr(&name)
        } else {
            syn::parse_quote_spanned!(name.span() => true)
        };
        Ok(Attribute {
            name,
            value,
            bind,
            spread: false,
        })
    }
}

fn path_expr(name: &Ident) -> Expr {
    Expr::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: name.clone().into(),
    })
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
        } else if let Some(format) = parse_format_literal(input)? {
            nodes.push(Node::Block(format));
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
                    if else_branch.is_some() {
                        return Err(syn::Error::new(
                            tag.span(),
                            "`{:else}` must be the last branch of `{#if}`, and can only appear once",
                        ));
                    }
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

/// Splits what follows `as` in `{#each}` into the item pattern, the optional
/// index and the optional `(key)`. A trailing parenthesised group is only a
/// key when the rest still parses, so a pattern like `Some(x)` stays whole.
fn parse_each_binding(
    tokens: Vec<proc_macro2::TokenTree>,
) -> Result<(Pat, Option<Ident>, Option<Expr>)> {
    use proc_macro2::{Delimiter, TokenTree};
    if let Some(TokenTree::Group(group)) = tokens.last()
        && group.delimiter() == Delimiter::Parenthesis
        && tokens.len() > 1
        && let Ok(key) = syn::parse2::<Expr>(group.stream())
    {
        let before: Vec<TokenTree> = tokens[..tokens.len() - 1].to_vec();
        if let Ok((pat, index)) = parse_item_and_index(before) {
            return Ok((pat, index, Some(key)));
        }
    }
    let (pat, index) = parse_item_and_index(tokens)?;
    Ok((pat, index, None))
}

/// `item` or `item, i`. The comma is found at the top level only, so
/// tuple patterns like `(a, b)` keep theirs.
fn parse_item_and_index(tokens: Vec<proc_macro2::TokenTree>) -> Result<(Pat, Option<Ident>)> {
    use proc_macro2::TokenTree;
    let comma = tokens
        .iter()
        .position(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ','));
    let Some(comma) = comma else {
        let pat = Pat::parse_single.parse2(tokens.into_iter().collect())?;
        return Ok((pat, None));
    };
    let pat = Pat::parse_single.parse2(tokens[..comma].iter().cloned().collect())?;
    let index: Ident = syn::parse2(tokens[comma + 1..].iter().cloned().collect())?;
    Ok((pat, Some(index)))
}

fn parse_each_node(input: ParseStream, first_content: ParseStream) -> Result<EachNode> {
    let mut expr_tokens = TokenStream::new();
    while !first_content.peek(Token![as]) && !first_content.is_empty() {
        let tt: proc_macro2::TokenTree = first_content.parse()?;
        expr_tokens.extend(std::iter::once(tt));
    }
    first_content.parse::<Token![as]>()?;
    let expr: Expr = syn::parse2(expr_tokens)?;

    let rest_tokens: Vec<proc_macro2::TokenTree> = {
        let mut tts = Vec::new();
        while !first_content.is_empty() {
            tts.push(first_content.parse()?);
        }
        tts
    };

    let (pat, index, key) = parse_each_binding(rest_tokens)?;

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
            if else_branch.is_some() {
                return Err(syn::Error::new(
                    tag.span(),
                    "`{:else}` can only appear once in an `{#each}` block",
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
        index,
        key,
        body,
        else_branch,
    })
}

/// The pattern after `then` or `catch`, left out for a bare tag.
fn parse_optional_pat(content: ParseStream) -> Result<Option<Pat>> {
    if content.is_empty() {
        Ok(None)
    } else {
        Ok(Some(Pat::parse_single(content)?))
    }
}

fn parse_await_node(input: ParseStream, first_content: ParseStream) -> Result<AwaitNode> {
    let future: Expr = first_content.parse()?;
    let mut then_branch = None;
    let mut catch_branch = None;

    // `{#await fut then v}` skips the pending state and starts straight in
    // the resolved branch.
    let loading = if first_content.peek(Ident) && first_content.fork().parse::<Ident>()? == "then" {
        first_content.parse::<Ident>()?;
        let pat = parse_optional_pat(first_content)?;
        then_branch = Some((pat, parse_nodes_until(input, is_control_boundary)?));
        Vec::new()
    } else {
        parse_nodes_until(input, is_control_boundary)?
    };

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
            if (tag == "then" && then_branch.is_some())
                || (tag == "catch" && catch_branch.is_some())
            {
                return Err(syn::Error::new(
                    tag.span(),
                    format!("`{{:{tag}}}` can only appear once in an `{{#await}}` block"),
                ));
            }
            if tag == "then" {
                let pat = parse_optional_pat(&real_content)?;
                let body = parse_nodes_until(input, is_control_boundary)?;
                then_branch = Some((pat, body));
            } else if tag == "catch" {
                let pat = parse_optional_pat(&real_content)?;
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

fn parse_key_node(input: ParseStream, first_content: ParseStream) -> Result<KeyNode> {
    let expr: Expr = first_content.parse()?;
    let body = parse_nodes_until(input, is_control_boundary)?;
    if input.is_empty() {
        return Err(input.error("Unclosed `{#key}` block, expected `{/key}`"));
    }
    let real_content;
    syn::braced!(real_content in input);
    real_content.parse::<Token![/]>()?;
    let tag = real_content.call(Ident::parse_any)?;
    if tag != "key" {
        return Err(syn::Error::new(
            tag.span(),
            format!("Expected `{{/key}}`, found `{{/{tag}}}`"),
        ));
    }
    Ok(KeyNode { expr, body })
}

/// Parses an interpolated string, written `$"Items: {n}"` (or `f "..."`,
/// since `f"..."` is a reserved prefix that Rust refuses to lex). Plain
/// strings stay literal, so their braces need no escaping. Returns `None`
/// when the input doesn't start one.
fn parse_format_literal(input: ParseStream) -> Result<Option<Expr>> {
    let marker = input.peek(Token![$])
        || (input.peek(Ident) && input.peek2(LitStr) && input.fork().parse::<Ident>()? == "f");
    if !marker {
        return Ok(None);
    }
    if input.peek(Token![$]) {
        input.parse::<Token![$]>()?;
    } else {
        input.parse::<Ident>()?;
    }
    let lit: LitStr = input.parse()?;
    interpolate(&lit).map(Some)
}

/// Turns the text of an interpolated string into a `format!` call. Each
/// `{expr}` or `{expr:spec}` becomes a hole with `expr` as its argument,
/// and `{{` and `}}` stay literal braces. `format!` itself can only take
/// plain names inline, which is why we do the splitting here.
fn interpolate(lit: &LitStr) -> Result<Expr> {
    let text = lit.value();
    let mut format = String::new();
    let mut args: Vec<Expr> = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        match c {
            '{' if matches!(chars.peek(), Some((_, '{'))) => {
                chars.next();
                format.push_str("{{");
            }
            '}' if matches!(chars.peek(), Some((_, '}'))) => {
                chars.next();
                format.push_str("}}");
            }
            '}' => {
                return Err(syn::Error::new(
                    lit.span(),
                    "Unmatched `}` in an interpolated string. Write `}}` for a literal brace.",
                ));
            }
            '{' => {
                let mut depth = 1;
                let mut end = None;
                for (i, c) in chars.by_ref() {
                    match c {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                end = Some(i);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let Some(end) = end else {
                    return Err(syn::Error::new(
                        lit.span(),
                        "Unclosed `{` in an interpolated string. Write `{{` for a literal brace.",
                    ));
                };
                let (expr, spec) = split_hole(&text[start + 1..end], lit)?;
                format.push('{');
                format.push_str(&spec);
                format.push('}');
                args.push(expr);
            }
            c => format.push(c),
        }
    }
    let format = LitStr::new(&format, lit.span());
    Ok(syn::parse_quote_spanned!(lit.span() => ::std::format!(#format #(, #args)*)))
}

/// Splits `expr` or `expr:spec`. The whole hole is tried as an expression
/// first, so a `::` or a ternary-looking `a:b` inside it is never cut.
fn split_hole(hole: &str, lit: &LitStr) -> Result<(Expr, String)> {
    if let Ok(expr) = syn::parse_str::<Expr>(hole) {
        return Ok((expr, String::new()));
    }
    let bytes = hole.as_bytes();
    let colon = (0..bytes.len()).rev().find(|&i| {
        bytes[i] == b':' && (i == 0 || bytes[i - 1] != b':') && bytes.get(i + 1) != Some(&b':')
    });
    if let Some(i) = colon
        && let Ok(expr) = syn::parse_str::<Expr>(&hole[..i])
    {
        return Ok((expr, hole[i..].to_string()));
    }
    Err(syn::Error::new(
        lit.span(),
        format!("`{{{hole}}}` in an interpolated string is not a Rust expression."),
    ))
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

/// Parses what follows `{@render`. `x?.(args)` isn't valid Rust, so we spot
/// the trailing `?.(..)` ourselves and parse the call without the `?.`.
fn parse_render_target(content: ParseStream) -> Result<RenderNode> {
    use proc_macro2::TokenTree;
    let tokens: Vec<TokenTree> = std::iter::from_fn(|| content.parse().ok()).collect();
    let optional = match tokens.as_slice() {
        [
            ..,
            TokenTree::Punct(q),
            TokenTree::Punct(dot),
            TokenTree::Group(g),
        ] => {
            q.as_char() == '?'
                && dot.as_char() == '.'
                && g.delimiter() == proc_macro2::Delimiter::Parenthesis
        }
        _ => false,
    };
    let stream: TokenStream = if optional {
        let split = tokens.len() - 3;
        tokens[..split]
            .iter()
            .chain(&tokens[split + 2..])
            .cloned()
            .collect()
    } else {
        tokens.into_iter().collect()
    };
    let target: Expr = syn::parse2(stream)?;
    Ok(RenderNode { target, optional })
}

fn parse_brace_node(input: ParseStream) -> Result<Node> {
    let content;
    syn::braced!(content in input);

    if content.peek(Token![#]) {
        content.parse::<Token![#]>()?;
        let tag = content.call(Ident::parse_any)?;
        if tag == "key" {
            Ok(Node::Key(parse_key_node(input, &content)?))
        } else if tag == "if" {
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
                format!(
                    "Unknown block directive `#{tag}`. Valid blocks are `{{#if}}`, \
                     `{{#each}}`, `{{#await}}`, `{{#snippet}}` and `{{#key}}`."
                ),
            ))
        }
    } else if content.peek(Token![@]) {
        content.parse::<Token![@]>()?;
        let tag = content.call(Ident::parse_any)?;
        if tag == "render" {
            Ok(Node::Render(parse_render_target(&content)?))
        } else if tag == "const" {
            let pat = Pat::parse_single(&content)?;
            content.parse::<Token![=]>()?;
            let expr: Expr = content.parse()?;
            Ok(Node::Const(ConstNode { pat, expr }))
        } else {
            Err(syn::Error::new(
                tag.span(),
                format!(
                    "Unknown directive `@{tag}`. Valid directives are `{{@render}}` and `{{@const}}`."
                ),
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
        let name = input.call(syn::Path::parse_mod_style)?;

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
        let close_name = input.call(syn::Path::parse_mod_style)?;
        input.parse::<Token![>]>()?;

        if name != close_name {
            return Err(syn::Error::new(
                close_name.span(),
                format!(
                    "Expected closing tag </{}>, found </{}>",
                    quote::quote!(#name),
                    quote::quote!(#close_name)
                ),
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
        } else if let Some(format) = parse_format_literal(input)? {
            Ok(ViewTree {
                root: Node::Block(format),
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
