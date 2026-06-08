use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::models::{Recommendation, StockRatingData};
use crate::providers::StockDataProvider;

#[derive(Clone)]
pub struct AppState {
    pub data_provider: std::sync::Arc<std::sync::RwLock<Box<dyn StockDataProvider + Send + Sync>>>,
}

#[derive(Deserialize)]
struct QueryParams {
    #[serde(rename = "ticker")]
    ticker: String,
}

pub fn format_recommendation(rec: &Recommendation) -> String {
    match rec {
        Recommendation::StrongBuy => "STRONG BUY".to_string(),
        Recommendation::Buy => "BUY".to_string(),
        Recommendation::Hold => "HOLD".to_string(),
        Recommendation::Sell => "SELL".to_string(),
        Recommendation::StrongSell => "STRONG SELL".to_string(),
    }
}

pub fn format_growth(growth: Option<f64>) -> String {
    match growth {
        Some(g) => format!("{:.1}%", g * 100.0),
        None => "N/A".to_string(),
    }
}

pub fn format_ratio(value: Option<f64>) -> String {
    match value {
        Some(v) => format!("{:.2}", v),
        None => "N/A".to_string(),
    }
}

pub fn format_market_cap(value: Option<i64>) -> String {
    match value {
        Some(v) if v >= 1_000_000_000_000 => format!("${:.2}T", v as f64 / 1_000_000_000_000.0),
        Some(v) if v >= 1_000_000_000 => format!("${:.2}B", v as f64 / 1_000_000_000.0),
        Some(v) if v >= 1_000_000 => format!("${:.2}M", v as f64 / 1_000_000.0),
        Some(v) => format!("${}", v),
        None => "N/A".to_string(),
    }
}

