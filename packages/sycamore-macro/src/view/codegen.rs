//! Codegen for `view!` macro.
//!
//! Note: we are not using the `ToTokens` trait from `quote` because we need to keep track
//! of some internal state during the entire codegen.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::view::ir::*;

/// A struct for keeping track of the state when emitting Rust code.
pub struct Codegen {
    ctx: Ident,
}

impl Codegen {
    pub fn view_root(&self, view_root: &ViewRoot) -> TokenStream {
        match &view_root.0[..] {
            [] => quote! {
                ::sycamore::view::View::empty()
            },
            [node] => self.view_node(node),
            nodes => {
                let append_nodes: TokenStream = nodes
                    .iter()
                    .map(|node| {
                        let quoted = self.view_node(node);
                        quote! { children.push(#quoted); }
                    })
                    .collect();
                quote! {
                    ::sycamore::view::View::new_fragment({
                        let mut children = ::std::vec::Vec::new();
                        #append_nodes
                        children
                    })
                }
            }
        }
    }

    pub fn view_node(&self, view_node: &ViewNode) -> TokenStream {
        match view_node {
            ViewNode::Element(elem) => self.element(elem),
            ViewNode::Component(comp) => self.component(comp),
            ViewNode::Text(txt) => self.text(txt),
            ViewNode::Dyn(d) => self.dyn_node(d),
        }
    }

    pub fn element(&self, elem: &Element) -> TokenStream {
        let Element {
            tag,
            attrs,
            children,
        } = elem;

        let tag = match tag {
            ElementTag::Builtin(id) => id.to_string(),
            ElementTag::Custom(s) => s.clone(),
        };

        let quote_attrs: TokenStream = attrs.iter().map(|attr| self.attribute(attr)).collect();

        let quote_children = {
            let multi = children.len() >= 2;
            let mut children = children.iter().peekable();
            let mut quoted = TokenStream::new();
            while let Some(child) = children.next() {
                let is_dyn = child.is_dynamic();
                if is_dyn {
                    // Child is dynamic.

                    // If __el is a HydrateNode, use get_next_marker as initial node value.
                    let initial = if cfg!(feature = "experimental-hydrate") {
                        quote! {
                            if let ::std::option::Some(__el) = <dyn ::std::any::Any>::downcast_ref::<::sycamore::HydrateNode>(&__el) {
                                let __initial = ::sycamore::utils::hydrate::web::get_next_marker(&__el.inner_element());
                                // Do not drop the HydrateNode because it will be cast into a GenericNode.
                                let __initial = ::std::mem::ManuallyDrop::new(__initial);
                                // SAFETY: This is safe because we already checked that the type is HydrateNode.
                                // __initial is wrapped inside ManuallyDrop to prevent double drop.
                                unsafe { ::std::ptr::read(&__initial as *const _ as *const _) }
                            } else { None }
                        }
                    } else {
                        quote! { None }
                    };

                    match child {
                        ViewNode::Component(comp) => todo!(),
                        ViewNode::Dyn(_) => todo!(),
                        _ => unreachable!("only component and dyn node can be dynamic"),
                    };

                    // Do not perform non dynamic codegen.
                    continue;
                }
                match child {
                    ViewNode::Element(elem) => quoted.extend({
                        let elem = self.element(elem);
                        quote! {
                            ::sycamore::generic_node::GenericNode::append_child(&__el, &#elem);
                        }
                    }),
                    ViewNode::Component(_) => unreachable!("component is always dynamic"),
                    ViewNode::Text(Text { value }) => {
                        let intern = quote! {
                            // Since this is static text, intern it as it will likely be constructed many times.
                            #[cfg(target_arch = "wasm32")]
                            ::sycamore::rt::intern(#value);
                        };
                        quoted.extend(match multi {
                            true => quote! {
                                #intern
                                ::sycamore::generic_node::GenericNode::append_child(
                                    &__el,
                                    &::sycamore::generic_node::GenericNode::text_node(#value),
                                );
                            },
                            // Only one child, directly set innerText instead of creating a text node.
                            false => quote! {
                                #intern
                                ::sycamore::generic_node::GenericNode::update_inner_text(&__el, #value);
                            },
                        });
                    }
                    ViewNode::Dyn(_) => {
                        assert!(!is_dyn);
                        todo!()
                    }
                }
            }
            quoted
        };

        quote! {
            let __el = ::sycamore::generic_node::GenericNode::element(#tag);
            #quote_attrs
            #quote_children
            __el
        }
    }

    pub fn attribute(&self, attr: &Attribute) -> TokenStream {
        todo!();
    }

    pub fn component(&self, comp: &Component) -> TokenStream {
        todo!();
    }

    pub fn text(&self, txt: &Text) -> TokenStream {
        let s = &txt.value;
        quote! { #s }
    }

    pub fn dyn_node(&self, d: &Dyn) -> TokenStream {
        todo!();
    }
}
