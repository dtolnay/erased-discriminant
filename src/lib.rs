#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)]

use std::any::TypeId;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::mem::{self, MaybeUninit};

/// A type-erased version of `std::mem::Discriminant<T>`.
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
                    .cast::<std::mem::Discriminant<T>>()
                    .write(discriminant);
            }
            data
        } else {
            MaybeUninit::new(Box::into_raw(Box::new(discriminant)).cast())
        };
        Discriminant {
            data,
            vtable: &DiscriminantVTable {
                eq: discriminant_eq::<T>,
                hash: discriminant_hash::<T>,
                debug: discriminant_debug::<T>,
                drop: discriminant_drop::<T>,
                type_id: typeid::of::<std::mem::Discriminant<T>>,
            },
        }
    }
}

fn small_discriminant<T>() -> bool {
    mem::size_of::<std::mem::Discriminant<T>>() <= mem::size_of::<*const ()>()
}

struct DiscriminantVTable {
    eq: unsafe fn(this: &Discriminant, other: &Discriminant) -> bool,
    hash: unsafe fn(this: &Discriminant, hasher: &mut dyn Hasher),
    debug: unsafe fn(this: &Discriminant, formatter: &mut fmt::Formatter) -> fmt::Result,
    drop: unsafe fn(this: &mut Discriminant),
    type_id: fn() -> TypeId,
}

unsafe fn as_ref<T>(this: &Discriminant) -> &std::mem::Discriminant<T> {
    unsafe {
        &*if small_discriminant::<T>() {
            this.data.as_ptr().cast()
        } else {
            this.data.assume_init().cast()
        }
    }
}

unsafe fn discriminant_eq<T>(this: &Discriminant, other: &Discriminant) -> bool {
    (other.vtable.type_id)() == typeid::of::<std::mem::Discriminant<T>>()
        && unsafe { as_ref::<T>(this) } == unsafe { as_ref::<T>(other) }
}

unsafe fn discriminant_hash<T>(this: &Discriminant, mut hasher: &mut dyn Hasher) {
    typeid::of::<std::mem::Discriminant<T>>().hash(&mut hasher);
    unsafe { as_ref::<T>(this) }.hash(&mut hasher);
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
            unsafe { Box::from_raw(this.data.assume_init().cast::<std::mem::Discriminant<T>>()) };
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
