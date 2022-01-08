//! Signals - The building blocks of reactivity.

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
        // Subscriber order is reversed because effects attach subscribers at the end of the
        // effect scope. This will ensure that outer effects re-execute before inner effects,
        // preventing inner effects from running twice.
        for subscriber in subscribers.values().rev() {
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
    /// Get the current value of the state. When called inside a reactive scope, calling this will
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
    #[must_use = "to only subscribe the signal without using the value, use .track() instead"]
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
    #[must_use = "discarding the returned value does nothing"]
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
    #[must_use]
    pub fn map<U>(&self, ctx: ScopeRef<'a>, mut f: impl FnMut(&T) -> U + 'a) -> &'a ReadSignal<U> {
        ctx.create_memo(move || f(&self.get()))
    }

    /// When called inside a reactive scope, calling this will add itself to the scope's dependencies.
    ///
    /// To both track and get the value of the signal, use [`Signal::get`] instead.
    pub fn track(&self) {
        self.emitter.track();
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

    /// Set the current value of the state _without_ triggering subscribers.
    ///
    /// Make sure you know what you are doing because this can make state inconsistent.
    pub fn set_silent(&self, value: T) {
        *self.0.value.borrow_mut() = Rc::new(value);
    }
}

impl<'a, T: Default> Signal<'a, T> {
    /// Take the current value out and replace it with the default value.
    ///
    /// This will notify and update any effects and memos that depend on this value.
    pub fn take(&self) -> Rc<T> {
        let ret = self.0.value.take();
        self.0.emitter.trigger_subscribers();
        ret
    }

    /// Take the current value out and replace it with the default value _without_ triggering subscribers.
    ///
    /// Make sure you know what you are doing because this can make state inconsistent.
    pub fn take_silent(&self) -> Rc<T> {
        self.0.value.take()
    }
}

impl<'a, T> Deref for Signal<'a, T> {
    type Target = ReadSignal<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A trait that is implemented for all signals that are allocated on a [`Scope`].
pub(crate) trait AnySignal<'a> {}
impl<'a, T> AnySignal<'a> for Signal<'a, T> {}
impl<'a, T> AnySignal<'a> for ReadSignal<'a, T> {}

/// A signal that is not bound to a [`Scope`].
///
/// Sometimes, it is useful to have a signal that can escape the enclosing [reactive scope](Scope).
/// However, this cannot be achieved simply with [`Scope::create_signal`] because the resulting [`Signal`]
/// is tied to the [`Scope`] by it's lifetime. The [`Signal`] can only live as long as the [`Scope`].
///
/// With [`RcSignal`] on the other hand, the lifetime is not tied to a [`Scope`]. Memory is managed using a
/// reference-counted smart pointer ([`Rc`]). What this means is that [`RcSignal`] cannot implement the [`Copy`]
/// trait and therefore needs to be manually cloned into all closures where it is used.
///
/// In general, [`Scope::create_signal`] should be preferred, both for performance and ergonomics.
///
/// # Usage
///
/// To create a [`RcSignal`], use the [`create_rc_signal`] function.
///
/// # Example
/// ```
/// # use sycamore_reactive::*;
/// let mut outer = None;
///
/// create_scope_immediate(|ctx| {
/// // Even though the RcSignal is created inside a reactive scope, it can escape out of it.
/// let rc_state = create_rc_signal(0);
/// let rc_state_cloned = rc_state.clone();
/// let double = ctx.create_memo(move || *rc_state_cloned.get() * 2);
/// assert_eq!(*double.get(), 0);
///
/// rc_state.set(1);
/// assert_eq!(*double.get(), 2);
///
/// // This isn't possible with simply ctx.create_signal()
/// outer = Some(rc_state);
/// });
/// ```
#[derive(Clone)]
pub struct RcSignal<T>(Rc<Signal<'static, T>>);

impl<T> Deref for RcSignal<T> {
    type Target = Signal<'static, T>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// Create a new [`RcSignal`] with the specified initial value.
///
/// For more details, check the documentation for [`RcSignal`].
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
    fn set_silent_signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            let double = state.map(ctx, |&x| x * 2);

            assert_eq!(*double.get(), 0);
            state.set_silent(1);
            assert_eq!(*double.get(), 0); // double value is unchanged.
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
    fn map_signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);
            let double = state.map(ctx, |&x| x * 2);

            assert_eq!(*double.get(), 0);
            state.set(1);
            assert_eq!(*double.get(), 2);
        });
    }

    #[test]
    fn take_signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(123);

            let x = state.take();
            assert_eq!(*x, 123);
            assert_eq!(*state.get(), 0);
        });
    }

    #[test]
    fn take_silent_signal() {
        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(123);
            let double = state.map(ctx, |&x| x * 2);

            // Do not trigger subscribers.
            state.take_silent();
            assert_eq!(*state.get(), 0);
            assert_eq!(*double.get(), 246);
        });
    }

    #[test]
    fn rc_signal() {
        let mut outer = None;
        create_scope_immediate(|ctx| {
            let rc_state = create_rc_signal(0);
            let rc_state_cloned = rc_state.clone();
            let double = ctx.create_memo(move || *rc_state_cloned.get() * 2);
            assert_eq!(*double.get(), 0);

            rc_state.set(1);
            assert_eq!(*double.get(), 2);

            outer = Some(rc_state);
        });
        assert_eq!(*outer.unwrap().get(), 1);
    }
}
