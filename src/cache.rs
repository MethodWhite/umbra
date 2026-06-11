// Zone 6 — Research/Stubs (server-gated)
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

pub struct TtlCache<K, V> {
    inner: HashMap<K, CacheEntry<V>>,
    ttl: Duration,
}

impl<K: Eq + Hash, V: Clone> TtlCache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: HashMap::new(),
            ttl,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key).and_then(|entry| {
            if Instant::now() < entry.expires_at {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub fn set(&mut self, key: K, value: V) {
        self.inner.insert(
            key,
            CacheEntry {
                value,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }

    pub fn invalidate(&mut self, key: &K) {
        self.inner.remove(key);
    }

    pub fn clear_expired(&mut self) {
        let now = Instant::now();
        self.inner.retain(|_, entry| now < entry.expires_at);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct ProviderCache {
    pub model_cache: TtlCache<String, Vec<String>>,
    pub status_cache: TtlCache<String, bool>,
    pub connection_cache: TtlCache<String, reqwest::Client>,
}

impl ProviderCache {
    pub fn new() -> Self {
        Self {
            model_cache: TtlCache::new(Duration::from_secs(300)),
            status_cache: TtlCache::new(Duration::from_secs(60)),
            connection_cache: TtlCache::new(Duration::from_secs(3600)),
        }
    }

    pub fn get_or_create_client(&mut self, base_url: &str) -> reqwest::Client {
        let key = base_url.to_string();
        if let Some(client) = self.connection_cache.get(&key) {
            return client;
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        self.connection_cache.set(key, client.clone());
        client
    }
}

impl Default for ProviderCache {
    fn default() -> Self {
        Self::new()
    }
}
