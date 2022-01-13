pub mod generic_node;
pub mod reactive {
    pub use sycamore_reactive::*;
}
pub mod component;
pub mod utils;
pub mod view;

#[cfg(feature = "dom")]
pub use generic_node::dom_node::{render, render_to};
pub use sycamore_macro::*;

/// The sycamore prelude.
pub mod prelude {
    pub use crate::component::Component;
    pub use crate::generic_node::{GenericNode, Html};
    pub use crate::reactive::*;
    pub use crate::view::View;
    pub use sycamore_macro::*;
}

/// Re-exports for use by `sycamore-macro`. Not intended for use by end-users.
#[doc(hidden)]
pub mod rt {
    pub use js_sys::Reflect;
    pub use wasm_bindgen::{intern, JsCast, JsValue};
    pub use web_sys::Event;
}
