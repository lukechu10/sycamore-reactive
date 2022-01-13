#![deny(unsafe_code)]

pub mod generic_node;
pub mod reactive {
    pub use sycamore_reactive::*;
}
pub mod component;
pub mod utils;
pub mod view;