fn html_page(data: &StockRatingData) -> String {
    use chrono::Utc;
    let rec_text = format_recommendation(
        &data.market_sentiment.recommendation_consensus.clone().unwrap_or(Recommendation::Hold)
    );
    let upside = match (data.market_sentiment.target_price_consensus, data.market_sentiment.current_price) {
        (Some(target), Some(current)) => {
            let upside_pct = ((target - current) / current) * 100.0;
            format!("{:+.1}%", upside_pct)
        }
        _ => "N/A".to_string(),
    };

    let rec_color = match data.market_sentiment.recommendation_consensus {
        Some(Recommendation::StrongBuy) | Some(Recommendation::Buy) => "#22c55e",
        Some(Recommendation::Hold) => "#eab308",
        Some(Recommendation::Sell) | Some(Recommendation::StrongSell) => "#ef4444",
        None => "#6b7280",
    };

    let roe = data.financial_health.return_on_equity.unwrap_or(0.0) * 100.0;
    let d_to_e = data.financial_health.debt_to_equity.unwrap_or(0.0);
    let fcf = data.financial_health.free_cash_flow.unwrap_or(0);

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{ticker} - StockRating Dashboard</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0f172a; color: #e2e8f0; min-height: 100vh; }}
        .container {{ max-width: 1200px; margin: 0 auto; padding: 20px; }}
        header {{ display: flex; justify-content: space-between; align-items: center; padding: 20px 0; border-bottom: 1px solid #1e293b; margin-bottom: 30px; }}
        h1 {{ font-size: 1.8rem; color: #f8fafc; }}
        h1 span {{ color: #60a5fa; }}
        .ticker-badge {{ background: #1e293b; padding: 8px 16px; border-radius: 8px; font-weight: bold; font-size: 1.2rem; }}
        .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(350px, 1fr)); gap: 20px; margin-bottom: 30px; }}
        .card {{ background: #1e293b; border-radius: 12px; padding: 24px; border: 1px solid #334155; }}
        .card h2 {{ font-size: 1.1rem; color: #94a3b8; margin-bottom: 16px; text-transform: uppercase; letter-spacing: 0.05em; }}
        .metric {{ display: flex; justify-content: space-between; padding: 10px 0; border-bottom: 1px solid #334155; }}
        .metric:last-child {{ border-bottom: none; }}
        .metric-label {{ color: #94a3b8; }}
        .metric-value {{ color: #f8fafc; font-weight: 600; font-variant-numeric: tabular-nums; }}
        .sentiment-box {{ text-align: center; padding: 30px; }}
        .sentiment-label {{ font-size: 0.9rem; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.1em; margin-bottom: 10px; }}
        .sentiment-value {{ font-size: 2.5rem; font-weight: 800; color: {rec_color}; margin-bottom: 10px; }}
        .upside {{ font-size: 1.2rem; color: #94a3b8; }}
        .upside span {{ font-weight: 600; }}
        .upside.positive {{ color: #22c55e; }}
        .upside.negative {{ color: #ef4444; }}
        .company-info {{ margin-bottom: 20px; }}
        .company-name {{ font-size: 1.1rem; color: #cbd5e1; margin-bottom: 4px; }}
        .last-updated {{ font-size: 0.8rem; color: #64748b; }}
        .full-width {{ grid-column: 1 / -1; }}
        .progress-bar {{ height: 8px; background: #334155; border-radius: 4px; overflow: hidden; margin-top: 8px; }}
        .progress-fill {{ height: 100%; border-radius: 4px; transition: width 0.3s; }}
        .analyst-stats {{ display: flex; gap: 20px; justify-content: center; margin-top: 20px; }}
        .analyst-stat {{ text-align: center; }}
        .analyst-stat-value {{ font-size: 1.5rem; font-weight: 700; color: #f8fafc; }}
        .analyst-stat-label {{ font-size: 0.75rem; color: #64748b; text-transform: uppercase; }}
        .search-section {{ text-align: center; margin: 40px 0; }}
        .search-section input {{ padding: 12px 20px; border-radius: 8px; border: 1px solid #334155; background: #1e293b; color: #f8fafc; font-size: 1rem; width: 300px; outline: none; }}
        .search-section input:focus {{ border-color: #60a5fa; }}
        .search-section button {{ padding: 12px 24px; border-radius: 8px; border: none; background: #3b82f6; color: white; font-size: 1rem; cursor: pointer; font-weight: 600; margin-left: 10px; }}
        .search-section button:hover {{ background: #2563eb; }}
        .error {{ text-align: center; padding: 40px; color: #ef4444; }}
        .not-found {{ text-align: center; padding: 60px 20px; }}
        .not-found h2 {{ font-size: 2rem; margin-bottom: 10px; }}
        .not-found p {{ color: #94a3b8; margin-bottom: 20px; }}
        .not-found a {{ color: #60a5fa; text-decoration: none; }}
        .not-found a:hover {{ text-decoration: underline; }}
        .health-indicator {{ display: inline-block; width: 10px; height: 10px; border-radius: 50%; margin-right: 6px; }}
        .health-good {{ background: #22c55e; }}
        .health-moderate {{ background: #eab308; }}
        .health-poor {{ background: #ef4444; }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1><span>Stock</span>Rating</h1>
            <div class="ticker-badge">{ticker}</div>
        </header>

        <div class="company-info">
            <div class="company-name">{company_name}</div>
            <div class="last-updated">Last updated: {last_updated}</div>
        </div>

        <div class="grid">
            <div class="card">
                <h2>Market Sentiment</h2>
                <div class="sentiment-box">
                    <div class="sentiment-label">Analyst Recommendation</div>
                    <div class="sentiment-value">{rec_text}</div>
                    <div class="upside {upside_class}">
                        Upside: <span>{upside}</span>
                    </div>
                    <div class="analyst-stats">
                        <div class="analyst-stat">
                            <div class="analyst-stat-value">{analyst_count}</div>
                            <div class="analyst-stat-label">Analysts</div>
                        </div>
                        <div class="analyst-stat">
                            <div class="analyst-stat-value">${target_price}</div>
                            <div class="analyst-stat-label">Target Price</div>
                        </div>
                        <div class="analyst-stat">
                            <div class="analyst-stat-value">${current_price}</div>
                            <div class="analyst-stat-label">Current Price</div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="card">
                <h2>Valuation Ratios</h2>
                <div class="metric">
                    <span class="metric-label">P/E Ratio</span>
                    <span class="metric-value">{pe_ratio}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Forward P/E</span>
                    <span class="metric-value">{forward_pe}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">EV/EBITDA</span>
                    <span class="metric-value">{ev_ebitda}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">P/B Ratio</span>
                    <span class="metric-value">{pb_ratio}</span>
                </div>
            </div>

            <div class="card">
                <h2>Financial Health</h2>
                <div class="metric">
                    <span class="metric-label">Return on Equity</span>
                    <span class="metric-value">{roe}%</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Debt-to-Equity</span>
                    <span class="metric-value">{d_to_e}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Free Cash Flow</span>
                    <span class="metric-value">{fcf_formatted}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Current Ratio</span>
                    <span class="metric-value">{current_ratio}</span>
                </div>
            </div>

            <div class="card">
                <h2>Growth Metrics</h2>
                <div class="metric">
                    <span class="metric-label">Revenue Growth (3Y)</span>
                    <span class="metric-value">{revenue_growth}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">EPS Growth (3Y)</span>
                    <span class="metric-value">{eps_growth}</span>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"#,
        ticker = data.ticker,
        company_name = data.company_name,
        last_updated = data.last_updated.unwrap_or(Utc::now()).format("%Y-%m-%d %H:%M UTC").to_string(),
        rec_text = rec_text,
        rec_color = rec_color,
        upside = upside,
        upside_class = if upside.starts_with('+') { "positive" } else if upside.starts_with('-') { "negative" } else { "" },
        analyst_count = data.market_sentiment.analyst_count.unwrap_or(0),
        target_price = format_ratio(data.market_sentiment.target_price_consensus),
        current_price = format_ratio(data.market_sentiment.current_price),
        pe_ratio = format_ratio(data.valuation_ratios.pe_ratio),
        forward_pe = format_ratio(data.valuation_ratios.forward_pe_ratio),
        ev_ebitda = format_ratio(data.valuation_ratios.ev_to_ebitda),
        pb_ratio = format_ratio(data.valuation_ratios.pb_ratio),
        roe = roe,
        d_to_e = format_ratio(Some(d_to_e)),
        fcf_formatted = format_market_cap(Some(fcf)),
        current_ratio = format_ratio(data.financial_health.current_ratio),
        revenue_growth = format_growth(data.growth_metrics.revenue_growth_3y),
        eps_growth = format_growth(data.growth_metrics.eps_growth_3y),
    )
}

pub fn html_index(tickers: &[String]) -> String {
    let mut ticker_list = String::new();
    for ticker in tickers {
        ticker_list.push_str(&format!(
            r#"<a href="/dashboard?ticker={}" class="ticker-link">{}</a>"#,
            ticker, ticker
        ));
    }

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>StockRating Dashboard</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0f172a; color: #e2e8f0; min-height: 100vh; }}
        .container {{ max-width: 800px; margin: 0 auto; padding: 40px 20px; text-align: center; }}
        h1 {{ font-size: 2.5rem; margin-bottom: 10px; }}
        h1 span {{ color: #60a5fa; }}
        .subtitle {{ color: #94a3b8; margin-bottom: 40px; font-size: 1.1rem; }}
        .ticker-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 16px; max-width: 600px; margin: 0 auto; }}
        .ticker-link {{ display: block; padding: 20px; background: #1e293b; border-radius: 12px; color: #f8fafc; text-decoration: none; font-weight: 700; font-size: 1.2rem; border: 1px solid #334155; transition: all 0.2s; }}
        .ticker-link:hover {{ background: #334155; border-color: #60a5fa; transform: translateY(-2px); }}
        .footer {{ margin-top: 60px; color: #64748b; font-size: 0.85rem; }}
    </style>
</head>
<body>
    <div class="container">
        <h1><span>Stock</span>Rating</h1>
        <p class="subtitle">Select a ticker to view detailed analysis</p>
        <div class="ticker-grid">
            {tickers}
        </div>
        <p class="footer">Mock data provider • Dashboard v1.0</p>
    </div>
</body>
</html>"#,
        tickers = ticker_list,
    )
}

pub fn setup_router(data_provider: Box<dyn StockDataProvider + Send + Sync>) -> Router {
    let state = AppState {
        data_provider: std::sync::Arc::new(std::sync::RwLock::new(data_provider)),
    };

    Router::new()
        .route("/", get(index_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/api/query", get(api_query_handler))
        .with_state(state)
}

async fn index_handler() -> String {
    let tickers: Vec<String> = vec![
        "AAPL".to_string(),
        "MSFT".to_string(),
        "GOOGL".to_string(),
        "TSLA".to_string(),
        "AMZN".to_string(),
    ];
    html_index(&tickers)
}

async fn dashboard_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> String {
    let ticker = params.ticker.to_uppercase();
    let data_provider = state.data_provider.read().unwrap();
    match data_provider.get_stock_data(&ticker) {
        Some(data) => html_page(&data),
        None => {
            let tickers = data_provider.list_supported_tickers();
            let tickers_list: Vec<String> = tickers.iter().map(|t| format!("/dashboard?ticker={}", t)).collect();
            format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{ticker} - StockRating</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0f172a; color: #e2e8f0; min-height: 100vh; display: flex; align-items: center; justify-content: center; }}
        .container {{ text-align: center; padding: 40px; }}
        h2 {{ font-size: 2rem; margin-bottom: 10px; color: #f8fafc; }}
        p {{ color: #94a3b8; margin-bottom: 30px; }}
        a {{ color: #60a5fa; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Ticker Not Found</h2>
        <p>Stock data for "{ticker}" is not available in our mock database.</p>
        <p>Available tickers: <a href="/">Click here to view all tickers</a></p>
    </div>
</body>
</html>"#,
                ticker = ticker,
            )
        }
    }
}

async fn api_query_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<StockRatingData>, (StatusCode, String)> {
    let ticker = params.ticker;
    let data_provider = state.data_provider.read().unwrap();
    match data_provider.get_stock_data(&ticker) {
        Some(data) => Ok(Json(data)),
        None => Err((
            StatusCode::NOT_FOUND,
            format!("Ticker '{}' not found", ticker),
        )),
    }
}
