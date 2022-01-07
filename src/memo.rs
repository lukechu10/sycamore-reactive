use std::cell::Cell;

use crate::*;

impl<'a> Ctx<'a> {
    pub fn create_memo<U: 'a>(&'a self, f: impl FnMut() -> U + 'a) -> &'a ReadSignal<'a, U> {
        self.create_selector_with(f, |_, _| false)
    }

    pub fn create_selector<U: PartialEq + 'a>(
        &'a self,
        f: impl FnMut() -> U + 'a,
    ) -> &'a ReadSignal<'a, U> {
        self.create_selector_with(f, PartialEq::eq)
    }

    pub fn create_selector_with<U: 'a>(
        &'a self,
        mut f: impl FnMut() -> U + 'a,
        eq_f: impl Fn(&U, &U) -> bool + 'a,
    ) -> &'a ReadSignal<'a, U> {
        let signal: Rc<Cell<Option<&Signal<U>>>> = Default::default();

        self.create_effect({
            let signal = signal.clone();
            move || {
                if let Some(signal) = signal.get() {
                    let new = f();
                    // Check if new value is different from old value.
                    if !eq_f(&new, &*signal.get()) {
                        signal.set(f())
                    }
                } else {
                    signal.set(Some(self.create_signal(f())))
                }
            }
        });

        signal.get().unwrap()
    }
}
