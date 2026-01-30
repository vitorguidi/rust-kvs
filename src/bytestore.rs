use rkyv::{AlignedVec, Serialize, Deserialize, Archive, check_archived_root};
use rkyv::ser::Serializer;
use rkyv::ser::serializers::AllocSerializer;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;
use crate::CacheKey;

#[derive(Clone, Debug)]
pub struct ByteEntry {
    pub data: Arc<AlignedVec>,
    pub expires_at: Option<Instant>,
}

impl ByteEntry {
    pub fn new(data: AlignedVec, ttl:Option<Duration>) -> Self {
        let expires_at = ttl.map(|d| Instant::now() + d);
        Self {
            data: Arc::new(data),
            expires_at
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(deadline) = self.expires_at {
            Instant::now() > deadline
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct ByteStore<K> {
    inner: Arc<RwLock<HashMap<K, ByteEntry>>>,
}

impl<K: CacheKey> ByteStore<K> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn get(&self, key: &K) -> Option<Arc<AlignedVec>> {
        let read_guard = self.inner.read().await;
        if let Some(entry) = read_guard.get(key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    pub async fn set(
        &self,
        key: K,
        data: AlignedVec,
        ttl: Option<Duration>
    ) {
        let mut write_guard = self.inner.write().await;
        let entry = ByteEntry::new(data, ttl);
        write_guard.insert(key, entry);
    }
}

pub trait RkyvCacheable:
    Archive
    + for<'a> Serialize<AllocSerializer<256>>
    + Send
    + Sync
    + 'static
{}

impl<T> RkyvCacheable for T
where 
    T: Archive
    + for<'a> Serialize<AllocSerializer<256>>
    + Send
    + Sync
    + 'static
{}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug, PartialEq))]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub active_sessions: Vec<String>,
    pub metadata: Vec<u8>,
}

pub fn to_bytes<T>(value: &T) -> AlignedVec
where
    T: Serialize<AllocSerializer<256>>,
{
    let mut serializer = AllocSerializer::<256>::default();
    serializer
        .serialize_value(value)
        .expect("Failed to serialize value.");

    let buffer = serializer.into_serializer().into_inner();
    buffer
}

pub fn access_user(bytes: &[u8]) -> &ArchivedUser {
    let archived = check_archived_root::<User>(bytes)
        .expect("Data corruption detected");
    archived
}

pub struct TypedCache<K> {
    inner: ByteCache<K>,
}

impl<K: CacheKey> TypedCache<K> {
    pub fn new() -> Self {
        Self {
            inner: ByteCache::new(),
        }
    }
    pub async fn set<V>(&self, key: K, value: &V, ttl: Option<Duration>)
    where
        V: Serialize<AllocSerializer<256>>,
    {
        let mut serializer = AllocSerializer::<256>::default();
        serializer.serialize_value(value).unwrap();
        let bytes = serializer.into_serializer().into_inner();
        self.inner.set(key, bytes, ttl);
    }
    pub async fn get_raw(&self, key: &K) -> Option<Arc<AlignedVec>> {
        self.inner.get(key)
    }
}

#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub data: Arc<AlignedVec>,
    pub expires_at: Option<Instant>,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        if let Some(deadline) = self.expires_at {
            Instant::now() > deadline
        } else {
            false
        }
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
        if let Some(entry) = self.store.get(key) {
            if entry.is_expired() {
                return None;
            }
            return Some(entry.data.clone());
        }
        None
    }
    pub fn set(&self, key: K, value: AlignedVec, ttl: Option<Duration>) -> Option<Arc<AlignedVec>> {
        let expires_at = ttl.map(|d| Instant::now() + d);
        let new_entry = CacheEntry {
            data: Arc::new(value),
            expires_at,
        };
        self.store
            .insert(key, new_entry)
            .map(|old_entry| old_entry.data)
    }
}

impl<K: CacheKey> ByteCache<K> {
    pub async fn run_eviction(&self) {
        let mut expired_keys = Vec::new();
        for r in self.store.iter() {
            if r.value().is_expired() {
                expired_keys.push(r.key().clone());
            }
        }
        for k in expired_keys {
            self.store.remove(&k);
        }
    }
}