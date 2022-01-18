use std::hash::Hash;

use crate::prelude::*;

/// Props for [`Keyed`].
pub struct KeyedProps<'id, 'a, T, F, G: GenericNode, K, Key>
where
    F: Fn(ScopeRef, &T) -> View<G> + 'a,
    K: Fn(&T) -> Key + 'a,
    Key: Clone + Hash + Eq,
    T: Clone + PartialEq,
{
    pub iterable: &'a ReadSignal<'id, 'a, Vec<T>>,
    pub template: F,
    pub key: K,
}

/// Keyed iteration. Use this instead of directly rendering an array of [`View`]s.
/// Using this will minimize re-renders instead of re-rendering every single node on every state
/// change.
///
/// For non keyed iteration, see [`Indexed`].
///
/// # Example
/// ```no_run
/// use sycamore::prelude::*;
///
/// let count = Signal::new(vec![1, 2]);
///
/// let node = view! {
///     Keyed(KeyedProps {
///         iterable: count.handle(),
///         template: |item| view! {
///             li { (item) }
///         },
///         key: |item| *item,
///     })
/// };
/// # let _ : View<DomNode> = node;
/// ```
#[component]
pub fn Keyed<'id, 'a, G: GenericNode, T, F, K, Key>(
    ctx: ScopeRef<'id, 'a>,
    props: KeyedProps<'id, 'a, T, F, G, K, Key>,
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
pub struct IndexedProps<'id, 'a, G: GenericNode, T, F>
where
    F: Fn(ScopeRef, &T) -> View<G> + 'a,
{
    pub iterable: &'a ReadSignal<'id, 'a, Vec<T>>,
    pub template: F,
}

/// Non keyed iteration (or keyed by index). Use this instead of directly rendering an array of
/// [`View`]s. Using this will minimize re-renders instead of re-rendering every single
/// node on every state change.
///
/// For keyed iteration, see [`Keyed`].
///
/// # Example
/// ```no_run
/// use sycamore::prelude::*;
///
/// let count = Signal::new(vec![1, 2]);
///
/// let node = view! {
///     Indexed(IndexedProps {
///         iterable: count.handle(),
///         template: |item| view! {
///             li { (item) }
///         },
///     })
/// };
/// # let _ : View<DomNode> = node;
/// ```
#[component]
pub fn Indexed<'id, 'a, G: GenericNode, T, F>(
    ctx: ScopeRef<'id, 'a>,
    props: IndexedProps<'id, 'a, G, T, F>,
) -> View<G>
where
    T: Clone + PartialEq,
    F: Fn(ScopeRef, &T) -> View<G> + 'a,
{
    let IndexedProps { iterable, template } = props;

    let mapped = ctx.map_indexed(iterable, template);
    View::new_dyn(ctx, || View::new_fragment(mapped.get().as_ref().clone()))
}
