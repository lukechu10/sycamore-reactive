//! Utilities for components.

use crate::generic_node::GenericNode;
use crate::reactive::*;
use crate::view::View;

/// Runs the given closure inside a new component scope. In other words, this does the following:
/// * If hydration is enabled, create a new hydration context.
/// * Create a new untracked scope (see [`untrack`]).
/// * Call the closure `f` passed to this function.
#[doc(hidden)]
pub fn component_scope<G: GenericNode>(f: impl FnOnce() -> View<G>) -> View<G> {
    if G::USE_HYDRATION_CONTEXT {
        #[cfg(feature = "experimental-hydrate")]
        return crate::utils::hydrate::hydrate_component(|| untrack(f));
        #[cfg(not(feature = "experimental-hydrate"))]
        return untrack(f);
    } else {
        untrack(f)
    }
}
