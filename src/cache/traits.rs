use async_trait::async_trait;
use std::fmt::Debug;
use std::hash::Hash;

pub trait CacheKey: Hash + Eq + Send + Sync + Clone + Debug + 'static {}

impl<T> CacheKey for T where T: Hash + Eq + Send + Sync + Clone + Debug + 'static {}

pub trait CacheValue: Send + Sync + Clone + Debug + 'static {}

impl<T> CacheValue for T where T: Send + Sync + Clone + Debug + 'static {}

#[async_trait]
pub trait Cache<K: CacheKey, V: CacheValue> {
    async fn get(&self, key: &K) -> Option<V>;
    async fn set(&self, key: K, value: V) -> Option<V>;
    async fn remove(&self, key: &K) -> Option<V>;
}
