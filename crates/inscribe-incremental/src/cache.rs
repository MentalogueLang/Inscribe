use std::collections::HashMap;
use std::hash::Hash;

use crate::fingerprint::Fingerprint;

#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    pub fingerprint: Fingerprint,
    pub value: V,
}

#[derive(Debug, Default)]
pub struct Cache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash,
{
    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).map(|entry| &entry.value)
    }

    pub fn get_if_fresh(&self, key: &K, fingerprint: Fingerprint) -> Option<&V> {
        self.entries
            .get(key)
            .filter(|entry| entry.fingerprint == fingerprint)
            .map(|entry| &entry.value)
    }

    pub fn fingerprint(&self, key: &K) -> Option<Fingerprint> {
        self.entries.get(key).map(|entry| entry.fingerprint)
    }

    pub fn insert(&mut self, key: K, fingerprint: Fingerprint, value: V) {
        let entry = CacheEntry { fingerprint, value };
        self.entries.insert(key, entry);
    }

    pub fn invalidate(&mut self, key: &K) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Cache;
    use crate::fingerprint::Fingerprint;

    #[test]
    fn respects_fingerprint_freshness() {
        let mut cache = Cache::default();
        let key = "value";
        let fingerprint = Fingerprint::of(&"v1");
        cache.insert(key, fingerprint, 42);

        assert_eq!(cache.get_if_fresh(&key, fingerprint), Some(&42));
        assert_eq!(
            cache.get_if_fresh(&key, Fingerprint::of(&"v2")),
            None
        );
    }
}
