
use std::hash::Hasher;

use std::hash::Hash;

use std::rc::Rc;

/// Allows hashing a `Rc<T>` value by its address and not its contents.
/// This struct additionally allows cloning and comparing equality
/// by pointer reference.
#[derive(Debug)]
pub struct RcHashable<T>(pub Rc<T>);

impl<T> Hash for RcHashable<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state)
    }
}

impl<T> Clone for RcHashable<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> Eq for RcHashable<T> {}

impl<T> PartialEq for RcHashable<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> RcHashable<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }
}
