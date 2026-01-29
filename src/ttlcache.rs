use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct Entry<V> {
    value: V,
    expires_at: Option<Instant>,
}

impl<V> Entry<V> {
    fn new(value: V, ttl: Option<Duration>) -> Self {
        let expires_at = ttl.map(|d| Instant::now() + d);
        Self {value, expires_at}
    }

    fn is_expired(&self) -> bool {
        if let Some(deadline) =self.expires_at {
            Instant::now() > deadline
        } else {
            false
        }
    }
}

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use crate::{Cache, CacheKey, CacheValue};

#[derive(Clone)]
pub struct  TtlCache<K,V> {
    store: Arc<RwLock<HashMap<K,Entry<V>>>>,
}

impl<K,V> TtlCache<K,V> {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl<K,V> Cache<K,V> for TtlCache<K,V>
where
    K: CacheKey,
    V: CacheValue,
{
    async fn get(&self, key:&K) -> Option<V> {
        let read_guard = self.store.read().await;

        if let Some(entry) = read_guard.get(key) {
            if entry.is_expired() {
                return None;
            }
            return Some(entry.value.clone());
        }

        None
    }

    async fn set(&self, key: K, value: V) -> Option<V> {
        let mut write_guard = self.store.write().await;
        let entry = Entry::new(
            value,
            Some(Duration::from_secs(300))
        );
        write_guard
            .insert(key, entry)
            .map(|old| old.value)
    }

    async fn remove(&self, key: &K) -> Option<V> {
        let mut write_guard = self.store.write().await;
        write_guard.remove(key).map(|e| e.value)
    }
}

impl <K, V> TtlCache<K,V>
where
    K: CacheKey,
    V: CacheValue,
{
    pub fn start_monitor(&self, interval: Duration) {
        let store_clone = self.store.clone();
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                let keys_to_remove = {
                    let read_guard = store_clone.read().await;
                    let mut expired_keys = Vec::new();
                    for (k,v) in read_guard.iter() {
                        if v.is_expired() {
                            expired_keys.push(k.clone());
                        }
                    }
                    expired_keys
                };

                if keys_to_remove.is_empty() {
                    continue;
                }

                let mut write_guard = store_clone
                    .write()
                    .await;

                for k in keys_to_remove {
                    if let Some(v) = write_guard.get(&k) {
                        if v.is_expired() {
                            write_guard.remove(&k);
                        }
                    }
                }
            }
        });


    }
}