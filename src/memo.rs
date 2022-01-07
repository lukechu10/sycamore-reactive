use std::cell::Cell;

use crate::*;

impl<'a> Ctx<'a> {
    pub fn create_memo<U: 'a>(&'a self, mut f: impl FnMut() -> U + 'a) -> &'a ReadSignal<'a, U> {
        let signal: Rc<Cell<Option<&Signal<U>>>> = Default::default();

        self.create_effect({
            let signal = signal.clone();
            move || {
                if let Some(signal) = signal.get() {
                    signal.set(f())
                } else {
                    signal.set(Some(self.create_signal(f())))
                }
            }
        });

        signal.get().unwrap()
    }
}
