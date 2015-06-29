#![feature(const_fn)]

#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
use std::cell::UnsafeCell;
use lazy_static::Lazy;

#[test]
fn lazy_alone() {
    static X: Lazy<u32> = Lazy::new(0);
    X.get(|| 1);
    assert_eq!(*X.get(|| 2), 1);
}

lazy_static_unboxed! {
    static NAT: u32 = { 0; times_two(3) };
    static ARRS: [Option<Box<u32>>; 3] = {
        [None, None, None];
        [Some(Box::new(1)), Some(Box::new(2)), Some(Box::new(3))]
    };
}

#[test]
fn unboxed() {
    assert_eq!(*NAT.get_or_init(), 6);
    assert_eq!(&*ARRS.get_or_init(),
               &[Some(Box::new(1)), Some(Box::new(2)), Some(Box::new(3))]);
}

lazy_static! {
    static ref NUMBER: u32 = times_two(3);
    static ref ARRAY_BOXES: [Box<u32>; 3] = [Box::new(1), Box::new(2), Box::new(3)];
    static ref STRING: String = "hello".to_string();
    static ref HASHMAP: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0, "abc");
        m.insert(1, "def");
        m.insert(2, "ghi");
        m
    };
    // This should not compile if the unsafe is removed.
    static ref UNSAFE: u32 = unsafe {
        std::mem::transmute::<i32, u32>(-1)
    };
    // This *should* triggger warn(dead_code) by design.
    static ref UNUSED: () = ();

}

fn times_two(n: u32) -> u32 {
    n * 2
}

#[test]
fn test_basic() {
    assert_eq!(&**STRING, "hello");
    assert_eq!(*NUMBER, 6);
    assert!(HASHMAP.get(&1).is_some());
    assert!(HASHMAP.get(&3).is_none());
    assert_eq!(&*ARRAY_BOXES, &[Box::new(1), Box::new(2), Box::new(3)]);
    assert_eq!(*UNSAFE, std::u32::MAX);
}

#[test]
fn test_repeat() {
    assert_eq!(*NUMBER, 6);
    assert_eq!(*NUMBER, 6);
    assert_eq!(*NUMBER, 6);
}

mod visibility {
    lazy_static! {
        pub static ref FOO: Box<u32> = Box::new(0);
    }
}

#[test]
fn test_visibility() {
    assert_eq!(*visibility::FOO, Box::new(0));
}

// This should not cause a warning about a missing Copy implementation
lazy_static! {
    pub static ref VAR: i32 = { 0 };
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct X;
struct Once(X);
const ONCE_INIT: Once = Once(X);
static DATA: X = X;
static ONCE: X = X;
fn require_sync() -> X { X }
fn transmute() -> X { X }
fn __builder() -> X { X }
fn test(_: Vec<X>) -> X { X }

// All these names should not be shadowed
lazy_static! {
    static ref ITEM_NAME_TEST: X = {
        test(vec![X, Once(X).0, ONCE_INIT.0, DATA, ONCE,
                  require_sync(), transmute(),
                  // Except this, which will sadly be shadowed by internals:
                  // __builder()
                  ])
    };
}

#[test]
fn item_name_shadowing() {
    assert_eq!(*ITEM_NAME_TEST, X);
}
