use crate::models::*;

pub trait StockDataProvider {
    fn get_stock_data(&self, ticker: &str) -> Option<StockRatingData>;
    fn list_supported_tickers(&self) -> Vec<String>;
    fn provider_name(&self) -> &'static str;
}

pub mod mock;
pub mod second_mock;
