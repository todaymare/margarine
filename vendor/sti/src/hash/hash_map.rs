use crate::mem::{NonNull, PhantomData};
use crate::borrow::Borrow;
use crate::byte_mask::ByteMask8;
use crate::alloc::{Alloc, GlobalAlloc, Layout, cat_join, cat_next_mut};
use crate::hash::{Hash, HashFn, fxhash::FxHashFn};


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotIdx(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hash32(pub u32);


pub struct HashMap<K, V, A: Alloc = GlobalAlloc, H: HashFn<K, u32> = FxHashFn> {
    h: H,
    alloc: A,

    groups_num: u32,
    empty: u32,
    used: u32,
    groups_ptr: NonNull<Group>,

    phantom: PhantomData<(K, V)>,
}

impl<K: Hash, V> HashMap<K, V, GlobalAlloc, FxHashFn> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self::new_in(GlobalAlloc)
    }

    #[inline(always)]
    pub fn with_cap(cap: usize) -> Self {
        Self::with_cap_in(GlobalAlloc, cap)
    }
}

impl<K: Hash, V, A: Alloc> HashMap<K, V, A, FxHashFn> {
    #[inline(always)]
    pub const fn new_in(alloc: A) -> Self {
        Self::with_hash_fn_in(alloc, FxHashFn)
    }

    #[inline(always)]
    pub fn with_cap_in(alloc: A, cap: usize) -> Self {
        Self::with_hash_and_cap_in(alloc, FxHashFn, cap)
    }
}

impl<K, V, A: Alloc, H: HashFn<K, u32>> HashMap<K, V, A, H> {
    #[inline]
    pub const fn with_hash_fn_in(alloc: A, h: H) -> Self {
        Self {
            h,
            alloc,
            groups_num: 0,
            empty: 0,
            used: 0,
            groups_ptr: NonNull::dangling(),
            phantom: PhantomData,
        }
    }

    pub fn with_hash_and_cap_in(alloc: A, h: H, cap: usize) -> Self {
        let mut this = Self::with_hash_fn_in(alloc, h);
        let num_groups = num_groups_for_cap(cap).unwrap();
        this.resize(Some(num_groups));
        return this;
    }

    #[inline]
    pub fn alloc(&self) -> &A {
        &self.alloc
    }

    #[inline]
    pub fn hash_fn(&self) -> &H {
        &self.h
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.groups_num as usize * Group::WIDTH
    }

    #[inline]
    pub fn cap(&self) -> usize {
        self.groups_num as usize * EMPTY_PER_GROUP as usize
    }

    #[inline]
    pub fn resident(&self) -> usize {
        self.cap() - self.empty as usize
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.used as usize
    }


    pub fn reserve(&mut self, min_cap: usize) {
        let min_groups_num = num_groups_for_cap(min_cap).unwrap();
        if min_groups_num > self.groups_num {
            self.resize(Some(min_groups_num));
        }
    }

    #[inline]
    pub fn reserve_for_insert(&mut self) {
        if self.empty == 0 {
            self.resize(None);
        }
    }


    #[inline(always)]
    pub fn hash<Q>(&self, k: &Q) -> Hash32
    where Q: ?Sized, K: Borrow<Q>, H: HashFn<Q, u32> {
        Hash32(self.h.hash(k))
    }


    #[inline(always)]
    pub fn lookup<Q>(&self, q: &Q, hash: Hash32) -> (bool, SlotIdx)
    where Q: ?Sized + Eq, K: Borrow<Q> {
        return self.lookup_cmp(hash, move |k| q == k.borrow());
    }

    pub fn lookup_cmp(&self, hash: Hash32, cmp: impl Fn(&K) -> bool) -> (bool, SlotIdx) {
        if self.groups_num == 0 {
            return (false, SlotIdx(0));
        }

        let slots = Self::slots_ptr(self.groups_num, self.groups_ptr);

        let mut tomb = None;

        let mut group_idx = Group::first_idx(hash.0, self.groups_num);
        loop { unsafe {
            let group = &*self.groups_ptr.as_ptr().add(group_idx);
            let group_slots = slots.add(Group::WIDTH*group_idx);

            for i in group.match_hash(hash) {
                let slot = &*group_slots.add(i);
                if cmp(&slot.0) {
                    return (true, SlotIdx((Group::WIDTH*group_idx + i) as u32));
                }
            }

            if tomb.is_none() {
                if let Some(i) = group.match_tomb().next() {
                    tomb = Some(SlotIdx((Group::WIDTH*group_idx + i) as u32));
                }
            }

            if let Some(i) = group.match_fresh().next() {
                let idx = tomb.unwrap_or(SlotIdx((Group::WIDTH*group_idx + i) as u32));
                return (false, idx);
            }

            group_idx += 1;
            if group_idx == self.groups_num as usize {
                group_idx = 0;
            }
        }}
    }

    #[inline]
    pub fn lookup_for_insert<Q>(&mut self, k: &Q, hash: Hash32) -> (bool, SlotIdx)
    where Q: ?Sized + Eq, K: Borrow<Q> {
        self.reserve_for_insert();
        self.lookup(k, hash)
    }


    pub fn entry<Q>(&self, k: &Q) -> (bool, SlotIdx)
    where Q: ?Sized + Eq, K: Borrow<Q>, H: HashFn<Q, u32> {
        self.lookup(k, self.hash(k))
    }

    #[inline]
    pub fn entry_for_insert<Q>(&mut self, k: &Q) -> (bool, SlotIdx)
    where Q: ?Sized + Eq, K: Borrow<Q>, H: HashFn<Q, u32> {
        self.lookup_for_insert(k, self.hash(k))
    }


    #[inline]
    pub fn slot_present(&self, idx: SlotIdx) -> bool {
        return (idx.0 as usize) < self.size()
            && unsafe { (&*Self::group_ptr(self.groups_ptr, idx)).is_used(idx) };
    }

