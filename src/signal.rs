use std::ops::Deref;

use crate::effect::EFFECTS;
use crate::*;

type WeakEffectCallback<'a> = Weak<RefCell<dyn FnMut() + 'a>>;
type EffectCallbackPtr<'a> = *const RefCell<dyn FnMut() + 'a>;

/// A struct for managing subscriptions to signals.
#[derive(Default)]
pub struct SignalEmitter<'a>(RefCell<IndexMap<EffectCallbackPtr<'a>, WeakEffectCallback<'a>>>);

impl<'a> SignalEmitter<'a> {
    /// Adds a callback to the subscriber list. If the callback is already a subscriber, does nothing.
    pub(crate) fn subscribe(&self, cb: WeakEffectCallback<'a>) {
        self.0.borrow_mut().insert(cb.as_ptr(), cb);
    }

    /// Removes a callback from the subscriber list. If the callback is not a subscriber, does
    /// nothing.
    pub(crate) fn unsubscribe(&self, cb: EffectCallbackPtr<'a>) {
        self.0.borrow_mut().remove(&cb);
    }

    /// Track the current signal in the effect scope.
    pub fn track(&self) {
        EFFECTS.with(|effects| {
            if let Some(last) = effects.borrow().last() {
                // SAFETY: See guarantee on EffectState within EFFECTS.
                let last = unsafe { &mut **last };
                // SAFETY: TODO
                last.add_dependency(unsafe { std::mem::transmute(self) });
            }
        });
    }

    /// Calls all the subscribers without modifying the state.
    /// This can be useful when using patterns such as inner mutability where the state updated will
    /// not be automatically triggered. In the general case, however, it is preferable to use
    /// [`Signal::set()`] instead.
    pub fn trigger_subscribers(&self) {
        // Clone subscribers to prevent modifying list when calling callbacks.
        let subscribers = self.0.borrow().clone();
        for subscriber in subscribers.values() {
            // subscriber might have already been destroyed in the case of nested effects
            if let Some(callback) = subscriber.upgrade() {
                // Might already be inside a callback, if infinite loop.
                // Do nothing if infinite loop.
                if let Ok(mut callback) = callback.try_borrow_mut() {
                    callback()
                }
            }
        }
    }
}

/// A read-only [`Signal`].
pub struct ReadSignal<'a, T> {
    value: RefCell<Rc<T>>,
    emitter: SignalEmitter<'a>,
}

impl<'a, T> ReadSignal<'a, T> {
    // Get the current value of the state. When called inside a reactive scope, calling this will
    /// add itself to the scope's dependencies.
    ///
    /// # Example
    /// ```rust
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*state.get(), 1);
    /// # });
    /// ```
    pub fn get(&self) -> Rc<T> {
        self.emitter.track();
        self.value.borrow().clone()
    }

    /// Get the current value of the state, without tracking this as a dependency if inside a
    /// reactive context.
    ///
    /// # Example
    ///
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(1);
    /// let double = ctx.create_memo(|| *state.get_untracked() * 2);
    /// assert_eq!(*double.get(), 2);
    ///
    /// state.set(2);
    /// // double value should still be old value because state was untracked
    /// assert_eq!(*double.get(), 2);
    /// # });
    /// ```
    pub fn get_untracked(&self) -> Rc<T> {
        self.value.borrow().clone()
    }

    /// Creates a mapped [`ReadSignal`]. This is equivalent to using [`create_memo`](Scope::create_memo).
    ///
    /// # Example
    /// ```rust
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(1);
    /// let double = state.map(&ctx, |&x| x * 2);
    /// assert_eq!(*double.get(), 2);
    ///
    /// state.set(2);
    /// assert_eq!(*double.get(), 4);
    /// # });
    /// ```
    pub fn map<U>(&self, ctx: ScopeRef<'a>, mut f: impl FnMut(&T) -> U + 'a) -> &'a ReadSignal<U> {
        ctx.create_memo(move || f(&self.get()))
    }
}

/// Reactive state that can be updated and subscribed to.
pub struct Signal<'a, T>(ReadSignal<'a, T>);

impl<'a, T> Signal<'a, T> {
    /// Create a new [`Signal`] with the specified value.
    pub(crate) fn new(value: T) -> Self {
        Self(ReadSignal {
            value: RefCell::new(Rc::new(value)),
            emitter: Default::default(),
        })
    }

    /// Set the current value of the state.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// # create_scope_immediate(|ctx| {
    /// let state = ctx.create_signal(0);
    /// assert_eq!(*state.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*state.get(), 1);
    /// # });
    /// ```
    pub fn set(&self, value: T) {
        *self.0.value.borrow_mut() = Rc::new(value);
        self.0.emitter.trigger_subscribers();
    }
}

impl<'a, T> Deref for Signal<'a, T> {
    type Target = ReadSignal<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) trait AnySignal<'a> {
    /// Subscribe the effect to the signal.
    fn subscribe(&self, cb: WeakEffectCallback<'a>);
    /// Unsubscribe the effect from the signal.
    fn unsubscribe(&self, cb: EffectCallbackPtr<'a>);
}

impl<'a, T> AnySignal<'a> for Signal<'a, T> {
    fn subscribe(&self, cb: WeakEffectCallback<'a>) {
        self.emitter.subscribe(cb);
    }

    fn unsubscribe(&self, cb: EffectCallbackPtr<'a>) {
        self.emitter.unsubscribe(cb);
    }
}

impl<'a, T> AnySignal<'a> for ReadSignal<'a, T> {
    fn subscribe(&self, cb: WeakEffectCallback<'a>) {
        self.emitter.subscribe(cb);
    }

    fn unsubscribe(&self, cb: EffectCallbackPtr<'a>) {
        self.emitter.unsubscribe(cb);
    }
}

/// A signal that is not bound to a [`Scope`].
#[derive(Clone)]
pub struct RcSignal<T>(Rc<Signal<'static, T>>);

impl<T> Deref for RcSignal<T> {
    type Target = Signal<'static, T>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub fn create_rc_signal<T>(value: T) -> RcSignal<T> {
    RcSignal(Rc::new(Signal::new(value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            assert_eq!(*state.get(), 0);

            state.set(1);
            assert_eq!(*state.get(), 1);
        });
    }

    #[test]
    fn signal_composition() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);

            let double = || *state.get() * 2;

            assert_eq!(double(), 0);

            state.set(1);
            assert_eq!(double(), 2);
        });
    }

    #[test]
    fn read_signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            let readonly: &ReadSignal<i32> = state.deref();

            assert_eq!(*readonly.get(), 0);

            state.set(1);
            assert_eq!(*readonly.get(), 1);
        });
    }

    #[test]
    fn rc_signal() {
        create_scope_immediate(|ctx| {
            let rc_state = create_rc_signal(0);
            let rc_state_cloned = rc_state.clone();
            let double = ctx.create_memo(move || *rc_state_cloned.get() * 2);
            assert_eq!(*double.get(), 0);

            rc_state.set(1);
            assert_eq!(*double.get(), 2);
        });
    }
}
