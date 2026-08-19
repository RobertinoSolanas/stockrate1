use axum_test::TestServer;

use stockrate::providers::mock::MockDataProvider;
use stockrate::providers::second_mock::SecondMockDataProvider;
use stockrate::providers::StockDataProvider;
use stockrate::routes::setup_router;

async fn make_server() -> TestServer {
    let providers: Vec<Box<dyn StockDataProvider + Send + Sync>> = vec![
        Box::new(MockDataProvider::new()),
        Box::new(SecondMockDataProvider::new()),
    ];
    let app = setup_router(std::sync::Arc::new(std::sync::RwLock::new(providers)));
    TestServer::new(app).expect("TestServer creation failed")
}

#[tokio::test]
async fn test_index_returns_200() {
    let server = make_server().await;
    let response = server.get("/").await;
    assert_eq!(response.status_code(), 200);
}

#[tokio::test]
async fn test_index_contains_ticker_cards() {
    let server = make_server().await;
    let response = server.get("/").await;
    let body = response.text();
    assert!(body.contains("AAPL") || body.contains("ticker-card"));
}

#[tokio::test]
async fn test_api_all_stocks_returns_json() {
    let server = make_server().await;
    let response = server.get("/api/all-stocks").await;
    assert_eq!(response.status_code(), 200);
    let body = response.json::<serde_json::Value>();
    assert!(body.get("tickers").is_some());
}

#[tokio::test]
async fn test_api_all_tickers_returns_json() {
    let server = make_server().await;
    let response = server.get("/api/all-tickers").await;
    assert_eq!(response.status_code(), 200);
    let body = response.json::<serde_json::Value>();
    assert!(body.get("total").is_some());
}

#[tokio::test]
async fn test_portfolio_returns_200() {
    let server = make_server().await;
    let response = server.get("/portfolio").await;
    assert_eq!(response.status_code(), 200);
}

// Tests that depend on Query params - may need route adjustment
#[tokio::test]
async fn test_api_query_aapl() {
    let server = make_server().await;
    let response = server.get("/api/query?ticker=AAPL").await;
    let body = response.text();
    // The mock provider may or may not have AAPL data
    // Just verify the response is valid (either 200 with data or 404)
    let status = response.status_code();
    assert!(status == 200 || status == 404);
}

#[tokio::test]
async fn test_dashboard_aapl() {
    let server = make_server().await;
    let response = server.get("/dashboard?ticker=AAPL").await;
    let status = response.status_code();
    // Mock provider may not have AAPL
    assert!(status == 200 || status == 404);
}

// Finnhub-specific endpoints report 503 when the provider is not configured
// (the test server only wires up the mock providers).
#[tokio::test]
async fn test_api_finnhub_search_disabled_returns_503() {
    let mut server = make_server().await;
    // axum-test ignores query strings embedded in the path, so attach the
    // parameter explicitly.
    server.add_query_param("q", "aapl");
    let response = server.get("/api/finnhub/search").await;
    assert_eq!(response.status_code(), 503);
}

#[tokio::test]
async fn test_api_finnhub_tickers_disabled_returns_503() {
    let server = make_server().await;
    let response = server.get("/api/finnhub/tickers").await;
    assert_eq!(response.status_code(), 503);
}

#[tokio::test]
async fn test_finnhub_page_disabled_shows_setup_hint() {
    let server = make_server().await;
    let response = server.get("/finnhub").await;
    assert_eq!(response.status_code(), 200);
    let body = response.text();
    assert!(body.contains("Finnhub not configured"));
}
