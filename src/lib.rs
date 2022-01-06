pub mod effect;
pub mod signal;

use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};

use effect::EffectState;
use indexmap::IndexMap;
use signal::{AnySignal, Signal};

/// Reactive context.
#[derive(Default)]
pub struct Ctx<'a> {
    effects: RefCell<Vec<Rc<RefCell<Option<EffectState<'a>>>>>>,
    cleanups: RefCell<Vec<Box<dyn FnOnce() + 'a>>>,
    child_ctx: RefCell<Vec<*mut Ctx<'a>>>,
    // Ctx owns the raw pointers in the Vec.
    signals: RefCell<Vec<*mut (dyn AnySignal<'a> + 'a)>>,
}

pub type CtxRef<'a> = &'a Ctx<'a>;

/// Create a reactive scope.
///
/// # Examples
///
/// ```
/// use sycamore_reactive::create_scope;
///
/// let disposer = create_scope(|ctx| {
///     // Use ctx here.
/// });
/// disposer();
/// ```
#[must_use = "not calling the disposer function will result in a memory leak"]
pub fn create_scope(f: impl FnOnce(CtxRef<'_>)) -> impl FnOnce() + 'static {
    let ctx = ManuallyDrop::new(Ctx::default());
    let boxed = Box::new(ctx);
    let ptr = Box::into_raw(boxed);
    // SAFETY: Safe because heap allocated value has stable address.
    f(unsafe { &*ptr });
    move || unsafe {
        // SAFETY: Safe because ptr created using Box::into_raw.
        let boxed = Box::from_raw(ptr);
        boxed.dispose();
    }
}

impl<'a> Ctx<'a> {
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

    pub fn on_cleanup(&self, f: impl FnOnce() + 'a) {
        self.cleanups.borrow_mut().push(Box::new(f));
    }

    pub fn create_child_scope<F>(&self, f: F)
    where
        F: FnOnce(CtxRef<'a>),
    {
        let ctx = Ctx::default();
        let boxed = Box::new(ctx);
        let ptr = Box::into_raw(boxed);
        self.child_ctx.borrow_mut().push(ptr);
        // SAFETY: the address of the Ctx lives as long as 'a because:
        // - It is allocated on the heap and therefore has a stable address.
        // - self.child_ctx is append only. That means that the Box<Ctx> will not be dropped until Self is dropped.
        f(unsafe { &*ptr });
    }

    pub fn dispose(&self) {
        // Drop effects.
        drop(self.effects.take());
        // Drop child contexts.
        for i in self.child_ctx.take() {
            // SAFETY: These pointers were allocated in Self::create_child_scope.
            unsafe {
                drop(Box::from_raw(i));
            }
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

impl Drop for Ctx<'_> {
    fn drop(&mut self) {
        self.dispose();
    }
}
