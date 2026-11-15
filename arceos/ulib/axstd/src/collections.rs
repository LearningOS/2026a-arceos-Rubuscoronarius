//! Collections available to ArceOS applications.
pub use alloc::collections::*;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};

// A deterministic FNV-1a hasher; it is not intended for adversarial keys.
struct FnvHasher(u64);
impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
    }
}
fn hash<K: Hash + ?Sized>(key: &K) -> usize {
    let mut hasher = FnvHasher(0xcbf29ce484222325);
    key.hash(&mut hasher);
    hasher.finish() as usize
}

/// A hash table with separate chaining and deterministic hashing.
pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    len: usize,
}
impl<K: Eq + Hash, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self {
            buckets: Vec::new(),
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    fn grow(&mut self) {
        let count = (self.buckets.len() * 2).max(16);
        let mut buckets: Vec<Vec<(K, V)>> = (0..count).map(|_| Vec::new()).collect();
        for (key, value) in self.buckets.drain(..).flatten() {
            let index = hash(&key) % count;
            buckets[index].push((key, value));
        }
        self.buckets = buckets;
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.len + 1 > self.buckets.len() * 3 / 4 {
            self.grow();
        }
        let index = hash(&key) % self.buckets.len();
        for (k, v) in &mut self.buckets[index] {
            if *k == key {
                return Some(core::mem::replace(v, value));
            }
        }
        self.buckets[index].push((key, value));
        self.len += 1;
        None
    }
    pub fn get<Q: Hash + Eq + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: core::borrow::Borrow<Q>,
    {
        if self.buckets.is_empty() {
            return None;
        }
        self.buckets[hash(key) % self.buckets.len()]
            .iter()
            .find(|(k, _)| k.borrow() == key)
            .map(|(_, v)| v)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.buckets.iter().flatten().map(|(k, v)| (k, v))
    }
}
impl<K: Eq + Hash, V> Default for HashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
