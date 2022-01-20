#![allow(clippy::eval_order_dependence)] // Needed when using `syn::parenthesized!`.

pub mod codegen;
pub mod ir;
pub mod parse;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token};

use self::codegen::Codegen;
use self::ir::*;

pub struct WithCtxArg<T> {
    ctx: Ident,
    rest: T,
}

impl<T: Parse> Parse for WithCtxArg<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let ctx = input.parse()?;
        let _comma: Token![,] = input.parse().map_err(|_| input.error("expected `,` (help: make sure you pass the ctx variable to the macro as an argument)"))?;
        let rest = input.parse()?;
        Ok(Self { ctx, rest })
    }
}

pub fn view_impl(view_root: WithCtxArg<ViewRoot>) -> TokenStream {
    let ctx = view_root.ctx;
    let codegen_state = Codegen { ctx: ctx.clone() };
    let quoted = codegen_state.view_root(&view_root.rest);
    quote! {{
        let __ctx = &#ctx; // Make sure that ctx is used.
        #quoted
    }}
}

pub fn node_impl(node: WithCtxArg<ViewNode>) -> TokenStream {
    let ctx = node.ctx;
    let codegen_state = Codegen { ctx: ctx.clone() };
    let quoted = codegen_state.view_node(&node.rest);
    quote! {{
        let __ctx = &#ctx; // Make sure that ctx is used.
        #quoted
    }}
}
