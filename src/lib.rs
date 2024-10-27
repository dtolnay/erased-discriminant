//! [![github]](https://github.com/dtolnay/erased-discriminant)&ensp;[![crates-io]](https://crates.io/crates/erased-discriminant)&ensp;[![docs-rs]](https://docs.rs/erased-discriminant)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! This crate provides a `Discriminant` type that behaves like
//! `core::mem::Discriminant<T>` but without the generic type parameter `T`.
//! With this, we can build collections such as HashSet that contain
//! discriminants from a mixture of different enum types.
//!
//! ```
//! use erased_discriminant::Discriminant;
//! use std::collections::HashSet;
//!
//! enum Enum {
//!     A(i32),
//!     B,
//! }
//!
//! enum DifferentEnum {
//!     A,
//! }
//!
//! let mut set = HashSet::new();
//! set.insert(Discriminant::of(&Enum::A(99)));
//! set.insert(Discriminant::of(&Enum::B));
//! set.insert(Discriminant::of(&DifferentEnum::A));
//! assert_eq!(set.len(), 3);
//! ```

#![no_std]
#![doc(html_root_url = "https://docs.rs/erased-discriminant/1.0.0")]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::doc_markdown, clippy::missing_safety_doc)]

extern crate alloc;

use alloc::boxed::Box;
use core::any::TypeId;
use core::fmt::{self, Debug};
use core::hash::{Hash, Hasher};
use core::mem::{self, MaybeUninit};

/// A type-erased version of `core::mem::Discriminant<T>`.
pub struct Discriminant {
    data: MaybeUninit<*mut ()>,
    vtable: &'static DiscriminantVTable,
}

impl Discriminant {
    pub fn of<T>(value: &T) -> Self {
        let discriminant = mem::discriminant(value);
        let data = if small_discriminant::<T>() {
            let mut data = MaybeUninit::<*mut ()>::uninit();
            unsafe {
                data.as_mut_ptr()
                    .cast::<core::mem::Discriminant<T>>()
                    .write(discriminant);
            }
            data
        } else {
            MaybeUninit::new(Box::into_raw(Box::new(discriminant)).cast::<()>())
        };
        Discriminant {
            data,
            vtable: &DiscriminantVTable {
                eq: discriminant_eq::<T>,
                hash: discriminant_hash::<T>,
                clone: discriminant_clone::<T>,
                debug: discriminant_debug::<T>,
                drop: discriminant_drop::<T>,
                type_id: typeid::of::<core::mem::Discriminant<T>>,
            },
        }
    }
}

fn small_discriminant<T>() -> bool {
    mem::size_of::<core::mem::Discriminant<T>>() <= mem::size_of::<*const ()>()
}

struct DiscriminantVTable {
    eq: unsafe fn(this: &Discriminant, other: &Discriminant) -> bool,
    hash: unsafe fn(this: &Discriminant, hasher: &mut dyn Hasher),
    clone: unsafe fn(this: &Discriminant) -> Discriminant,
    debug: unsafe fn(this: &Discriminant, formatter: &mut fmt::Formatter) -> fmt::Result,
    drop: unsafe fn(this: &mut Discriminant),
    type_id: fn() -> TypeId,
}

unsafe fn as_ref<T>(this: &Discriminant) -> &core::mem::Discriminant<T> {
    unsafe {
        &*if small_discriminant::<T>() {
            this.data.as_ptr().cast::<core::mem::Discriminant<T>>()
        } else {
            this.data.assume_init().cast::<core::mem::Discriminant<T>>()
        }
    }
}

unsafe fn discriminant_eq<T>(this: &Discriminant, other: &Discriminant) -> bool {
    (other.vtable.type_id)() == typeid::of::<core::mem::Discriminant<T>>()
        && unsafe { as_ref::<T>(this) } == unsafe { as_ref::<T>(other) }
}

unsafe fn discriminant_hash<T>(this: &Discriminant, mut hasher: &mut dyn Hasher) {
    typeid::of::<core::mem::Discriminant<T>>().hash(&mut hasher);
    unsafe { as_ref::<T>(this) }.hash(&mut hasher);
}

unsafe fn discriminant_clone<T>(this: &Discriminant) -> Discriminant {
    if small_discriminant::<T>() {
        Discriminant {
            data: this.data,
            vtable: this.vtable,
        }
    } else {
        let discriminant = unsafe { *this.data.assume_init().cast::<core::mem::Discriminant<T>>() };
        Discriminant {
            data: MaybeUninit::new(Box::into_raw(Box::new(discriminant)).cast::<()>()),
            vtable: this.vtable,
        }
    }
}

unsafe fn discriminant_debug<T>(
    this: &Discriminant,
    formatter: &mut fmt::Formatter,
) -> fmt::Result {
    Debug::fmt(unsafe { as_ref::<T>(this) }, formatter)
}

unsafe fn discriminant_drop<T>(this: &mut Discriminant) {
    if !small_discriminant::<T>() {
        let _ =
            unsafe { Box::from_raw(this.data.assume_init().cast::<core::mem::Discriminant<T>>()) };
    }
}

impl Eq for Discriminant {}

impl PartialEq for Discriminant {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (self.vtable.eq)(self, other) }
    }
}

impl Hash for Discriminant {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        unsafe { (self.vtable.hash)(self, hasher) };
    }
}

impl Clone for Discriminant {
    fn clone(&self) -> Self {
        unsafe { (self.vtable.clone)(self) }
    }
}

impl Debug for Discriminant {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        unsafe { (self.vtable.debug)(self, formatter) }
    }
}

impl Drop for Discriminant {
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self) };
    }
}
