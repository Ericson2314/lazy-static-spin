/*!
A macro for declaring lazily evaluated statics.

Using this macro, it is possible to have `static`s that require code to be
executed at runtime in order to be initialized.
This includes anything requiring heap allocations, like vectors or hash maps,
as well as anything that requires function calls to be computed.

# Syntax

```ignore
lazy_static! {
    [pub] static ref NAME_1: TYPE_1 = EXPR_1;
    [pub] static ref NAME_2: TYPE_2 = EXPR_2;
    ...
    [pub] static ref NAME_N: TYPE_N = EXPR_N;
}
```

# Semantic

For a given `static ref NAME: TYPE = EXPR;`, the macro generates a
unique type that implements `Deref<TYPE>` and stores it in a static with name `NAME`.

On first deref, `EXPR` gets evaluated and stored internally, such that all further derefs
can return a reference to the same object.

Like regular `static mut`s, this macro only works for types that fulfill the `Sync`
trait.

# Example

Using the macro:

```rust
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

lazy_static! {
    static ref HASHMAP: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0, "foo");
        m.insert(1, "bar");
        m.insert(2, "baz");
        m
    };
    static ref COUNT: usize = HASHMAP.len();
    static ref NUMBER: u32 = times_two(21);
}

fn times_two(n: u32) -> u32 { n * 2 }

fn main() {
    println!("The map has {} entries.", *COUNT);
    println!("The entry for `0` is \"{}\".", HASHMAP.get(&0).unwrap());
    println!("A expensive calculation on a static results in: {}.", *NUMBER);
}
```

# Implementation details

The `Deref` implementation uses a hidden `static mut` that is guarded by a atomic check
using the `sync::Once` abstraction. All lazily evaluated values are currently
put in a heap allocated box, due to the Rust language currently not providing any way to
define uninitialized `static mut` values.

*/

pub use self::lazy::Lazy;

mod lazy;

#[macro_export]
macro_rules! lazy_static {
    (static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        lazy_static!(PRIV static ref $N : $T = $e; $($t)*);
    };
    (pub static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        lazy_static!(PUB static ref $N : $T = $e; $($t)*);
    };
    ($VIS:ident static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        lazy_static_unboxed!($VIS static $N : ::std::ptr::Unique<$T> = {
            ::std::ptr::Unique::new(0 as *mut $T);
            ::std::ptr::Unique::new(unsafe {
                ::std::mem::transmute::<Box<$T>, *mut $T>(Box::new($e))
            })
        };);
        impl ::std::ops::Deref for $N {
            type Target = $T;
            fn deref<'a>(&'a self) -> &'a $T {
                unsafe {
                    let slf: &'static Self = ::std::mem::transmute(self);
                    ::std::mem::transmute::<_, &'a $T>(slf.get_or_init().get())
                }
            }
        }

        lazy_static!($($t)*);
    };
    () => ()
}


#[macro_export]
macro_rules! lazy_static_unboxed {
    (static $N:ident : $T:ty = { $u:expr ; $e:expr}; $($t:tt)*) => {
        lazy_static_unboxed!(PRIV static $N : $T = $e; $($t)*);
    };
    (pub static $N:ident : $T:ty = { $u:expr ; $e:expr}; $($t:tt)*) => {
        lazy_static_unboxed!(PUB static $N : $T = $e; $($t)*);
    };
    ($VIS:ident static $N:ident : $T:ty = { $u:expr ; $e:expr}; $($t:tt)*) => {
        lazy_static_unboxed!(MK $VIS struct $N<$T>);
        lazy_static_unboxed!(MK $VIS static $N : $N = $N {
            inner: ::lazy_static::Lazy(
                ::std::cell::UnsafeCell {
                    value: $u
                },
                ::std::sync::ONCE_INIT)
        });
        impl $N {
            fn get_or_init<'a>(&'static self) -> &'static $T {
                fn builder() -> $T { $e }
                self.inner.get(builder)
            }
        }

        lazy_static_unboxed!($($t)*);
    };
    (MK PUB struct $N:ident<$T:ty>) => {
        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        pub struct $N { inner: ::lazy_static::Lazy<$T> }
    };
    (MK PRIV struct $N:ident<$T:ty>) => {
        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        struct $N { inner: ::lazy_static::Lazy<$T> }
    };
    (MK PUB  static $i:ident : $t:ty = $e:expr) => {pub static $i : $t = $e;};
    (MK PRIV static $i:ident : $t:ty = $e:expr) =>     {static $i : $t = $e;};
    () => ();
}
