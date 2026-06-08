use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockRatingData {
    pub ticker: String,
    pub company_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<DateTime<Utc>>,
    pub valuation_ratios: ValuationRatios,
    pub financial_health: FinancialHealth,
    pub growth_metrics: GrowthMetrics,
    pub market_sentiment: MarketSentiment,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ValuationRatios {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pe_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_pe_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ev_to_ebitda: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pb_ratio: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FinancialHealth {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_on_equity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debt_to_equity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_cash_flow: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_ratio: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GrowthMetrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_growth_3y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eps_growth_3y: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Recommendation {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarketSentiment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_price_consensus: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation_consensus: Option<Recommendation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analyst_count: Option<i32>,
}
