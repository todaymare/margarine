use crate::hash::{Hash, Hasher, HashFn};
use crate::ops::BitXor;


const ROL: u32 = 5;
// golden ratio in fixed point.
const MUL32: u32 = 0x9e3779b9;
const MUL64: u64 = 0x9e3779b97f4a7c15;
// pi in fixed point.
const INI32: u32 = 0x517cc1b7;
const INI64: u64 = 0x517cc1b727220a95;

#[inline(always)]
fn add_to_hash_u32(hash: &mut u32, value: u32) {
    *hash = hash.rotate_left(ROL).bitxor(value).wrapping_mul(MUL32);
}

#[inline(always)]
fn add_to_hash_u64(hash: &mut u64, value: u64) {
    *hash = hash.rotate_left(ROL).bitxor(value).wrapping_mul(MUL64);
}


#[inline]
pub fn fxhash32<T: Hash + ?Sized>(value: &T) -> u32 {
    let mut hasher = FxHasher32::new();
    value.hash(&mut hasher);
    hasher.hash
}

#[inline]
pub fn fxhash64<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = FxHasher64::new();
    value.hash(&mut hasher);
    hasher.hash
}


pub struct FxHasher32 {
    pub hash: u32,
}

impl FxHasher32 {
    pub const DEFAULT_SEED: u32 = INI32;


    #[inline(always)]
    pub fn new() -> Self { Self { hash: INI32 } }

    #[inline(always)]
    pub fn from_seed(seed: u32) -> Self { Self { hash: seed } }


    #[inline(always)]
    pub fn finish_u32(&self) -> u32 {
        self.hash
    }

    #[inline]
    pub fn finish_u64(&self) -> u64 {
        let mut hasher = FxHasher64::new();
        hasher.write_u32(self.hash);
        hasher.hash
    }


    pub fn write_bytes(&mut self, mut bytes: &[u8]) {
        let mut hash = self.hash;

        while bytes.len() >= 4 {
            let value = unsafe {
                (bytes.as_ptr() as *const u32).read_unaligned()
            };
            add_to_hash_u32(&mut hash, value);
            bytes = &bytes[4..];
        }

        if bytes.len() > 0 {
            let rest =
                if bytes.len() == 3 {
                      ((bytes[0] as u32) <<  0)
                    | ((bytes[1] as u32) <<  8)
                    | ((bytes[2] as u32) << 16)
                }
                else if bytes.len() == 2 {
                      ((bytes[0] as u32) << 0)
                    | ((bytes[1] as u32) << 8)
                }
                else {
                    bytes[0] as u32
                };
            add_to_hash_u32(&mut hash, rest);
        }

        self.hash = hash;
    }
}

impl Hasher for FxHasher32 {
    #[inline(always)]
    fn finish(&self) -> u64 {
        // it's kinda dumb that we have to return a `u64`.
        // as the user expects properly distributed "entropy",
        // we expand the hash to 64 bits.
        // the sti hash map doesn't use this trait anyway,
        // so we don't care about the perf hit.
        self.finish_u64()
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.write_bytes(bytes);
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        add_to_hash_u32(&mut self.hash, i as u32);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        add_to_hash_u32(&mut self.hash, i as u32);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        add_to_hash_u32(&mut self.hash, i);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        let mut hash = self.hash;
        add_to_hash_u32(&mut hash, (i >>  0) as u32);
        add_to_hash_u32(&mut hash, (i >> 32) as u32);
        self.hash = hash;
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        let mut hash = self.hash;
        add_to_hash_u32(&mut hash, (i >>  0) as u32);
        add_to_hash_u32(&mut hash, (i >> 32) as u32);
        add_to_hash_u32(&mut hash, (i >> 64) as u32);
        add_to_hash_u32(&mut hash, (i >> 96) as u32);
        self.hash = hash;
    }

    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.write_u32(i as u32)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.write_u64(i as u64)
    }
}


pub struct FxHasher64 {
    pub hash: u64,
}

impl FxHasher64 {
    pub const DEFAULT_SEED: u64 = INI64;


    #[inline(always)]
    pub fn new() -> Self { Self { hash: INI64 } }

    #[inline(always)]
    pub fn from_seed(seed: u64) -> Self { Self { hash: seed } }


    pub fn write_bytes(&mut self, mut bytes: &[u8]) {
        let mut hash = self.hash;

        while bytes.len() >= 8 {
            let value = unsafe {
                (bytes.as_ptr() as *const u64).read_unaligned()
            };
            add_to_hash_u64(&mut hash, value);
            bytes = &bytes[8..];
        }

        while bytes.len() >= 4 {
            let value = unsafe {
                (bytes.as_ptr() as *const u32).read_unaligned()
            };
            add_to_hash_u64(&mut hash, value as u64);
            bytes = &bytes[4..];
        }

        if bytes.len() > 0 {
            let rest =
                if bytes.len() == 3 {
                      ((bytes[0] as u64) <<  0)
                    | ((bytes[1] as u64) <<  8)
                    | ((bytes[2] as u64) << 16)
                }
                else if bytes.len() == 2 {
                      ((bytes[0] as u64) << 0)
                    | ((bytes[1] as u64) << 8)
                }
                else {
                    bytes[0] as u64
                };
            add_to_hash_u64(&mut hash, rest);
        }

        self.hash = hash;
    }
}

impl Hasher for FxHasher64 {
    #[inline(always)]
    fn finish(&self) -> u64 { self.hash }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.write_bytes(bytes);
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        add_to_hash_u64(&mut self.hash, i as u64);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        add_to_hash_u64(&mut self.hash, i as u64);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        add_to_hash_u64(&mut self.hash, i as u64);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        add_to_hash_u64(&mut self.hash, i);
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        let mut hash = self.hash;
        add_to_hash_u64(&mut hash, (i >>  0) as u64);
        add_to_hash_u64(&mut hash, (i >> 64) as u64);
        self.hash = hash;
    }

    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.write_u32(i as u32)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.write_u64(i as u64)
    }
}



#[derive(Clone, Copy, Debug, Default)]
pub struct FxHashFn;

impl<T: Hash + ?Sized> HashFn<T, u32> for FxHashFn {
    #[inline(always)]
    fn hash(&self, value: &T) -> u32 {
        fxhash32(value)
    }
}

impl<T: Hash + ?Sized> HashFn<T, u64> for FxHashFn {
    #[inline(always)]
    fn hash(&self, value: &T) -> u64 {
        fxhash64(value)
    }
}

