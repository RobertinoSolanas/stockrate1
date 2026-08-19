mod credentials;
mod models;
mod providers;
mod routes;
mod services;
#[cfg(test)]
mod tests;

use clap::Parser;
use providers::mock::MockDataProvider;
use providers::second_mock::SecondMockDataProvider;
use providers::finnhub::FinnhubDataProvider;
use providers::StockDataProvider;
use providers::cache::{
    spawn_background_cache_warmer, CacheStore, CachedProvider, DEFAULT_CACHE_TTL,
    DEFAULT_REFRESH_INTERVAL,
};
use std::sync::{Arc, RwLock};

#[derive(Parser)]
#[command(name = "stockrate", about = "StockRating Dashboard Server")]
struct Cli {
    /// Suppress startup info messages
    #[arg(long)]
    release: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut providers: Vec<Box<dyn StockDataProvider + Send + Sync>> = vec![
        Box::new(MockDataProvider::new()),
        Box::new(SecondMockDataProvider::new()),
    ];

    let cred_file = "resources/credentials.txt";
    let creds = credentials::Credentials::load_from_file(cred_file);

    let finnhub_key = std::env::var("FINNHUB_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .or_else(|| creds.get("FINNHUB_API_KEY"));

    match &finnhub_key {
        Some(key) if !key.is_empty() => {
            providers.push(Box::new(FinnhubDataProvider::new(key.clone())));
        }
        Some(_) => {
            println!("WARNING: FINNHUB_API_KEY is empty in environment or credentials file ({})", cred_file);
        }
        None => {
            println!("INFO: No FINNHUB_API_KEY found. Skipped FinnhubDataProvider.");
            println!("INFO: Set FINNHUB_API_KEY env var or create {} with 'FINNHUB_API_KEY=your_key'", cred_file);
        }
    }

    let has_finnhub = providers.len() > 2;

    // Wrap every provider with a transparent TTL cache, then share the provider
    // list between the router and a background thread that keeps the cache
    // populated while the app runs.
    let store = Arc::new(CacheStore::new(DEFAULT_CACHE_TTL));
    let cached: Vec<Box<dyn StockDataProvider + Send + Sync>> = providers
        .into_iter()
        .map(|p| -> Box<dyn StockDataProvider + Send + Sync> {
            Box::new(CachedProvider::new(p, store.clone()))
        })
        .collect();
    let providers: Arc<RwLock<Vec<Box<dyn StockDataProvider + Send + Sync>>>> =
        Arc::new(RwLock::new(cached));

    // Pre-warm the cache in the background, then refresh on an interval.
    spawn_background_cache_warmer(providers.clone(), DEFAULT_REFRESH_INTERVAL);

    let app = routes::setup_router(providers);

    let addr = "0.0.0.0:3000";
    if !cli.release {
        println!("StockRating Dashboard running at http://{}", addr);
        println!("Available providers: {}", if has_finnhub { "MockDataProvider, SecondMockProvider, FinnhubDataProvider" } else { "MockDataProvider, SecondMockProvider" });
        println!("Available tickers: AAPL, MSFT, GOOGL, TSLA, AMZN, NVDA, META, AMD");
        println!("API endpoint: http://localhost:3000/api/query?ticker=AAPL");
        println!("Comparison: http://localhost:3000/compare?ticker=AAPL");
        println!(
            "Caching: enabled (TTL {:?}), pre-warming all providers in the background",
            DEFAULT_CACHE_TTL
        );
    }

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
