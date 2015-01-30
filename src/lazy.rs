use core::prelude::*;

use core::cell::UnsafeCell;
use core::atomic::{self, AtomicUint};

// TODO polymorphic statics so the fields can be private
pub struct Lazy<T: Sync>(pub UnsafeCell<T>, pub AtomicUint);

#[inline]
impl<T: Sync> Lazy<T> {
    fn force_get<'a>(&'a self) -> &'a T {
        unsafe { &*self.0.get() }
    }

    pub fn get<'a, F>(&'a self, builder: F) -> &'a T
        where F: FnOnce() -> T
    {
        let mut status = self.1.load(atomic::Ordering::SeqCst);

        loop {
            match status {
                0 => {
                    status = self.1.compare_and_swap(0, 1, atomic::Ordering::SeqCst);
                    if status == 0 { // we init
                        unsafe { *self.0.get() = builder() };
                        status = 2;
                        self.1.store(status, atomic::Ordering::SeqCst);
                        return self.force_get(); // this line is strictly an optomization
                    }
                },
                1 => status = self.1.load(atomic::Ordering::SeqCst), // we spin
                _ => return self.force_get(),
            }
        }
    }
}

unsafe impl<T: Sync> Sync for Lazy<T> { }
