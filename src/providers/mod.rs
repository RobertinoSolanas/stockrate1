use crate::models::*;

pub trait StockDataProvider {
    fn get_stock_data(&self, ticker: &str) -> Option<StockRatingData>;
    fn list_supported_tickers(&self) -> Vec<String>;
}

pub mod mock;
