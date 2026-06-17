use crate::models::*;
use chrono::Utc;
use crate::providers::StockDataProvider;

pub struct SecondMockDataProvider;

impl SecondMockDataProvider {
    pub fn new() -> Self {
        Self
    }
    
    #[allow(dead_code)]
    pub fn provider_name(&self) -> &'static str {
        "SecondMockProvider"
    }
}

impl StockDataProvider for SecondMockDataProvider {
    fn get_stock_data(&self, ticker: &str) -> Option<StockRatingData> {
        match ticker.to_uppercase().as_str() {
            "AAPL" => Some(self.aapl_data()),
            "MSFT" => Some(self.msft_data()),
            "NVDA" => Some(self.nvda_data()),
            "META" => Some(self.meta_data()),
            "AMD" => Some(self.amd_data()),
            _ => None,
        }
    }

    fn list_supported_tickers(&self) -> Vec<String> {
        vec!["AAPL".to_string(), "MSFT".to_string(), "NVDA".to_string(), "META".to_string(), "AMD".to_string()]
    }

    fn provider_name(&self) -> &'static str {
        "SecondMockDataProvider"
    }
}

impl SecondMockDataProvider {
    fn base_data(&self, ticker: &str, company_name: &str) -> StockRatingData {
        StockRatingData {
            ticker: ticker.to_string(),
            company_name: company_name.to_string(),
            provider: "SecondMockProvider".to_string(),
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
            pe_ratio: Some(31.2),
            forward_pe_ratio: Some(27.8),
            ev_to_ebitda: Some(20.1),
            pb_ratio: Some(42.5),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(1.62),
            debt_to_equity: Some(1.78),
            free_cash_flow: Some(101200000000),
            current_ratio: Some(0.95),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.062),
            eps_growth_3y: Some(0.095),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(195.00),
            current_price: Some(185.25),
            recommendation_consensus: Some(Recommendation::Hold),
            analyst_count: Some(38),
        };
        data
    }

    fn msft_data(&self) -> StockRatingData {
        let mut data = self.base_data("MSFT", "Microsoft Corporation");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(38.5),
            forward_pe_ratio: Some(33.2),
            ev_to_ebitda: Some(25.8),
            pb_ratio: Some(14.2),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.38),
            debt_to_equity: Some(0.42),
            free_cash_flow: Some(58000000000),
            current_ratio: Some(1.62),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.098),
            eps_growth_3y: Some(0.135),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(480.00),
            current_price: Some(445.80),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(44),
        };
        data
    }

    fn nvda_data(&self) -> StockRatingData {
        let mut data = self.base_data("NVDA", "NVIDIA Corporation");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(65.8),
            forward_pe_ratio: Some(48.5),
            ev_to_ebitda: Some(42.3),
            pb_ratio: Some(28.5),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.85),
            debt_to_equity: Some(0.22),
            free_cash_flow: Some(28500000000),
            current_ratio: Some(4.15),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.520),
            eps_growth_3y: Some(0.680),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(165.00),
            current_price: Some(138.50),
            recommendation_consensus: Some(Recommendation::StrongBuy),
            analyst_count: Some(58),
        };
        data
    }

    fn meta_data(&self) -> StockRatingData {
        let mut data = self.base_data("META", "Meta Platforms Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(26.5),
            forward_pe_ratio: Some(22.8),
            ev_to_ebitda: Some(16.8),
            pb_ratio: Some(7.8),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.32),
            debt_to_equity: Some(0.05),
            free_cash_flow: Some(42000000000),
            current_ratio: Some(2.85),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.145),
            eps_growth_3y: Some(0.210),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(620.00),
            current_price: Some(548.00),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(52),
        };
        data
    }

    fn amd_data(&self) -> StockRatingData {
        let mut data = self.base_data("AMD", "Advanced Micro Devices Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(58.2),
            forward_pe_ratio: Some(42.5),
            ev_to_ebitda: Some(35.8),
            pb_ratio: Some(4.2),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.08),
            debt_to_equity: Some(0.04),
            free_cash_flow: Some(1800000000),
            current_ratio: Some(2.42),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.185),
            eps_growth_3y: Some(0.245),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(210.00),
            current_price: Some(178.50),
            recommendation_consensus: Some(Recommendation::Hold),
            analyst_count: Some(42),
        };
        data
    }
}
