use std::cell::UnsafeCell;
use std::sync::{Once, ONCE_INIT};

pub struct Lazy<T: Sync>(UnsafeCell<T>, Once);

#[inline]
impl<T: Sync> Lazy<T> {
    #[inline]
    pub const fn new(init: T) -> Self {
        Lazy(UnsafeCell::new(init), ONCE_INIT)
    }

    #[inline]
    fn force_get<'a>(&'a self) -> &'a T {
        unsafe { &*self.0.get() }
    }

    #[inline]
    pub fn get<F>(&'static self, builder: F) -> &'static T
        where F: FnOnce() -> T
    {
        self.1.call_once(move || unsafe {
            *self.0.get() = builder()
        });
        self.force_get()
    }
}

unsafe impl<T: Sync> Sync for Lazy<T> { }
