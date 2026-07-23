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
            "JPM" => Some(self.jpm_data()),
            "V" => Some(self.v_data()),
            "JNJ" => Some(self.jnj_data()),
            "WMT" => Some(self.wmt_data()),
            "PG" => Some(self.pg_data()),
            "UNH" => Some(self.unh_data()),
            "HD" => Some(self.hd_data()),
            "DIS" => Some(self.dis_data()),
            "BAC" => Some(self.bac_data()),
            "INTC" => Some(self.intc_data()),
            _ => None,
        }
    }

    fn list_supported_tickers(&self) -> Vec<String> {
        vec![
            "AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string(), "TSLA".to_string(), "AMZN".to_string(),
            "JPM".to_string(), "V".to_string(), "JNJ".to_string(), "WMT".to_string(), "PG".to_string(),
            "UNH".to_string(), "HD".to_string(), "DIS".to_string(), "BAC".to_string(), "INTC".to_string(),
        ]
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

    fn jpm_data(&self) -> StockRatingData {
        let mut data = self.base_data("JPM", "JPMorgan Chase & Co.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(12.5),
            forward_pe_ratio: Some(11.2),
            ev_to_ebitda: Some(9.8),
            pb_ratio: Some(1.8),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.15),
            debt_to_equity: Some(1.10),
            free_cash_flow: Some(32000000000),
            current_ratio: Some(1.05),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.065),
            eps_growth_3y: Some(0.088),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(235.00),
            current_price: Some(218.50),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(28),
        };
        data
    }

    fn v_data(&self) -> StockRatingData {
        let mut data = self.base_data("V", "Visa Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(30.2),
            forward_pe_ratio: Some(26.5),
            ev_to_ebitda: Some(22.1),
            pb_ratio: Some(14.5),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.48),
            debt_to_equity: Some(0.55),
            free_cash_flow: Some(19500000000),
            current_ratio: Some(1.38),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.092),
            eps_growth_3y: Some(0.125),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(330.00),
            current_price: Some(298.75),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(32),
        };
        data
    }

    fn jnj_data(&self) -> StockRatingData {
        let mut data = self.base_data("JNJ", "Johnson & Johnson");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(22.1),
            forward_pe_ratio: Some(19.8),
            ev_to_ebitda: Some(14.5),
            pb_ratio: Some(5.8),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.22),
            debt_to_equity: Some(0.42),
            free_cash_flow: Some(18200000000),
            current_ratio: Some(1.18),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.035),
            eps_growth_3y: Some(0.052),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(195.00),
            current_price: Some(182.30),
            recommendation_consensus: Some(Recommendation::Hold),
            analyst_count: Some(25),
        };
        data
    }

    fn wmt_data(&self) -> StockRatingData {
        let mut data = self.base_data("WMT", "Walmart Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(28.8),
            forward_pe_ratio: Some(24.5),
            ev_to_ebitda: Some(16.2),
            pb_ratio: Some(5.2),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.18),
            debt_to_equity: Some(0.68),
            free_cash_flow: Some(12800000000),
            current_ratio: Some(0.85),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.045),
            eps_growth_3y: Some(0.078),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(180.00),
            current_price: Some(165.40),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(26),
        };
        data
    }

    fn pg_data(&self) -> StockRatingData {
        let mut data = self.base_data("PG", "Procter & Gamble Co.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(25.3),
            forward_pe_ratio: Some(22.8),
            ev_to_ebitda: Some(18.5),
            pb_ratio: Some(7.2),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.26),
            debt_to_equity: Some(0.52),
            free_cash_flow: Some(15200000000),
            current_ratio: Some(0.72),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.038),
            eps_growth_3y: Some(0.065),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(178.00),
            current_price: Some(168.90),
            recommendation_consensus: Some(Recommendation::Hold),
            analyst_count: Some(22),
        };
        data
    }

    fn unh_data(&self) -> StockRatingData {
        let mut data = self.base_data("UNH", "UnitedHealth Group");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(23.5),
            forward_pe_ratio: Some(20.8),
            ev_to_ebitda: Some(16.8),
            pb_ratio: Some(3.8),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.25),
            debt_to_equity: Some(0.62),
            free_cash_flow: Some(22500000000),
            current_ratio: Some(0.78),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.105),
            eps_growth_3y: Some(0.135),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(620.00),
            current_price: Some(572.30),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(30),
        };
        data
    }

    fn hd_data(&self) -> StockRatingData {
        let mut data = self.base_data("HD", "Home Depot Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(24.2),
            forward_pe_ratio: Some(21.5),
            ev_to_ebitda: Some(18.8),
            pb_ratio: Some(-45.2),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(-1.25),
            debt_to_equity: Some(-4.58),
            free_cash_flow: Some(14800000000),
            current_ratio: Some(1.12),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.042),
            eps_growth_3y: Some(0.085),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(420.00),
            current_price: Some(385.50),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(34),
        };
        data
    }

    fn dis_data(&self) -> StockRatingData {
        let mut data = self.base_data("DIS", "Walt Disney Co.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(52.8),
            forward_pe_ratio: Some(35.2),
            ev_to_ebitda: Some(28.5),
            pb_ratio: Some(2.1),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.04),
            debt_to_equity: Some(0.48),
            free_cash_flow: Some(4200000000),
            current_ratio: Some(0.95),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.055),
            eps_growth_3y: Some(0.165),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(125.00),
            current_price: Some(102.80),
            recommendation_consensus: Some(Recommendation::Hold),
            analyst_count: Some(28),
        };
        data
    }

    fn bac_data(&self) -> StockRatingData {
        let mut data = self.base_data("BAC", "Bank of America Corp.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(14.8),
            forward_pe_ratio: Some(12.5),
            ev_to_ebitda: Some(10.2),
            pb_ratio: Some(1.2),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.09),
            debt_to_equity: Some(1.25),
            free_cash_flow: Some(18500000000),
            current_ratio: Some(0.92),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.048),
            eps_growth_3y: Some(0.072),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(48.00),
            current_price: Some(42.30),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(30),
        };
        data
    }

    fn intc_data(&self) -> StockRatingData {
        let mut data = self.base_data("INTC", "Intel Corporation");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(128.5),
            forward_pe_ratio: Some(28.5),
            ev_to_ebitda: Some(32.8),
            pb_ratio: Some(1.5),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.02),
            debt_to_equity: Some(0.38),
            free_cash_flow: Some(8200000000),
            current_ratio: Some(1.48),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(-0.015),
            eps_growth_3y: Some(-0.085),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(38.00),
            current_price: Some(22.50),
            recommendation_consensus: Some(Recommendation::Hold),
            analyst_count: Some(36),
        };
        data
    }
}
