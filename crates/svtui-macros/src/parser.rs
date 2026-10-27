use crate::ast::{Attribute, Element, Node, ViewTree};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitStr, Result, Token};

pub fn parse_attr_value(input: ParseStream) -> Result<Expr> {
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
