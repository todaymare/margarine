use crate::fmt::Arguments;
use crate::alloc::{Alloc, GlobalAlloc};
use crate::vec::ZVec;


#[derive(Clone)]
pub struct String<A: Alloc = GlobalAlloc>(ZVec<u8, A>);

impl String<GlobalAlloc> {
    #[inline(always)]
    pub fn new() -> Self {
        Self::new_in(GlobalAlloc)
    }

    #[inline(always)]
    pub fn with_cap(cap: usize) -> Self {
        Self::with_cap_in(GlobalAlloc, cap)
    }


    #[inline(always)]
    pub fn from_str(str: &str) -> Self {
        Self::from_str_in(GlobalAlloc, str)
    }

    #[inline(always)]
    pub fn from_fmt(args: Arguments) -> Self {
        Self::from_fmt_in(GlobalAlloc, args)
    }
}

impl<A: Alloc> String<A> {
    #[inline(always)]
    pub fn new_in(alloc: A) -> Self {
        Self(ZVec::new_in(alloc))
    }

    #[inline(always)]
    pub fn with_cap_in(alloc: A, cap: usize) -> Self {
        Self(ZVec::with_cap_in(alloc, cap))
    }


    #[inline]
    pub fn from_str_in(alloc: A, str: &str) -> Self {
        Self(ZVec::from_slice_in(alloc, str.as_bytes()))
    }

    #[inline(always)]
    pub fn from_fmt_in(alloc: A, args: Arguments) -> Self {
        let mut this = String::new_in(alloc);
        this.push_fmt(args);
        return this;
    }


    #[inline(always)]
    pub fn cap(&self) -> usize { self.0.cap() }

    #[inline(always)]
    pub fn len(&self) -> usize { self.0.len() }

    #[inline(always)]
    pub unsafe fn inner_mut(&mut self) -> &mut ZVec<u8, A> { &mut self.0 }


    #[inline(always)]
    pub fn as_str(&self) -> &str {
        unsafe { crate::str::from_utf8_unchecked(self.0.as_slice()) }
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }


    #[inline]
    pub fn push(&mut self, s: &str) {
        self.0.extend_from_slice(s.as_bytes());
    }

    #[inline]
    pub fn push_char(&mut self, c: char) {
        if (c as u32) < 128 {
            self.0.push(c as u8);
        }
        else {
            #[cold]
            fn cold<A: Alloc>(this: &mut String<A>, c: char) {
                this.0.extend_from_slice(
                    c.encode_utf8(&mut [0; 4]).as_bytes());
            }
            cold(self, c);
        }
    }

    #[inline(always)]
    pub fn push_fmt(&mut self, args: core::fmt::Arguments) {
        _ = core::fmt::Write::write_fmt(self, args);
    }


    #[inline(always)]
    pub fn clear(&mut self) {
        self.0.clear();
    }


    #[inline(always)]
    pub fn take(&mut self) -> Self where A: Clone {
        Self(self.0.take())
    }

    #[inline]
    pub fn leak<'a>(self) -> &'a str where A: 'a {
        let bytes = self.0.leak();
        unsafe { crate::str::from_utf8_unchecked(bytes) }
    }
}

impl<A: Alloc> crate::fmt::Debug for String<A> {
    fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<A: Alloc> crate::fmt::Display for String<A> {
    fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
        self.as_str().fmt(f)
    }
}


impl<A: Alloc> crate::ops::Deref for String<A> {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<A: Alloc> crate::borrow::Borrow<str> for String<A> {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}


impl<A: Alloc + Default> Default for String<A> {
    #[inline]
    fn default() -> Self {
        Self::new_in(A::default())
    }
}


impl From<&str> for String<GlobalAlloc> {
    #[inline(always)]
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}


impl<A: Alloc> crate::cmp::PartialEq for String<A> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<A: Alloc> crate::cmp::Eq for String<A> {}


impl<A: Alloc> crate::cmp::PartialEq<&str> for String<A> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}


impl<A: Alloc> crate::cmp::PartialOrd for String<A> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<crate::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: Alloc> crate::cmp::Ord for String<A> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> crate::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}


impl<A: Alloc> crate::hash::Hash for String<A> {
    #[inline]
    fn hash<H: crate::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}


impl<A: Alloc> crate::fmt::Write for String<A> {
    fn write_str(&mut self, s: &str) -> crate::fmt::Result {
        self.push(s);
        Ok(())
    }
}

