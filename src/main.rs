mod credentials;
mod models;
mod providers;
mod routes;
mod services;
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

    let app = routes::setup_router(providers);

    let addr = "0.0.0.0:3000";
    println!("StockRating Dashboard running at http://{}", addr);
    println!("Available providers: {}", if has_finnhub { "MockDataProvider, SecondMockProvider, FinnhubDataProvider" } else { "MockDataProvider, SecondMockProvider" });
    println!("Available tickers: AAPL, MSFT, GOOGL, TSLA, AMZN, NVDA, META, AMD");
    println!("API endpoint: http://localhost:3000/api/query?ticker=AAPL");
    println!("Comparison: http://localhost:3000/compare?ticker=AAPL");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
