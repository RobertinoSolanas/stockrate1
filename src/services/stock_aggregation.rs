use crate::models::*;
use crate::providers::StockDataProvider;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub metrics: Vec<MetricType>,
    pub provider_filter: Vec<String>,
    pub chart_type: ChartType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricType {
    PE,
    ForwardPE,
    EVToEBITDA,
    PB,
    ROE,
    DebtToEquity,
    CurrentRatio,
    RevenueGrowth3Y,
    EPSGrowth3Y,
    Upside,
    TargetPrice,
    CurrentPrice,
    AnalystCount,
    FCF,
}

impl MetricType {
    pub fn label(&self) -> &'static str {
        match self {
            MetricType::PE => "P/E Ratio",
            MetricType::ForwardPE => "Forward P/E",
            MetricType::EVToEBITDA => "EV/EBITDA",
            MetricType::PB => "P/B Ratio",
            MetricType::ROE => "ROE",
            MetricType::DebtToEquity => "Debt/Equity",
            MetricType::CurrentRatio => "Current Ratio",
            MetricType::RevenueGrowth3Y => "Revenue Growth 3Y",
            MetricType::EPSGrowth3Y => "EPS Growth 3Y",
            MetricType::Upside => "Upside %",
            MetricType::TargetPrice => "Target Price",
            MetricType::CurrentPrice => "Current Price",
            MetricType::AnalystCount => "Analyst Count",
            MetricType::FCF => "Free Cash Flow",
        }
    }

    pub fn unit(&self) -> &'static str {
        match self {
            MetricType::ROE | MetricType::RevenueGrowth3Y | MetricType::EPSGrowth3Y | MetricType::Upside | MetricType::DebtToEquity => "%",
            MetricType::FCF => "$",
            MetricType::TargetPrice | MetricType::CurrentPrice => "$",
            _ => "",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            MetricType::PE | MetricType::ForwardPE | MetricType::EVToEBITDA | MetricType::PB => "#3b82f6",
            MetricType::ROE => "#10b981",
            MetricType::DebtToEquity => "#f59e0b",
            MetricType::CurrentRatio => "#3b82f6",
            MetricType::RevenueGrowth3Y | MetricType::EPSGrowth3Y => "#22c55e",
            MetricType::Upside => "#8b5cf6",
            MetricType::TargetPrice => "#06b6d4",
            MetricType::CurrentPrice => "#64748b",
            MetricType::AnalystCount => "#ec4899",
            MetricType::FCF => "#14b8a6",
        }
    }

    pub fn from_str(s: &str) -> Option<MetricType> {
        match s.to_lowercase().as_str() {
            "pe" | "pe_ratio" => Some(MetricType::PE),
            "forwardpe" | "forward_pe" | "forwardpe_ratio" => Some(MetricType::ForwardPE),
            "ev" | "ev_to_ebitda" | "evebitda" => Some(MetricType::EVToEBITDA),
            "pb" | "pb_ratio" => Some(MetricType::PB),
            "roe" | "return_on_equity" => Some(MetricType::ROE),
            "debt" | "debt_to_equity" | "d2e" => Some(MetricType::DebtToEquity),
            "current" | "current_ratio" => Some(MetricType::CurrentRatio),
            "revenue" | "revenue_growth" | "rev_growth" => Some(MetricType::RevenueGrowth3Y),
            "eps" | "eps_growth" => Some(MetricType::EPSGrowth3Y),
            "upside" | "upside_pct" => Some(MetricType::Upside),
            "target" | "target_price" => Some(MetricType::TargetPrice),
            "price" | "current_price" => Some(MetricType::CurrentPrice),
            "analysts" | "analyst_count" => Some(MetricType::AnalystCount),
            "fcf" | "free_cash_flow" => Some(MetricType::FCF),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartType {
    Bar,
    HorizontalBar,
    Scatter,
}

impl ChartType {
    pub fn from_str(s: &str) -> Option<ChartType> {
        match s.to_lowercase().as_str() {
            "bar" => Some(ChartType::Bar),
            "horizontal" | "horizontalbar" | "hbar" => Some(ChartType::HorizontalBar),
            "scatter" => Some(ChartType::Scatter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StockChartData {
    pub ticker: String,
    pub company_name: String,
    pub provider: String,
    pub metrics: Vec<MetricValue>,
}

#[derive(Debug, Clone)]
pub struct MetricValue {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct ChartGroup {
    pub metric: MetricType,
    pub label: String,
    pub unit: String,
    pub color: String,
    pub entries: Vec<ChartEntry>,
}

#[derive(Debug, Clone)]
pub struct ChartEntry {
    pub ticker: String,
    pub company_name: String,
    pub provider: String,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct AggregatedStockData {
    pub tickers: Vec<String>,
    pub providers: Vec<String>,
    pub chart_groups: Vec<ChartGroup>,
    pub raw_data: Vec<StockChartData>,
}

#[derive(Debug, Clone)]
pub struct TickerInfo {
    pub ticker: String,
    pub providers: Vec<String>,
}

pub struct StockAggregationService {
    pub(crate) providers: std::sync::Arc<std::sync::RwLock<Vec<Box<dyn StockDataProvider + Send + Sync>>>>,
}

impl StockAggregationService {
    pub fn new(providers: std::sync::Arc<std::sync::RwLock<Vec<Box<dyn StockDataProvider + Send + Sync>>>>) -> Self {
        Self { providers }
    }

    pub fn get_aggregated_data(&self, config: &ChartConfig) -> AggregatedStockData {
        let providers = self.providers.read().unwrap();
        let mut all_tickers: HashSet<String> = HashSet::new();
        
        for provider in providers.iter() {
            let name = provider.provider_name().to_string();
            if Self::should_include_provider(&config.provider_filter, &name) {
                for ticker in provider.list_supported_tickers() {
                    all_tickers.insert(ticker);
                }
            }
        }

        let mut tickers: Vec<String> = all_tickers.into_iter().collect();
        tickers.sort();

        let mut providers_list: Vec<String> = providers.iter()
            .filter(|p| {
                let name = p.provider_name().to_string();
                Self::should_include_provider(&config.provider_filter, &name)
            })
            .map(|p| p.provider_name().to_string())
            .collect();
        providers_list.sort();

        let mut chart_groups: Vec<ChartGroup> = Vec::new();
        let mut raw_data: Vec<StockChartData> = Vec::new();

        for metric in &config.metrics {
            let mut entries: Vec<ChartEntry> = Vec::new();
            
            for ticker in &tickers {
                for provider in providers.iter() {
                    let name = provider.provider_name().to_string();
                    if !Self::should_include_provider(&config.provider_filter, &name) {
                        continue;
                    }
                    
                    if let Some(data) = provider.get_stock_data(ticker) {
                        let value = Self::extract_metric_value(metric, &data);
                        let unit = metric.unit();
                        let color = metric.color().to_string();

                        entries.push(ChartEntry {
                            ticker: ticker.clone(),
                            company_name: data.company_name.clone(),
                            provider: name.clone(),
                            value,
                        });

                        raw_data.push(StockChartData {
                            ticker: ticker.clone(),
                            company_name: data.company_name.clone(),
                            provider: name.clone(),
                            metrics: vec![MetricValue {
                                name: metric.label().to_string(),
                                value,
                                unit: unit.to_string(),
                                color: color.clone(),
                            }],
                        });
                    }
                }
            }

            if !entries.is_empty() {
                chart_groups.push(ChartGroup {
                    metric: *metric,
                    label: metric.label().to_string(),
                    unit: metric.unit().to_string(),
                    color: metric.color().to_string(),
                    entries,
                });
            }
        }

        AggregatedStockData {
            tickers,
            providers: providers_list,
            chart_groups,
            raw_data,
        }
    }

    pub fn get_all_tickers(&self) -> Vec<TickerInfo> {
        let providers = self.providers.read().unwrap();
        let mut ticker_providers: HashMap<String, Vec<String>> = HashMap::new();

        for provider in providers.iter() {
            let name = provider.provider_name().to_string();
            for ticker in provider.list_supported_tickers() {
                ticker_providers
                    .entry(ticker)
                    .or_default()
                    .push(name.clone());
            }
        }

        let mut result: Vec<TickerInfo> = ticker_providers
            .into_iter()
            .map(|(ticker, providers)| TickerInfo {
                ticker,
                providers,
            })
            .collect();
        result.sort_by(|a, b| a.ticker.cmp(&b.ticker));
        result
    }

    pub fn get_all_stock_data(&self) -> Vec<(String, String, StockRatingData)> {
        let providers = self.providers.read().unwrap();
        let mut result: Vec<(String, String, StockRatingData)> = Vec::new();
        
        for provider in providers.iter() {
            let name = provider.provider_name().to_string();
            for ticker in provider.list_supported_tickers() {
                if let Some(data) = provider.get_stock_data(&ticker) {
                    result.push((ticker, name.clone(), data));
                }
            }
        }
        
        result
    }

    pub(crate) fn should_include_provider(filter: &[String], name: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        filter.iter().any(|f| f.to_lowercase() == name.to_lowercase())
    }

    pub(crate) fn extract_metric_value(metric: &MetricType, data: &StockRatingData) -> f64 {
        match metric {
            MetricType::PE => data.valuation_ratios.pe_ratio.unwrap_or(0.0),
            MetricType::ForwardPE => data.valuation_ratios.forward_pe_ratio.unwrap_or(0.0),
            MetricType::EVToEBITDA => data.valuation_ratios.ev_to_ebitda.unwrap_or(0.0),
            MetricType::PB => data.valuation_ratios.pb_ratio.unwrap_or(0.0),
            MetricType::ROE => data.financial_health.return_on_equity.unwrap_or(0.0) * 100.0,
            MetricType::DebtToEquity => data.financial_health.debt_to_equity.unwrap_or(0.0),
            MetricType::CurrentRatio => data.financial_health.current_ratio.unwrap_or(0.0),
            MetricType::RevenueGrowth3Y => data.growth_metrics.revenue_growth_3y.unwrap_or(0.0) * 100.0,
            MetricType::EPSGrowth3Y => data.growth_metrics.eps_growth_3y.unwrap_or(0.0) * 100.0,
            MetricType::Upside => {
                match (data.market_sentiment.target_price_consensus, data.market_sentiment.current_price) {
                    (Some(target), Some(current)) if current > 0.0 => ((target - current) / current) * 100.0,
                    _ => 0.0,
                }
            }
            MetricType::TargetPrice => data.market_sentiment.target_price_consensus.unwrap_or(0.0),
            MetricType::CurrentPrice => data.market_sentiment.current_price.unwrap_or(0.0),
            MetricType::AnalystCount => data.market_sentiment.analyst_count.unwrap_or(0) as f64,
            MetricType::FCF => data.financial_health.free_cash_flow.unwrap_or(0) as f64,
        }
    }

    pub fn default_config() -> ChartConfig {
        ChartConfig {
            metrics: vec![
                MetricType::PE,
                MetricType::ROE,
                MetricType::RevenueGrowth3Y,
                MetricType::Upside,
            ],
            provider_filter: vec![],
            chart_type: ChartType::Bar,
        }
    }

    pub fn parse_query_params(params: &HashMap<String, String>) -> ChartConfig {
        let mut config = Self::default_config();

        if let Some(metrics_str) = params.get("metrics") {
            let metrics: Vec<MetricType> = metrics_str
                .split(',')
                .filter_map(|m| MetricType::from_str(m.trim()))
                .collect();
            if !metrics.is_empty() {
                config.metrics = metrics;
            }
        }

        if let Some(providers_str) = params.get("providers") {
            config.provider_filter = providers_str
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
        }

        if let Some(chart_type_str) = params.get("chart_type") {
            if let Some(chart_type) = ChartType::from_str(chart_type_str) {
                config.chart_type = chart_type;
            }
        }

        config
    }
}
