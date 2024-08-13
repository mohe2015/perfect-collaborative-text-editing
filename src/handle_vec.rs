use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct HandleVec<T>(Vec<T>);

impl<T> HandleVec<T> {
    pub fn new() -> Self {
        HandleVec(Vec::new())
    }

    pub fn push(&mut self, value: T) -> Handle<T> {
        let handle = self.0.len();
        self.0.push(value);
        Handle(handle, PhantomData)
    }
}

pub struct Handle<T>(usize, PhantomData<T>);

impl<T> Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Handle").field(&self.0).finish()
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Index<Handle<T>> for HandleVec<T> {
    type Output = T;

    fn index(&self, handle: Handle<T>) -> &T {
        &self.0[handle.0]
    }
}

impl<T> IndexMut<Handle<T>> for HandleVec<T> {
    fn index_mut(&mut self, handle: Handle<T>) -> &mut Self::Output {
        &mut self.0[handle.0]
    }
}
