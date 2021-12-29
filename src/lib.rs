use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

/// Reactive context.
pub struct Ctx {}

pub struct Signal<'a, T> {
    phantom: PhantomData<&'a ()>,
    value: RefCell<Rc<T>>,
}

pub fn create_scope(mut f: impl FnMut(&Ctx)) {
    let ctx = Ctx {};
    f(&ctx);
}

impl Ctx {
    pub fn create_signal<T>(&self, value: T) -> Signal<T> {
        Signal {
            phantom: PhantomData,
            value: RefCell::new(Rc::new(value)),
        }
    }
}

impl<'a, T> Signal<'a, T> {
    pub fn get(&self) -> Rc<T> {
        self.value.borrow().clone()
    }

    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = Rc::new(value);
    }
}
