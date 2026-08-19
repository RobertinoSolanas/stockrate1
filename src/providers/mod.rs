use crate::models::*;

pub trait StockDataProvider {
    fn get_stock_data(&self, ticker: &str) -> Option<StockRatingData>;
    fn list_supported_tickers(&self) -> Vec<String>;
    fn provider_name(&self) -> &'static str;

    /// Free-text symbol search (e.g. "apple" -> AAPL, "nv" -> NVDA).
    /// Default: not supported (empty list).
    fn search_symbols(&self, _query: &str) -> Vec<StockSearchResult> {
        Vec::new()
    }

    /// The full universe of available tickers, sorted alphabetically.
    /// Default: the curated supported-ticker list.
    fn list_all_tickers(&self) -> Vec<String> {
        self.list_supported_tickers()
    }
}

pub mod mock;
pub mod second_mock;
pub mod finnhub;
pub mod cache;
