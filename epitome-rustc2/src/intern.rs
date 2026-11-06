//! Interning: values stored once, named by a `u32`.
//!
//! Equal values get the same id, so comparing two ids compares the
//! values. The table is split into shards by hash, each with its own
//! locks, so that threads adding values rarely wait for each other; an
//! id's low bits say which shard holds it. Values are small (`Copy`, made
//! of other ids), so reading one copies it out under a read lock.

use crate::hash::{FastBuild, FastMap};
use std::hash::{BuildHasher, Hash};
use std::sync::{Mutex, RwLock};

const SHARD_BITS: u32 = 4;
const SHARDS: usize = 1 << SHARD_BITS;

struct Shard<T> {
    map: Mutex<FastMap<T, u32>>,
    values: RwLock<Vec<T>>,
}

pub struct Interner<T> {
    shards: Vec<Shard<T>>,
}

impl<T: Clone + Eq + Hash> Default for Interner<T> {
    fn default() -> Self {
        Interner {
            shards: (0..SHARDS)
                .map(|_| Shard { map: Mutex::new(FastMap::default()), values: RwLock::new(Vec::new()) })
                .collect(),
        }
    }
}

impl<T: Clone + Eq + Hash> Interner<T> {
    /// The id of `value`, adding it if it is new.
    pub fn intern(&self, value: T) -> u32 {
        let h = FastBuild::default().hash_one(&value);
        let s = (h as usize) & (SHARDS - 1);
        let shard = &self.shards[s];
        let mut map = shard.map.lock().unwrap();
        if let Some(&id) = map.get(&value) {
            return id;
        }
        let mut values = shard.values.write().unwrap();
        let id = ((values.len() as u32) << SHARD_BITS) | s as u32;
        values.push(value.clone());
        map.insert(value, id);
        id
    }

    /// The value with id `id`.
    pub fn get(&self, id: u32) -> T {
        let shard = &self.shards[(id as usize) & (SHARDS - 1)];
        shard.values.read().unwrap()[(id >> SHARD_BITS) as usize].clone()
    }

    /// How many values are stored.
    pub fn len(&self) -> usize {
        self.shards.iter().map(|s| s.values.read().unwrap().len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_value_same_id() {
        let i: Interner<(u32, u32)> = Interner::default();
        let a = i.intern((1, 2));
        let b = i.intern((3, 4));
        assert_ne!(a, b);
        assert_eq!(i.intern((1, 2)), a);
        assert_eq!(i.get(b), (3, 4));
        // From many threads at once.
        let ids: Vec<u32> = std::thread::scope(|s| {
            let hs: Vec<_> =
                (0..8).map(|_| s.spawn(|| (0..1000u32).map(|k| i.intern((k, k))).collect::<Vec<_>>())).collect();
            hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
        });
        for (n, &id) in ids.iter().enumerate() {
            assert_eq!(i.get(id), ((n % 1000) as u32, (n % 1000) as u32));
        }
        assert_eq!(i.len(), 1002);
    }
}