    #[inline]
    pub fn slot(&self, idx: SlotIdx) -> (&K, &V) {
        assert!(self.slot_present(idx));
        return unsafe { self.slot_unck(idx) };
    }

    #[inline]
    pub unsafe fn slot_unck(&self, idx: SlotIdx) -> (&K, &V) { unsafe {
        debug_assert!(self.slot_present(idx));
        let (k, v) = &*Self::slot_ptr(self.groups_num, self.groups_ptr, idx);
        return (k, v);
    }}

    #[inline]
    pub fn slot_mut(&mut self, idx: SlotIdx) -> (&K, &mut V) {
        assert!(self.slot_present(idx));
        return unsafe { self.slot_unck_mut(idx) };
    }

    #[inline]
    pub unsafe fn slot_unck_mut(&mut self, idx: SlotIdx) -> (&K, &mut V) { unsafe {
        debug_assert!(self.slot_present(idx));
        let (k, v) = &mut *Self::slot_ptr(self.groups_num, self.groups_ptr, idx);
        return (k, v);
    }}


    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where Q: ?Sized + Eq, K: Borrow<Q>, H: HashFn<Q, u32> {
        let (present, idx) = self.entry(k);
        if present {
            Some(unsafe { self.slot_unck(idx).1 })
        }
        else { None }
    }


