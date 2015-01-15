use std::cell::UnsafeCell;
use std::sync::Once; //ONCE_INIT

// TODO polymorphic statics so the fields can be private
pub struct Lazy<T: Sync>(pub UnsafeCell<T>, pub Once);

#[inline]
impl<T: Sync> Lazy<T> {
    fn force_get<'a>(&'a self) -> &'a T {
        unsafe { &*self.0.get() }
    }

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
