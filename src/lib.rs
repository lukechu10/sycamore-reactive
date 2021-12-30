use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::rc::Rc;

pub struct Scope<'a> {
    ctx: Pin<Box<Ctx<'a>>>,
}

/// Reactive context.
#[derive(Default)]
pub struct Ctx<'a> {
    effects: RefCell<Vec<Box<dyn FnMut() + 'a>>>,
    cleanups: RefCell<Vec<Box<dyn FnOnce() + 'a>>>,
    // Ctx owns the raw pointers in the Vec.
    signals: RefCell<Vec<*mut dyn Any>>,
    _pinned: PhantomPinned,
}

pub struct Signal<T: 'static> {
    value: RefCell<Rc<T>>,
}

trait AnySignal: Any {}

impl<T> AnySignal for Signal<T> {}

pub fn create_scope<'a, F>(f: F) -> Scope<'a>
where
    F: FnOnce(&'a Ctx<'a>),
{
    let ctx = Ctx::default();
    let scope = Scope { ctx: Box::pin(ctx) };
    // TODO: very unsafe
    // SAFETY: not
    f(unsafe { &*(scope.ctx.as_ref().get_ref() as *const Ctx<'a>) });
    scope
}

impl<'a> Ctx<'a> {
    pub fn create_signal<T>(&self, value: T) -> &Signal<T> {
        let signal = Signal {
            value: RefCell::new(Rc::new(value)),
        };
        let boxed = Box::new(signal);
        let ptr = Box::into_raw(boxed);
        self.signals.borrow_mut().push(ptr);
        // SAFETY: the address of the Signal<T> lives as long as 'a because:
        // - It is allocated on the heap and therefore has a stable address.
        // - self.signals is append only. That means that the Box<Signal<T>> will not be dropped until Self is dropped.
        // TODO: this is a violation of StackedBorrows when dropped.
        unsafe { &*ptr }
    }

    pub fn create_effect(&self, mut f: impl FnMut() + 'a) {
        f(); // TODO
        self.effects.borrow_mut().push(Box::new(f));
    }

    pub fn on_cleanup(&self, f: impl FnOnce() + 'a) {
        self.cleanups.borrow_mut().push(Box::new(f));
    }

    pub fn create_child_scope<F>(&self, f: F)
    where
        F: FnOnce(&Ctx),
    {
        let ctx = Ctx::default();
        f(&ctx);
    }
}

impl Drop for Ctx<'_> {
    fn drop(&mut self) {
        // Call cleanup functions.
        for cb in self.cleanups.take() {
            cb();
        }
        // Drop effects.
        drop(self.effects.take());
        // Drop signals.
        for i in self.signals.take() {
            // SAFETY: These pointers were allocated in Self::create_signal.
            unsafe {
                drop(Box::from_raw(i));
            }
        }
    }
}

impl<T> Signal<T> {
    pub fn get(&self) -> Rc<T> {
        self.value.borrow().clone()
    }

    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = Rc::new(value);
    }
}