    #[inline]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where Q: ?Sized + Eq, K: Borrow<Q>, H: HashFn<Q, u32> {
        let (present, idx) = self.entry(k);
        if present {
            Some(unsafe { self.slot_unck_mut(idx).1 })
        }
        else { None }
    }


    pub fn insert(&mut self, k: K, v: V) -> Option<(K, V)>
    where K: Eq {
        let hash = self.hash(&k);
        let (present, idx) = self.lookup_for_insert(&k, hash);
        unsafe {
            let result = self.insert_at_unck(idx, hash, k, v);
            crate::assume!(present == result.is_some());
            return result;
        }
    }

    pub fn insert_new(&mut self, k: K, v: V)
    where K: Eq {
        let hash = self.hash(&k);
        let (present, idx) = self.lookup_for_insert(&k, hash);
        assert!(!present);

        unsafe {
            let none = self.insert_at_unck(idx, hash, k, v);
            crate::assume!(none.is_none());
        }
    }

    #[inline]
    pub fn insert_at(&mut self, idx: SlotIdx, hash: Hash32, k: K, v: V) -> Option<(K, V)> {
        assert!((idx.0 as usize) < self.size() && self.empty > 0);
        return unsafe { self.insert_at_unck(idx, hash, k, v) };
    }

    #[inline]
    pub unsafe fn insert_at_unck(&mut self, idx: SlotIdx, hash: Hash32, k: K, v: V) -> Option<(K, V)> { unsafe {
        debug_assert!((idx.0 as usize) < self.size() && self.empty > 0);

        let slot = Self::slot_ptr(self.groups_num, self.groups_ptr, idx);
        let group = &mut *Self::group_ptr(self.groups_ptr, idx);

        let was_used = group.is_used(idx);
        let result =
            if was_used { Some(slot.read()) }
            else { None };

        self.empty -= group.is_fresh(idx) as u32;

        group.use_entry(idx, hash);

        crate::asan::unpoison_ptr(slot);
        slot.write((k, v));

        self.used += !was_used as u32;

        return result;
    }}


    pub fn remove<Q>(&mut self, k: &Q) -> Option<(K, V)>
    where Q: ?Sized + Eq,
          H: HashFn<Q, u32>,
          K: Borrow<Q>
    {
        let (present, idx) = self.lookup(k, self.hash(k));
        if present { Some(unsafe { self.remove_at_unck(idx) }) }
        else { None }
    }

    #[inline]
    pub fn remove_at(&mut self, idx: SlotIdx) -> (K, V) {
        assert!(self.slot_present(idx));
        return unsafe { self.remove_at_unck(idx) };
    }

    #[inline]
    pub unsafe fn remove_at_unck(&mut self, idx: SlotIdx) -> (K, V) { unsafe {
        debug_assert!((idx.0 as usize) < self.size());

        let slot = Self::slot_ptr(self.groups_num, self.groups_ptr, idx);
        let group = &mut *Self::group_ptr(self.groups_ptr, idx);
        debug_assert!(group.is_used(idx));

        let result = slot.read();
        crate::asan::poison_ptr(slot);

        self.empty += group.free_entry(idx);
        self.used -= 1;

        return result;
    }}


    pub fn clear(&mut self) { unsafe {
        if self.used == 0 { return }

        let slots = Self::slots_ptr(self.groups_num, self.groups_ptr);
        for group_idx in 0..self.groups_num as usize {
            let group = &mut *self.groups_ptr.as_ptr().add(group_idx);
            let group_slots = slots.add(Group::WIDTH*group_idx);

            for i in group.match_used() {
                crate::mem::drop_in_place(group_slots.add(i));
            }

            *group = Group::fresh();
        }
        crate::asan::poison_ptr_len(slots, Group::WIDTH.wrapping_mul(self.groups_num as usize));

        self.empty = EMPTY_PER_GROUP * self.groups_num;
        self.used = 0;
    }}


    pub fn clone_in<A2>(&self, alloc: A2) -> HashMap<K, V, A2, H>
    where K: Clone, V: Clone, H: Clone, A2: Alloc
    { unsafe {
        // allocate uninitialized hash map with same capacity.
        let mut result = {
            let layout = Self::layout(self.groups_num).unwrap_unchecked();

            let data = alloc.alloc(layout).unwrap();

            HashMap {
                h: self.h.clone(),
                alloc,
                groups_num: self.groups_num,
                empty: self.empty,
                used: 0,
                groups_ptr: data.cast(),
                phantom: PhantomData,
            }
        };

        // clone slots.
        let src_slots = Self::slots_ptr(self.groups_num, self.groups_ptr);
        let dst_slots = Self::slots_ptr(result.groups_num, result.groups_ptr);
        for group_idx in 0..self.groups_num as usize {
            let group = &*self.groups_ptr.as_ptr().add(group_idx);
            let src_group_slots = src_slots.add(Group::WIDTH*group_idx);
            let dst_group_slots = dst_slots.add(Group::WIDTH*group_idx);

            for i in group.match_used() {
                let src = &*src_group_slots.add(i);
                let dst = dst_group_slots.add(i);
                dst.write((src.0.clone(), src.1.clone()));
            }
        }

        // group metadata.
        core::ptr::copy_nonoverlapping(
            self.groups_ptr.as_ptr(),
            result.groups_ptr.as_ptr(),
            self.groups_num as usize);

        result.used = self.used;

        result
     }}


    #[inline]
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            groups: self.groups_ptr.as_ptr(),
            slots: Self::slots_ptr(self.groups_num, self.groups_ptr),
            rem: self.used,
            slots_num: self.groups_num * Group::WIDTH as u32,
            slot_idx: SlotIdx(0),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            groups: self.groups_ptr.as_ptr(),
            slots: Self::slots_ptr(self.groups_num, self.groups_ptr),
            rem: self.used,
            slots_num: self.groups_num * Group::WIDTH as u32,
            slot_idx: SlotIdx(0),
            phantom: PhantomData,
        }
    }


    fn resize(&mut self, new_groups_num: Option<u32>) {
        let new_groups_num = new_groups_num.unwrap_or_else(||
            self.groups_num.checked_mul(2).unwrap().max(1));

        let layout = Self::layout(new_groups_num).unwrap();
        let data = self.alloc.alloc(layout).unwrap();

        let new_groups_ptr: NonNull<Group> = data.cast();
        for i in 0..new_groups_num as usize {
            unsafe { new_groups_ptr.as_ptr().add(i).write(Group::fresh()) }
        }

        let new_slots = Self::slots_ptr(new_groups_num, new_groups_ptr);
        crate::asan::poison_ptr_len(new_slots, Group::WIDTH.wrapping_mul(new_groups_num as usize));

        let old_groups_num = self.groups_num;
        let old_used = self.used;
        let old_groups_ptr = self.groups_ptr;
        let old_slots = Self::slots_ptr(old_groups_num, old_groups_ptr);

        self.groups_num = new_groups_num;
        self.empty = EMPTY_PER_GROUP * new_groups_num;
        self.used  = 0;
        self.groups_ptr = new_groups_ptr;

        if old_groups_num == 0 {
            assert_eq!(old_used, 0);
            return;
        }

        for group_idx in 0..old_groups_num as usize { unsafe {
            let group = &*old_groups_ptr.as_ptr().add(group_idx);
            let group_slots = old_slots.add(Group::WIDTH*group_idx);

            for i in group.match_used() {
                assert!(self.empty > 0);

                let slot = group_slots.add(i).read();

                let hash = self.hash(&slot.0);

                let idx = {
                    let mut group_idx = Group::first_idx(hash.0, self.groups_num);
                    loop {
                        let group = &*self.groups_ptr.as_ptr().add(group_idx);

                        if let Some(i) = group.match_fresh().next() {
                            break SlotIdx((Group::WIDTH*group_idx + i) as u32);
                        }

                        group_idx += 1;
                        if group_idx == self.groups_num as usize {
                            group_idx = 0;
                        }
                    }
                };

                let slot_ptr = new_slots.add(idx.0 as usize);
                crate::asan::unpoison_ptr(slot_ptr);
                slot_ptr.write(slot);

                (&mut *Self::group_ptr(new_groups_ptr, idx)).use_entry(idx, hash);

                self.empty -= 1;
                self.used  += 1;
            }
        }}

        unsafe {
            let layout = Self::layout(old_groups_num).unwrap_unchecked();
            self.alloc.free(old_groups_ptr.cast(), layout);
        }
    }


    #[inline]
    fn layout(num_groups: u32) -> Option<Layout> {
        let num_groups: usize = num_groups.try_into().expect("unreachable");
        let num_slots = num_groups.checked_mul(Group::WIDTH)?;
        cat_join(
            Layout::array::<Group>(num_groups).ok()?,
            Layout::array::<(K, V)>(num_slots).ok()?)
    }

    #[inline]
    fn slots_ptr(groups_num: u32, groups_ptr: NonNull<Group>) -> *mut (K, V) {
        unsafe { cat_next_mut(groups_ptr.as_ptr(), groups_num as usize) }
    }

    #[inline]
    fn slot_ptr(groups_num: u32, groups_ptr: NonNull<Group>, idx: SlotIdx) -> *mut (K, V) {
        unsafe { Self::slots_ptr(groups_num, groups_ptr).add(idx.0 as usize) }
    }

    #[inline]
    fn group_ptr(groups_ptr: NonNull<Group>, idx: SlotIdx) -> *mut Group {
        unsafe { groups_ptr.as_ptr().add(idx.0 as usize / Group::WIDTH) }
    }
}


impl<K: Clone, V: Clone, A: Alloc + Clone, H: HashFn<K, u32> + Clone> Clone for HashMap<K, V, A, H> {
    fn clone(&self) -> Self {
        return self.clone_in(self.alloc.clone());
    }
}


impl<K, V, A: Alloc, H: HashFn<K, u32>> Drop for HashMap<K, V, A, H> {
    fn drop(&mut self) { unsafe {
        if self.groups_num == 0 { return }

        // nb: we don't use clear because that writes to the groups.
        // we assume most users have non-drop entries, so this entire loop
        // should get DCE'd.
        if self.used != 0 {
            let slots = Self::slots_ptr(self.groups_num, self.groups_ptr);
            for group_idx in 0..self.groups_num as usize {
                let group = &*self.groups_ptr.as_ptr().add(group_idx);
                let group_slots = slots.add(Group::WIDTH*group_idx);

                for i in group.match_used() {
                    crate::mem::drop_in_place(group_slots.add(i));
                }
            }
        }

        let layout = Self::layout(self.groups_num).unwrap_unchecked();
        self.alloc.free(self.groups_ptr.cast(), layout);
    }}
}


