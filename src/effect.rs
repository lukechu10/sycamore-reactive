use std::collections::HashSet;

use crate::signal::SignalEmitter;
use crate::*;

thread_local! {
    /// While the [`EffectState`] is inside the Vec, it is owned by [`EFFECTS`].
    /// Because this is a global variable, the lifetime is necessarily `'static`. However, that does not mean
    /// that it can last forever. The `EffectState` should only be used the time it is inside [`EFFECTS`].
    pub(crate) static EFFECTS: RefCell<Vec<*mut EffectState<'static>>> = Default::default();
}

pub(crate) struct EffectState<'a> {
    /// The callback when the effect is re-executed.
    cb: Rc<RefCell<dyn FnMut() + 'a>>,
    dependencies: HashSet<EffectDependency<'a>>,
}

/// Implements reference equality for [`AnySignal`]s.
pub struct EffectDependency<'a>(&'a SignalEmitter<'a>);

impl<'a> std::cmp::PartialEq for EffectDependency<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'a> std::cmp::Eq for EffectDependency<'a> {}

impl<'a> std::hash::Hash for EffectDependency<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as *const SignalEmitter<'a>).hash(state);
    }
}

impl<'a> EffectState<'a> {
    // Clears the dependencies (both links and backlinks).
    /// Should be called when re-executing an effect to recreate all dependencies.
    pub fn clear_dependencies(&mut self) {
        for dependency in &self.dependencies {
            dependency.0.unsubscribe(Rc::as_ptr(&self.cb));
        }
        self.dependencies.clear();
    }

    pub fn add_dependency(&mut self, signal: &'a SignalEmitter<'a>) {
        self.dependencies.insert(EffectDependency(signal));
    }
}

impl<'a> Ctx<'a> {
    pub fn create_effect(&self, f: impl FnMut() + 'a) {
        let f = Rc::new(RefCell::new(f));

        let effect = Rc::new(RefCell::new(None::<EffectState<'a>>));
        let cb = Rc::new(RefCell::new({
            let effect = Rc::downgrade(&effect);
            move || {
                EFFECTS.with(|effects| {
                    // Record initial effect stack length to verify that it is the same after.
                    let initial_effect_stack_len = effects.borrow().len();
                    // Upgrade the effect to an Rc now so that it is valid for the rest of the callback.
                    let effect_ref = effect.upgrade().unwrap();

                    // Take effect out.
                    let mut effect = effect_ref.take().unwrap();
                    effect.clear_dependencies();

                    // Push the effect onto the effect stack.
                    let boxed = Box::new(effect);
                    let ptr: *mut EffectState<'a> = Box::into_raw(boxed);
                    // Push the effect onto the effect stack so that it is visible by signals.
                    effects
                        .borrow_mut()
                        .push(ptr as *mut () as *mut EffectState<'static>);
                    // Now we can call the user-provided function.
                    f.borrow_mut()();
                    // Pop the effect from the effect stack.
                    effects.borrow_mut().pop().unwrap();

                    //  SAFETY: Now that the effect has been popped from EFFECTS,
                    // get a boxed EffectState with the correct lifetime back.
                    let boxed = unsafe { Box::from_raw(ptr) };

                    // For all the signals collected by the EffectState,
                    // we need to add backlinks from the signal to the effect, so that
                    // updating the signal will trigger the effect.
                    for emitter in &boxed.dependencies {
                        emitter.0.subscribe(Rc::downgrade(&boxed.cb));
                    }

                    // Get the effect state back into the Rc
                    *effect_ref.borrow_mut() = Some(*boxed);

                    debug_assert_eq!(effects.borrow().len(), initial_effect_stack_len);
                });
            }
        }));

        // Initialize initial effect state.
        *effect.borrow_mut() = Some(EffectState {
            cb: cb.clone(),
            dependencies: HashSet::new(),
        });

        // Initial callback call to get everything started.
        cb.borrow_mut()();

        self.effects.borrow_mut().push(effect);

        // let effect: EffectState<'a> = EffectState {
        //     cb,
        //     dependencies: HashSet::new(),
        // };
        // let boxed = Box::new(effect);
        // let ptr: *mut EffectState<'a> = Box::into_raw(boxed);
        // EFFECTS.with(move |effects| {
        //     // Push the effect onto the effect stack so that it is visible by signals.
        //     effects
        //         .borrow_mut()
        //         .push(ptr as *mut () as *mut EffectState<'static>);
        //     // Now we can call the user-provided function.
        //     f.borrow_mut()();
        //     // Pop the effect from the effect stack.
        //     effects.borrow_mut().pop().unwrap();
        // });
        // //  SAFETY: Now that the effect has been popped from EFFECTS,
        // // get a boxed EffectState with the correct lifetime back.
        // let boxed = unsafe { Box::from_raw(ptr) };
        // self.effects.borrow_mut().push(*boxed);
    }
}
