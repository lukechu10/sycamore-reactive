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

// /// Instantiates a component.
// #[inline(always)]
// pub fn instantiate_component<G: GenericNode, Props, C: Component<G, Props>>(
//     props: Props,
// ) -> View<G> {
//     if G::USE_HYDRATION_CONTEXT {
//         #[cfg(feature = "experimental-hydrate")]
//         return crate::utils::hydrate::hydrate_component(|| untrack(||
// C::create_component(props)));         #[cfg(not(feature = "experimental-hydrate"))]
//         return untrack(|| C::create_component(props));
//     } else {
//         untrack(|| C::create_component(props))
//     }
// }

// /// Alias to [`instantiate_component`]. For use in proc-macro output.
// ///
// /// The double underscores (`__`) are to prevent conflicts with other trait methods. This is
// /// because we cannot use fully qualified syntax here because it prevents type inference.
// #[doc(hidden)]
// pub trait __InstantiateComponent<G: GenericNode, Props>: Component<G, Props> {
//     /// Alias to [`instantiate_component`]. For use in proc-macro output.
//     ///
//     /// The double underscores (`__`) are to prevent conflicts with other trait methods. This is
//     /// because we cannot use fully qualified syntax here because it prevents type inference.
//     #[doc(hidden)]
//     fn __instantiate_component(props: Props) -> View<G>;
// }

// impl<C, G, Props> __InstantiateComponent<G, Props> for C
// where
//     C: Component<G, Props>,
//     G: GenericNode,
// {
//     #[inline(always)]
//     fn __instantiate_component(props: Props) -> View<G> {
//         instantiate_component::<G, Props, C>(props)
//     }
// }
