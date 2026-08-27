use crate::alloc::{Alloc, GlobalAlloc};
use crate::hash::{Hash, HashFn, HashMap, hash_map::{SlotIdx, Hash32}, fxhash::FxHashFn};


struct Entry<V> {
    prev: u32,
    next: u32,
    value: V,
}


pub struct Lru<K, V, A: Alloc = GlobalAlloc, H: HashFn<K, u32> = FxHashFn> {
    map: HashMap<K, Entry<V>, A, H>,
    cap: u32,
    head: u32,
    tail: u32,
}

impl<K: Hash, V> Lru<K, V, GlobalAlloc, FxHashFn> {
    #[inline(always)]
    pub fn with_cap(cap: usize) -> Self {
        Self::with_cap_in(GlobalAlloc, cap)
    }
}

impl<K: Hash, V, A: Alloc> Lru<K, V, A, FxHashFn> {
    #[inline(always)]
    pub fn with_cap_in(alloc: A, cap: usize) -> Self {
        Self::with_hash_and_cap_in(alloc, FxHashFn, cap)
    }
}

impl<K, V, A: Alloc, H: HashFn<K, u32>> Lru<K, V, A, H> {
    pub fn with_hash_and_cap_in(alloc: A, h: H, cap: usize) -> Self {
        assert!(cap > 1);
        Self {
            map: HashMap::with_hash_and_cap_in(alloc, h, cap),
            cap: cap as u32,
            head: u32::MAX,
            tail: u32::MAX,
        }
    }

    pub fn get_or_insert(&mut self, k: K, build: impl FnOnce() -> V) -> &V
    where K: Eq + Copy, A: Clone, H: Clone {
        let hash = self.map.hash(&k).0;
        return self.get_or_insert_cmp(hash, move |other| other == &k, move || (k, build()));
    }

    pub fn get_or_insert_cmp(&mut self,
        hash: u32,
        cmp: impl Fn(&K) -> bool,
        build: impl FnOnce() -> (K, V))
        -> &V
    where A: Clone, H: Clone
    {
        let hash = Hash32(hash);

        let (present, slot) = self.map.lookup_cmp(hash, &cmp);
        if present {
            // move this entry to the head.
            if slot.0 != self.head {
                let entry = self.map.slot_mut(slot).1;

                // remove from list.
                let prev = core::mem::replace(&mut entry.prev, self.head);
                let next = core::mem::replace(&mut entry.next, u32::MAX);

                // fix up prev entry.
                if prev != u32::MAX {
                    self.map.slot_mut(SlotIdx(prev)).1.next = next;
                }
                else {
                    self.tail = next;
                }

                // fix up next entry.
                self.map.slot_mut(SlotIdx(next)).1.prev = prev;

                // insert at head.
                self.map.slot_mut(SlotIdx(self.head)).1.next = slot.0;
                self.head = slot.0;
            }

            return unsafe { &self.map.slot_unck(slot).1.value };
        }

        // no entry found -> build new entry.
        let (k, v) = build();
        debug_assert!(self.map.hash(&k) == hash);


        // evict last entry if full.
        if self.map.len() == self.cap as usize {
            let (_, last) = self.map.remove_at(SlotIdx(self.tail));
            debug_assert!(last.prev == u32::MAX);

            self.map.slot_mut(SlotIdx(last.next)).1.prev = u32::MAX;
            self.tail = last.next;
        }

        // rehash.
        let mut slot = slot;
        if self.map.resident() == self.map.cap() {
            let mut new_map = HashMap::with_hash_and_cap_in(
                self.map.alloc().clone(),
                self.map.hash_fn().clone(),
                self.cap as usize);

            let mut current = self.head;
            while current != u32::MAX {
                let (k, v) = self.map.remove_at(SlotIdx(current));

                let prev = v.prev;
                let next = v.next;

                let hash = new_map.hash(&k);
                let new_idx = new_map.lookup_cmp(hash, &cmp).1;
                let None = new_map.insert_at(new_idx, hash, k, v) else { unreachable!() };

                if next != u32::MAX {
                    new_map.slot_mut(SlotIdx(next)).1.prev = new_idx.0;
                }
                else {
                    self.head = new_idx.0;
                }

                if prev != u32::MAX {
                    self.map.slot_mut(SlotIdx(prev)).1.next = new_idx.0;
                }
                else {
                    self.tail = new_idx.0;
                }

                current = prev;
            }

            self.map = new_map;

            slot = self.map.lookup_cmp(hash, &cmp).1;
        }

        // insert entry.
        self.map.insert_at(slot, hash, k, Entry {
            prev: self.head,
            next: u32::MAX,
            value: v,
        });

        // insert at head.
        if self.head != u32::MAX {
            self.map.slot_mut(SlotIdx(self.head)).1.next = slot.0;
        }
        else {
            self.tail = slot.0;
        }
        self.head = slot.0;

        return unsafe { &self.map.slot_unck(slot).1.value };
    }


