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
    pub(crate) fn track(&self) {
        EFFECTS.with(|effects| {
            if let Some(last) = effects.borrow().last() {
                // SAFETY: See guarantee on EffectState within EFFECTS.
                let last = unsafe { &mut **last };
                // SAFETY: TODO
                last.add_dependency(unsafe { std::mem::transmute(self) });
            }
        });
    }
}

pub struct Signal<'a, T> {
    value: RefCell<Rc<T>>,
    emitter: SignalEmitter<'a>,
}

impl<'a, T> Signal<'a, T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value: RefCell::new(Rc::new(value)),
            emitter: Default::default(),
        }
    }

    pub fn get(&self) -> Rc<T> {
        self.emitter.track();
        self.value.borrow().clone()
    }

    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = Rc::new(value);
    }
}

pub(crate) trait AnySignal<'a> {
    fn subscribe(&self, cb: WeakEffectCallback<'a>);
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
