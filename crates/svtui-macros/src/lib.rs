use proc_macro::TokenStream;
use syn::parse_macro_input;

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
