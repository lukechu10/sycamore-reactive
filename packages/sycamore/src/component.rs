//! The definition of the [`Component`] trait.

use crate::generic_node::GenericNode;
use crate::reactive::*;
use crate::view::View;

/// Trait that is implemented by components. Should not be implemented manually. Use the
/// [`component`](sycamore_macro::component) macro instead.
pub trait Component<G: GenericNode, Props> {
    /// Create a new component with an instance of the properties.
    fn create_component(&self, ctx: ScopeRef, props: Props) -> View<G>;
}

impl<G: GenericNode, Props, T> Component<G, Props> for T
where
    T: Fn(ScopeRef<'_, '_>, Props) -> View<G>,
{
    fn create_component(&self, ctx: ScopeRef, props: Props) -> View<G> {
        self(ctx, props)
    }
}

/// Instantiates a component.
#[inline(always)]
#[doc(hidden)]
pub fn instantiate<G: GenericNode, Props>(
    f: impl Fn(ScopeRef<'_, '_>, Props) -> View<G>,
    ctx: ScopeRef,
    props: Props,
) -> View<G> {
    if G::USE_HYDRATION_CONTEXT {
        #[cfg(feature = "experimental-hydrate")]
        return crate::utils::hydrate::hydrate_component(|| untrack(|| C::create_component(props)));
        #[cfg(not(feature = "experimental-hydrate"))]
        return untrack(|| f(ctx, props));
    } else {
        untrack(|| f(ctx, props))
    }
}
