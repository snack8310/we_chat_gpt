use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;

pub struct Cache {
    data: RwLock<HashMap<String, Instant>>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            data: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Instant> {
        let now = Instant::now();
        let mut data = self.data.write().await;
        if let Some(&expire_time) = data.get(key) {
            if now < expire_time {
                // the key is still valid
                return Some(expire_time);
            }
        }
        // the key is not valid or not found
        data.remove(key);
        None
    }

    pub async fn set(&self, key: &str, value: Instant, ttl: Duration) {
        let expire_time = value + ttl;
        let mut data = self.data.write().await;
        data.insert(key.to_string(), expire_time);
    }

    pub async fn _delete(&self, key: &str) {
        let mut data = self.data.write().await;
        data.remove(key);
    }

    pub async fn cleanup(&self, ttl: Duration) {
        let mut interval = interval(ttl);
        loop {
            interval.tick().await;
            let now = Instant::now();
            let mut data = self.data.write().await;
            let expired_keys: Vec<String> = data
                .iter()
                .filter(|&(_, &expire_time)| now >= expire_time)
                .map(|(key, _)| key.clone())
                .collect();
            for key in expired_keys {
                data.remove(&key);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    // use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache() {
        let cache = Cache::new();
        let key = "test_key";
        let value = Instant::now();
        let ttl = Duration::from_secs(1);

        // Test set and get methods
        cache.set(key, value, ttl).await;
        assert_eq!(cache.get(key).await, Some(value + ttl));
        assert_eq!(cache.get("nonexistent_key").await, None);

        // Test delete method
        cache._delete(key).await;
        assert_eq!(cache.get(key).await, None);

        // Test cleanup method
        // let ttl = Duration::from_millis(500);
        // cache.set(key, value, ttl).await;
        // sleep(Duration::from_millis(1000)).await;
        // cache.cleanup(ttl).await;
        // assert_eq!(cache.get(key).await, None);
    }
}
