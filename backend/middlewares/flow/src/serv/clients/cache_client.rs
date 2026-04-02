//! Redis cache helpers: distributed spin lock based on `SET NX` + token verification on release.
//!
//! 基于 `SET NX` 的分布式自旋锁，释放时校验 token，避免误删其他持有者写入的键。

use std::time::{Duration, Instant};

use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

/// Configuration for [`FlowCacheClient::spin_lock_acquire`] / [`FlowCacheClient::with_spin_lock`].
#[derive(Debug, Clone)]
pub struct CacheSpinLockConfig {
    /// Sleep interval between failed `set_nx` attempts.
    pub spin_interval: Duration,
    /// TTL (seconds) passed to [`TardisFunsInst::cache`] `expire` after a successful lock.
    pub lock_ttl_sec: i64,
    /// If set, [`spin_lock_acquire`] returns error when total wait exceeds this duration.
    pub max_wait: Option<Duration>,
}

impl Default for CacheSpinLockConfig {
    fn default() -> Self {
        Self {
            spin_interval: Duration::from_millis(50),
            lock_ttl_sec: 5,
            max_wait: None,
        }
    }
}

pub struct FlowCacheClient;

impl FlowCacheClient {
    /// Spins until `set_nx` succeeds, then sets key TTL. Returns an opaque token that must be passed to [`spin_lock_release`].
    pub async fn spin_lock_acquire(lock_key: &str, funs: &TardisFunsInst, config: &CacheSpinLockConfig) -> TardisResult<String> {
        let token = TardisFuns::field.nanoid();
        let start = Instant::now();
        loop {
            if funs.cache().set_nx(lock_key, &token).await? {
                funs.cache().expire(lock_key, config.lock_ttl_sec).await?;
                return Ok(token);
            }
            if let Some(max) = config.max_wait {
                if start.elapsed() >= max {
                    return Err(funs.err().internal_error(
                        "cache_client",
                        "spin_lock_acquire",
                        "cache spin lock acquire timeout",
                        "408-flow-cache-spin-lock-timeout",
                    ));
                }
            }
            tardis::tokio::time::sleep(config.spin_interval).await;
        }
    }

    /// Deletes the lock key only if the stored value still equals `token` (safe release).
    pub async fn spin_lock_release(lock_key: &str, token: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        if funs.cache().get(lock_key).await?.as_deref() == Some(token) {
            funs.cache().del(lock_key).await?;
        }
        Ok(())
    }

    /// Acquires the spin lock, runs `f`, then releases the lock (even if `f` returns `Err`).
    pub async fn with_spin_lock<F, Fut, T>(lock_key: &str, funs: &TardisFunsInst, config: &CacheSpinLockConfig, f: F) -> TardisResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = TardisResult<T>>,
    {
        let token = Self::spin_lock_acquire(lock_key, funs, config).await?;
        let out = f().await;
        let _ = Self::spin_lock_release(lock_key, &token, funs).await;
        out
    }
}
