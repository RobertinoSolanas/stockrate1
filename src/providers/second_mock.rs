use crate::models::*;
use chrono::Utc;
use crate::providers::StockDataProvider;

pub struct SecondMockDataProvider;

impl SecondMockDataProvider {
    pub fn new() -> Self {
        Self
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
            "AAPL".to_string(), "MSFT".to_string(), "NVDA".to_string(), "META".to_string(), "AMD".to_string(),
            "JPM".to_string(), "V".to_string(), "JNJ".to_string(), "WMT".to_string(), "PG".to_string(),
            "UNH".to_string(), "HD".to_string(), "DIS".to_string(), "BAC".to_string(), "INTC".to_string(),
        ]
    }

    fn provider_name(&self) -> &'static str {
        "SecondMockProvider"
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

    fn jpm_data(&self) -> StockRatingData {
        let mut data = self.base_data("JPM", "JPMorgan Chase & Co.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(13.2),
            forward_pe_ratio: Some(11.8),
            ev_to_ebitda: Some(10.5),
            pb_ratio: Some(2.0),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.16),
            debt_to_equity: Some(1.15),
            free_cash_flow: Some(30500000000),
            current_ratio: Some(1.08),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.072),
            eps_growth_3y: Some(0.095),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(228.00),
            current_price: Some(218.50),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(30),
        };
        data
    }

    fn v_data(&self) -> StockRatingData {
        let mut data = self.base_data("V", "Visa Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(32.8),
            forward_pe_ratio: Some(28.2),
            ev_to_ebitda: Some(24.5),
            pb_ratio: Some(15.8),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.52),
            debt_to_equity: Some(0.62),
            free_cash_flow: Some(20800000000),
            current_ratio: Some(1.42),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.105),
            eps_growth_3y: Some(0.142),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(318.00),
            current_price: Some(298.75),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(35),
        };
        data
    }

    fn jnj_data(&self) -> StockRatingData {
        let mut data = self.base_data("JNJ", "Johnson & Johnson");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(23.8),
            forward_pe_ratio: Some(20.5),
            ev_to_ebitda: Some(15.2),
            pb_ratio: Some(6.2),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.24),
            debt_to_equity: Some(0.48),
            free_cash_flow: Some(17500000000),
            current_ratio: Some(1.22),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.042),
            eps_growth_3y: Some(0.058),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(202.00),
            current_price: Some(182.30),
            recommendation_consensus: Some(Recommendation::Hold),
            analyst_count: Some(22),
        };
        data
    }

    fn wmt_data(&self) -> StockRatingData {
        let mut data = self.base_data("WMT", "Walmart Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(30.5),
            forward_pe_ratio: Some(26.2),
            ev_to_ebitda: Some(17.8),
            pb_ratio: Some(5.8),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.22),
            debt_to_equity: Some(0.75),
            free_cash_flow: Some(11500000000),
            current_ratio: Some(0.82),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.052),
            eps_growth_3y: Some(0.088),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(175.00),
            current_price: Some(165.40),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(24),
        };
        data
    }

    fn pg_data(&self) -> StockRatingData {
        let mut data = self.base_data("PG", "Procter & Gamble Co.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(26.8),
            forward_pe_ratio: Some(24.2),
            ev_to_ebitda: Some(19.8),
            pb_ratio: Some(7.8),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.28),
            debt_to_equity: Some(0.58),
            free_cash_flow: Some(14800000000),
            current_ratio: Some(0.75),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.045),
            eps_growth_3y: Some(0.072),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(185.00),
            current_price: Some(168.90),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(20),
        };
        data
    }

    fn unh_data(&self) -> StockRatingData {
        let mut data = self.base_data("UNH", "UnitedHealth Group");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(25.2),
            forward_pe_ratio: Some(22.5),
            ev_to_ebitda: Some(18.2),
            pb_ratio: Some(4.2),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.28),
            debt_to_equity: Some(0.68),
            free_cash_flow: Some(21800000000),
            current_ratio: Some(0.82),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.112),
            eps_growth_3y: Some(0.148),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(605.00),
            current_price: Some(572.30),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(28),
        };
        data
    }

    fn hd_data(&self) -> StockRatingData {
        let mut data = self.base_data("HD", "Home Depot Inc.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(26.5),
            forward_pe_ratio: Some(23.2),
            ev_to_ebitda: Some(20.5),
            pb_ratio: Some(-48.5),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(-1.38),
            debt_to_equity: Some(-4.82),
            free_cash_flow: Some(13200000000),
            current_ratio: Some(1.15),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.048),
            eps_growth_3y: Some(0.092),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(405.00),
            current_price: Some(385.50),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(32),
        };
        data
    }

    fn dis_data(&self) -> StockRatingData {
        let mut data = self.base_data("DIS", "Walt Disney Co.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(55.2),
            forward_pe_ratio: Some(38.5),
            ev_to_ebitda: Some(30.2),
            pb_ratio: Some(2.3),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.05),
            debt_to_equity: Some(0.52),
            free_cash_flow: Some(3800000000),
            current_ratio: Some(0.98),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.062),
            eps_growth_3y: Some(0.185),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(132.00),
            current_price: Some(102.80),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(26),
        };
        data
    }

    fn bac_data(&self) -> StockRatingData {
        let mut data = self.base_data("BAC", "Bank of America Corp.");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(15.5),
            forward_pe_ratio: Some(13.2),
            ev_to_ebitda: Some(11.5),
            pb_ratio: Some(1.3),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.10),
            debt_to_equity: Some(1.32),
            free_cash_flow: Some(17200000000),
            current_ratio: Some(0.95),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(0.055),
            eps_growth_3y: Some(0.082),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(45.00),
            current_price: Some(42.30),
            recommendation_consensus: Some(Recommendation::Buy),
            analyst_count: Some(32),
        };
        data
    }

    fn intc_data(&self) -> StockRatingData {
        let mut data = self.base_data("INTC", "Intel Corporation");
        data.valuation_ratios = ValuationRatios {
            pe_ratio: Some(142.5),
            forward_pe_ratio: Some(32.8),
            ev_to_ebitda: Some(38.5),
            pb_ratio: Some(1.8),
        };
        data.financial_health = FinancialHealth {
            return_on_equity: Some(0.015),
            debt_to_equity: Some(0.42),
            free_cash_flow: Some(7500000000),
            current_ratio: Some(1.52),
        };
        data.growth_metrics = GrowthMetrics {
            revenue_growth_3y: Some(-0.025),
            eps_growth_3y: Some(-0.105),
        };
        data.market_sentiment = MarketSentiment {
            target_price_consensus: Some(35.00),
            current_price: Some(22.50),
            recommendation_consensus: Some(Recommendation::Hold),
            analyst_count: Some(38),
        };
        data
    }
}
