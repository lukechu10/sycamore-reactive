//! Reactive primitives for Sycamore.

#![warn(missing_docs)]

pub mod effect;
pub mod iter;
pub mod memo;
pub mod signal;

pub use effect::*;
pub use signal::*;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use indexmap::IndexMap;
use slotmap::{DefaultKey, SlotMap};

/// A trait that is implemented for everything.
trait ReallyAny {}
impl<T> ReallyAny for T {}

/// A reactive scope.
///
/// The only way to ever use a scope should be behind a reference.
/// It should never be possible to access a raw [`Scope`] on the stack.
///
/// The intended way to access a [`Scope`] is with the [`create_scope`] function.
///
/// For convenience, the [`ScopeRef`] type alias is defined as a reference to a [`Scope`].
pub struct Scope<'a> {
    effects: RefCell<Vec<Rc<RefCell<Option<EffectState<'a>>>>>>,
    cleanups: RefCell<Vec<Box<dyn FnOnce() + 'a>>>,
    child_ctx: RefCell<SlotMap<DefaultKey, *mut Scope<'a>>>,
    signals: RefCell<Vec<*mut (dyn AnySignal<'a> + 'a)>>,
    refs: RefCell<Vec<*mut (dyn ReallyAny + 'a)>>,
    parent: Option<*const Self>,
}

impl<'a> Scope<'a> {
    /// Create a new [`Scope`]. This function is deliberately not `pub` because it should not be
    /// possible to access a [`Scope`] directly on the stack.
    pub(crate) fn new() -> Self {
        Self {
            effects: Default::default(),
            cleanups: Default::default(),
            child_ctx: Default::default(),
            signals: Default::default(),
            refs: Default::default(),
            parent: None,
        }
    }
}

/// A reference to a [`Scope`].
pub type ScopeRef<'a> = &'a Scope<'a>;

/// Creates a reactive scope.
///
/// Returns a disposer function which will release the memory owned by the [`Scope`].
/// Failure to call the disposer function will result in a memory leak.
///
/// # Examples
///
/// ```
/// # use sycamore_reactive::*;
/// let disposer = create_scope(|ctx| {
///     // Use ctx here.
/// });
/// disposer();
/// ```
#[must_use = "not calling the disposer function will result in a memory leak"]
pub fn create_scope(f: impl FnOnce(ScopeRef<'_>)) -> impl FnOnce() {
    let ctx = Scope::new();
    let boxed = Box::new(ctx);
    let ptr = Box::into_raw(boxed);
    // SAFETY: Safe because heap allocated value has stable address.
    f(unsafe { &*ptr });
    // Ownership of the context is passed into the closure.
    move || {
        // SAFETY: Safe because ptr created using Box::into_raw.
        let boxed = unsafe { Box::from_raw(ptr) };
        // SAFETY: Outside of call to f.
        unsafe { boxed.dispose(); }
    }
}

/// Creates a reactive scope, runs the callback, and disposes the scope immediately.
pub fn create_scope_immediate(f: impl FnOnce(ScopeRef<'_>)) {
    let disposer = create_scope(f);
    disposer();
}

impl<'a> Scope<'a> {
    /// Create a new [`Signal`] under the current [`Scope`].
    /// The created signal lasts as long as the scope and cannot be used outside of the scope.
    pub fn create_signal<T>(&'a self, value: T) -> &'a Signal<'a, T> {
        let signal = Signal::new(value);
        let boxed = Box::new(signal);
        let ptr = Box::into_raw(boxed);
        self.signals.borrow_mut().push(ptr);
        // SAFETY: the address of the Signal<T> lives as long as 'a because:
        // - It is allocated on the heap and therefore has a stable address.
        // - self.signals is append only. That means that the Box<Signal<T>> will not be dropped
        //   until Self is dropped.
        unsafe { &*ptr }
    }

    /// Allocate a new arbitrary value under the current [`Scope`].
    /// The allocated value lasts as long as the scope and cannot be used outside of the scope.
    pub fn create_ref<T: 'a>(&'a self, value: T) -> &'a T {
        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed);
        self.refs.borrow_mut().push(ptr);
        // SAFETY: the address of the ref lives as long as 'a because:
        // - It is allocated on the heap and therefore has a stable address.
        // - self.signals is append only. That means that the Box<_> will not be dropped until Self
        //   is dropped.
        unsafe { &*ptr }
    }

    /// Adds a callback that is called when the scope is destroyed.
    pub fn on_cleanup(&self, f: impl FnOnce() + 'a) {
        self.cleanups.borrow_mut().push(Box::new(f));
    }

    /// Create a child scope.
    ///
    /// Returns a disposer function which will release the memory owned by the [`Scope`]. If the
    /// disposer function is never called, the child scope will be disposed automatically when the
    /// parent scope is disposed.
    ///
    /// # Lifetime of the child scope
    ///
    /// The lifetime of the child scope is strictly a subset of the lifetime of the parent scope.
    /// ```txt
    /// [------------'a-------------]
    ///      [---------'b--------]
    /// 'a: lifetime of parent
    /// 'b: lifetime of child
    /// ```
    /// If the disposer is never called, the lifetime `'b` lasts as long as `'a`.
    /// As such, it is impossible for anything allocated in the child scope to escape into the
    /// parent scope.
    // TODO: should be compile_fail
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let mut outer = None;
    /// let disposer = ctx.create_child_scope(|ctx| {
    ///     outer = Some(ctx);
    ///     //           ^^^
    /// });
    /// disposer();
    /// let _ = outer.unwrap();
    /// # });
    /// ```
    pub fn create_child_scope<'b, F>(&'a self, f: F) -> impl FnOnce() + 'a
    where
        'a: 'b,
        F: FnOnce(ScopeRef<'b>),
    {
        let mut ctx: Scope = Scope::new();
        // SAFETY: TODO
        ctx.parent = Some(unsafe { std::mem::transmute(self as *const _) });
        let boxed = Box::new(ctx);
        let ptr = Box::into_raw(boxed);
        let key = self
            .child_ctx
            .borrow_mut()
            // SAFETY: TODO
            .insert(unsafe { std::mem::transmute(ptr) });
        // SAFETY: the address of the Ctx lives as long as 'a because:
        // - It is allocated on the heap and therefore has a stable address.
        // - self.child_ctx is append only. That means that the Box<Ctx> will not be dropped until
        //   Self is dropped.
        f(unsafe { &*ptr });
        move || {
            let ctx = self.child_ctx.borrow_mut().remove(key).unwrap();
            // SAFETY: Safe because ptr created using Box::into_raw and closure cannot live longer
            // than 'a.
            let ctx = unsafe { Box::from_raw(ctx) };
            // SAFETY: Outside of call to f.
            unsafe {
                ctx.dispose();
            }
        }
    }

    /// Cleanup the resources owned by the [`Scope`]. This is not automatically called in [`Drop`]
    /// because that would violate Rust's aliasing rules. However, [`dispose`](Self::dispose)
    /// only needs to take `&self` instead of `&mut self`. Dropping a [`Scope`] will
    /// automatically call [`dispose`](Self::dispose).
    ///
    /// If a [`Scope`] has already been disposed, calling it again does nothing.
    ///
    /// # Safety
    ///
    /// `dispose` should not be called inside the `create_scope` or `create_child_scope` closure.
    pub(crate) unsafe fn dispose(&self) {
        // Drop child contexts.
        for &i in self.child_ctx.take().values() {
            // SAFETY: These pointers were allocated in Self::create_child_scope.
            let ctx = Box::from_raw(i);
            // Dispose of ctx if it has not already been disposed.
            ctx.dispose()
        }
        // Drop effects.
        drop(self.effects.take());
        // Call cleanup functions in an untracked scope.
        untrack(|| {
            for cb in self.cleanups.take() {
                cb();
            }
        });
        // Drop signals.
        for i in self.signals.take() {
            // SAFETY: These pointers were allocated in Self::create_signal.
            drop(Box::from_raw(i));
        }
        // Drop refs.
        for i in self.refs.take() {
            // SAFETY: These pointers were allocated in Self::create_ref.
            drop(Box::from_raw(i));
        }
    }
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        // SAFETY: scope cannot be dropped while it is borrowed inside closure.
        unsafe { self.dispose() };
    }
}

