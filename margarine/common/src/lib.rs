pub mod string_map;
pub mod fuck_map;
pub mod hashables;
pub mod source;

use std::{ops::{Deref, DerefMut}, time::Instant};

pub trait Slice: Deref {
    fn as_slice(&self) -> &<Self as Deref>::Target;
    fn as_mut(&mut self) -> &mut <Self as Deref>::Target;
}


impl<A, T: Deref<Target = [A]> + DerefMut> Slice for T {
    fn as_slice(&self) -> &<Self as Deref>::Target {
        self.deref()
    }

    fn as_mut(&mut self) -> &mut <Self as Deref>::Target {
        self.deref_mut()
    }
}



pub struct DropTimer<'a> {
    message: &'a str,
    time: Instant,
}


impl<'a> DropTimer<'a> {
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            time: Instant::now(),
        }
    }


    #[inline(always)]
    pub fn with_timer<T, F: FnOnce() -> T>(message: &'a str, block: F) -> T {
        let _drop = DropTimer::new(message);
        block()
    }
}


impl Drop for DropTimer<'_> {
    fn drop(&mut self) {
        println!("droptimer: ran '{}' in {} seconds", self.message, self.time.elapsed().as_secs_f32());
    }
}
