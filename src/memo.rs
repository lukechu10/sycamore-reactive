use std::cell::Cell;

use crate::*;

impl<'a> Ctx<'a> {
    /// Creates a memoized value from some signals. Also know as "derived stores".
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// # let disposer = create_scope(|ctx| {
    /// let state = ctx.create_signal(0);
    ///
    /// let double = ctx.create_memo(|| *state.get() * 2);
    /// assert_eq!(*double.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*double.get(), 2);
    /// # });
    /// # disposer();
    /// ```
    pub fn create_memo<U: 'a>(&'a self, f: impl FnMut() -> U + 'a) -> &'a ReadSignal<'a, U> {
        self.create_selector_with(f, |_, _| false)
    }

    /// Creates a memoized value from some signals. Also know as "derived stores".
    /// Unlike [`create_memo`](Self::create_memo), this function will not notify dependents of a change if the output is
    /// the same. That is why the output of the function must implement [`PartialEq`].
    ///
    /// To specify a custom comparison function, use [`create_selector_with`](Self::create_selector_with).
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// # let disposer = create_scope(|ctx| {
    /// let state = ctx.create_signal(0);
    ///
    /// let double = ctx.create_selector(|| *state.get() * 2);
    /// assert_eq!(*double.get(), 0);
    ///
    /// state.set(1);
    /// assert_eq!(*double.get(), 2);
    /// # });
    /// # disposer();
    /// ```
    pub fn create_selector<U: PartialEq + 'a>(
        &'a self,
        f: impl FnMut() -> U + 'a,
    ) -> &'a ReadSignal<'a, U> {
        self.create_selector_with(f, PartialEq::eq)
    }

    /// Creates a memoized value from some signals. Also know as "derived stores".
    /// Unlike [`create_memo`](Self::create_memo), this function will not notify dependents of a change if the output is
    /// the same.
    ///
    /// It takes a comparison function to compare the old and new value, which returns `true` if they
    /// are the same and `false` otherwise.
    ///
    /// To use the type's [`PartialEq`] implementation instead of a custom function, use
    /// [`create_selector`](Self::create_selector).
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

    /// An alternative to [`Signal::new`] that uses a reducer to get the next value.
    ///
    /// It uses a reducer function that takes the previous value and a message and returns the next
    /// value.
    ///
    /// Returns a [`ReadSignal`] and a dispatch function to send messages to the reducer.
    ///
    /// # Params
    /// * `initial` - The initial value of the state.
    /// * `reducer` - A function that takes the previous value and a message and returns the next value.
    ///
    /// # Example
    /// ```
    /// # use sycamore_reactive::*;
    /// enum Msg {
    ///     Increment,
    ///     Decrement,
    /// }
    ///
    /// # let disposer = create_scope(|ctx| {
    /// let (state, dispatch) = ctx.create_reducer(0, |state, msg: Msg| match msg {
    ///     Msg::Increment => *state + 1,
    ///     Msg::Decrement => *state - 1,
    /// });
    ///
    /// assert_eq!(*state.get(), 0);
    /// dispatch(Msg::Increment);
    /// assert_eq!(*state.get(), 1);
    /// dispatch(Msg::Decrement);
    /// assert_eq!(*state.get(), 0);
    /// # });
    /// # disposer();
    /// ```
    pub fn create_reducer<U, Msg>(
        &'a self,
        initial: U,
        reduce: impl Fn(&U, Msg) -> U + 'a,
    ) -> (&'a ReadSignal<U>, Rc<impl Fn(Msg) + 'a>) {
        let memo = self.create_signal(initial);

        let dispatcher = move |msg| {
            memo.set(reduce(&memo.get_untracked(), msg));
        };

        (&*memo, Rc::new(dispatcher))
    }
}
