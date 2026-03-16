use std::fmt;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Fingerprint(u64);

impl Fingerprint {
    pub fn of<T: Hash>(value: &T) -> Self {
        let mut hasher = FnvHasher::new();
        value.hash(&mut hasher);
        Self(hasher.finish())
    }

    pub fn combine(self, other: Fingerprint) -> Self {
        let mut builder = FingerprintBuilder::new();
        builder.update_u64(self.0);
        builder.update_u64(other.0);
        builder.finish()
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

pub struct FingerprintBuilder {
    hasher: FnvHasher,
}

impl FingerprintBuilder {
    pub fn new() -> Self {
        Self {
            hasher: FnvHasher::new(),
        }
    }

    pub fn update<T: Hash>(&mut self, value: &T) {
        value.hash(&mut self.hasher);
    }

    pub fn update_bytes(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes);
    }

    pub fn update_u64(&mut self, value: u64) {
        self.hasher.write(&value.to_le_bytes());
    }

    pub fn update_i64(&mut self, value: i64) {
        self.hasher.write(&value.to_le_bytes());
    }

    pub fn update_usize(&mut self, value: usize) {
        self.hasher.write(&value.to_le_bytes());
    }

    pub fn update_bool(&mut self, value: bool) {
        self.hasher.write(&[value as u8]);
    }

    pub fn update_str(&mut self, value: &str) {
        self.hasher.write(value.as_bytes());
    }

    pub fn update_fingerprint(&mut self, value: Fingerprint) {
        self.update_u64(value.0);
    }

    pub fn finish(self) -> Fingerprint {
        Fingerprint(self.hasher.finish())
    }
}

impl Default for FingerprintBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct FnvHasher(u64);

impl FnvHasher {
    fn new() -> Self {
        Self(FNV_OFFSET_BASIS)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Fingerprint, FingerprintBuilder};

    #[test]
    fn fingerprints_match_for_same_data() {
        let direct = Fingerprint::of(&"inscribe");
        let mut builder = FingerprintBuilder::new();
        builder.update_str("inscribe");
        assert_eq!(direct, builder.finish());
    }
}