    pub fn iter(&self) -> impl Iterator<Item=(&K, &V)> {
        let mut prev = u32::MAX;
        let mut current = self.head;

        debug_assert!((current != u32::MAX) == (self.map.len() != 0));

        return core::iter::from_fn(move || {
            if current == u32::MAX {
                debug_assert!(self.tail == prev);
                return None;
            }

            let (k, entry) = self.map.slot(SlotIdx(current));
            debug_assert!(entry.next == prev);
            prev = current;
            current = entry.prev;

            return Some((k, &entry.value));
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lru_basic() {
        use crate::string::String;

        let mut lru: Lru<u32, String> = Lru::with_cap(3);

        let mut iter = lru.iter();
        assert_eq!(iter.next(), None);
        drop(iter);


        lru.get_or_insert(1, || String::from_str("one"));

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), None);
        drop(iter);

        lru.get_or_insert(1, || unreachable!());

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), None);
        drop(iter);


        lru.get_or_insert(2, || String::from_str("two"));

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), None);
        drop(iter);

        lru.get_or_insert(1, || unreachable!());

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), None);
        drop(iter);

        lru.get_or_insert(2, || unreachable!());

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), None);
        drop(iter);


        lru.get_or_insert(3, || String::from_str("three"));

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&3, &String::from_str("three"))));
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), None);
        drop(iter);

        lru.get_or_insert(1, || unreachable!());

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), Some((&3, &String::from_str("three"))));
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), None);
        drop(iter);

        lru.get_or_insert(2, || unreachable!());

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), Some((&3, &String::from_str("three"))));
        assert_eq!(iter.next(), None);
        drop(iter);

        lru.get_or_insert(3, || unreachable!());

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&3, &String::from_str("three"))));
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), None);
        drop(iter);

        lru.get_or_insert(2, || unreachable!());

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), Some((&3, &String::from_str("three"))));
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), None);
        drop(iter);

        lru.get_or_insert(1, || unreachable!());

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), Some((&3, &String::from_str("three"))));
        assert_eq!(iter.next(), None);
        drop(iter);


        lru.get_or_insert(4, || String::from_str("four"));

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&4, &String::from_str("four"))));
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), None);
        drop(iter);

        lru.get_or_insert(2, || unreachable!());

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), Some((&4, &String::from_str("four"))));
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), None);
        drop(iter);

        lru.get_or_insert(1, || unreachable!());

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), Some((&4, &String::from_str("four"))));
        assert_eq!(iter.next(), None);
        drop(iter);

        lru.get_or_insert(3, || String::from_str("three"));

        let mut iter = lru.iter();
        assert_eq!(iter.next(), Some((&3, &String::from_str("three"))));
        assert_eq!(iter.next(), Some((&1, &String::from_str("one"))));
        assert_eq!(iter.next(), Some((&2, &String::from_str("two"))));
        assert_eq!(iter.next(), None);
        drop(iter);
    }

    #[test]
    fn lru_rehash() {
        let mut num_rehashes = 0;
        let mut prev_resident = 0;

        let mut lru: Lru<u32, (), GlobalAlloc, FxHashFn> = Lru::with_hash_and_cap_in(GlobalAlloc, FxHashFn, 23);
        for i in 0..1000 {
            lru.get_or_insert(i, || ());

            // we do +1 here to make sure it's really a rehash.
            // when we evict, we may get one empty slot back,
            // but then on insert, we may reuse a tombstone.
            if lru.map.resident() + 1 < prev_resident {
                num_rehashes += 1;

                let mut expected = i;
                for (val, _) in lru.iter() {
                    assert_eq!(*val, expected);
                    expected -= 1;
                }
                assert_eq!(expected, i - lru.cap);
            }
            prev_resident = lru.map.resident();
        }

        assert!(num_rehashes > 0);
    }
}

