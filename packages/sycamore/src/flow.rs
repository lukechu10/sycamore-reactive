use std::hash::Hash;

use crate::prelude::*;

/// Props for [`Keyed`].
pub struct KeyedProps<'a, T, F, G: GenericNode, K, Key>
where
    F: Fn(ScopeRef, &T) -> View<G> + 'a,
    K: Fn(&T) -> Key + 'a,
    Key: Clone + Hash + Eq,
    T: Clone + PartialEq,
{
    pub iterable: &'a ReadSignal<Vec<T>>,
    pub template: F,
    pub key: K,
}

/// Keyed iteration. Use this instead of directly rendering an array of [`View`]s.
/// Using this will minimize re-renders instead of re-rendering every single node on every state
/// change.
///
/// For non keyed iteration, see [`Indexed`].
#[component]
pub fn Keyed<'a, G: GenericNode, T, F, K, Key>(
    ctx: ScopeRef<'a>,
    props: KeyedProps<'a, T, F, G, K, Key>,
) -> View<G>
where
    F: Fn(ScopeRef, &T) -> View<G> + 'a,
    K: Fn(&T) -> Key + 'a,
    Key: Clone + Hash + Eq,
    T: Clone + Eq,
{
    let KeyedProps {
        iterable,
        template,
        key,
    } = props;

    let mapped = ctx.map_keyed(iterable, template, key);
    View::new_dyn(ctx, || View::new_fragment(mapped.get().as_ref().clone()))
}

/// Props for [`Indexed`].
pub struct IndexedProps<'a, G: GenericNode, T, F>
where
    F: Fn(ScopeRef, &T) -> View<G> + 'a,
{
    pub iterable: &'a ReadSignal<Vec<T>>,
    pub template: F,
}

/// Non keyed iteration (or keyed by index). Use this instead of directly rendering an array of
/// [`View`]s. Using this will minimize re-renders instead of re-rendering every single
/// node on every state change.
///
/// For keyed iteration, see [`Keyed`].
#[component]
pub fn Indexed<'a, G: GenericNode, T, F>(
    ctx: ScopeRef<'a>,
    props: IndexedProps<'a, G, T, F>,
) -> View<G>
where
    T: Clone + PartialEq,
    F: Fn(ScopeRef, &T) -> View<G> + 'a,
{
    let IndexedProps { iterable, template } = props;

    let mapped = ctx.map_indexed(iterable, template);
    View::new_dyn(ctx, || View::new_fragment(mapped.get().as_ref().clone()))
}