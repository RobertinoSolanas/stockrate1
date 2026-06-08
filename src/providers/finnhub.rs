use crate::models::*;
use chrono::Utc;
use crate::providers::StockDataProvider;
use reqwest;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
struct FinnhubMetrics {
    metrics: Option<FinnhubMetricData>,
}

#[derive(Debug, Deserialize, Default)]
struct FinnhubMetricData {
    #[serde(rename = "peAnnual")]
    pe_ratio: Option<Vec<f64>>,
    #[serde(rename = "pbRatioAnnual")]
    pb_ratio: Option<Vec<f64>>,
    #[serde(rename = "evToEbitdaAnnual")]
    ev_to_ebitda: Option<Vec<f64>>,
    #[serde(rename = "evToSalesAnnual")]
    ev_to_sales: Option<Vec<f64>>,
    #[serde(rename = "revenuePerEmployeeAnnual")]
    revenue_per_employee: Option<Vec<f64>>,
    #[serde(rename = "returnOnEquityAnnual")]
    roe: Option<Vec<f64>>,
    #[serde(rename = "debtEquityAnnual")]
    debt_equity: Option<Vec<f64>>,
    #[serde(rename = "freeCashFlowPerShareAnnual")]
    fcf_per_share: Option<Vec<f64>>,
    #[serde(rename = "dividendYieldAnnualizedAnnual")]
    dividend_yield: Option<Vec<f64>>,
    #[serde(rename = "dividendYieldUnitAnnualizedAnnual")]
    dividend_yield_unit: Option<Vec<f64>>,
    #[serde(rename = "epsAnnual")]
    eps: Option<Vec<f64>>,
    #[serde(rename = "revenuePerEmployeeTTM")]
    revenue_per_employee_ttm: Option<Vec<f64>>,
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
    client: reqwest::Client,
}

impl FinnhubDataProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    async fn fetch<T: serde::de::DeserializeOwned>(api_key: &str, url: &str) -> Result<T, String> {
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
        let metrics: FinnhubMetrics = match Self::fetch(&api_key, &metrics_url).await {
            Ok(m) => m,
            Err(_) => return None,
        };
        let md = metrics.metrics.unwrap_or_default();

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

        // Build PE ratio from metrics (take last non-zero value)
        let pe_ratio = md.pe_ratio.as_ref().and_then(|v| {
            v.iter().rev().find(|&&x| x > 0.0).copied()
        });

        // Get forward PE approximation: use trailing PE / (1 + expected_growth)
        // Finnhub doesn't provide forward PE directly, use trailing as fallback
        let forward_pe_ratio = pe_ratio;

        // EV/EBITDA
        let ev_to_ebitda = md.ev_to_ebitda.as_ref().and_then(|v| {
            v.iter().rev().find(|&&x| x > 0.0).copied()
        });

        // P/B Ratio
        let pb_ratio = md.pb_ratio.as_ref().and_then(|v| {
            v.iter().rev().find(|&&x| x > 0.0).copied()
        });

        // ROE
        let roe = md.roe.as_ref().and_then(|v| {
            v.iter().rev().find(|&&x| x > 0.0).copied()
        });

        // Debt/Equity
        let debt_to_equity = md.debt_equity.as_ref().and_then(|v| {
            v.iter().rev().find(|&&x| x > 0.0).copied()
        });

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

        // Revenue growth (3Y): approximate from EPS growth trend
        // Finnhub metrics have per-year EPS, we approximate
        let eps_values = md.eps.as_ref();
        let eps_growth_3y = match eps_values {
            Some(v) if v.len() >= 3 => {
                let recent = v.iter().rev().take(3).collect::<Vec<&f64>>();
                if recent[0] > &0.0 && recent[2] > &0.0 {
                    Some(((recent[0] / recent[2]).powf(1.0 / 2.0) - 1.0).abs())
                } else {
                    None
                }
            }
            _ => None,
        };

        // Revenue growth approximation: use EPS growth as proxy when revenue data unavailable
        let revenue_growth_3y = eps_growth_3y;

        let fcf = md.fcf_per_share.as_ref().and_then(|v| {
            v.iter().rev().find(|&&x| x > 0.0).copied()
        });

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
                free_cash_flow: fcf.map(|x| (x * 1_000_000.0) as i64), // rough approximation per share to total
                current_ratio: None, // Finnhub doesn't provide this directly
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
