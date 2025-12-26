use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use std::env;

/// Redis connection wrapper
pub struct RedisClient {
    connection: MultiplexedConnection,
}

impl RedisClient {
    /// Initialize Redis connection from environment variable
    pub async fn init() -> Result<Self, String> {
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let client =
            Client::open(redis_url).map_err(|e| format!("Failed to create Redis client: {}", e))?;

        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

        println!("✅ Connected successfully to Redis");

        Ok(Self { connection })
    }

    /// Get the Redis connection
    pub fn get_connection(&self) -> MultiplexedConnection {
        self.connection.clone()
    }
}

/// Redis service for session and cache management
#[derive(Clone)]
pub struct RedisService {
    connection: MultiplexedConnection,
}

impl RedisService {
    /// Create a new Redis service
    pub fn new(client: &RedisClient) -> Self {
        Self {
            connection: client.get_connection(),
        }
    }

    // ============================================
    // Session Management (JWT + Redis)
    // ============================================

    /// Store a session token in Redis
    pub async fn store_session(
        &self,
        user_id: &str,
        token: &str,
        expiry_seconds: u64,
    ) -> Result<(), String> {
        let mut conn = self.connection.clone();
        let key = format!("session:{}", user_id);

        conn.set_ex::<_, _, ()>(&key, token, expiry_seconds)
            .await
            .map_err(|e| format!("Failed to store session: {}", e))?;

        // Also store reverse lookup (token -> user_id) for validation
        let token_key = format!("token:{}", token);
        conn.set_ex::<_, _, ()>(&token_key, user_id, expiry_seconds)
            .await
            .map_err(|e| format!("Failed to store token mapping: {}", e))?;

        Ok(())
    }

    /// Validate a session token
    pub async fn validate_session(&self, token: &str) -> Result<Option<String>, String> {
        let mut conn = self.connection.clone();
        let token_key = format!("token:{}", token);

        let user_id: Option<String> = conn
            .get(&token_key)
            .await
            .map_err(|e| format!("Failed to validate session: {}", e))?;

        Ok(user_id)
    }

    /// Get user's current session token
    pub async fn get_session(&self, user_id: &str) -> Result<Option<String>, String> {
        let mut conn = self.connection.clone();
        let key = format!("session:{}", user_id);

        let token: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| format!("Failed to get session: {}", e))?;

        Ok(token)
    }

    /// Invalidate a user's session (logout)
    pub async fn invalidate_session(&self, user_id: &str) -> Result<(), String> {
        let mut conn = self.connection.clone();
        let session_key = format!("session:{}", user_id);

        // Get the token first to delete the reverse lookup
        if let Some(token) = self.get_session(user_id).await? {
            let token_key = format!("token:{}", token);
            conn.del::<_, ()>(&token_key)
                .await
                .map_err(|e| format!("Failed to delete token: {}", e))?;
        }

        conn.del::<_, ()>(&session_key)
            .await
            .map_err(|e| format!("Failed to delete session: {}", e))?;

        Ok(())
    }

    /// Invalidate all sessions for a user
    pub async fn invalidate_all_sessions(&self, user_id: &str) -> Result<(), String> {
        self.invalidate_session(user_id).await
    }

    // ============================================
    // Caching
    // ============================================

    /// Set a cache value with expiration
    pub async fn cache_set(
        &self,
        key: &str,
        value: &str,
        expiry_seconds: u64,
    ) -> Result<(), String> {
        let mut conn = self.connection.clone();
        let cache_key = format!("cache:{}", key);

        conn.set_ex::<_, _, ()>(&cache_key, value, expiry_seconds)
            .await
            .map_err(|e| format!("Failed to set cache: {}", e))?;

        Ok(())
    }

    /// Get a cached value
    pub async fn cache_get(&self, key: &str) -> Result<Option<String>, String> {
        let mut conn = self.connection.clone();
        let cache_key = format!("cache:{}", key);

        let value: Option<String> = conn
            .get(&cache_key)
            .await
            .map_err(|e| format!("Failed to get cache: {}", e))?;

        Ok(value)
    }

    /// Delete a cached value
    pub async fn cache_delete(&self, key: &str) -> Result<(), String> {
        let mut conn = self.connection.clone();
        let cache_key = format!("cache:{}", key);

        conn.del::<_, ()>(&cache_key)
            .await
            .map_err(|e| format!("Failed to delete cache: {}", e))?;

        Ok(())
    }

    /// Set a cache value with JSON serialization
    pub async fn cache_set_json<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
        expiry_seconds: u64,
    ) -> Result<(), String> {
        let json = serde_json::to_string(value)
            .map_err(|e| format!("Failed to serialize value: {}", e))?;
        self.cache_set(key, &json, expiry_seconds).await
    }

    /// Get a cached JSON value
    pub async fn cache_get_json<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, String> {
        match self.cache_get(key).await? {
            Some(json) => {
                let value = serde_json::from_str(&json)
                    .map_err(|e| format!("Failed to deserialize value: {}", e))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Check if a cache key exists
    pub async fn cache_exists(&self, key: &str) -> Result<bool, String> {
        let mut conn = self.connection.clone();
        let cache_key = format!("cache:{}", key);

        let exists: bool = conn
            .exists(&cache_key)
            .await
            .map_err(|e| format!("Failed to check cache existence: {}", e))?;

        Ok(exists)
    }

    /// Set cache TTL (time to live)
    pub async fn cache_expire(&self, key: &str, seconds: u64) -> Result<(), String> {
        let mut conn = self.connection.clone();
        let cache_key = format!("cache:{}", key);

        conn.expire::<_, ()>(&cache_key, seconds as i64)
            .await
            .map_err(|e| format!("Failed to set cache expiry: {}", e))?;

        Ok(())
    }

    // ============================================
    // Rate Limiting Helper
    // ============================================

    /// Increment a rate limit counter
    pub async fn rate_limit_increment(
        &self,
        key: &str,
        window_seconds: u64,
    ) -> Result<u64, String> {
        let mut conn = self.connection.clone();
        let rate_key = format!("ratelimit:{}", key);

        // Increment the counter
        let count: u64 = conn
            .incr(&rate_key, 1)
            .await
            .map_err(|e| format!("Failed to increment rate limit: {}", e))?;

        // Set expiry on first increment
        if count == 1 {
            conn.expire::<_, ()>(&rate_key, window_seconds as i64)
                .await
                .map_err(|e| format!("Failed to set rate limit expiry: {}", e))?;
        }

        Ok(count)
    }

    /// Check if rate limit is exceeded
    pub async fn is_rate_limited(
        &self,
        key: &str,
        max_requests: u64,
        window_seconds: u64,
    ) -> Result<bool, String> {
        let count = self.rate_limit_increment(key, window_seconds).await?;
        Ok(count > max_requests)
    }
}

/// Convenience function to connect to Redis
pub async fn connect_to_redis() -> Result<RedisClient, String> {
    RedisClient::init().await
}
