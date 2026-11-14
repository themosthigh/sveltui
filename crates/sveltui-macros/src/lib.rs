use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemFn, parse_macro_input, spanned::Spanned};

mod ast;
mod codegen;
mod parser;

use ast::ViewTree;
use codegen::codegen_node;

#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let tree = parse_macro_input!(input as ViewTree);
    let expanded = codegen_node(&tree.root);
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let vis = &input_fn.vis;
    let block = &input_fn.block;
    let attrs = &input_fn.attrs;
    let fn_name = &input_fn.sig.ident;

    let root = quote! { ::sveltui::reactive };

    let props_ident = quote::format_ident!("{}Props", fn_name);
    let builder_ident = quote::format_ident!("{}PropsBuilder", fn_name);

    let (synthesized_props, new_sig) = if input_fn.sig.inputs.is_empty() {
        let mut new_sig = input_fn.sig.clone();
        new_sig.inputs.push(syn::parse_quote!(_props: #props_ident));

        let props_def = quote! {
            #[derive(Clone, Copy, Default)]
            #vis struct #props_ident;

            #[derive(Clone, Copy, Default)]
            #vis struct #builder_ident;

            impl #props_ident {
                pub fn builder() -> #builder_ident {
                    #builder_ident
                }
            }

            impl #builder_ident {
                pub fn build(self) -> #props_ident {
                    #props_ident
                }
            }
        };
        (Some(props_def), new_sig)
    } else {
        (None, input_fn.sig.clone())
    };

    let expanded = quote! {
        #synthesized_props

        #(#attrs)*
        #[allow(non_snake_case)]
        #vis #new_sig {
            #root::ComponentScope::run_scoped(stringify!(#fn_name), move || {
                #block
            })
        }
    };

    TokenStream::from(expanded)
}

/// The `T` in `Wrapper<T>` when `ty`'s last path segment is `wrapper`.
/// Whether `ty` is a number, `bool` or `char`.
fn is_primitive(ty: &syn::Type) -> bool {
    const PRIMITIVES: [&str; 16] = [
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64", "bool", "char",
    ];
    matches!(ty, syn::Type::Path(path) if path.qself.is_none()
        && path.path.get_ident().is_some_and(|ident| PRIMITIVES.iter().any(|p| ident == p)))
}

fn extract_generic_inner<'a>(ty: &'a syn::Type, wrapper: &str) -> Option<&'a syn::Type> {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == wrapper
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        Some(inner)
    } else {
        None
    }
}

fn extract_option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        Some(inner)
    } else {
        None
    }
}

fn is_snippet_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Snippet" {
                    return true;
                }
                if segment.ident == "Option"
                    && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                    && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                {
                    return is_snippet_type(inner);
                }
            }
            false
        }
        _ => false,
    }
}

type SetterFn =
    Box<dyn Fn(proc_macro2::TokenStream, proc_macro2::TokenStream) -> proc_macro2::TokenStream>;

/// A required prop: its setters are written once we know the builder's
/// full list of type parameters.
struct Required {
    name: syn::Ident,
    make: SetterFn,
}

