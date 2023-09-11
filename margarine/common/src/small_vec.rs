use std::mem::MaybeUninit;

use crate::static_vec::StaticVec;

pub struct SmallVec<T, const N: usize>(Internal<T, N>);


enum Internal<T, const N: usize> {
    Static(StaticVec<T, N>),
    Vec(Vec<T>),
}


impl<T, const N: usize> SmallVec<T, N> {
    pub fn new() -> Self {
        Self(Internal::Static(StaticVec::new()))
    }


    pub fn push(&mut self, val: T) {
        self.reserve(1);
        match &mut self.0 {
            Internal::Static(v) => {
                if v.push(val).is_err() {
                    panic!("Huh?");
                }
            },

            
            Internal::Vec(v) => {
                v.push(val)
            },
        }
    }


    pub fn reserve(&mut self, size: usize) {
        match &mut self.0 {
            Internal::Static(v) => {
                if v.len() + size < N {
                    return
                }

                
                let mut vec = Vec::with_capacity(N + size);

                unsafe { std::ptr::copy_nonoverlapping(
                    v.as_ptr(),
                    vec.as_mut_ptr(),
                    v.len(),
                )};
                
                self.0 = Internal::Vec(vec);
            },

            
            Internal::Vec(v) => {
                v.reserve(size)
            },
        }
    }


    pub fn len(&self) -> usize {
        match &self.0 {
            Internal::Static(v) => v.len(),
            Internal::Vec(v) => v.len(),
        }
    }


    pub fn is_empty(&self) -> bool {
        match &self.0 {
            Internal::Static(v) => v.is_empty(),
            Internal::Vec(v) => v.is_empty(),
        }
    }


    pub fn as_slice(&self) -> &[T] {
        match &self.0 {
            Internal::Static(v) => v.as_slice(),
            Internal::Vec(v) => v.as_slice(),
        }
    }


    pub fn as_mut(&mut self) -> &mut [T] {
        match &mut self.0 {
            Internal::Static(v) => v.as_mut(),
            Internal::Vec(v) => v.as_mut(),
        }
    }
}


impl<T: core::fmt::Debug, const N: usize> core::fmt::Debug for SmallVec<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.as_slice())
            .finish()
    }
}


impl<T: Clone, const N: usize> Clone for SmallVec<T, N> {
    fn clone(&self) -> Self {
        let mut me = SmallVec::new();
        me.reserve(self.len());

        for i in self.as_slice() {
            me.push(i.clone())
        }

        me
    }
}


impl<T, const N: usize> IntoIterator for SmallVec<T, N> {
    type Item = T;

    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            len: self.len(),
            index: 0,
            kind: match self.0 {
                Internal::Static(mut v) => IntoIterType::Static(
                    std::mem::replace(
                        &mut v.values,
                        std::array::from_fn(|_| MaybeUninit::uninit()),
                    ).into_iter()),
                Internal::Vec(v) => IntoIterType::Vec(v.into_iter()),
            },
        }
    }
}


pub struct IntoIter<T, const N: usize> {
    index: usize,
    len: usize,
    kind: IntoIterType<T, N>,
}


enum IntoIterType<T, const N: usize> {
    Static(std::array::IntoIter<MaybeUninit<T>, N>),
    Vec(std::vec::IntoIter<T>),
}


impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;

        if self.index > self.len {
            return None
        }

        match &mut self.kind {
            IntoIterType::Static(v) => v.next().map(|x| unsafe { x.assume_init() }),
            IntoIterType::Vec(v) => v.next(),
        }
    }
}
