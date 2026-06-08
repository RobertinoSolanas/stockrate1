mod models;
mod providers;
mod routes;
mod tests;

use providers::mock::MockDataProvider;
use providers::second_mock::SecondMockDataProvider;
use providers::finnhub::FinnhubDataProvider;
use providers::StockDataProvider;

#[tokio::main]
async fn main() {
    let mut providers: Vec<Box<dyn StockDataProvider + Send + Sync>> = vec![
        Box::new(MockDataProvider::new()),
        Box::new(SecondMockDataProvider::new()),
    ];

    // Add Finnhub provider if API key is set
    if let Ok(api_key) = std::env::var("FINNHUB_API_KEY") {
        if !api_key.is_empty() {
            providers.push(Box::new(FinnhubDataProvider::new(api_key)));
        }
    }

    let has_finnhub = providers.len() > 2;

    let app = routes::setup_router(providers);

    let addr = "0.0.0.0:3000";
    println!("StockRating Dashboard running at http://{}", addr);
    println!("Available providers: {}", if has_finnhub { "MockDataProvider, SecondMockProvider, FinnhubDataProvider" } else { "MockDataProvider, SecondMockProvider" });
    println!("Available tickers: AAPL, MSFT, GOOGL, TSLA, AMZN, NVDA, META, AMD");
    println!("API endpoint: http://localhost:3000/api/query?ticker=AAPL");
    println!("Comparison: http://localhost:3000/compare?ticker=AAPL");
    println!("Finnhub: set FINNHUB_API_KEY env var to enable live data");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
