mod models;
mod providers;
mod routes;

use providers::mock::MockDataProvider;
use providers::StockDataProvider;

#[tokio::main]
async fn main() {
    let data_provider: Box<dyn StockDataProvider + Send + Sync> = Box::new(MockDataProvider::new());

    let app = routes::setup_router(data_provider);

    let addr = "0.0.0.0:3000";
    println!("StockRating Dashboard running at http://{}", addr);
    println!("Available tickers: AAPL, MSFT, GOOGL, TSLA, AMZN");
    println!("API endpoint: http://localhost:3000/api/stocks/{{ticker}}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
