use crate::models::*;
use chrono::Utc;
use crate::providers::StockDataProvider;

pub struct MockDataProvider;

impl MockDataProvider {
    pub fn new() -> Self {
        Self
    }
}

impl StockDataProvider for MockDataProvider {
    fn get_stock_data(&self, ticker: &str) -> Option<StockRatingData> {
        match ticker.to_uppercase().as_str() {
            "AAPL" => Some(self.aapl_data()),
            "MSFT" => Some(self.msft_data()),
            "GOOGL" => Some(self.googl_data()),
            "TSLA" => Some(self.tsla_data()),
            "AMZN" => Some(self.amzn_data()),
            _ => None,
        }
    }

    fn list_supported_tickers(&self) -> Vec<String> {
        vec!["AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string(), "TSLA".to_string(), "AMZN".to_string()]
    }

    fn provider_name(&self) -> &'static str {
        "MockDataProvider"
    }
}

impl MockDataProvider {
    fn base_data(&self, ticker: &str, company_name: &str) -> StockRatingData {
        StockRatingData {
            ticker: ticker.to_string(),
            company_name: company_name.to_string(),
            provider: "MockDataProvider".to_string(),
            last_updated: Some(Utc::now()),
            valuation_ratios: ValuationRatios::default(),
            financial_health: FinancialHealth::default(),
            growth_metrics: GrowthMetrics::default(),
            market_sentiment: MarketSentiment::default(),
        }
    }

    fn aapl_data(&self) -> StockRatingData {
        let mut data = self.base_data("AAPL", "Apple Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(28.5),
            forward_pe_ratio: Some(25.2),
            ev_to_ebitda: Some(18.3),
            pb_ratio: Some(35.1),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(1.54),
            debt_to_equity: Some(1.45),
            free_cash_flow: Some(97500000000),
            current_ratio: Some(0.98),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.085),
            eps_growth_3y: Some(0.112),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(210.50),
            current_price: Some(185.25),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(42),
        };
        data
    }

    fn msft_data(&self) -> StockRatingData {
        let mut data = self.base_data("MSFT", "Microsoft Corporation");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(35.2),
            forward_pe_ratio: Some(30.1),
            ev_to_ebitda: Some(22.5),
            pb_ratio: Some(12.8),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.42),
            debt_to_equity: Some(0.35),
            free_cash_flow: Some(65000000000),
            current_ratio: Some(1.75),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.125),
            eps_growth_3y: Some(0.158),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(520.00),
            current_price: Some(445.80),
            recommendation_consensus: Some(Recommendation::StrongBuy),
            analyst_count: Some(48),
        };
        data
    }

    fn googl_data(&self) -> StockRatingData {
        let mut data = self.base_data("GOOGL", "Alphabet Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(24.8),
            forward_pe_ratio: Some(21.3),
            ev_to_ebitda: Some(14.2),
            pb_ratio: Some(6.5),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.28),
            debt_to_equity: Some(0.12),
            free_cash_flow: Some(73000000000),
            current_ratio: Some(2.15),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.095),
            eps_growth_3y: Some(0.125),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(215.00),
            current_price: Some(178.50),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(52),
        };
        data
    }

    fn tsla_data(&self) -> StockRatingData {
        let mut data = self.base_data("TSLA", "Tesla Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(85.3),
            forward_pe_ratio: Some(65.0),
            ev_to_ebitda: Some(45.2),
            pb_ratio: Some(14.5),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.25),
            debt_to_equity: Some(0.08),
            free_cash_flow: Some(7500000000),
            current_ratio: Some(1.85),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.350),
            eps_growth_3y: Some(0.420),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(280.00),
            current_price: Some(245.60),
            recommendation_consensus: Some(Recommendation::Hold),
            analyst_count: Some(35),
        };
        data
    }

    fn amzn_data(&self) -> StockRatingData {
        let mut data = self.base_data("AMZN", "Amazon.com Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(52.5),
            forward_pe_ratio: Some(42.0),
            ev_to_ebitda: Some(28.3),
            pb_ratio: Some(8.2),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.18),
            debt_to_equity: Some(0.52),
            free_cash_flow: Some(35000000000),
            current_ratio: Some(1.05),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.145),
            eps_growth_3y: Some(0.285),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(245.00),
            current_price: Some(205.30),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(55),
        };
        data
    }
}
