use crate::alloc::Alloc;
use core::iter::{Copied, Map};


pub trait FromIn<T, A: Alloc> {
    fn from_in(alloc: A, value: T) -> Self;
}


pub trait MapIt<A>: IntoIterator<Item = A> {
    fn map_it<B, F: FnMut(A) -> B>(self, f: F) -> Map<Self::IntoIter, F>;
}

impl<A, I: IntoIterator<Item = A>> MapIt<A> for I {
    fn map_it<B, F: FnMut(A) -> B>(self, f: F) -> Map<Self::IntoIter, F> {
        self.into_iter().map(f)
    }
}


pub trait CopyIt<'a, T: 'a + Copy>: IntoIterator<Item = &'a T> {
    fn copy_it(self) -> Copied<Self::IntoIter>;

    fn copy_map_it<B, F: FnMut(T) -> B>(self, f: F) -> Map<Copied<Self::IntoIter>, F>;
}

impl<'a, T: 'a + Copy, I: IntoIterator<Item = &'a T>> CopyIt<'a, T> for I {
    fn copy_it(self) -> Copied<Self::IntoIter> {
        self.into_iter().copied()
    }

    fn copy_map_it<B, F: FnMut(T) -> B>(self, f: F) -> Map<Copied<Self::IntoIter>, F> {
        self.into_iter().copied().map(f)
    }
}


pub trait InsertNew<T> {
    fn insert_new(&mut self, value: T) -> &mut T;
}

impl<T> InsertNew<T> for Option<T> {
    #[inline]
    fn insert_new(&mut self, value: T) -> &mut T {
        if self.is_some() { unreachable!() };
        self.insert(value)
    }
}


pub trait OkVal<T> {
    fn ok_val(self) -> (bool, T);
}

impl<T> OkVal<T> for Result<T, T> {
    #[inline]
    fn ok_val(self) -> (bool, T) {
        match self {
            Ok(val)  => (true, val),
            Err(val) => (false, val),
        }
    }
}

