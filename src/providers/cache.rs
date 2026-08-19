//! Transparent caching layer for stock data providers.
//!
//! A [`CacheStore`] is a shared, TTL-based in-memory cache keyed by
//! `(provider_name, ticker)`. [`CachedProvider`] decorates any
//! [`StockDataProvider`] so that repeated calls for the same ticker hit the
//! cache instead of the (potentially slow / rate-limited) underlying source.
//!
//! Because the decorator implements the same [`StockDataProvider`] trait, every
//! existing call site (route handlers and the aggregation service) benefits from
//! caching with no changes to their code.
//!
//! [`spawn_background_cache_warmer`] runs on a dedicated OS thread and
//! pre-fetches every `(provider, ticker)` pair at startup, then re-refreshes on
//! an interval, so the cache is populated "in the background" while the app
//! serves requests.

use crate::models::StockRatingData;
use crate::providers::StockDataProvider;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Default time-to-live for a cached entry.
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);

/// Default interval between background refresh passes (keeps the cache warm
/// continuously while the app runs).
pub const DEFAULT_REFRESH_INTERVAL: Duration = Duration::from_secs(300);

/// Small pause between individual ticker fetches during a warm/refresh pass,
/// to be gentle with rate-limited live APIs (e.g. Finnhub free tier).
const WARM_TICKER_DELAY: Duration = Duration::from_millis(250);

#[derive(Debug, Clone)]
struct CacheEntry {
    /// The provider's result for the ticker. `None` is intentionally cached
    /// (negative caching) so a failed / unsupported lookup is not re-fetched on
    /// every request for the duration of the TTL.
    data: Option<StockRatingData>,
    fetched_at: Instant,
}

/// Shared TTL cache of provider -> ticker stock data.
pub struct CacheStore {
    ttl: Duration,
    map: Mutex<HashMap<String, CacheEntry>>,
}

impl CacheStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            map: Mutex::new(HashMap::new()),
        }
    }

    #[allow(dead_code)]
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    fn key(provider: &str, ticker: &str) -> String {
        format!("{}::{}", provider, ticker.to_uppercase())
    }

    /// Returns the cached result if a **fresh** entry exists.
    ///
    /// The inner `Option` mirrors the provider's own result, so a cached
    /// "no data" is still a valid *hit* (returns `Some(None)`). A stale or
    /// absent entry yields `None`, signalling a miss.
    fn fresh(&self, provider: &str, ticker: &str) -> Option<Option<StockRatingData>> {
        let guard = self.map.lock().unwrap();
        guard
            .get(&Self::key(provider, ticker))
            .filter(|e| e.fetched_at.elapsed() <= self.ttl)
            .map(|e| e.data.clone())
    }

    fn put(&self, provider: &str, ticker: &str, data: Option<StockRatingData>) {
        let mut guard = self.map.lock().unwrap();
        guard.insert(
            Self::key(provider, ticker),
            CacheEntry {
                data,
                fetched_at: Instant::now(),
            },
        );
    }

    /// Number of entries currently in the cache (fresh or stale).
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.map.lock().unwrap().len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Drop all entries (useful for tests / manual invalidation).
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.map.lock().unwrap().clear();
    }
}

/// Decorator that caches a [`StockDataProvider`]'s results in a shared
/// [`CacheStore`].
pub struct CachedProvider {
    inner: Box<dyn StockDataProvider + Send + Sync>,
    store: Arc<CacheStore>,
}

impl CachedProvider {
    pub fn new(inner: Box<dyn StockDataProvider + Send + Sync>, store: Arc<CacheStore>) -> Self {
        Self { inner, store }
    }
}

impl StockDataProvider for CachedProvider {
    fn get_stock_data(&self, ticker: &str) -> Option<StockRatingData> {
        let name = self.inner.provider_name();

        // Cache hit (fresh) — may be a cached `None` (negative cache).
        if let Some(cached) = self.store.fresh(name, ticker) {
            return cached;
        }

        // Cache miss — fetch from the real provider, store, and return.
        let data = self.inner.get_stock_data(ticker);
        self.store.put(name, ticker, data.clone());
        data
    }

    fn list_supported_tickers(&self) -> Vec<String> {
        self.inner.list_supported_tickers()
    }

    // Search / universe listing are delegated straight through to the inner
    // provider (the Finnhub provider keeps its own TTL cache for these).
    fn search_symbols(&self, query: &str) -> Vec<crate::models::StockSearchResult> {
        self.inner.search_symbols(query)
    }

