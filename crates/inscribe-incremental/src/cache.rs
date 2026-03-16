use std::collections::HashMap;
use std::hash::Hash;
use std::io;
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::fingerprint::Fingerprint;

#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    pub fingerprint: Fingerprint,
    pub value: V,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskCacheEntry<V> {
    pub fingerprint: Fingerprint,
    pub value: V,
}

#[derive(Debug)]
pub struct Cache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
}

impl<K, V> Default for Cache<K, V> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
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

#[derive(Debug, Clone)]
pub struct DiskCache {
    root: PathBuf,
    query: &'static str,
}

impl DiskCache {
    pub fn new(root: impl Into<PathBuf>, query: &'static str) -> Self {
        Self {
            root: root.into(),
            query,
        }
    }

    pub fn load<K, V>(&self, key: &K) -> io::Result<Option<DiskCacheEntry<V>>>
    where
        K: Hash,
        V: DeserializeOwned,
    {
        let path = self.entry_path(key);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(path)?;
        let entry = bincode::deserialize::<DiskCacheEntry<V>>(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        Ok(Some(entry))
    }

    pub fn load_if_fresh<K, V>(
        &self,
        key: &K,
        fingerprint: Fingerprint,
    ) -> io::Result<Option<V>>
    where
        K: Hash,
        V: DeserializeOwned,
    {
        let entry = match self.load::<K, V>(key)? {
            Some(entry) => entry,
            None => return Ok(None),
        };
        if entry.fingerprint == fingerprint {
            Ok(Some(entry.value))
        } else {
            Ok(None)
        }
    }

    pub fn store<K, V>(
        &self,
        key: &K,
        fingerprint: Fingerprint,
        value: &V,
    ) -> io::Result<()>
    where
        K: Hash,
        V: Serialize,
    {
        let path = self.entry_path(key);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let entry = DiskCacheEntry {
            fingerprint,
            value,
        };
        let bytes = bincode::serialize(&entry)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    fn entry_path<K: Hash>(&self, key: &K) -> PathBuf {
        let key_hash = Fingerprint::of(key).as_u64();
        let filename = format!("{:016x}.bin", key_hash);
        self.root.join(self.query).join(filename)
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
