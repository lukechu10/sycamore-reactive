use std::hash::Hash;

use crate::prelude::*;

/// Props for [`Keyed`].
pub struct KeyedProps<'a, T: 'static, F, G: GenericNode, K, Key>
where
    F: Fn(ScopeRef, T) -> View<G>,
    K: Fn(&T) -> Key,
    Key: Clone + Hash + Eq,
    T: Clone + PartialEq,
{
    pub iterable: &'a ReadSignal<'a, 'a, Vec<T>>,
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
pub fn Keyed<'a, G: GenericNode, T: 'static, F: 'static, K: 'static, Key: 'static>(
    ctx: ScopeRef<'a, 'a>,
    props: KeyedProps<'a, T, F, G, K, Key>,
) -> View<G>
where
    F: Fn(ScopeRef, T) -> View<G>,
    K: Fn(&T) -> Key,
    Key: Clone + Hash + Eq,
    T: Clone + Eq,
{
    let KeyedProps {
        iterable,
        template,
        key,
    } = props;

    let mapped = ctx.map_keyed(iterable, move |ctx, x| template(ctx, x.clone()), key);
    View::new_dyn(ctx, || View::new_fragment(mapped.get().as_ref().clone()))
}

// /// Props for [`Indexed`].
// pub struct IndexedProps<G: GenericNode, T: 'static, F>
// where
//     F: Fn(T) -> View<G>,
// {
//     pub iterable: ReadSignal<Vec<T>>,
//     pub template: F,
// }

// /// Non keyed iteration (or keyed by index). Use this instead of directly rendering an array of
// /// [`View`]s. Using this will minimize re-renders instead of re-rendering every single
// /// node on every state change.
// ///
// /// For keyed iteration, see [`Keyed`].
// ///
// /// # Example
// /// ```no_run
// /// use sycamore::prelude::*;
// ///
// /// let count = Signal::new(vec![1, 2]);
// ///
// /// let node = view! {
// ///     Indexed(IndexedProps {
// ///         iterable: count.handle(),
// ///         template: |item| view! {
// ///             li { (item) }
// ///         },
// ///     })
// /// };
// /// # let _ : View<DomNode> = node;
// /// ```
// #[component(Indexed<G>)]
// pub fn Indexed<T: 'static, F: 'static>(props: IndexedProps<T, F, G>) -> View<G>
// where
//     T: Clone + PartialEq,
//     F: Fn(T) -> View<G>,
// {
//     let IndexedProps { iterable, template } = props;

//     let mut mapped = map_indexed(iterable, move |x| template(x.clone()));
//     View::new_dyn(move || View::new_fragment(mapped()))
// }