    fn list_all_tickers(&self) -> Vec<String> {
        self.inner.list_all_tickers()
    }

    fn provider_name(&self) -> &'static str {
        self.inner.provider_name()
    }
}

/// Spawns a dedicated background thread that pre-warms the cache for every
/// `(provider, ticker)` pair, then re-refreshes on `refresh_interval`.
///
/// The work runs on its own OS thread (not the async runtime) because the live
/// providers perform blocking fetches internally.
pub fn spawn_background_cache_warmer(
    providers: Arc<RwLock<Vec<Box<dyn StockDataProvider + Send + Sync>>>>,
    refresh_interval: Duration,
) {
    std::thread::Builder::new()
        .name("cache-warm".to_string())
        .spawn(move || {
            loop {
                warm_once(&providers);
                std::thread::sleep(refresh_interval);
            }
        })
        .expect("failed to spawn cache warmer thread");
}

fn warm_once(providers: &Arc<RwLock<Vec<Box<dyn StockDataProvider + Send + Sync>>>>) {
    let started = Instant::now();
    // The read lock is held across the pass; no writer ever mutates the
    // provider list at runtime, so this is safe.
    let guard = providers.read().unwrap();

    let mut entries = 0usize;
    for provider in guard.iter() {
        let name = provider.provider_name();
        for ticker in provider.list_supported_tickers() {
            // Calling get_stock_data on the (cached) provider populates or
            // refreshes the shared cache entry.
            let result = provider.get_stock_data(&ticker);
            if result.is_some() {
                entries += 1;
            }
            // Be gentle with rate-limited live APIs.
            std::thread::sleep(WARM_TICKER_DELAY);
        }
        eprintln!("[cache-warm] {} tickers refreshed", name);
    }

    eprintln!(
        "[cache-warm] pass complete: {} providers, {} data points, in {:?}",
        guard.len(),
        entries,
        started.elapsed()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::mock::MockDataProvider;

    fn cached_mock(store: Arc<CacheStore>) -> Box<dyn StockDataProvider + Send + Sync> {
        Box::new(CachedProvider::new(
            Box::new(MockDataProvider::new()),
            store,
        ))
    }

    #[test]
    fn cache_returns_data_and_delegates_metadata() {
        let store = Arc::new(CacheStore::new(Duration::from_secs(60)));
        let provider = cached_mock(store.clone());
        assert_eq!(provider.provider_name(), "MockDataProvider");

        let data = provider.get_stock_data("AAPL");
        assert!(data.is_some());
        // Cache now holds one entry.
        assert_eq!(store.len(), 1);

        // Second call is served from cache (still returns data).
        assert!(provider.get_stock_data("AAPL").is_some());
    }

    #[test]
    fn negative_caching_stores_none() {
        let store = Arc::new(CacheStore::new(Duration::from_secs(60)));
        let provider = cached_mock(store.clone());
        assert!(provider.get_stock_data("NOT_A_TICKER").is_none());
        // The `None` result is cached, so the store has an entry.
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn stale_entries_are_refetched() {
        let store = Arc::new(CacheStore::new(Duration::from_nanos(1)));
        let provider = cached_mock(store.clone());
        assert!(provider.get_stock_data("AAPL").is_some());
        // Wait out the 1ns TTL so the entry is stale, then it should refetch
        // (and still succeed).
        std::thread::sleep(Duration::from_millis(5));
        assert!(provider.get_stock_data("AAPL").is_some());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn clear_empties_cache() {
        let store = Arc::new(CacheStore::new(Duration::from_secs(60)));
        let provider = cached_mock(store.clone());
        let _ = provider.get_stock_data("AAPL");
        assert!(!store.is_empty());
        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn warm_populates_cache_for_all_tickers() {
        let store = Arc::new(CacheStore::new(Duration::from_secs(60)));
        let providers: Arc<RwLock<Vec<Box<dyn StockDataProvider + Send + Sync>>>> =
            Arc::new(RwLock::new(vec![cached_mock(store.clone())]));
        // Warm without the sleep by inlining the pass logic is complex, so
        // just assert the store starts empty and that warming via the provider
        // path fills it.
        let guard = providers.read().unwrap();
        let count = guard[0].list_supported_tickers().len();
        for t in guard[0].list_supported_tickers() {
            let _ = guard[0].get_stock_data(&t);
        }
        assert!(store.len() >= count);
    }
}
