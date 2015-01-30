/*!
A macro for declaring lazily evaluated statics.

Using this macro, it is possible to have `static`s that require code to be
executed at runtime in order to be initialized.
This includes anything requiring heap allocations, like vectors or hash maps,
as well as anything that requires function calls to be computed.

# Syntax

```ignore
lazy_static_spin! {
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
extern crate core;
extern crate alloc;
#[macro_use]
extern crate lazy_static_spin;

use std::collections::HashMap;

lazy_static_spin! {
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

In a freestanding context with allocation:

```rust
#![no_std]
#![feature(core)]
#![feature(alloc)]
#![feature(collections)]


extern crate core;
extern crate alloc;
extern crate collections;
#[macro_use]
extern crate std;
#[macro_use]
extern crate lazy_static_spin;

use collections::Vec;

lazy_static_spin! {
    static ref NUMBER: u32 = 5;
    static ref ARRAY: Vec<u32> = {
        let mut v = Vec::new();
        v.push(*NUMBER);
        v.push(*NUMBER);
        v
    };
}

fn times_two(n: u32) -> u32 { n * 2 }

fn main() {
    assert_eq!(*ARRAY, vec!(5, 5));
}
```

In a freestanding context Without allocation:

```rust
#![no_std]

extern crate core;
#[macro_use]
extern crate std;
#[macro_use]
extern crate lazy_static_spin;

lazy_static_unboxed_spin! {
    static COUNT: u32 = {0; 5};
    static NUMBER: u32 = {0; times_two(*COUNT.get_or_init() as u32)};
}

fn times_two(n: u32) -> u32 { n * 2 }

fn main() {
    assert_eq!(*NUMBER.get_or_init(), 10);
}
```

# Implementation details

The `Deref` implementation uses a hidden `static mut` that is guarded by a atomic check
using the `sync::Once` abstraction. All lazily evaluated values are currently
put in a heap allocated box, due to the Rust language currently not providing any way to
define uninitialized `static mut` values.

*/

#![no_std]
#![feature(core)]

extern crate core;

#[cfg(test)]
extern crate std;


pub use self::lazy::Lazy;

mod lazy;

#[macro_export]
macro_rules! lazy_static_spin {
    (static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        lazy_static_spin!(PRIV static ref $N : $T = $e; $($t)*);
    };
    (pub static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        lazy_static_spin!(PUB static ref $N : $T = $e; $($t)*);
    };
    ($VIS:ident static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        lazy_static_unboxed_spin!($VIS static $N : ::core::ptr::Unique<$T> = {
            ::core::ptr::Unique(0 as *mut $T);
            ::core::ptr::Unique(unsafe {
                ::core::mem::transmute::<::alloc::boxed::Box<$T>, *mut $T>(box() ($e))
            })
        };);
        impl ::core::ops::Deref for $N {
            type Target = $T;
            fn deref<'a>(&'a self) -> &'a $T {
                unsafe {
                    ::core::mem::transmute::<*mut $T, &'a $T>(self.get_or_init().0)
                }
            }
        }

        lazy_static_spin!($($t)*);
    };
    () => ()
}


#[macro_export]
macro_rules! lazy_static_unboxed_spin {
    (static $N:ident : $T:ty = { $u:expr ; $e:expr}; $($t:tt)*) => {
        lazy_static_unboxed_spin!(PRIV static $N : $T = {$u; $e}; $($t)*);
    };
    (pub static $N:ident : $T:ty = { $u:expr ; $e:expr}; $($t:tt)*) => {
        lazy_static_unboxed_spin!(PUB static $N : $T = {$u; $e}; $($t)*);
    };
    ($VIS:ident static $N:ident : $T:ty = { $u:expr ; $e:expr}; $($t:tt)*) => {
        lazy_static_unboxed_spin!(MK $VIS struct $N<$T>);
        lazy_static_unboxed_spin!(MK $VIS static $N : $N = $N {
            inner: ::lazy_static_spin::Lazy(
                ::core::cell::UnsafeCell {
                    value: $u
                },
                ::core::atomic::ATOMIC_UINT_INIT)
        });
        impl $N {
            fn get_or_init<'a>(&'a self) -> &'a $T {
                fn builder() -> $T { $e }
                self.inner.get(builder)
            }
        }

        lazy_static_unboxed_spin!($($t)*);
    };
    (MK PUB struct $N:ident<$T:ty>) => {
        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        pub struct $N { inner: ::lazy_static_spin::Lazy<$T> }
    };
    (MK PRIV struct $N:ident<$T:ty>) => {
        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        struct $N { inner: ::lazy_static_spin::Lazy<$T> }
    };
    (MK PUB  static $i:ident : $t:ty = $e:expr) => {pub static $i : $t = $e;};
    (MK PRIV static $i:ident : $t:ty = $e:expr) =>     {static $i : $t = $e;};
    () => ();
}
