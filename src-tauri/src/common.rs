use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniqueId(Uuid);

impl UniqueId {
    pub fn create() -> Self {
        UniqueId(Uuid::new_v4())
    }
}

/// Because JavaScript's int64 type loses precision, it needs to be split into two sets of data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsSafeHash(u32, u32);

impl std::hash::Hash for JsSafeHash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash_u64())
    }
}

impl JsSafeHash {
    pub fn from_u64(hash: u64) -> Self {
        let hash_upper = (hash >> 32) as u32;
        let hash_lower = hash as u32;
        Self(hash_lower, hash_upper)
    }

    pub fn hash_u64(&self) -> u64 {
        ((self.1 as u64) << 32) | (self.0 as u64)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_js_safe_hash() {
        todo!()
    }
}
