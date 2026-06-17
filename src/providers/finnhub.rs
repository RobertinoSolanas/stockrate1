use crate::models::*;
use chrono::Utc;
use crate::providers::StockDataProvider;
use reqwest;
use serde::Deserialize;
use serde_json;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FinnhubProfile {
    #[serde(rename = "name")]
    company_name: String,
    #[serde(rename = "financialCurrency")]
    currency: Option<String>,
    #[serde(rename = "exchange")]
    exchange: Option<String>,
    #[serde(rename = "ipo")]
    ipo: Option<String>,
    #[serde(rename = "marketCapitalization")]
    market_cap: Option<f64>,
    #[serde(rename = "price")]
    price: Option<f64>,
    #[serde(rename = "stockExchange")]
    stock_exchange: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FinnhubQuote {
    c: Option<f64>, // current price
    d: Option<f64>, // change
    dp: Option<f64>, // change percent
    h: Option<f64>, // high
    l: Option<f64>, // low
    o: Option<f64>, // open
    pc: Option<f64>, // previous close
    t: Option<i64>, // timestamp
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct FinnhubMetricEntry {
    period: Option<String>,
    v: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct FinnhubSeriesAnnual {
    #[serde(rename = "currentRatio")]
    current_ratio: Option<Vec<FinnhubMetricEntry>>,
    #[serde(rename = "longtermDebtTotalAsset")]
    longterm_debt_total_asset: Option<Vec<FinnhubMetricEntry>>,
    #[serde(rename = "totalDebt/totalEquity")]
    total_debt_total_equity: Option<Vec<FinnhubMetricEntry>>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct FinnhubSeries {
    annual: Option<FinnhubSeriesAnnual>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct FinnhubMetricScalar {
    #[serde(rename = "peAnnual")]
    pe_annual: Option<f64>,
    #[serde(rename = "peTTM")]
    pe_ttm: Option<f64>,
    #[serde(rename = "peNormalizedAnnual")]
    pe_normalized: Option<f64>,
    #[serde(rename = "forwardPE")]
    forward_pe: Option<f64>,
    #[serde(rename = "evEbitdaTTM")]
    ev_ebitda_ttm: Option<f64>,
    #[serde(rename = "pb")]
    pb: Option<f64>,
    #[serde(rename = "pbAnnual")]
    pb_annual: Option<f64>,
    #[serde(rename = "pbQuarterly")]
    pb_quarterly: Option<f64>,
    #[serde(rename = "roeTTM")]
    roe_ttm: Option<f64>,
    #[serde(rename = "roe5Y")]
    roe_5y: Option<f64>,
    #[serde(rename = "roeRfy")]
    roe_rfy: Option<f64>,
    #[serde(rename = "totalDebt/totalEquityAnnual")]
    total_debt_equity_annual: Option<f64>,
    #[serde(rename = "totalDebt/totalEquityQuarterly")]
    total_debt_equity_quarterly: Option<f64>,
    #[serde(rename = "epsGrowth3Y")]
    eps_growth_3y: Option<f64>,
    #[serde(rename = "revenueGrowth3Y")]
    revenue_growth_3y: Option<f64>,
    #[serde(rename = "revenueGrowthTTMYoy")]
    revenue_growth_ttm: Option<f64>,
    #[serde(rename = "currentRatioAnnual")]
    current_ratio_annual: Option<f64>,
    #[serde(rename = "currentRatioQuarterly")]
    current_ratio_quarterly: Option<f64>,
    #[serde(rename = "freeCashFlowPerShareAnnual")]
    fcf_per_share_annual: Option<f64>,
    #[serde(rename = "freeCashFlowPerShareTTM")]
    fcf_per_share_ttm: Option<f64>,
    #[serde(rename = "marketCapitalization")]
    market_cap: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct FinnhubMetrics {
    metric: Option<FinnhubMetricScalar>,
    series: Option<FinnhubSeries>,
}

#[derive(Debug, Deserialize)]
struct FinnhubRecommendation {
    #[serde(rename = "strongBuy")]
    strong_buy: Option<i32>,
    #[serde(rename = "buy")]
    buy: Option<i32>,
    #[serde(rename = "hold")]
    hold: Option<i32>,
    #[serde(rename = "sell")]
    sell: Option<i32>,
    #[serde(rename = "strongSell")]
    strong_sell: Option<i32>,
}

pub struct FinnhubDataProvider {
    api_key: String,
    #[allow(dead_code)]
    client: reqwest::Client,
}

impl FinnhubDataProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    async fn fetch<T: serde::de::DeserializeOwned>(_api_key: &str, url: &str) -> Result<T, String> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await.map_err(|e| e.to_string())?;
        let body = response.text().await.map_err(|e| e.to_string())?;
        serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {}", e))
    }

    async fn get_stock_data_from_api_inner(api_key: &str, ticker: &str) -> Option<StockRatingData> {
        let ticker_upper = ticker.to_uppercase();

        // Fetch company profile
        let profile_url = format!(
            "https://finnhub.io/api/v1/stock/profile2?symbol={}&token={}",
            ticker_upper, api_key
        );
        let profile: FinnhubProfile = match Self::fetch(&api_key, &profile_url).await {
            Ok(p) => p,
            Err(_) => return None,
        };
        if profile.company_name.is_empty() {
            return None;
        }

        // Fetch quote
        let quote_url = format!(
            "https://finnhub.io/api/v1/quote?symbol={}&token={}",
            ticker_upper, api_key
        );
        let quote: FinnhubQuote = match Self::fetch(&api_key, &quote_url).await {
            Ok(q) => q,
            Err(_) => return None,
        };

        // Fetch metrics
        let metrics_url = format!(
            "https://finnhub.io/api/v1/stock/metric?symbol={}&metric=all&token={}",
            ticker_upper, api_key
        );
        let metrics_raw = match Self::fetch::<serde_json::Value>(&api_key, &metrics_url).await {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Finnhub metrics fetch error: {}", e);
                return None;
            }
        };
        eprintln!("Finnhub metrics raw: {}", serde_json::to_string_pretty(&metrics_raw).unwrap_or_default());
        let metrics: FinnhubMetrics = match serde_json::from_value(metrics_raw) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Finnhub metrics deserialization error: {}", e);
                return None;
            }
        };

        // Fetch recommendation trends
         let rec_url = format!(
            "https://finnhub.io/api/v1/recommendation-trends?symbol={}&token={}",
            ticker_upper, api_key
        );
        let rec_trends: Vec<FinnhubRecommendation> = match Self::fetch(&api_key, &rec_url).await {
            Ok(r) => r,
            Err(_) => Vec::new(),
        };

        // Get the most recent recommendation
        let latest_rec = rec_trends.first();

        let m = metrics.metric.unwrap_or_default();

        // PE ratio: prefer TTM, fall back to annual
        let pe_ratio = m.pe_ttm.or(m.pe_normalized).or(m.pe_annual);

        // Forward PE
        let forward_pe_ratio = m.forward_pe.or(pe_ratio);

        // EV/EBITDA
        let ev_to_ebitda = m.ev_ebitda_ttm;

        // P/B Ratio: prefer quarterly, fall back to annual
        let pb_ratio = m.pb_quarterly.or(m.pb).or(m.pb_annual);

        // ROE: prefer TTM, fall back to annual (fy) or 5y
        let roe = m.roe_ttm.or(m.roe_rfy).or(m.roe_5y).map(|x| x / 100.0);

        // Debt/Equity: prefer quarterly, fall back to annual
        let debt_to_equity = m.total_debt_equity_quarterly.or(m.total_debt_equity_annual);

        // Current ratio: prefer quarterly, fall back to annual
        let current_ratio = m.current_ratio_quarterly.or(m.current_ratio_annual);

        // EPS growth 3Y: Finnhub returns percentage, convert to decimal
        let eps_growth_3y = m.eps_growth_3y.map(|x| x / 100.0);

        // Revenue growth 3Y: Finnhub returns percentage, convert to decimal
        let revenue_growth_3y = m.revenue_growth_3y.map(|x| x / 100.0).or(m.revenue_growth_ttm.map(|x| x / 100.0));

        // Free cash flow per share (TTM)
        let fcf_per_share = m.fcf_per_share_ttm.or(m.fcf_per_share_annual);

        // Current price from quote
        let current_price = quote.c;

        // Target price: use analyst consensus if available, else estimate
        let target_price_consensus = match latest_rec {
            Some(r) => {
                let total = r.strong_buy.unwrap_or(0) + r.buy.unwrap_or(0)
                    + r.hold.unwrap_or(0) + r.sell.unwrap_or(0) + r.strong_sell.unwrap_or(0);
                if total > 0 {
                    let score = (r.strong_buy.unwrap_or(0) as f64 * 5.0
                        + r.buy.unwrap_or(0) as f64 * 4.0
                        + r.hold.unwrap_or(0) as f64 * 3.0
                        + r.sell.unwrap_or(0) as f64 * 2.0
                        + r.strong_sell.unwrap_or(0) as f64 * 1.0)
                        / total as f64;
                    // Convert score to target price estimate
                    // Score 5.0 = Strong Buy -> target ~20% above current
                    // Score 1.0 = Strong Sell -> target ~20% below current
                    let current = current_price.unwrap_or(0.0);
                    if current > 0.0 {
                        let multiplier = 0.8 + (score / 5.0) * 0.4;
                        Some(current * multiplier)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => None,
        };

        // Recommendation consensus from Finnhub
        let recommendation_consensus = match latest_rec {
            Some(r) => {
                let strong_buy = r.strong_buy.unwrap_or(0);
                let buy = r.buy.unwrap_or(0);
                let hold = r.hold.unwrap_or(0);
                let sell = r.sell.unwrap_or(0);
                let strong_sell = r.strong_sell.unwrap_or(0);
                let total = strong_buy + buy + hold + sell + strong_sell;
                if total == 0 {
                    None
                } else if strong_buy as f64 / total as f64 > 0.4 {
                    Some(Recommendation::StrongBuy)
                } else if (strong_buy + buy) as f64 / total as f64 > 0.4 {
                    Some(Recommendation::Buy)
                } else if hold as f64 / total as f64 > 0.3 {
                    Some(Recommendation::Hold)
                } else if sell as f64 / total as f64 > 0.3 {
                    Some(Recommendation::Sell)
                } else {
                    Some(Recommendation::StrongSell)
                }
            }
            None => None,
        };

        let analyst_count = match latest_rec {
            Some(r) => Some(
                r.strong_buy.unwrap_or(0) + r.buy.unwrap_or(0)
                    + r.hold.unwrap_or(0) + r.sell.unwrap_or(0)
                    + r.strong_sell.unwrap_or(0)
            ),
            None => None,
        };

        // Revenue growth approximation: use EPS growth as proxy when revenue data unavailable
        let revenue_growth_3y = revenue_growth_3y.or(eps_growth_3y);

        Some(StockRatingData {
            ticker: ticker_upper,
            company_name: profile.company_name,
            provider: "FinnhubDataProvider".to_string(),
            last_updated: Some(Utc::now()),
            valuation_ratios: ValuationRatios {
                pe_ratio,
                forward_pe_ratio,
                ev_to_ebitda,
                pb_ratio,
            },
            financial_health: FinancialHealth {
                return_on_equity: roe,
                debt_to_equity,
                free_cash_flow: fcf_per_share.map(|x| x as i64),
                current_ratio,
            },
            growth_metrics: GrowthMetrics {
                revenue_growth_3y,
                eps_growth_3y,
            },
            market_sentiment: MarketSentiment {
                target_price_consensus,
                current_price,
                recommendation_consensus,
                analyst_count,
            },
        })
    }
}

impl StockDataProvider for FinnhubDataProvider {
    fn get_stock_data(&self, ticker: &str) -> Option<StockRatingData> {
        let api_key = self.api_key.clone();
        let ticker = ticker.to_string();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build tokio runtime");
            rt.block_on(Self::get_stock_data_from_api_inner(&api_key, &ticker))
        })
        .join()
        .unwrap_or(None)
    }

    fn list_supported_tickers(&self) -> Vec<String> {
        vec![
            "AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string(),
            "TSLA".to_string(), "AMZN".to_string(), "NVDA".to_string(),
            "META".to_string(), "AMD".to_string(), "NFLX".to_string(),
            "DIS".to_string(), "INTC".to_string(), "BA".to_string(),
        ]
    }

    fn provider_name(&self) -> &'static str {
        "FinnhubDataProvider"
    }
}
