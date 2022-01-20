#![allow(clippy::eval_order_dependence)] // Needed when using `syn::parenthesized!`.

pub mod codegen;
pub mod ir;
pub mod parse;

use proc_macro2::TokenStream;
use quote::format_ident;

use self::codegen::Codegen;
use self::ir::*;

pub fn view_impl(view_root: ViewRoot) -> TokenStream {
    let codegen_state = Codegen {
        ctx: format_ident!("ctx"),
    };
    codegen_state.view_root(&view_root)
}

pub fn node_impl(node: ViewNode) -> TokenStream {
    let codegen_state = Codegen {
        ctx: format_ident!("ctx"),
    };
    codegen_state.view_node(&node)
}