#[proc_macro_derive(Props, attributes(prop, snippet))]
pub fn derive_props(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let builder_name = quote::format_ident!("{}Builder", struct_name);

    let root = quote! { ::sveltui };

    let fields = match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(f) => &f.named,
            _ => {
                return syn::Error::new(
                    input.span(),
                    "#[derive(Props)] only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new(input.span(), "#[derive(Props)] only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut builder_fields = Vec::new();
    let mut builder_names = Vec::new();
    let mut spread_stmts = Vec::new();
    let mut default_inits = Vec::new();
    let mut setters = Vec::new();
    let mut build_inits = Vec::new();
    // Required props are tracked in the builder's type, so `build()` only
    // exists once all of them are set. Their setters change the type, so
    // we write them after the loop, when we know every required name.
    let mut required: Vec<Required> = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        let mut is_snippet = is_snippet_type(field_ty);
        let mut is_optional = false;
        let mut default_expr = None;
        let mut is_default = false;
        let mut is_bindable = false;

        for attr in &field.attrs {
            if attr.path().is_ident("prop") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("optional") {
                        is_optional = true;
                        Ok(())
                    } else if meta.path.is_ident("bindable") {
                        is_bindable = true;
                        Ok(())
                    } else if meta.path.is_ident("default") {
                        if meta.input.peek(syn::Token![=]) {
                            let value: syn::Expr = meta.value()?.parse()?;
                            default_expr = Some(value);
                        } else {
                            is_default = true;
                        }
                        Ok(())
                    } else {
                        Ok(())
                    }
                });
            } else if attr.path().is_ident("snippet") {
                is_snippet = true;
                if let syn::Meta::List(list) = &attr.meta {
                    let _ = list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("optional") {
                            is_optional = true;
                            Ok(())
                        } else if meta.path.is_ident("default") {
                            if meta.input.peek(syn::Token![=]) {
                                let value: syn::Expr = meta.value()?.parse()?;
                                default_expr = Some(value);
                            } else {
                                is_default = true;
                            }
                            Ok(())
                        } else {
                            Ok(())
                        }
                    });
                }
            }
        }

        if is_snippet && extract_option_inner(field_ty).is_some() {
            is_optional = true;
        }

        // `#[prop(bindable)]` fields hold a `Bindable<T>`: `x(..)` sets the
        // read-only value and `bind_x(..)` the bound state, so the view
        // macro's `bind:x={state}` lands on the same field.
        if is_bindable {
            let Some(inner_ty) = extract_generic_inner(field_ty, "Bindable") else {
                return syn::Error::new(
                    field_ty.span(),
                    "#[prop(bindable)] fields must have the type `Bindable<T>`",
                )
                .to_compile_error()
                .into();
            };
            let bind_fn = quote::format_ident!("bind_{}", field_name);
            builder_fields.push(quote! {
                pub #field_name: Option<#field_ty>,
            });
            builder_names.push(field_name.clone());
            spread_stmts.push(quote! { self.#field_name = Some(__props.#field_name); });
            default_inits.push(quote! {
                #field_name: None,
            });
            setters.push(quote! {
                pub fn #field_name(mut self, val: impl #root::IntoPropOption<#inner_ty>) -> Self {
                    let mut bindable = self.#field_name.take().unwrap_or_default();
                    bindable.set_value(val.into_prop_option());
                    self.#field_name = Some(bindable);
                    self
                }
                pub fn #bind_fn(mut self, val: impl #root::IntoBinding<#inner_ty>) -> Self {
                    // Passing a parent's `Bindable` on carries its read-only
                    // value too, but a bare state leaves a value set earlier.
                    let (value, bound) = val.into_binding().into_parts();
                    let mut bindable = self.#field_name.take().unwrap_or_default();
                    if value.is_some() {
                        bindable.set_value(value);
                    }
                    bindable.set_bound(bound);
                    self.#field_name = Some(bindable);
                    self
                }
            });
            build_inits.push(quote! {
                #field_name: self.#field_name.unwrap_or_default(),
            });
            continue;
        }

        builder_fields.push(quote! {
            pub #field_name: Option<#field_ty>,
        });
        builder_names.push(field_name.clone());
        // An optional prop that is `None` in the spread leaves what was
        // set before it alone. Everything else takes the spread's value.
        if is_optional && extract_option_inner(field_ty).is_some() {
            spread_stmts.push(quote! {
                if __props.#field_name.is_some() {
                    self.#field_name = Some(__props.#field_name);
                }
            });
        } else {
            spread_stmts.push(quote! { self.#field_name = Some(__props.#field_name); });
        }

        default_inits.push(quote! {
            #field_name: None,
        });

        if is_optional {
            if let Some(inner_ty) = extract_option_inner(field_ty) {
                if field_name == "children" {
                    setters.push(quote! {
                        pub fn slot_children(mut self, val: impl #root::IntoPropOption<#inner_ty>) -> Self {
                            self.#field_name = Some(val.into_prop_option());
                            self
                        }
                        pub fn children(self, val: impl #root::IntoPropOption<#inner_ty>) -> Self {
                            self.slot_children(val)
                        }
                    });
                } else if is_snippet {
                    let slot_fn = quote::format_ident!("slot_{}", field_name);
                    setters.push(quote! {
                        pub fn #slot_fn(mut self, val: impl #root::IntoPropOption<#inner_ty>) -> Self {
                            self.#field_name = Some(val.into_prop_option());
                            self
                        }
                        pub fn #field_name(self, val: impl #root::IntoPropOption<#inner_ty>) -> Self {
                            self.#slot_fn(val)
                        }
                    });
                } else {
                    // Every setter for an optional prop accepts both `T` and
                    // `Option<T>`, so forwarding an optional value needs no
                    // separate `opt_<field>` setter.
                    let s = quote!(#inner_ty).to_string().replace(" ", "");
                    let (param, value) = if s.contains("Rc<dynFn()") || s.contains("Rc<(dynFn())") {
                        (
                            quote!(impl #root::IntoClickHandler),
                            quote!(val.into_click_handler()),
                        )
                    } else if s.contains("KeyHandler") || s.contains("KeyEvent") {
                        (
                            quote!(impl #root::IntoKeyHandler),
                            quote!(val.into_key_handler()),
                        )
                    } else {
                        (
                            quote!(impl #root::IntoPropOption<#inner_ty>),
                            quote!(val.into_prop_option()),
                        )
                    };
                    setters.push(quote! {
                        pub fn #field_name(mut self, val: #param) -> Self {
                            self.#field_name = Some(#value);
                            self
                        }
                    });
                }
            } else {
                if field_name == "children" {
                    setters.push(quote! {
                        pub fn slot_children(mut self, val: impl Into<#field_ty>) -> Self {
                            self.#field_name = Some(val.into());
                            self
                        }
                        pub fn children(self, val: impl Into<#field_ty>) -> Self {
                            self.slot_children(val)
                        }
                    });
                } else if is_snippet {
                    let slot_fn = quote::format_ident!("slot_{}", field_name);
                    setters.push(quote! {
                        pub fn #slot_fn(mut self, val: impl Into<#field_ty>) -> Self {
                            self.#field_name = Some(val.into());
                            self
                        }
                    });
                } else if is_primitive(field_ty) {
                    // A number takes its own type, so a literal like `gap=2`
                    // becomes that type instead of an `i32` with no `Into`.
                    setters.push(quote! {
                        pub fn #field_name(mut self, val: #field_ty) -> Self {
                            self.#field_name = Some(val);
                            self
                        }
                    });
                } else {
                    setters.push(quote! {
                        pub fn #field_name(mut self, val: impl Into<#field_ty>) -> Self {
                            self.#field_name = Some(val.into());
                            self
                        }
                    });
                }
            }
            build_inits.push(quote! {
                #field_name: self.#field_name.flatten(),
            });
        } else {
            let is_dimension = matches!(field_ty, syn::Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Dimension"));
            let is_children = field_name == "children";
            let is_prim = is_primitive(field_ty);
            let (root_c, name_c, ty_c) = (root.clone(), field_name.clone(), field_ty.clone());
            // `ret` is the builder type a setter hands back and `fin` the
            // expression that produces it from the updated `self`.
            let make: SetterFn = Box::new(move |ret, fin| {
                let (root, field_name, field_ty) = (&root_c, &name_c, &ty_c);
                if is_dimension {
                    quote! {
                        pub fn #field_name(mut self, val: impl #root::IntoDimension) -> #ret {
                            self.#field_name = Some(val.into_dimension());
                            #fin
                        }
                    }
                } else if is_children {
                    quote! {
                        pub fn slot_children(mut self, val: impl Into<#field_ty>) -> #ret {
                            self.#field_name = Some(val.into());
                            #fin
                        }
                        pub fn children(self, val: impl Into<#field_ty>) -> #ret {
                            self.slot_children(val)
                        }
                    }
                } else if is_snippet {
                    let slot_fn = quote::format_ident!("slot_{}", field_name);
                    quote! {
                        pub fn #slot_fn(mut self, val: impl Into<#field_ty>) -> #ret {
                            self.#field_name = Some(val.into());
                            #fin
                        }
                    }
                } else if is_prim {
                    quote! {
                        pub fn #field_name(mut self, val: #field_ty) -> #ret {
                            self.#field_name = Some(val);
                            #fin
                        }
                    }
                } else {
                    quote! {
                        pub fn #field_name(mut self, val: impl Into<#field_ty>) -> #ret {
                            self.#field_name = Some(val.into());
                            #fin
                        }
                    }
                }
            });
            if let Some(expr) = default_expr {
                setters.push(make(quote!(Self), quote!(self)));
                build_inits.push(quote! {
                    #field_name: self.#field_name.unwrap_or_else(|| #expr),
                });
            } else if is_default {
                setters.push(make(quote!(Self), quote!(self)));
                build_inits.push(quote! {
                    #field_name: self.#field_name.unwrap_or_default(),
                });
            } else {
                required.push(Required {
                    name: field_name.clone(),
                    make,
                });
                build_inits.push(quote! {
                    #field_name: self.#field_name.expect("a required prop is set by the builder's type"),
                });
            }
        }
    }

    // The builder gets one extra type parameter per required prop, `Missing`
    // until its setter runs and `PropSet` after.
    let plain_params: Vec<syn::GenericParam> = input
        .generics
        .params
        .iter()
        .cloned()
        .map(|mut param| {
            match &mut param {
                syn::GenericParam::Type(t) => t.default = None,
                syn::GenericParam::Const(c) => c.default = None,
                syn::GenericParam::Lifetime(_) => {}
            }
            param
        })
        .collect();
    let plain_args: Vec<proc_macro2::TokenStream> = input
        .generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Lifetime(l) => {
                let lifetime = &l.lifetime;
                quote!(#lifetime)
            }
            syn::GenericParam::Type(t) => {
                let ident = &t.ident;
                quote!(#ident)
            }
            syn::GenericParam::Const(c) => {
                let ident = &c.ident;
                quote!(#ident)
            }
        })
        .collect();
    let state_params: Vec<syn::Ident> = (0..required.len())
        .map(|i| quote::format_ident!("__R{}", i))
        .collect();
    let missing_types: Vec<syn::Ident> = required
        .iter()
        .map(|r| quote::format_ident!("__{}_missing_{}", struct_name, r.name))
        .collect();
    let args_with = |states: Vec<proc_macro2::TokenStream>| {
        quote! { <#(#plain_args,)* #(#states),*> }
    };
    let own_states: Vec<proc_macro2::TokenStream> =
        state_params.iter().map(|p| quote!(#p)).collect();
    let missing_states: Vec<proc_macro2::TokenStream> =
        missing_types.iter().map(|m| quote!(#m)).collect();
    let own_args = args_with(own_states.clone());
    let missing_args = args_with(missing_states.clone());
    let decl_params: Vec<proc_macro2::TokenStream> = plain_params
        .iter()
        .map(|p| quote!(#p))
        .chain(
            state_params
                .iter()
                .zip(&missing_types)
                .map(|(p, m)| quote!(#p = #m)),
        )
        .collect();
    let impl_params: Vec<proc_macro2::TokenStream> = plain_params
        .iter()
        .map(|p| quote!(#p))
        .chain(state_params.iter().map(|p| quote!(#p)))
        .collect();
    let plain_impl_params: Vec<proc_macro2::TokenStream> =
        plain_params.iter().map(|p| quote!(#p)).collect();

    let mut required_setters = Vec::new();
    let mut mark_fns = Vec::new();
    for (i, req) in required.iter().enumerate() {
        let mut states = own_states.clone();
        states[i] = quote!(#root::PropSet);
        let ret_args = args_with(states);
        let ret = quote!(#builder_name #ret_args);
        let mark = quote::format_ident!("__mark_{}", req.name);
        required_setters.push((req.make)(ret.clone(), quote!(self.#mark())));
        let moves = builder_names.iter().map(|n| quote!(#n: self.#n,));
        mark_fns.push(quote! {
            fn #mark(self) -> #ret {
                #builder_name {
                    #(#moves)*
                    __required: ::core::marker::PhantomData,
                }
            }
        });
    }
    let all_set = args_with(
        state_params
            .iter()
            .map(|_| quote!(#root::PropSet))
            .collect(),
    );
    let spread_moves = builder_names.iter().map(|n| quote!(#n: self.#n,));
    let build_bounds: Vec<proc_macro2::TokenStream> = state_params
        .iter()
        .map(|p| quote!(#p: #root::PropIsSet))
        .collect();
    let where_tokens = where_clause.map(|w| &w.predicates);
    let build_where = quote! { where #(#build_bounds,)* #where_tokens };
    let type_where = where_clause.map(|w| quote!(#w));

    let expanded = quote! {
        #(
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            pub struct #missing_types;
        )*

        #[allow(non_snake_case)]
        pub struct #builder_name <#(#decl_params),*> #type_where {
            #(#builder_fields)*
            #[doc(hidden)]
            pub __required: ::core::marker::PhantomData<(#(#own_states,)*)>,
        }

        impl <#(#plain_impl_params),*> Default for #builder_name #missing_args #type_where {
            fn default() -> Self {
                Self {
                    #(#default_inits)*
                    __required: ::core::marker::PhantomData,
                }
            }
        }

        impl <#(#impl_params),*> #builder_name #own_args #type_where {
            #(#setters)*
            #(#required_setters)*
            #(#mark_fns)*

            /// Merges a whole props value in, as `<Comp {..props} />` does.
            /// A complete props value has every required prop, so the
            /// builder it returns is ready to build. Props set after the
            /// spread win over it.
            pub fn spread(mut self, __props: #struct_name #ty_generics) -> #builder_name #all_set {
                #(#spread_stmts)*
                #builder_name {
                    #(#spread_moves)*
                    __required: ::core::marker::PhantomData,
                }
            }

            pub fn build(self) -> #struct_name #ty_generics #build_where {
                #struct_name {
                    #(#build_inits)*
                }
            }
        }

        impl #impl_generics #struct_name #ty_generics #where_clause {
            pub fn builder() -> #builder_name #missing_args {
                #builder_name::default()
            }
        }
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified_tags_resolve_props_and_defer_children() {
        let tree = syn::parse2::<ViewTree>(quote! {
            <widgets::DropdownMenu::Root>
                <widgets::DropdownMenu::Item>"Copy"</widgets::DropdownMenu::Item>
                <widgets::DropdownMenu::Separator />
            </widgets::DropdownMenu::Root>
        })
        .expect("qualified tags should parse");
        let generated = codegen_node(&tree.root).to_string();
        assert!(generated.contains("widgets :: DropdownMenu :: RootProps :: builder"));
        assert!(generated.contains("widgets :: DropdownMenu :: ItemProps :: builder"));
        assert!(generated.contains("slot_children (:: sveltui :: element :: Snippet :: new"));
    }

    #[test]
    fn closing_tags_match_the_entire_path() {
        let result = syn::parse2::<ViewTree>(quote! {
            <DropdownMenu::Root></ContextMenu::Root>
        });
        let error = result.err().expect("different namespaces must not match");
        assert!(error.to_string().contains("Expected closing tag"));
    }

    fn generated(tokens: proc_macro2::TokenStream) -> String {
        let tree = syn::parse2::<ViewTree>(tokens).expect("view should parse");
        codegen_node(&tree.root).to_string()
    }

    #[test]
    fn bind_attributes_call_the_bind_setter() {
        let code = generated(quote! { <TextInput bind:value={name} placeholder="Name" /> });
        assert!(code.contains(". bind_value (name)"));
        assert!(code.contains(". placeholder (\"Name\")"));
        assert!(!code.contains(". value ("));
    }

    #[test]
    fn bind_and_plain_attributes_mix_without_braces() {
        let code = generated(quote! {
            <Checkbox label=title bind:checked=accepted disabled=false />
        });
        assert!(code.contains(". label (title)"));
        assert!(code.contains(". bind_checked (accepted)"));
        assert!(code.contains(". disabled (false)"));
    }

    #[test]
    fn paths_in_values_are_not_mistaken_for_prefixes() {
        let code = generated(quote! {
            <View border_color=Color::White bind:open=show />
        });
        assert!(code.contains(". border_color (Color :: White)"));
        assert!(code.contains(". bind_open (show)"));
    }

    #[test]
    fn unknown_prefixes_are_rejected() {
        let error = syn::parse2::<ViewTree>(quote! { <TextInput on:value={x} /> })
            .err()
            .expect("only bind: is a prefix");
        assert!(error.to_string().contains("Only `bind:` is supported"));
    }

    fn tokens(source: &str) -> proc_macro2::TokenStream {
        source.parse().expect("valid tokens")
    }

    fn parse_error(tokens: proc_macro2::TokenStream) -> String {
        syn::parse2::<ViewTree>(tokens)
            .err()
            .expect("view should not parse")
            .to_string()
    }

    #[test]
    fn bind_shorthand_binds_the_same_name() {
        let code = generated(quote! { <TextInput bind:value /> });
        assert!(code.contains(". bind_value (value)"));
    }

    #[test]
    fn braced_names_are_shorthand_attributes() {
        let code = generated(quote! { <TextInput {placeholder} bind:value {size} /> });
        assert!(code.contains(". placeholder (placeholder)"));
        assert!(code.contains(". bind_value (value)"));
        assert!(code.contains(". size (size)"));
    }

    #[test]
    fn shorthand_must_be_a_single_name() {
        assert!(parse_error(quote! { <TextInput {a + b} /> }).contains("single name"));
        assert!(parse_error(quote! { <TextInput {a b} /> }).contains("single name"));
    }

    #[test]
    fn unbraced_values_are_literals_and_paths() {
        let code = generated(quote! {
            <View width=40 gap=-1 ratio=-0.5 flag=true name="x" title=title
                color=Color::White self_ref=self::x none=None />
        });
        assert!(code.contains(". width (40)"));
        assert!(code.contains(". gap (- 1)"));
        assert!(code.contains(". ratio (- 0.5)"));
        assert!(code.contains(". flag (true)"));
        assert!(code.contains(". name (\"x\")"));
        assert!(code.contains(". title (title)"));
        assert!(code.contains(". color (Color :: White)"));
        assert!(code.contains(". none (None)"));
    }

    #[test]
    fn unbraced_values_end_at_the_tag_or_the_next_attribute() {
        let code = generated(quote! { <View a=1 b=x c=Color::Red {d} {..e} /> });
        assert!(code.contains(". c (Color :: Red)"));
        assert!(code.contains(". d (d)"));
        let code = generated(quote! { <View a=x>"text"</View> });
        assert!(code.contains(". a (x)"));
    }

    #[test]
    fn unbraced_expressions_name_the_exact_fix() {
        let rejected = [
            (
                quote! { <View padding=Padding::all(1) /> },
                "padding={Padding::all(1)}",
            ),
            (quote! { <View width=props.width /> }, "width={props.width}"),
            (quote! { <View name=x.len() /> }, "name={x.len()}"),
            (quote! { <View pair=(a, 0) /> }, "pair={(a, 0)}"),
            (
                quote! { <View cols=[length(3), fr(1)] /> },
                "cols={[length(3), fr(1)]}",
            ),
            (
                quote! { <View on_click=move || go() /> },
                "on_click={move || go()}",
            ),
            (quote! { <View width=a / 2 /> }, "width={a / 2}"),
            (quote! { <View width=a + 1 height=2 /> }, "width={a + 1}"),
            (
                quote! { <View c=if a { 1 } else { 2 } /> },
                "c={if a { 1 } else { 2 }}",
            ),
            (quote! { <View bind:open=!x /> }, "bind:open={!x}"),
            (
                quote! { <View margin=Margin { left: 1 } /> },
                "margin={Margin { left : 1 }}",
            ),
            (
                quote! { <View color=Some(Color::Red) /> },
                "color={Some(Color::Red)}",
            ),
            (quote! { <View width= /> }, "width={}"),
        ];
        for (tokens, fix) in rejected {
            let message = parse_error(tokens);
            assert!(message.contains("Wrap the value in braces"), "{message}");
            assert!(message.contains(&format!("`{fix}`")), "{fix} in {message}");
        }
    }

    #[test]
    fn a_comparison_value_is_not_cut_at_the_greater_than() {
        let message = parse_error(tokens("<View disabled=a > b>\"x\"</View>"));
        assert!(message.contains("`disabled={a > b}`"), "{message}");
        let message = parse_error(tokens("<View disabled=a > 3 />"));
        assert!(message.contains("`disabled={a > 3}`"), "{message}");
        // A child string after the tag's `>` is still just a child.
        generated(tokens("<View disabled=a>\"x\"</View>"));
    }

    #[test]
    fn braced_values_can_hold_any_expression() {
        let code = generated(quote! { <View width={a / 2} disabled={a > b} /> });
        assert!(code.contains(". width (a / 2)"));
        assert!(code.contains(". disabled (a > b)"));
    }

    #[test]
    fn repeated_else_is_rejected() {
        let message = parse_error(tokens(r#"{#if a} "x" {:else} "y" {:else} "z" {/if}"#));
        assert!(message.contains("can only appear once"), "{message}");
        let message = parse_error(tokens(r#"{#if a} "x" {:else} "y" {:else if b} "z" {/if}"#));
        assert!(message.contains("can only appear once"), "{message}");
        let message = parse_error(tokens(
            r#"{#each xs as x} "x" {:else} "y" {:else} "z" {/each}"#,
        ));
        assert!(message.contains("can only appear once"), "{message}");
    }

    #[test]
    fn unknown_tags_list_the_valid_ones() {
        let message = parse_error(tokens("{#foo a} {/foo}"));
        assert!(message.contains("`{#if}`") && message.contains("`{#snippet}`"));
        let message = parse_error(quote! { {@html x} });
        assert!(message.contains("`{@render}`"));
    }

    #[test]
    fn optional_render_checks_the_snippet_first() {
        let code = generated(quote! { {@render props.footer?.(item)} });
        assert!(code.contains("is_snippet_present"));
        assert!(code.contains("((props) . footer) . render (item)"));
        let code = generated(quote! { {@render footer?.()} });
        assert!(code.contains("(footer) . is_snippet_present"));
        let code = generated(quote! { {@render props.footer} });
        assert!(!code.contains("is_snippet_present"));
    }

    #[test]
    fn each_accepts_an_index_and_a_key() {
        let code = generated(tokens("{#each xs as item, i (item.id)} \"x\" {/each}"));
        assert!(code.contains("for (i , item) in (xs) . into_iter () . enumerate ()"));
        assert!(code.contains("run_keyed"));
        let code = generated(tokens("{#each xs as (a, b), i} \"x\" {/each}"));
        assert!(code.contains("for (i , (a , b)) in"));
        let code = generated(tokens("{#each xs as (a, b)} \"x\" {/each}"));
        assert!(code.contains("for (a , b) in (xs) . into_iter () {"));
    }

    #[test]
    fn each_index_must_be_a_name() {
        assert!(!parse_error(tokens("{#each xs as item, 1} \"x\" {/each}")).is_empty());
    }

    #[test]
    fn bare_then_and_catch_bind_nothing() {
        let code = generated(tokens(
            "{#await fut} \"wait\" {:then} \"done\" {:catch} \"oops\" {/await}",
        ));
        assert!(code.contains("if let Some (_) = __val"));
        assert!(code.contains("if let Some (_) = __err"));
    }

    #[test]
    fn then_shorthand_skips_the_loading_branch() {
        let code = generated(tokens("{#await fut then v} {v} {:catch e} {e} {/await}"));
        assert!(code.contains("if let Some (v) = __val"));
        assert!(code.contains("if let Some (e) = __err"));
        let code = generated(tokens("{#await fut then} \"done\" {/await}"));
        assert!(code.contains("if let Some (_) = __val"));
    }

    #[test]
    fn key_blocks_run_the_body_in_a_keyed_scope() {
        let code = generated(tokens("{#key page} <Row /> {/key}"));
        assert!(code.contains("run_keyed (\"key\" , __key"));
        assert!(parse_error(tokens("{#key page} <Row /> {/each}")).contains("{/key}"));
    }

    #[test]
    fn const_and_snippets_become_lets_at_the_top() {
        let code = generated(tokens(
            "{#if ok} {@const total = a + b} {#snippet row(x, y)} {x} {/snippet} \"hi\" {@render row(1, 2)} {/if}",
        ));
        let total = code.find("let total = a + b ;").expect("const binding");
        let row = code.find("let row = :: sveltui").expect("snippet binding");
        let first_node = code.find("\"hi\"").expect("text");
        assert!(total < row && row < first_node);
        assert!(code.contains("(row) . render ((1 , 2))"));
    }

    #[test]
    fn single_argument_render_is_not_wrapped() {
        let code = generated(quote! { {@render row(x)} });
        assert!(code.contains("(row) . render (x)"));
        let code = generated(quote! { {@render row()} });
        assert!(code.contains("(row) . render (())"));
    }

    #[test]
    fn const_outside_a_block_is_an_error() {
        let tree = syn::parse2::<ViewTree>(tokens("{@const a = 1}")).expect("parses");
        assert!(
            codegen_node(&tree.root)
                .to_string()
                .contains("compile_error")
        );
    }

    #[test]
    fn interpolated_strings_become_format_calls() {
        let code = generated(tokens(
            r#"<View> $"Items: {tasks.get().len()} {{ok}} {ratio:.1}" "{plain}" </View>"#,
        ));
        assert!(code.contains(
            ":: std :: format ! (\"Items: {} {{ok}} {:.1}\" , tasks . get () . len () , ratio)"
        ));
        // A plain string keeps its braces as they are.
        assert!(code.contains("\"{plain}\""));
    }

    #[test]
    fn interpolation_works_in_attributes_and_with_a_spaced_f() {
        let code = generated(tokens(r#"<View title=$"n={n}" /> "#));
        assert!(code.contains(". title (:: std :: format ! (\"n={}\" , n))"));
        let code = generated(tokens(r#"f "a{b::c}""#));
        assert!(code.contains("format ! (\"a{}\" , b :: c)"));
    }

    #[test]
    fn interpolation_reports_bad_braces() {
        assert!(parse_error(tokens(r#"$"a {b""#)).contains("Unclosed"));
        assert!(parse_error(tokens(r#"$"a } b""#)).contains("Unmatched"));
        assert!(parse_error(tokens(r#"$"a {1 +} b""#)).contains("not a Rust expression"));
    }

    #[test]
    fn function_bindings_take_the_same_path_as_any_binding() {
        let code = generated(quote! {
            <TextInput bind:value={(|| name.get(), |next| name.set(next))} />
        });
        assert!(code.contains(". bind_value ((| | name . get () , | next | name . set (next)))"));
        assert!(!code.contains("on_value_change"));
        assert!(!code.contains(". value ("));
    }

    #[test]
    fn spread_attributes_merge_a_props_value() {
        let code = generated(quote! { <Button {..base} label="Go" /> });
        assert!(code.contains(". spread (base) . label (\"Go\")"));
    }

    #[test]
    fn bind_this_passes_the_refs_focus_id() {
        let code = generated(quote! { <TextInput bind:this={field} /> });
        assert!(code.contains(". id ((field) . id ())"));
        assert!(!code.contains("bind_this"));
    }
}
