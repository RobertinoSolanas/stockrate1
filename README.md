# StockRating
A Rust web application that aggregates and visualizes stock ratings from multiple data sources via API and provides a dashboard to show all stocks from different sources.
Architecture: Axum web server + typed Rust models matching stock_data_generic_api.yaml OpenAPI spec + trait-based provider plugin system.
WARNING: i will never ever store keys into git or github i always make a reference to git ignored files like /resources/credentials.txt or by user interactions
Providers implement `StockDataProvider` trait (mock, yahoo, alphavantage, etc.) enabling zero-downtime source swapping.
Data flows: provider → serde-deserialized models → routes render HTML/JSON responses.
Run: `cargo run` then open http://localhost:3000 (dashboard) or http://localhost:3000/api/query?ticker=AAPL (JSON API).
Ends: `GET /` index, `GET /dashboard?ticker=XYZ` HTML dashboard, `GET /api/query?ticker=XYZ` JSON endpoint.
Models in `src/models.rs` mirror the YAML schema: StockRatingData, ValuationRatios, FinancialHealth, GrowthMetrics, MarketSentiment.
MockDataProvider ships with AAPL/MSFT/GOOGL/TSLA/AMZN sample data for offline development.
Dependencies: axum 0.7, serde, tokio, chrono — all async with RwLock for thread-safe state.
