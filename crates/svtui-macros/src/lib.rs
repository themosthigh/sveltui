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
    let sig = &input_fn.sig;
    let block = &input_fn.block;
    let attrs = &input_fn.attrs;
    let fn_name = &sig.ident;

    let pkg = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
    let root = if pkg == "svtui-reactive" {
        quote! { crate }
    } else {
        quote! { ::svtui::reactive }
    };

    let expanded = quote! {
        #(#attrs)*
        #[allow(non_snake_case)]
        #vis #sig {
            #root::ComponentScope::run_scoped(stringify!(#fn_name), move || {
                #block
            })
        }
    };

    TokenStream::from(expanded)
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

#[proc_macro_derive(Props, attributes(prop))]
pub fn derive_props(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let builder_name = quote::format_ident!("{}Builder", struct_name);

    let pkg = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
    let root = if pkg == "svtui-element" {
        quote! { crate }
    } else {
        quote! { ::svtui }
    };

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

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut builder_fields = Vec::new();
    let mut default_inits = Vec::new();
    let mut setters = Vec::new();
    let mut build_inits = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        let mut is_optional = false;
        let mut default_expr = None;
        let mut is_default = false;

        for attr in &field.attrs {
            if attr.path().is_ident("prop") {
                let _ = attr.parse_nested_meta(|meta| {
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

        builder_fields.push(quote! {
            pub #field_name: Option<#field_ty>,
        });

        default_inits.push(quote! {
            #field_name: None,
        });

        let is_snippet = is_snippet_type(field_ty);

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
                    });
                } else {
                    setters.push(quote! {
                        pub fn #field_name(mut self, val: impl #root::IntoPropOption<#inner_ty>) -> Self {
                            self.#field_name = Some(val.into_prop_option());
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
            } else {
                setters.push(quote! {
                    pub fn #field_name(mut self, val: impl Into<#field_ty>) -> Self {
                        self.#field_name = Some(val.into());
                        self
                    }
                });
            }
            if let Some(expr) = default_expr {
                build_inits.push(quote! {
                    #field_name: self.#field_name.unwrap_or_else(|| #expr),
                });
            } else if is_default {
                build_inits.push(quote! {
                    #field_name: self.#field_name.unwrap_or_default(),
                });
            } else {
                let err_msg = format!(
                    "Field '{}' is required for struct '{}'",
                    field_name, struct_name
                );
                build_inits.push(quote! {
                    #field_name: self.#field_name.expect(#err_msg),
                });
            }
        }
    }

    let expanded = quote! {
        #[allow(non_snake_case)]
        pub struct #builder_name #generics #where_clause {
            #(#builder_fields)*
        }

        impl #impl_generics Default for #builder_name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(#default_inits)*
                }
            }
        }

        impl #impl_generics #builder_name #ty_generics #where_clause {
            #(#setters)*

            pub fn build(self) -> #struct_name #ty_generics {
                #struct_name {
                    #(#build_inits)*
                }
            }
        }

        impl #impl_generics #struct_name #ty_generics #where_clause {
            pub fn builder() -> #builder_name #ty_generics {
                #builder_name::default()
            }
        }
    };

    TokenStream::from(expanded)
}
