use crate::providers::finnhub::FinnhubDataProvider;
use crate::providers::StockDataProvider;

fn finnhub_api_key() -> String {
    let cred_path = "resources/credentials.txt";
    let content = std::fs::read_to_string(cred_path).unwrap_or_default();
    for line in content.lines() {
        if let Some(key) = line.strip_prefix("FINNHUB_API_KEY=") {
            let key = key.trim();
            if !key.is_empty() {
                return key.to_string();
            }
        }
    }
    panic!("FINNHUB_API_KEY not found in {}", cred_path);
}

#[test]
fn test_finnhub_provider_new() {
    let _provider = FinnhubDataProvider::new(finnhub_api_key());
}

#[test]
fn test_finnhub_provider_name() {
    let provider = FinnhubDataProvider::new(finnhub_api_key());
    assert_eq!(provider.provider_name(), "FinnhubDataProvider");
}

#[test]
fn test_finnhub_list_supported_tickers() {
    let provider = FinnhubDataProvider::new(finnhub_api_key());
    let tickers = provider.list_supported_tickers();
    assert!(tickers.len() >= 5);
    assert!(tickers.contains(&"AAPL".to_string()));
    assert!(tickers.contains(&"MSFT".to_string()));
    assert!(tickers.contains(&"GOOGL".to_string()));
    assert!(tickers.contains(&"TSLA".to_string()));
    assert!(tickers.contains(&"AMZN".to_string()));
}

#[ignore]
#[test]
fn test_finnhub_returns_aapl_data() {
    println!("Fetching AAPL data from Finnhub...");
    let provider = FinnhubDataProvider::new(finnhub_api_key());
    let data = provider.get_stock_data("AAPL");
    if data.is_none() {
        println!("AAPL: No data returned");
        panic!("Expected data for AAPL");
    }
    let data = data.unwrap();
    println!("AAPL: ticker={}, company={}, provider={}", data.ticker, data.company_name, data.provider);
    println!("AAPL: valuation_ratios={:?}", data.valuation_ratios);
    println!("AAPL: financial_health={:?}", data.financial_health);
    println!("AAPL: growth_metrics={:?}", data.growth_metrics);
    println!("AAPL: market_sentiment={:?}", data.market_sentiment);
    assert_eq!(data.ticker, "AAPL");
    assert!(!data.company_name.is_empty());
    assert_eq!(data.provider, "FinnhubDataProvider");
}

#[ignore]
#[test]
fn test_finnhub_returns_msft_data() {
    println!("Fetching MSFT data from Finnhub...");
    let provider = FinnhubDataProvider::new(finnhub_api_key());
    let data = provider.get_stock_data("MSFT");
    if data.is_none() {
        println!("MSFT: No data returned");
        panic!("Expected data for MSFT");
    }
    let data = data.unwrap();
    println!("MSFT: ticker={}, company={}, provider={}", data.ticker, data.company_name, data.provider);
    println!("MSFT: valuation_ratios={:?}", data.valuation_ratios);
    println!("MSFT: financial_health={:?}", data.financial_health);
    println!("MSFT: growth_metrics={:?}", data.growth_metrics);
    println!("MSFT: market_sentiment={:?}", data.market_sentiment);
    assert_eq!(data.ticker, "MSFT");
    assert!(!data.company_name.is_empty());
}

#[ignore]
#[test]
fn test_finnhub_returns_googl_data() {
    println!("Fetching GOOGL data from Finnhub...");
    let provider = FinnhubDataProvider::new(finnhub_api_key());
    let data = provider.get_stock_data("GOOGL");
    if data.is_none() {
        println!("GOOGL: No data returned");
        panic!("Expected data for GOOGL");
    }
    let data = data.unwrap();
    println!("GOOGL: ticker={}, company={}, provider={}", data.ticker, data.company_name, data.provider);
    println!("GOOGL: valuation_ratios={:?}", data.valuation_ratios);
    println!("GOOGL: financial_health={:?}", data.financial_health);
    println!("GOOGL: growth_metrics={:?}", data.growth_metrics);
    println!("GOOGL: market_sentiment={:?}", data.market_sentiment);
    assert_eq!(data.ticker, "GOOGL");
    assert!(!data.company_name.is_empty());
}

#[ignore]
#[test]
fn test_finnhub_returns_nvda_data() {
    println!("Fetching NVDA data from Finnhub...");
    let provider = FinnhubDataProvider::new(finnhub_api_key());
    let data = provider.get_stock_data("NVDA");
    if data.is_none() {
        println!("NVDA: No data returned");
        panic!("Expected data for NVDA");
    }
    let data = data.unwrap();
    println!("NVDA: ticker={}, company={}, provider={}", data.ticker, data.company_name, data.provider);
    println!("NVDA: valuation_ratios={:?}", data.valuation_ratios);
    println!("NVDA: financial_health={:?}", data.financial_health);
    println!("NVDA: growth_metrics={:?}", data.growth_metrics);
    println!("NVDA: market_sentiment={:?}", data.market_sentiment);
    assert_eq!(data.ticker, "NVDA");
    assert!(!data.company_name.is_empty());
}

#[ignore]
#[test]
fn test_finnhub_returns_none_for_unknown() {
    println!("Fetching unknown ticker from Finnhub...");
    let provider = FinnhubDataProvider::new(finnhub_api_key());
    let data = provider.get_stock_data("NONEXISTENT_TICKER_12345");
    if let Some(ref d) = data {
        println!("Unknown ticker returned data: ticker={}, company={}", d.ticker, d.company_name);
    } else {
        println!("Unknown ticker: No data returned (expected)");
    }
    assert!(data.is_none());
}

#[ignore]
#[test]
fn test_finnhub_aapl_has_valuation_ratios() {
    println!("Fetching AAPL valuation ratios from Finnhub...");
    let provider = FinnhubDataProvider::new(finnhub_api_key());
    let data = provider.get_stock_data("AAPL").unwrap();
    println!("AAPL: pe_ratio={:?}, forward_pe_ratio={:?}, ev_to_ebitda={:?}, pb_ratio={:?}",
        data.valuation_ratios.pe_ratio, data.valuation_ratios.forward_pe_ratio,
        data.valuation_ratios.ev_to_ebitda, data.valuation_ratios.pb_ratio);
    assert!(data.valuation_ratios.pe_ratio.is_some() || data.valuation_ratios.pb_ratio.is_some());
}

#[ignore]
#[test]
fn test_finnhub_aapl_has_market_sentiment() {
    println!("Fetching AAPL market sentiment from Finnhub...");
    let provider = FinnhubDataProvider::new(finnhub_api_key());
    let data = provider.get_stock_data("AAPL").unwrap();
    println!("AAPL: current_price={:?}, target_price_consensus={:?}, recommendation_consensus={:?}, analyst_count={:?}",
        data.market_sentiment.current_price, data.market_sentiment.target_price_consensus,
        data.market_sentiment.recommendation_consensus, data.market_sentiment.analyst_count);
    assert!(data.market_sentiment.current_price.is_some());
    assert!(data.market_sentiment.current_price.unwrap() > 0.0);
}
