use dashmap::DashMap;
use rkyv::AlignedVec;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::traits::CacheKey;

#[derive(Clone, Debug)]
struct CacheEntry {
    data: Arc<AlignedVec>,
    expires_at: Option<Instant>,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.expires_at
            .map(|deadline| Instant::now() > deadline)
            .unwrap_or(false)
    }
}

#[derive(Clone)]
pub struct ByteCache<K> {
    store: Arc<DashMap<K, CacheEntry>>,
}

impl<K: CacheKey> ByteCache<K> {
    pub fn new() -> Self {
        Self {
            store: Arc::new(DashMap::new()),
        }
    }

    pub fn get(&self, key: &K) -> Option<Arc<AlignedVec>> {
        self.store.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    pub fn set(
        &self,
        key: K,
        value: AlignedVec,
        ttl: Option<Duration>,
    ) -> Option<Arc<AlignedVec>> {
        let expires_at = ttl.map(|d| Instant::now() + d);
        let new_entry = CacheEntry {
            data: Arc::new(value),
            expires_at,
        };
        self.store.insert(key, new_entry).map(|old| old.data)
    }

    pub fn remove(&self, key: &K) -> Option<Arc<AlignedVec>> {
        self.store.remove(key).map(|(_, entry)| entry.data)
    }

    pub async fn run_eviction(&self) {
        let expired_keys: Vec<K> = self
            .store
            .iter()
            .filter(|r| r.value().is_expired())
            .map(|r| r.key().clone())
            .collect();

        for key in expired_keys {
            self.store.remove(&key);
        }
    }
}

impl<K: CacheKey> Default for ByteCache<K> {
    fn default() -> Self {
        Self::new()
    }
}
