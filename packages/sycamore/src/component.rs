//! The definition of the [`Component`] trait.

use crate::generic_node::GenericNode;
use crate::reactive::*;
use crate::view::View;

/// Trait that is implemented by components. Should not be implemented manually. Use the
/// [`component`](sycamore_macro::component) macro instead.
pub trait Component<'a, G: GenericNode, Props: 'a> {
    /// Create a new component with an instance of the properties.
    fn create_component(&self, ctx: ScopeRef<'_, 'a>, props: Props) -> View<G>;
}

impl<'a, G: GenericNode, Props: 'a, T> Component<'a, G, Props> for T
where
    T: Fn(ScopeRef, Props) -> View<G>,
{
    fn create_component(&self, ctx: ScopeRef, props: Props) -> View<G> {
        self(ctx, props)
    }
}

/// Instantiates a component.
#[inline(always)]
#[doc(hidden)]
pub fn instantiate<'a, G: GenericNode, Props: 'a>(
    f: &dyn Component<'a, G, Props>,
    ctx: ScopeRef<'_, 'a>,
    props: Props,
) -> View<G> {
    if G::USE_HYDRATION_CONTEXT {
        #[cfg(feature = "experimental-hydrate")]
        return crate::utils::hydrate::hydrate_component(|| untrack(|| C::create_component(props)));
        #[cfg(not(feature = "experimental-hydrate"))]
        return untrack(|| f.create_component(ctx, props));
    } else {
        untrack(|| f.create_component(ctx, props))
    }
}
