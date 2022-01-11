//! Context state management.

use crate::*;

impl<'id, 'a> Scope<'id, 'a> {
    /// TODO: docs
    pub fn provide_context<T: 'static>(&'a self, value: T) {
        let type_id = TypeId::of::<T>();
        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed);
        if self.contexts.borrow_mut().insert(type_id, ptr).is_some() {
            panic!("existing context with type exists already");
        }
    }

    /// TODO: docs
    pub fn try_use_context<T: 'static>(&'a self) -> Option<DataRef<'id, 'a, T>> {
        let type_id = TypeId::of::<T>();
        let this = Some(self);
        while let Some(current) = this {
            if let Some(value) = current.contexts.borrow_mut().get(&type_id) {
                // SAFETY: value lives at least as long as 'a:
                // - Lifetime of value is 'a if it is allocated on the current scope.
                // - Lifetime of value is longer than 'a if it is allocated on a parent scope.
                // - 'a is variant because it is an immutable reference.
                let value = unsafe { &**value };
                let value = value.downcast_ref::<T>().unwrap();
                let data = DataRef {
                    _phantom: InvariantLifetime::default(),
                    value,
                };
                return Some(data);
            }
        }
        None
    }

    /// TODO: docs
    #[track_caller]
    pub fn use_context<T: 'static>(&'a self) -> DataRef<'id, 'a, T> {
        self.try_use_context().expect("context not found for type")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context() {
        create_scope_immediate(|ctx| {
            ctx.provide_context(42i32);
            let x = ctx.use_context::<i32>();
            assert_eq!(*x, 42);
        });
    }
}
