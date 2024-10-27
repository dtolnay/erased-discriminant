use std::any::TypeId;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::mem;
use std::ptr;

/// A type-erased version of `std::mem::Discriminant<T>`.
pub struct Discriminant {
    erased: Box<dyn ErasedDiscriminant>,
}

impl Discriminant {
    pub fn of<T>(value: &T) -> Self {
        let erased_nonstatic = Box::new(mem::discriminant(value)) as Box<dyn ErasedDiscriminant>;
        // SAFETY: while the enum type T may contain borrows, the discriminant
        // type Discriminant<T> definitely does not, despite T's type appearing
        // in Discriminant<T>'s type. All of the ErasedDiscriminant APIs (eq,
        // hash, fmt) operate correctly on Discriminant<T> after T's lifetime is
        // expired.
        let erased = unsafe {
            mem::transmute::<Box<dyn ErasedDiscriminant>, Box<dyn ErasedDiscriminant>>(
                erased_nonstatic,
            )
        };
        Discriminant { erased }
    }
}

trait ErasedDiscriminant: Debug + NonStaticAny {
    fn erased_eq(&self, other: &dyn ErasedDiscriminant) -> bool;
    fn erased_hash(&self, hasher: &mut dyn Hasher);
}

impl<T> ErasedDiscriminant for std::mem::Discriminant<T> {
    fn erased_eq(&self, other: &dyn ErasedDiscriminant) -> bool {
        other.type_id() == typeid::of::<std::mem::Discriminant<T>>()
            && PartialEq::eq(
                self,
                // SAFETY: self and other have the same type modulo lifetimes,
                // and std::mem::Discriminant<T>'s PartialEq is not sensitive to
                // lifetimes.
                unsafe { &*ptr::from_ref(other).cast::<std::mem::Discriminant<T>>() },
            )
    }

    fn erased_hash(&self, mut hasher: &mut dyn Hasher) {
        typeid::of::<Self>().hash(&mut hasher);
        self.hash(&mut hasher);
    }
}

impl Eq for Discriminant {}

impl PartialEq for Discriminant {
    fn eq(&self, other: &Self) -> bool {
        self.erased.erased_eq(&*other.erased)
    }
}

impl Hash for Discriminant {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.erased.erased_hash(hasher);
    }
}

impl Debug for Discriminant {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.erased, formatter)
    }
}

// Unsafe because unsafe code must be allowed to rely on `type_id` being
// correctly implemented.
unsafe trait NonStaticAny {
    fn type_id(&self) -> TypeId;
}

// SAFETY: correct implementation of `type_id`.
unsafe impl<T> NonStaticAny for T {
    fn type_id(&self) -> TypeId {
        typeid::of::<T>()
    }
}
