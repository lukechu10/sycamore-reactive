use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Reactive context.
#[derive(Default)]
pub struct Ctx<'a> {
    signals: RefCell<Vec<Box<dyn Any>>>,
    effects: RefCell<Vec<Box<dyn Fn() + 'a>>>,
    cleanups: RefCell<Vec<Box<dyn FnOnce() + 'a>>>,
}

pub struct Signal<T: 'static> {
    value: RefCell<Rc<T>>,
}

trait AnySignal: Any {}

impl<T> AnySignal for Signal<T> {}

pub fn create_scope<'a, F>(f: F)
where
    F: FnOnce(&Ctx<'a>),
{
    let ctx = Ctx::default();
    f(&ctx);
}

impl<'a> Ctx<'a> {
    pub fn create_signal<T>(&self, value: T) -> &'a Signal<T> {
        let signal = Signal {
            value: RefCell::new(Rc::new(value)),
        };
        let boxed = Box::new(signal);
        let ptr = boxed.as_ref() as *const Signal<T>;
        self.signals.borrow_mut().push(boxed);
        // SAFETY: the address of the Signal<T> lives as long as 'a because:
        // - It is allocated on the heap and therefore has a stable address.
        // - self.signals is append only. That means that the Box<Signal<T>> will not be dropped until Self is dropped.
        unsafe { &*ptr }
    }

    pub fn create_effect(&self, f: impl Fn() + 'a) {
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
        let cleanups = self.cleanups.take();
        for cb in cleanups {
            cb();
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