impl<K, V, A: Alloc + Default, H: HashFn<K, u32> + Default> Default for HashMap<K, V, A, H> {
    #[inline]
    fn default() -> Self {
        Self::with_hash_fn_in(A::default(), H::default())
    }
}


impl<K: crate::fmt::Debug, V: crate::fmt::Debug, A: Alloc, H: HashFn<K, u32>> crate::fmt::Debug for HashMap<K, V, A, H> {
    fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}


impl<Q, K, V, A: Alloc + Default, H: HashFn<K, u32> + Default> crate::ops::Index<&Q> for HashMap<K, V, A, H>
where Q: ?Sized + Eq, K: Borrow<Q>, H: HashFn<Q, u32> {
    type Output = V;

    #[inline]
    fn index(&self, k: &Q) -> &Self::Output {
        self.get(k).unwrap()
    }
}

impl<Q, K, V, A: Alloc + Default, H: HashFn<K, u32> + Default> crate::ops::IndexMut<&Q> for HashMap<K, V, A, H>
where Q: ?Sized + Eq, K: Borrow<Q>, H: HashFn<Q, u32> {
    #[inline]
    fn index_mut(&mut self, k: &Q) -> &mut Self::Output {
        self.get_mut(k).unwrap()
    }
}


pub struct Iter<'a, K, V> {
    groups: *const Group,
    slots: *const (K, V),
    rem: u32,
    slots_num: u32,
    slot_idx: SlotIdx,
    phantom: PhantomData<&'a (K, V)>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> { unsafe {
        while self.slot_idx.0 < self.slots_num {
            let group = &*self.groups.add(self.slot_idx.0 as usize / Group::WIDTH);

            if group.is_used(self.slot_idx) {
                self.rem -= 1;

                let idx = self.slot_idx.0 as usize;
                self.slot_idx.0 += 1;

                let (k, v) = &*self.slots.add(idx);
                return Some((k, v));
            }

            if !group.match_used().any() {
                self.slot_idx.0 = (self.slot_idx.0 & !(Group::WIDTH as u32 - 1)) + Group::WIDTH as u32;
            }
            else {
                self.slot_idx.0 += 1;
            }
        }
        None
    }}

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.rem as usize, Some(self.rem as usize))
    }
}


pub struct IterMut<'a, K, V> {
    groups: *const Group,
    slots: *mut (K, V),
    rem: u32,
    slots_num: u32,
    slot_idx: SlotIdx,
    phantom: PhantomData<(&'a K, &'a mut V)>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> { unsafe {
        while self.slot_idx.0 < self.slots_num {
            let group = &*self.groups.add(self.slot_idx.0 as usize / Group::WIDTH);

            if group.is_used(self.slot_idx) {
                self.rem -= 1;

                let idx = self.slot_idx.0 as usize;
                self.slot_idx.0 += 1;

                let (k, v) = &mut *self.slots.add(idx);
                return Some((k, v));
            }

            if !group.match_used().any() {
                self.slot_idx.0 = (self.slot_idx.0 & !(Group::WIDTH as u32 - 1)) + Group::WIDTH as u32;
            }
            else {
                self.slot_idx.0 += 1;
            }
        }
        None
    }}

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.rem as usize, Some(self.rem as usize))
    }
}


// max load: 14/16 = 7/8.

const EMPTY_PER_GROUP: u32 = Group::WIDTH as u32 * 7/8;
crate::static_assert!(Group::WIDTH % 8 == 0);

#[inline]
const fn num_groups_for_cap(cap: usize) -> Option<u32> {
    const W32: u32 = Group::WIDTH as u32;
    const W: usize = Group::WIDTH;

    // max cap we can support:
    //  cap*8/7 + w-1 <= u32::MAX
    //  cap <= (u32::MAX - w-1)*7/8
    const MAX_CAP: usize = (((u32::MAX - (W32-1)) as u64)*7/8) as usize;

    const _MAX_GROUPS: usize = (MAX_CAP + MAX_CAP/7 + (W-1)) / W;
    crate::static_assert!(MAX_CAP <= _MAX_GROUPS*W);

    if cap <= MAX_CAP {
        let groups = (cap + cap/7 + (W-1)) / W;
        Some(groups as u32)
    }
    else { None }
}


struct Group(u64);

impl Group {
    const WIDTH: usize = crate::mem::size_of::<Group>();


    #[inline]
    fn first_idx(hash: u32, groups_num: u32) -> usize {
        ((hash as u64 * groups_num as u64) >> 32) as usize
    }


    const FRESH:     u8  = 0xff;
    const TOMBSTONE: u8  = 0x80;
    const HASH_MASK: u32 = 0x7f;


    #[inline]
    const fn fresh() -> Group {
        Self(crate::byte_mask::splat_8(Self::FRESH))
    }

    #[inline]
    const fn mask_hash(hash: Hash32) -> u8 {
        (hash.0 & Self::HASH_MASK) as u8
    }


    #[inline]
    fn match_hash(&self, hash: Hash32) -> ByteMask8 {
        ByteMask8::find_equal_bytes(self.0, Self::mask_hash(hash))
    }

    #[inline]
    fn match_tomb(&self) -> ByteMask8 {
        // high bit -> not used.
        // second highest bit -> fresh.
        ByteMask8::find_high_bit_bytes(self.0 & !(self.0 << 1))
    }

    #[inline]
    fn match_fresh(&self) -> ByteMask8 {
        // high bit -> not used.
        // second highest bit -> fresh.
        ByteMask8::find_high_bit_bytes(self.0 & (self.0 << 1))
    }

