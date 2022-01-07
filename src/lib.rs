//! Reactive primitives for Sycamore.

pub mod effect;
pub mod memo;
pub mod signal;

use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};

pub use indexmap::IndexMap;

pub use effect::*;
pub use signal::*;

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
    child_ctx: RefCell<Vec<*mut Scope<'a>>>,
    // Ctx owns the raw pointers in the Vec.
    signals: RefCell<Vec<*mut (dyn AnySignal<'a> + 'a)>>,
    parent: Option<&'a Self>,
}

impl<'a> Scope<'a> {
    /// Create a new [`Scope`]. This function is deliberately not `pub` because it should not be possible to
    /// access a [`Scope`] directly on the stack.
    pub(crate) fn new() -> Self {
        Self {
            effects: Default::default(),
            cleanups: Default::default(),
            child_ctx: Default::default(),
            signals: Default::default(),
            parent: None,
        }
    }
}

/// A reference to a [`Scope`].
pub type ScopeRef<'a> = &'a Scope<'a>;

/// Creates a reactive scope.
///
/// Returns a disposer function which will release the memory owned by the [`Ctx`].
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
    let ctx = ManuallyDrop::new(Scope::new());
    let boxed = Box::new(ctx);
    let ptr = Box::into_raw(boxed);
    // SAFETY: Safe because heap allocated value has stable address.
    f(unsafe { &*ptr });
    // Ownership of the context is passed into the closure.
    move || {
        // SAFETY: Safe because ptr created using Box::into_raw.
        let boxed = unsafe { Box::from_raw(ptr) };
        boxed.dispose();
    }
}

/// Creates a reactive scope, runs the callback, and disposes the scope immediately.
pub fn create_scope_immediate(f: impl FnOnce(ScopeRef<'_>)) {
    let disposer = create_scope(f);
    disposer();
}

impl<'a> Scope<'a> {
    /// Create a new [`Signal`] under the current [`Ctx`].
    /// The created signal lasts as long as the context and cannot be used outside of the context.
    pub fn create_signal<T>(&'a self, value: T) -> &'a Signal<'a, T> {
        let signal = Signal::new(value);
        let boxed = Box::new(signal);
        let ptr = Box::into_raw(boxed);
        self.signals.borrow_mut().push(ptr);
        // SAFETY: the address of the Signal<T> lives as long as 'a because:
        // - It is allocated on the heap and therefore has a stable address.
        // - self.signals is append only. That means that the Box<Signal<T>> will not be dropped until Self is dropped.
        unsafe { &*ptr }
    }

    /// Adds a callback that is called when the scope is destroyed.
    pub fn on_cleanup(&self, f: impl FnOnce() + 'a) {
        self.cleanups.borrow_mut().push(Box::new(f));
    }

    /// Create a child scope.
    pub fn create_child_scope(&'a self, f: impl FnOnce(ScopeRef<'a>)) -> impl FnOnce() + 'a {
        let mut ctx = Scope::new();
        ctx.parent = Some(self);
        let boxed = Box::new(ctx);
        let ptr = Box::into_raw(boxed);
        self.child_ctx.borrow_mut().push(ptr);
        // SAFETY: the address of the Ctx lives as long as 'a because:
        // - It is allocated on the heap and therefore has a stable address.
        // - self.child_ctx is append only. That means that the Box<Ctx> will not be dropped until Self is dropped.
        f(unsafe { &*ptr });
        move || {
            // SAFETY: Safe because ptr created using Box::into_raw and closure cannot live longer than 'a.
            let ctx = unsafe { &*ptr };
            ctx.dispose();
        }
    }

    /// Cleanup the resources owned by the [`Ctx`]. This is not automatically called in [`Drop`] because that
    /// would violate Rust's aliasing rules. However, [`dispose`](Self::dispose) only needs to take `&self`
    /// instead of `&mut self`. Dropping a [`Ctx`] will automatically call [`dispose`](Self::dispose).
    ///
    /// If a [`Ctx`] has already been disposed, calling it again does nothing.
    pub(crate) fn dispose(&self) {
        // Drop effects.
        drop(self.effects.take());
        // Drop child contexts.
        for i in self.child_ctx.take() {
            // SAFETY: These pointers were allocated in Self::create_child_scope.
            let ctx = unsafe { Box::from_raw(i) };
            // Dispose of ctx if it has not already been disposed.
            ctx.dispose()
        }
        // Call cleanup functions.
        for cb in self.cleanups.take() {
            cb();
        }
        // Drop signals.
        for i in self.signals.take() {
            // SAFETY: These pointers were allocated in Self::create_signal.
            unsafe {
                drop(Box::from_raw(i));
            }
        }
    }
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        self.dispose();
    }
}
