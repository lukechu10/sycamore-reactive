//! Arena allocator for [`Scope`].

use std::cell::UnsafeCell;
use std::ops::Deref;

use crate::*;

/// A trait that is implemented for everything.
pub(crate) trait ReallyAny {}
impl<T> ReallyAny for T {}

/// A ref to data allocated on a [`Scope`].
#[derive(Debug, PartialEq, Eq)]
pub struct DataRef<'id, 'a, T: 'a> {
    _phantom: InvariantLifetime<'id>,
    value: &'a T,
}

impl<'id, 'a, T> DataRef<'id, 'a, T> {
    /// Create a new [`DataRef`] wrapping a raw reference.
    pub(crate) fn new(value: &'a T) -> Self {
        Self {
            _phantom: InvariantLifetime::default(),
            value,
        }
    }
}

impl<'id, 'a, T> Deref for DataRef<'id, 'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

// Manually implement `Clone` and `Copy` for `DataRef` to prevent having over-constrained type
// bounds.
impl<'id, 'a, T> Clone for DataRef<'id, 'a, T> {
    fn clone(&self) -> Self {
        Self {
            _phantom: InvariantLifetime::default(),
            value: self.value,
        }
    }
}
impl<'id, 'a, T> Copy for DataRef<'id, 'a, T> {}

/// Owned data that is tied to the lifetime of a [`Scope`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Data<'id, T> {
    _phantom: InvariantLifetime<'id>,
    value: T,
}

impl<'id, T> Data<'id, T> {
    /// Create a new [`Data`] wrapping a raw reference.
    pub(crate) fn new(value: T) -> Self {
        Self {
            _phantom: InvariantLifetime::default(),
            value,
        }
    }
}

impl<'id, T> Deref for Data<'id, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Default)]
pub(crate) struct ScopeArena<'a> {
    inner: UnsafeCell<Vec<*mut (dyn ReallyAny + 'a)>>,
}

impl<'a> ScopeArena<'a> {
    /// Allocate a value onto the arena. Returns a reference that lasts as long as the arena itself.
    pub fn alloc<T: 'a>(&'a self, value: T) -> &'a T {
        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed);
        unsafe {
            // SAFETY: The only place where self.inner.get() is mutably borrowed is right here.
            // It is impossible to have two alloc() calls on the same ScopeArena at the same time so
            // the mutable reference here is effectively unique.
            let inner_exclusive = &mut *self.inner.get();
            inner_exclusive.push(ptr);
        };

        // SAFETY: the address of the ptr lives as long as 'a because:
        // - It is allocated on the heap and therefore has a stable address.
        // - self.inner is append only. That means that the Box<_> will not be dropped until Self is
        //   dropped.
        // - The drop code for ScopeRef never reads the allocated value and therefore does not
        //   create a stacked-borrows violation.
        unsafe { &*ptr }
    }

    /// Cleanup the resources owned by the [`ScopeArena`]. This is automatically called in [`Drop`].
    /// However, [`dispose`](Self::dispose) only needs to take `&self` instead of `&mut self`.
    /// Dropping a [`ScopeArena`] will automatically call [`dispose`](Self::dispose).
    ///
    /// If a [`ScopeArena`] has already been disposed, calling it again does nothing.
    pub unsafe fn dispose(&self) {
        for &ptr in &*self.inner.get() {
            // SAFETY: the ptr was allocated in Self::alloc using Box::into_raw.
            let boxed: Box<dyn ReallyAny> = Box::from_raw(ptr);
            // Call the drop code for the allocated value.
            drop(boxed);
        }
        // Clear the inner Vec to prevent dangling references.
        drop(std::mem::take(&mut *self.inner.get()));
    }
}

impl<'a> Drop for ScopeArena<'a> {
    fn drop(&mut self) {
        unsafe { self.dispose() }
    }
}