use crate::*;

pub(crate) struct EffectState<'a> {
    /// The callback when the effect is re-executed.
    cb: Box<dyn FnMut() + 'a>,
}

impl<'a> Ctx<'a> {
    pub fn create_effect(&self, mut f: impl FnMut() + 'a) {
        f(); // TODO
        let effect = EffectState {
            cb: Box::new(f),
        };
        self.effects.borrow_mut().push(effect);
    }
}
