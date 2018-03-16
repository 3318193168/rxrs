//taken from https://github.com/crossbeam-rs/crossbeam-utils/blob/master/src/atomic_option.rs

use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

unsafe impl<T: Send> Send for AtomicOption<T> {}
unsafe impl<T: Send> Sync for AtomicOption<T> {}

#[derive(Debug)]
pub struct AtomicOption<T> {
    inner: AtomicPtr<T>,
}

impl<T> Drop for AtomicOption<T> {
    #[inline(always)]
    fn drop(&mut self) {
        let inner = self.inner.load(Ordering::Relaxed);
        if !inner.is_null() {
            unsafe {
                drop(Box::from_raw(inner));
            }
        }
    }
}

impl<T> AtomicOption<T> {

    pub fn new() -> Self {
        AtomicOption { inner: AtomicPtr::new(ptr::null_mut()) }
    }

    pub fn with(t:T) -> Self {
        let o = AtomicOption::new();
        o.swap(t, Ordering::SeqCst);
        o
    }

    #[inline(always)]
    fn swap_inner(&self, ptr: *mut T, order: Ordering) -> Option<Box<T>> {
        let old = self.inner.swap(ptr, order);
        if old.is_null() {
            None
        } else {
            Some(unsafe { Box::from_raw(old) })
        }
    }

    // allows re-use of allocation
    #[inline(always)]
    pub fn swap_box(&self, t: Box<T>, order: Ordering) -> Option<Box<T>> {
        self.swap_inner(Box::into_raw(t), order)
    }

    #[inline(always)]
    pub fn swap(&self, t: T, order: Ordering) -> Option<T> {
        self.swap_box(Box::new(t), order).map(|old| *old)
    }

    #[inline(always)]
    pub fn take(&self, order: Ordering) -> Option<T> {
        self.swap_inner(ptr::null_mut(), order).map(|old| *old)
    }
}

impl<T> Default for AtomicOption<T> {
    fn default() -> Self {
        Self::new()
    }
}