mod models;
mod providers;
mod routes;

use providers::mock::MockDataProvider;
use providers::second_mock::SecondMockDataProvider;
use providers::StockDataProvider;

#[tokio::main]
async fn main() {
    let providers: Vec<Box<dyn StockDataProvider + Send + Sync>> = vec![
        Box::new(MockDataProvider::new()),
        Box::new(SecondMockDataProvider::new()),
    ];

    let app = routes::setup_router(providers);

    let addr = "0.0.0.0:3000";
    println!("StockRating Dashboard running at http://{}", addr);
    println!("Available tickers: AAPL, MSFT, GOOGL, TSLA, AMZN, NVDA, META, AMD");
    println!("API endpoint: http://localhost:3000/api/query?ticker=AAPL");
    println!("Providers: MockDataProvider, SecondMockProvider");
    println!("Comparison: http://localhost:3000/compare?ticker=AAPL");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
