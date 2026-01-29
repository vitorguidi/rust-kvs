use std::sync::Arc;
use tokio::sync::Mutex;
use lru::LruCache;
use std::num::NonZeroUsize;
use async_trait::async_trait;

use crate::{Cache, CacheKey, CacheValue};

#[derive(Clone)]
pub struct LruCacheStore<K,V> {
    inner: Arc<Mutex<LruCache<K,V>>>,
}

impl <K,V> LruCacheStore<K,V>
where
    K: CacheKey,
    V: CacheValue,
{
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity)
            .expect("Capacity must be > 0");
        Self {
            inner: Arc::new(Mutex::new(
                LruCache::new(cap)
            )),
        }
    }
}

#[async_trait]
impl<K,V> Cache<K,V> for LruCacheStore<K,V>
where
    K: CacheKey,
    V: CacheValue,
{
    async fn get(&self, key: &K) -> Option<V> {
        let mut guard = self.inner.lock().await;
        guard.get(key).cloned()
    }

    async fn set(&self, key: K, value: V) -> Option<V> {
        let mut guard = self.inner.lock().await;
        guard.put(key, value)
    }

    async fn remove(&self, key: &K) -> Option<V> {
        let mut guard = self.inner.lock().await;
        guard.pop(key)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_cache_get_none() {
        let cache = LruCacheStore::<u32, u32>::new(2);
        let mut result = cache.set(2,2).await;
        assert_eq!(result, None);
        result = cache.get(&2).await;
        assert_eq!(result, Some(2));
        result = cache.set(3,3).await;
        assert_eq!(result, None);
        result = cache.get(&3).await;
        assert_eq!(result, Some(3));
        result = cache.set(4,4).await;
        assert_eq!(result, None);
        result = cache.get(&2).await;
        assert_eq!(result, None);
    }
}