#[cfg(test)]
mod tests {
    use crate::{create_scope, create_scope_immediate};

    #[test]
    fn refs() {
        let disposer = create_scope(|ctx| {
            let r = ctx.create_ref(0);
            ctx.on_cleanup(move || {
                let _ = r; // r can be accessed inside scope here.
                dbg!(r);
            })
        });
        disposer();
    }

    #[test]
    fn cleanup() {
        create_scope_immediate(|ctx| {
            let cleanup_called = ctx.create_signal(false);
            let disposer = ctx.create_child_scope(|ctx| {
                ctx.on_cleanup(move || {
                    cleanup_called.set(true);
                });
            });
            assert!(!*cleanup_called.get());
            disposer();
            assert!(*cleanup_called.get());
        });
    }

    #[test]
    fn cleanup_in_effect() {
        create_scope_immediate(|ctx| {
            let trigger = ctx.create_signal(());

            let counter = ctx.create_signal(0);

            ctx.create_effect_scoped(|ctx| {
                trigger.track();

                ctx.on_cleanup(|| {
                    counter.set(*counter.get() + 1);
                });
            });

            assert_eq!(*counter.get(), 0);

            trigger.set(());
            assert_eq!(*counter.get(), 1);

            trigger.set(());
            assert_eq!(*counter.get(), 2);
        });
    }

    #[test]
    fn cleanup_is_untracked() {
        create_scope_immediate(|ctx| {
            let trigger = ctx.create_signal(());

            let counter = ctx.create_signal(0);

            ctx.create_effect_scoped(|ctx| {
                counter.set(*counter.get_untracked() + 1);

                ctx.on_cleanup(|| {
                    trigger.track(); // trigger should not be tracked
                });
            });

            assert_eq!(*counter.get(), 1);

            trigger.set(());
            assert_eq!(*counter.get(), 1);
        });
    }

    #[test]
    fn can_store_disposer_in_own_signal() {
        create_scope_immediate(|ctx| {
            let signal = ctx.create_signal(None);
            let disposer = ctx.create_child_scope(|_ctx| {});
            signal.set(Some(disposer));
        });
    }
}
