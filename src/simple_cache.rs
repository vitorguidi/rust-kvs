use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::{Cache, CacheKey, CacheValue};

#[derive(Clone)]
pub struct SimpleCache<K,V> {
    inner: Arc<RwLock<HashMap<K,V>>>
}

impl<K,V> SimpleCache<K,V> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl<K,V> Cache<K,V> for SimpleCache<K,V>
where
    K: CacheKey,
    V: CacheValue
{
    async fn get(&self, key: &K) -> Option<V> {
        let read_guard = self.inner
            .read()
            .await;
        read_guard
            .get(key)
            .cloned()
    }

    async fn set(&self, key: K, value: V) -> Option<V> {
        let mut write_guard = self.inner
            .write()
            .await;

        write_guard.insert(key, value)
    }

    async fn remove(&self, key: &K) -> Option<V> {
        let mut write_guard = self.inner
            .write()
            .await;
        write_guard.remove(key)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_cache_get_none() {
        let cache = SimpleCache::<u32, u32>::new();
        let mut result = cache.set(2,2).await;
        assert_eq!(result, None);
        result = cache.get(&2).await;
        assert_eq!(result, Some(2));
    }
}