    #[inline]
    fn match_used(&self) -> ByteMask8 {
        // used entries don't have the high bit.
        ByteMask8::find_high_bit_bytes(self.0).not()
    }


    #[inline]
    fn get(&self, idx: SlotIdx) -> u8 { unsafe {
        let bytes = (&self.0 as *const u64).cast::<u8>();
        return bytes.add(idx.0 as usize % Group::WIDTH).read();
    }}

    #[inline]
    fn set(&mut self, idx: SlotIdx, value: u8) { unsafe {
        let bytes = (&mut self.0 as *mut u64).cast::<u8>();
        bytes.add(idx.0 as usize % Group::WIDTH).write(value);
    }}

    #[inline]
    fn is_used(&self, idx: SlotIdx) -> bool {
        self.get(idx) & 0x80 == 0
    }

    #[inline]
    fn is_fresh(&self, idx: SlotIdx) -> bool {
        self.get(idx) == 0xff
    }

    #[inline]
    fn use_entry(&mut self, idx: SlotIdx, hash: Hash32) {
        self.set(idx, Self::mask_hash(hash))
    }

    #[inline]
    fn free_entry(&mut self, idx: SlotIdx) -> u32 {
        if self.match_fresh().any() {
            self.set(idx, Self::FRESH);
            return 1;
        }
        else {
            self.set(idx, Self::TOMBSTONE);
            return 0;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::string::String;

    /*
    struct DumbHash;
    impl HashFn<u32, u32> for DumbHash {
        fn hash(&self, value: &u32) -> u32 { *value % 32 / 2 }
    }
    */

    struct IdHash;
    impl HashFn<u32, u32> for IdHash {
        fn hash(&self, value: &u32) -> u32 { *value }
    }

    struct ConstHash;
    impl<T> HashFn<T, u32> for ConstHash {
        fn hash(&self, _value: &T) -> u32 { 0 }
    }
    impl Default for ConstHash { fn default() -> Self { ConstHash } }


    #[test]
    fn hm_basic() {
        let mut hm: HashMap<String, u32> = HashMap::with_cap(69);

        assert!(hm.get("hi").is_none());
        assert!(hm.remove("ho").is_none());

        let size = (69*8/7 + Group::WIDTH) / Group::WIDTH * Group::WIDTH;
        let cap = size*7/8;
        assert_eq!((hm.size(), hm.cap(), hm.resident(), hm.len()), (size, cap, 0, 0));

        assert!(hm.insert("hi".into(), 42).is_none());
        assert_eq!(hm["hi"], 42);
        assert_eq!((hm.size(), hm.cap(), hm.resident(), hm.len()), (size, cap, 1, 1));

        hm.insert_new("ho".into(), 69);
        assert_eq!(hm["hi"], 42);
        assert_eq!(hm["ho"], 69);
        assert_eq!((hm.size(), hm.cap(), hm.resident(), hm.len()), (size, cap, 2, 2));

        hm["hi"] = 17;
        assert_eq!(hm["hi"], 17);
        assert_eq!(hm["ho"], 69);
        assert_eq!((hm.size(), hm.cap(), hm.resident(), hm.len()), (size, cap, 2, 2));

        let (old_k, old_v) = hm.insert("ho".into(), 19).unwrap();
        assert_eq!(old_k, "ho");
        assert_eq!(old_v, 69);
        assert_eq!(hm["hi"], 17);
        assert_eq!(hm["ho"], 19);
        assert_eq!((hm.size(), hm.cap(), hm.resident(), hm.len()), (size, cap, 2, 2));

        let (hi_k, hi_v) = hm.remove("hi").unwrap();
        assert_eq!(hi_k, "hi");
        assert_eq!(hi_v, 17);
        assert!(hm.get("hi").is_none());
        assert_eq!(hm["ho"], 19);
        assert_eq!((hm.size(), hm.cap(), hm.resident(), hm.len()), (size, cap, 1, 1));
    }

    #[test]
    fn hm_growing() {
        let mut hm: HashMap<String, u32> = HashMap::new();

        hm.insert("a".into(), 0);

        let size_0 = hm.size();
        let mut i = 1;
        while hm.len() < hm.cap() {
            hm.insert(crate::format!("{i}"), i);
            i += 1;
        }
        assert_eq!(hm.size(), size_0);
        assert_eq!(hm.len(), hm.resident());

        let len_1 = hm.len();

        hm.insert("b".into(), 1);
        assert_eq!(hm.size(), size_0*2);
        assert_eq!(hm.len(), hm.resident());
        assert_eq!(hm.len(), len_1 + 1);

        assert_eq!(hm["a"], 0);
        assert_eq!(hm["b"], 1);
        for i in 1..i {
            assert_eq!(hm[&crate::format!("{i}")], i);
            assert_eq!(hm[&*crate::format!("{i}")], i);
        }
    }

    #[test]
    fn hm_tombstone() {
        let mut hm: HashMap<u32, u8, GlobalAlloc, ConstHash> = HashMap::default();
        hm.reserve(Group::WIDTH);
        assert_eq!(hm.size(), 2*Group::WIDTH);
        assert_eq!(hm.len(), hm.resident());
        assert_eq!(hm.len(), 0);

        for i in 0..Group::WIDTH-1 {
            hm.insert_new(i as u32, i as u8 + 1);
        }
        assert_eq!(hm.size(), 2*Group::WIDTH);
        assert_eq!(hm.len(), hm.resident());
        assert_eq!(hm.len(), Group::WIDTH-1);


        // can remove without needing tombstone.
        for i in 0..Group::WIDTH-1 {
            assert_eq!(hm.remove(&(i as u32)).unwrap(), (i as u32, i as u8 + 1));
            assert_eq!(hm.size(), 2*Group::WIDTH);
            assert_eq!(hm.len(), hm.resident());
            assert_eq!(hm.len(), Group::WIDTH-1 - 1);

            hm.insert_new(i as u32, i as u8 + 1);
            assert_eq!(hm.size(), 2*Group::WIDTH);
            assert_eq!(hm.len(), hm.resident());
            assert_eq!(hm.len(), Group::WIDTH-1);
        }


        hm.insert_new(42, 69);
        assert_eq!(hm.size(), 2*Group::WIDTH);
        assert_eq!(hm.len(), hm.resident());
        assert_eq!(hm.len(), Group::WIDTH);

        // now we need a tombstone.
        assert_eq!(hm.remove(&42).unwrap(), (42, 69));
        assert_eq!(hm.size(), 2*Group::WIDTH);
        assert_eq!(hm.len(), hm.resident() - 1);
        assert_eq!(hm.len(), Group::WIDTH-1);
        assert_eq!(hm.entry(&42), (false, SlotIdx(Group::WIDTH as u32 - 1)));


        // remove another one.
        assert_eq!(hm.remove(&0).unwrap(), (0, 1));
        assert_eq!(hm.entry(&0), (false, SlotIdx(0)));
        assert_eq!(hm.size(), 2*Group::WIDTH);
        assert_eq!(hm.len(), hm.resident() - 2);
        assert_eq!(hm.len(), Group::WIDTH-2);

        hm.insert_new(0, 1);
        assert_eq!(hm.entry(&0), (true, SlotIdx(0)));


        // but we can reuse it.
        hm.insert_new(42, 69);
        assert_eq!(hm.size(), 2*Group::WIDTH);
        assert_eq!(hm.len(), hm.resident());
        assert_eq!(hm.len(), Group::WIDTH);
        assert_eq!(hm.entry(&42), (true, SlotIdx(Group::WIDTH as u32 - 1)));
    }

    /*
    #[test]
    fn hm_probe_length() {
        let mut hm: HashMap<u32, u32, _, DumbHash> = HashMap::new();

        assert_eq!(hm.probe_length(&0),  (0, 0));
        assert_eq!(hm.probe_length(&69), (0, 0));

        hm.insert(0, 0);
        assert_eq!(hm.probe_length(&0), (1, 1));
        assert_eq!(hm.probe_length(&1), (1, 1));
        assert_eq!(hm.probe_length(&2), (1, 0));
        assert_eq!(hm.probe_length(&3), (1, 0));

        hm.insert(1, 1);
        assert_eq!(hm.probe_length(&32), (1, 2));

        for i in 2..Group::WIDTH as u32 {
            hm.insert(2*i, 2*i);
        }
        assert_eq!(hm.size(), 2*Group::WIDTH);
        assert_eq!(hm.probe_length(&32), (2, 2));


        assert_eq!(hm.resident(), hm.len());
        hm.remove(&1).unwrap();
        assert_eq!(hm.resident(), hm.len() + 1);
        assert_eq!(hm.probe_length(&32), (2, 1));

        hm.insert(32, 32);
        assert_eq!(hm.probe_length(&32), (1, 2));
    }
    */

    #[test]
    fn hm_iter() {
        let mut hm: HashMap<u32, i8> = HashMap::with_cap(69);
        assert!(hm.iter().next().is_none());

        hm.insert_new(42, 69);
        let mut iter = hm.iter();
        assert_eq!(iter.next().unwrap(), (&42, &69));
        assert!(iter.next().is_none());


        let mut hm: HashMap<u32, i8, _, IdHash> = HashMap::with_hash_fn_in(GlobalAlloc, IdHash);
        assert!(hm.iter().next().is_none());

        let n = 3*Group::WIDTH as u32 + 3;

        for i in 0..n {
            hm.insert(i, i as i8 + 1);
        }

        let mut iter = hm.iter();
        for i in 0..n {
            let (k, v) = iter.next().unwrap();
            assert_eq!(*k, i);
            assert_eq!(*v, i as i8 + 1);
        }
        assert!(iter.next().is_none());


        for (k, v) in hm.iter_mut() {
            *v += *k as i8;
        }

        let mut iter = hm.iter();
        for i in 0..n {
            let (k, v) = iter.next().unwrap();
            assert_eq!(*k, i);
            assert_eq!(*v, i as i8 + 1 + i as i8);
        }
        assert!(iter.next().is_none());
    }

    /*
    #[test]
    fn hm_clone() {
        let mut hm1: HashMap<String, Vec<i8>, _, ConstHash> = HashMap::fnew();

        assert!(hm1.iter().next().is_none());

        for i in 0..2*Group::WIDTH as u32 {
            let mut v = Vec::new();
            for k in 0..i { v.push(k as i8) }

            hm1.insert(format!("{i}"), v);
        }

        let hm2 = hm1.clone();
        assert_eq!(hm1.len(), hm2.len());
        assert_eq!(hm1.cap(), hm2.cap());

        let iter1 = hm1.iter();
        let iter2 = hm2.iter();
        let mut iter = iter1.zip(iter2);
        for i in 0..2*Group::WIDTH as u32 {
            let mut v = Vec::new();
            for k in 0..i { v.push(k as i8) }

            let ((k1, v1), (k2, v2)) = iter.next().unwrap();
            assert_eq!(*k1, format!("{i}"));
            assert_eq!(*v1, v);
            assert_eq!(*k1, *k2);
            assert_eq!(*v1, *v2);
        }
        assert!(iter.next().is_none());
    }

    #[test]
    fn hm_retain() {
        let mut hm: HashMapF<String, Vec<i8>, ConstHash> = HashMap::fnew();
        hm.insert("a".to_string(), vec![1]);
        hm.insert("b".to_string(), vec![1, 2]);
        hm.insert("d".to_string(), vec![1, 2, 3, 4]);
        hm.insert("c".to_string(), vec![1, 2, 3]);
        assert_eq!(hm.len(), 4);
        assert_eq!(hm.resident(), 4);

        let mut iter = hm.iter();
        assert_eq!(iter.next(), Some((&"a".to_string(), &vec![1])));
        assert_eq!(iter.next(), Some((&"b".to_string(), &vec![1, 2])));
        assert_eq!(iter.next(), Some((&"d".to_string(), &vec![1, 2, 3, 4])));
        assert_eq!(iter.next(), Some((&"c".to_string(), &vec![1, 2, 3])));
        assert_eq!(iter.next(), None);

        hm.retain(|_, v| v.len() % 2 == 0);
        assert_eq!(hm.len(), 2);
        assert_eq!(hm.resident(), 2);

        let mut iter = hm.iter();
        assert_eq!(iter.next(), Some((&"b".to_string(), &vec![1, 2])));
        assert_eq!(iter.next(), Some((&"d".to_string(), &vec![1, 2, 3, 4])));
        assert_eq!(iter.next(), None);
    }
    */

    /*
    #[test]
    fn hm_copy() {
        let mut hm1: HashMap<u32, [u32; 4], _, ConstHash> = HashMap::new();

        assert!(hm1.iter().next().is_none());

        for i in 0..2*Group::WIDTH as u32 {
            hm1.insert(i, [i, i+1, i+2, i+3]);
        }

        let hm2 = hm1.copy();
        assert_eq!(hm1.len(), hm2.len());
        assert_eq!(hm1.cap(), hm2.cap());

        let iter1 = hm1.iter();
        let iter2 = hm2.iter();
        let mut iter = iter1.zip(iter2);
        for i in 0..2*Group::WIDTH as u32 {
            let ((k1, v1), (k2, v2)) = iter.next().unwrap();
            assert_eq!(*k1, i);
            assert_eq!(*v1, [i, i+1, i+2, i+3]);
            assert_eq!(*k1, *k2);
            assert_eq!(*v1, *v2);
        }
        assert!(iter.next().is_none());
    }
    */

    /*
    #[test]
    fn hm_get_or_insert() {
        use crate::mem::Cell;
        use core::hash::{Hash, Hasher};


        struct MagicString {
            str: MagicStr<'static>,
        }

        impl PartialEq for MagicString {
            fn eq(&self, other: &Self) -> bool { self.str.value == other.str.value }
        }
        impl Eq for MagicString {}

        impl Hash for MagicString {
            fn hash<H: Hasher>(&self, state: &mut H) { self.str.value.hash(state) }
        }

        impl<'a> Borrow<MagicStr<'a>> for MagicString {
            fn borrow(&self) -> &MagicStr<'a> { &self.str }
        }


        struct MagicStr<'a> {
            value: &'static str,
            counter: Option<&'a Cell<u32>>,
        }

        impl<'a> PartialEq for MagicStr<'a> {
            fn eq(&self, other: &Self) -> bool { self.value == other.value }
        }
        impl<'a> Eq for MagicStr<'a> {}

        impl<'a> Hash for MagicStr<'a> {
            fn hash<H: Hasher>(&self, state: &mut H) { self.value.hash(state) }
        }

        impl<'a> From<&MagicStr<'a>> for MagicString {
            fn from(value: &MagicStr<'a>) -> Self {
                let counter = value.counter.as_ref().unwrap();
                counter.set(counter.get() + 1);

                MagicString { str: MagicStr {
                    value: value.value,
                    counter: None,
                }}
            }
        }


        let mut hm: HashMap<MagicString, i32> = HashMap::new();

        let counter = Cell::new(0);
        let str = |value: &'static str| -> MagicStr {
            MagicStr { counter: Some(&counter), value }
        };
        let string = |value: &'static str| -> MagicString {
            MagicString { str: MagicStr { counter: None, value } }
        };

        let inc = || { counter.set(counter.get() + 1); };

        assert_eq!(counter.get(), 0);
        assert_eq!(hm.resident(), 0);
        assert_eq!(hm.len(), 0);
        assert_eq!(hm.cap(), 0);

        // get_or_insert.
        let v = hm.get_or_insert(&str("hi"), 69);
        assert_eq!(*v, 69);
        assert_eq!(counter.get(), 1);
        assert_eq!(hm.resident(), 1);
        assert_eq!(hm.len(), 1);


        // insert same key does nothing.

        let v = hm.get_or_insert(&str("hi"), 70);
        assert_eq!(*v, 69);
        assert_eq!(counter.get(), 1);
        assert_eq!(hm.resident(), 1);
        assert_eq!(hm.len(), 1);

        let v = hm.get_or_insert_with(&str("hi"), || { inc(); 70 });
        assert_eq!(*v, 69);
        assert_eq!(counter.get(), 1);
        assert_eq!(hm.resident(), 1);
        assert_eq!(hm.len(), 1);

        let v = hm.get_or_insert_with_key(&str("hi"),
            |k| { inc(); (k.into(), 70) });
        assert_eq!(*v, 69);
        assert_eq!(counter.get(), 1);
        assert_eq!(hm.resident(), 1);
        assert_eq!(hm.len(), 1);

        let v = hm.kget_or_insert(string("hi"), 70);
        assert_eq!(*v, 69);
        assert_eq!(counter.get(), 1);
        assert_eq!(hm.resident(), 1);
        assert_eq!(hm.len(), 1);

        let v = hm.kget_or_insert_with(string("hi"), || { inc(); 70 });
        assert_eq!(*v, 69);
        assert_eq!(counter.get(), 1);
        assert_eq!(hm.resident(), 1);
        assert_eq!(hm.len(), 1);


        // get_or_insert_with.
        let v = hm.get_or_insert_with(&str("ho"), || { inc(); 12 });
        assert_eq!(*v, 12);
        assert_eq!(counter.get(), 3);
        assert_eq!(hm.resident(), 2);
        assert_eq!(hm.len(), 2);

        // get_or_insert_with_key.
        let v = hm.get_or_insert_with_key(&str("hu"), |k| {
            assert_eq!(counter.get(), 3);
            inc();
            (k.into(), 8)
        });
        assert_eq!(*v, 8);
        assert_eq!(counter.get(), 5);
        assert_eq!(hm.resident(), 3);
        assert_eq!(hm.len(), 3);

        // kget_or_insert
        let v = hm.kget_or_insert(string("he"), 123);
        assert_eq!(*v, 123);
        assert_eq!(counter.get(), 5);
        assert_eq!(hm.resident(), 4);
        assert_eq!(hm.len(), 4);

        // kget_or_insert_with
        let v = hm.kget_or_insert_with(string("ha"), || { inc(); 231 });
        assert_eq!(*v, 231);
        assert_eq!(counter.get(), 6);
        assert_eq!(hm.resident(), 5);
        assert_eq!(hm.len(), 5);
    }

    #[test]
    fn hm_get_or_insert_present_on_grow() {
        let mut hm: HashMap<u32, u32> = HashMap::new();
        hm.kget_or_insert(0, 0);

        while hm.len() < hm.cap() {
            let i = hm.len() as u32;
            let x = *hm.kget_or_insert(i, i);
            assert_eq!(x, i);
        }

        let x = *hm.kget_or_insert(0, 1);
        assert_eq!(x, 0);
    }
    */


    #[test]
    fn hm_drop_and_clear() {
        use crate::mem::Cell;

        #[derive(PartialEq, Eq)]
        struct Dropper<'a> {
            ticket: u32,
            counter: &'a Cell<u32>
        }

        impl<'a> Drop for Dropper<'a> {
            fn drop(&mut self) {
                assert_eq!(self.counter.get(), self.ticket);
                self.counter.set(self.counter.get() + 1);
            }
        }

        impl<'a> core::hash::Hash for Dropper<'a> {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                self.ticket.hash(state)
            }
        }

        let counter = Cell::new(0);
        let d = |ticket: u32| {
            Dropper { ticket, counter: &counter }
        };

        // basic drop.
        let mut hm = HashMap::new();
        hm.insert(d(0), d(1));
        hm.insert(d(2), d(3));
        hm.insert(d(4), d(5));
        drop(hm);
        assert_eq!(counter.get(), 6);

        // clear.
        counter.set(0);
        let mut hm = HashMap::new();
        hm.insert(d(0), d(1));
        hm.insert(d(2), d(3));
        hm.insert(d(4), d(5));
        hm.clear();
        assert_eq!(hm.len(), 0);
        assert_eq!(counter.get(), 6);

        counter.set(0);
        hm.insert(d(0), d(1));
        hm.insert(d(2), d(3));
        hm.insert(d(4), d(5));
        drop(hm);
        assert_eq!(counter.get(), 6);

        counter.set(0);
        let mut hm = HashMap::new();
        hm.insert(d(0), d(1));
        hm.insert(d(2), d(3));
        hm.insert(d(4), d(5));
        hm.clear();

        counter.set(0);
        drop(hm);
        assert_eq!(counter.get(), 0);

        // use after clear.
        counter.set(0);
        let mut hm = HashMap::new();
        hm.insert(0, 1);
        hm.insert(2, 3);
        hm.insert(4, 5);
        hm.clear();
        hm.insert(6, 7);
        assert!(hm.get(&0).is_none());
        assert!(hm.get(&2).is_none());
        assert!(hm.get(&4).is_none());
        assert_eq!(hm.get(&6), Some(&7));
    }

