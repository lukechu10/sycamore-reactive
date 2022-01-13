pub mod generic_node;
pub mod reactive {
    pub use sycamore_reactive::*;
}
pub mod component;
pub mod utils;
pub mod view;

/// The sycamore prelude.
pub mod prelude {
    pub use crate::component::Component;
    pub use crate::generic_node::{GenericNode, Html};
    pub use crate::reactive::*;
    pub use crate::view::View;
}

pub use sycamore_macro::*;