    /*
    #[test]
    fn hm_move_into() {
        struct Seed {
            _dummy: String,
        }

        impl<K: Hash> HashFnSeed<K> for Seed {
            type Seed = <DefaultSeed as HashFnSeed<K>>::Seed;

            type Hash = <DefaultSeed as HashFnSeed<K>>::Hash;

            type F = <DefaultSeed as HashFnSeed<K>>::F;

            fn seed(&self) -> Self::Seed {
                FxHasher32::DEFAULT_SEED
            }
        }

        let mut hm = HashMap::with_seed_in(
            GlobalAlloc,
            Seed { _dummy: "ok".to_string() });

        hm.insert("5".to_string(), "3".to_string());
        hm.insert("8".to_string(), "12".to_string());
        hm.insert("398".to_string(), "128".to_string());

        let hm = hm.move_into(GlobalAlloc);

        assert_eq!(hm.get(&"5".to_string()), Some(&"3".to_string()));
        assert_eq!(hm.get(&"8".to_string()), Some(&"12".to_string()));
        assert_eq!(hm.get(&"398".to_string()), Some(&"128".to_string()));
    }
    */

    #[test]
    fn hm_lookup_stopped_at_first_tombstone_regression() {
        let mut hm = HashMap::with_hash_fn_in(GlobalAlloc, ConstHash);

        for i in 0..2*Group::WIDTH {
            assert_eq!(hm.insert(i, i), None);
        }

        assert_eq!(hm.remove(&0), Some((0,0)));
        // this would fail if we stopped at the first tombstone (where 0 was).
        assert_eq!(hm.remove(&Group::WIDTH), Some((Group::WIDTH,Group::WIDTH)));

        // in first group.
        assert_eq!(hm.insert(5*Group::WIDTH, 42), None);
        // this is now inserted in the second group.
        assert_eq!(hm.insert(0, 1), None);
        // leave tombstone in first group.
        assert_eq!(hm.remove(&(5*Group::WIDTH)), Some((5*Group::WIDTH,42)));

        // this would fail if we stopped at the tombstone in the first group.
        assert_eq!(hm.get(&0), Some(&1));

        // this would insert into the first group.
        assert_eq!(hm.insert(0, 2), Some((0, 1)));

        assert_eq!(hm.get(&0), Some(&2));
    }
}

