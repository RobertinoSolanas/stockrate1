use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Json},
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
        Some(g) => format!("{:+.1}%", g * 100.0),
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

fn health_score_color(roe: f64, d_to_e: f64) -> &'static str {
    let score = if roe > 0.3 { 3 } else if roe > 0.1 { 2 } else { 1 };
    let debt_score = if d_to_e < 0.5 { 3 } else if d_to_e < 1.0 { 2 } else { 1 };
    let total = score + debt_score;
    match total {
        5..=6 => "#10b981",
        3..=4 => "#f59e0b",
        _ => "#ef4444",
    }
}

fn health_score_label(score: f64, d_to_e: f64) -> (&'static str, &'static str) {
    let score = if score > 0.3 { 3 } else if score > 0.1 { 2 } else { 1 };
    let debt_score = if d_to_e < 0.5 { 3 } else if d_to_e < 1.0 { 2 } else { 1 };
    let total = score + debt_score;
    match total {
        5..=6 => ("Excellent", "#10b981"),
        3..=4 => ("Moderate", "#f59e0b"),
        _ => ("Weak", "#ef4444"),
    }
}

fn valuation_assessment(pe_ratio: Option<f64>, ev_ebitda: Option<f64>) -> (&'static str, &'static str) {
    let pe = pe_ratio.unwrap_or(30.0);
    let ev = ev_ebitda.unwrap_or(25.0);
    if pe < 15.0 && ev < 10.0 {
        ("Attractive", "#10b981")
    } else if pe < 25.0 && ev < 20.0 {
        ("Fair", "#3b82f6")
    } else if pe < 40.0 && ev < 30.0 {
        ("Expensive", "#f59e0b")
    } else {
        ("Overvalued", "#ef4444")
    }
}

fn growth_assessment(rev: Option<f64>, eps: Option<f64>) -> (&'static str, &'static str) {
    let avg = (rev.unwrap_or(0.0) + eps.unwrap_or(0.0)) / 2.0;
    if avg > 0.20 {
        ("Strong", "#10b981")
    } else if avg > 0.10 {
        ("Moderate", "#3b82f6")
    } else if avg > 0.05 {
        ("Slow", "#f59e0b")
    } else {
        ("Declining", "#ef4444")
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

    let upside_numeric = match (data.market_sentiment.target_price_consensus, data.market_sentiment.current_price) {
        (Some(target), Some(current)) => ((target - current) / current) * 100.0,
        _ => 0.0,
    };

    let rec_color = match data.market_sentiment.recommendation_consensus {
        Some(Recommendation::StrongBuy) => "#10b981",
        Some(Recommendation::Buy) => "#22c55e",
        Some(Recommendation::Hold) => "#f59e0b",
        Some(Recommendation::Sell) => "#f97316",
        Some(Recommendation::StrongSell) => "#ef4444",
        None => "#6b7280",
    };

    let roe = data.financial_health.return_on_equity.unwrap_or(0.0);
    let d_to_e = data.financial_health.debt_to_equity.unwrap_or(0.0);
    let fcf = data.financial_health.free_cash_flow.unwrap_or(0);
    let current_ratio = data.financial_health.current_ratio.unwrap_or(0.0);

    let health_label = health_score_label(roe, d_to_e);
    let val_assessment = valuation_assessment(data.valuation_ratios.pe_ratio, data.valuation_ratios.ev_to_ebitda);
    let growth_assessment = growth_assessment(data.growth_metrics.revenue_growth_3y, data.growth_metrics.eps_growth_3y);

    let roe_pct = roe * 100.0;
    let pe = data.valuation_ratios.pe_ratio.unwrap_or(0.0);
    let fcf_formatted = format_market_cap(Some(fcf));

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{ticker} - StockRating</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&display=swap" rel="stylesheet">
    <style>
        :root {{
            --bg-primary: #0b0f19;
            --bg-secondary: #111827;
            --bg-card: #1a2236;
            --bg-card-hover: #1f2a42;
            --border: #2d3a52;
            --border-light: #3b4a66;
            --text-primary: #f1f5f9;
            --text-secondary: #94a3b8;
            --text-muted: #64748b;
            --accent-blue: #3b82f6;
            --accent-blue-glow: rgba(59, 130, 246, 0.3);
            --green: #10b981;
            --green-soft: rgba(16, 185, 129, 0.15);
            --yellow: #f59e0b;
            --yellow-soft: rgba(245, 158, 11, 0.15);
            --red: #ef4444;
            --red-soft: rgba(239, 68, 68, 0.15);
            --blue-soft: rgba(59, 130, 246, 0.15);
        }}

        * {{ margin: 0; padding: 0; box-sizing: border-box; }}

        body {{
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            min-height: 100vh;
            overflow-x: hidden;
        }}

        .bg-grid {{
            position: fixed;
            top: 0; left: 0; right: 0; bottom: 0;
            background-image:
                linear-gradient(rgba(59, 130, 246, 0.03) 1px, transparent 1px),
                linear-gradient(90deg, rgba(59, 130, 246, 0.03) 1px, transparent 1px);
            background-size: 50px 50px;
            pointer-events: none;
            z-index: 0;
        }}

        .bg-glow {{
            position: fixed;
            top: -200px;
            left: 50%;
            transform: translateX(-50%);
            width: 800px;
            height: 600px;
            background: radial-gradient(circle, rgba(59, 130, 246, 0.08) 0%, transparent 70%);
            pointer-events: none;
            z-index: 0;
        }}

        .container {{
            max-width: 1400px;
            margin: 0 auto;
            padding: 0 24px;
            position: relative;
            z-index: 1;
        }}

        /* Header */
        header {{
            padding: 24px 0;
            border-bottom: 1px solid var(--border);
            margin-bottom: 32px;
            backdrop-filter: blur(12px);
        }}

        .header-inner {{
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}

        .logo {{
            display: flex;
            align-items: center;
            gap: 12px;
        }}

        .logo-icon {{
            width: 40px;
            height: 40px;
            background: linear-gradient(135deg, var(--accent-blue), #8b5cf6);
            border-radius: 10px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: 800;
            font-size: 1.1rem;
            color: white;
        }}

        .logo-text {{
            font-size: 1.3rem;
            font-weight: 700;
            letter-spacing: -0.02em;
        }}

        .logo-text span {{ color: var(--accent-blue); }}

        .ticker-badge {{
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 10px 20px;
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 12px;
        }}

        .ticker-symbol {{
            font-weight: 700;
            font-size: 1.1rem;
            letter-spacing: 0.05em;
        }}

        .ticker-company {{
            color: var(--text-secondary);
            font-size: 0.9rem;
        }}

        /* Navigation */
        .nav-bar {{
            display: flex;
            gap: 8px;
            margin-bottom: 32px;
            padding: 6px;
            background: var(--bg-secondary);
            border-radius: 14px;
            border: 1px solid var(--border);
            width: fit-content;
        }}

        .nav-link {{
            padding: 10px 20px;
            border-radius: 10px;
            text-decoration: none;
            color: var(--text-secondary);
            font-weight: 500;
            font-size: 0.9rem;
            transition: all 0.2s;
        }}

        .nav-link:hover {{
            color: var(--text-primary);
            background: var(--bg-card);
        }}

        .nav-link.active {{
            background: var(--accent-blue);
            color: white;
        }}

        /* Search Bar */
        .search-section {{
            margin-bottom: 32px;
        }}

        .search-bar {{
            display: flex;
            gap: 12px;
            max-width: 600px;
            margin: 0 auto;
        }}

        .search-input {{
            flex: 1;
            padding: 14px 20px;
            border-radius: 12px;
            border: 1px solid var(--border);
            background: var(--bg-card);
            color: var(--text-primary);
            font-size: 1rem;
            font-family: inherit;
            outline: none;
            transition: all 0.2s;
        }}

        .search-input:focus {{
            border-color: var(--accent-blue);
            box-shadow: 0 0 0 3px var(--accent-blue-glow);
        }}

        .search-input::placeholder {{ color: var(--text-muted); }}

        .search-btn {{
            padding: 14px 28px;
            border-radius: 12px;
            border: none;
            background: linear-gradient(135deg, var(--accent-blue), #8b5cf6);
            color: white;
            font-weight: 600;
            font-size: 1rem;
            cursor: pointer;
            font-family: inherit;
            transition: all 0.2s;
        }}

        .search-btn:hover {{ transform: translateY(-1px); box-shadow: 0 4px 12px var(--accent-blue-glow); }}

        /* Sentiment Hero */
        .sentiment-hero {{
            background: linear-gradient(135deg, var(--bg-card) 0%, var(--bg-card-hover) 100%);
            border: 1px solid var(--border);
            border-radius: 20px;
            padding: 40px;
            margin-bottom: 32px;
            text-align: center;
            position: relative;
            overflow: hidden;
        }}

        .sentiment-hero::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 3px;
            background: linear-gradient(90deg, var(--green), var(--accent-blue), #8b5cf6);
        }}

        .sentiment-label {{
            font-size: 0.85rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.15em;
            color: var(--text-muted);
            margin-bottom: 12px;
        }}

        .sentiment-value {{
            font-size: 3.5rem;
            font-weight: 800;
            color: {rec_color};
            margin-bottom: 8px;
            letter-spacing: -0.02em;
        }}

        .upside {{
            font-size: 1.1rem;
            font-weight: 500;
            color: var(--text-secondary);
            margin-bottom: 24px;
        }}

        .upside.positive {{ color: var(--green); }}
        .upside.negative {{ color: var(--red); }}

        .price-comparison {{
            display: flex;
            justify-content: center;
            gap: 40px;
            margin-top: 24px;
            padding-top: 24px;
            border-top: 1px solid var(--border);
        }}

        .price-item {{ text-align: center; }}

        .price-label {{
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.1em;
            color: var(--text-muted);
            margin-bottom: 6px;
        }}

        .price-value {{
            font-size: 1.3rem;
            font-weight: 700;
            color: var(--text-primary);
        }}

        .analyst-count {{
            margin-top: 20px;
            padding-top: 20px;
            border-top: 1px solid var(--border);
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 8px;
            color: var(--text-muted);
            font-size: 0.9rem;
        }}

        .analyst-icon {{
            width: 20px;
            height: 20px;
            background: var(--blue-soft);
            border-radius: 50%;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            font-size: 0.7rem;
        }}

        /* Grid */
        .metrics-grid {{
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 24px;
            margin-bottom: 32px;
        }}

        @media (max-width: 900px) {{
            .metrics-grid {{ grid-template-columns: 1fr; }}
            .price-comparison {{ flex-direction: column; gap: 20px; }}
        }}

        /* Cards */
        .card {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 16px;
            padding: 28px;
            transition: all 0.2s;
        }}

        .card:hover {{
            border-color: var(--border-light);
            background: var(--bg-card-hover);
        }}

        .card-header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 24px;
        }}

        .card-title {{
            font-size: 1rem;
            font-weight: 600;
            color: var(--text-secondary);
            text-transform: uppercase;
            letter-spacing: 0.08em;
        }}

        .card-badge {{
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}

        .badge-green {{ background: var(--green-soft); color: var(--green); }}
        .badge-yellow {{ background: var(--yellow-soft); color: var(--yellow); }}
        .badge-red {{ background: var(--red-soft); color: var(--red); }}
        .badge-blue {{ background: var(--blue-soft); color: var(--accent-blue); }}

        /* Metrics */
        .metric-row {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 14px 0;
            border-bottom: 1px solid rgba(45, 58, 82, 0.5);
        }}

        .metric-row:last-child {{ border-bottom: none; }}

        .metric-label {{
            font-size: 0.9rem;
            color: var(--text-secondary);
        }}

        .metric-value {{
            font-size: 1rem;
            font-weight: 600;
            font-variant-numeric: tabular-nums;
            color: var(--text-primary);
        }}

        .metric-value.positive {{ color: var(--green); }}
        .metric-value.negative {{ color: var(--red); }}

        /* Progress bar */
        .progress-container {{
            margin-top: 16px;
        }}

        .progress-bar-bg {{
            height: 8px;
            background: rgba(45, 58, 82, 0.5);
            border-radius: 4px;
            overflow: hidden;
        }}

        .progress-bar-fill {{
            height: 100%;
            border-radius: 4px;
            transition: width 0.5s ease;
        }}

        /* Footer */
        footer {{
            text-align: center;
            padding: 40px 0;
            color: var(--text-muted);
            font-size: 0.85rem;
            border-top: 1px solid var(--border);
            margin-top: 40px;
        }}

        /* Quick stats */
        .quick-stats {{
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 16px;
            margin-bottom: 32px;
        }}

        @media (max-width: 900px) {{
            .quick-stats {{ grid-template-columns: repeat(2, 1fr); }}
        }}

        .quick-stat {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 20px;
            text-align: center;
        }}

        .quick-stat-value {{
            font-size: 1.5rem;
            font-weight: 700;
            margin-bottom: 4px;
        }}

        .quick-stat-label {{
            font-size: 0.75rem;
            color: var(--text-muted);
            text-transform: uppercase;
            letter-spacing: 0.1em;
        }}
    </style>
</head>
<body>
    <div class="bg-grid"></div>
    <div class="bg-glow"></div>

    <div class="container">
        <header>
            <div class="header-inner">
                <div class="logo">
                    <div class="logo-icon">S</div>
                    <div class="logo-text"><span>Stock</span>Rating</div>
                </div>
                <div class="ticker-badge">
                    <span class="ticker-symbol">{ticker}</span>
                    <span class="ticker-company">{company_name}</span>
                </div>
            </div>
        </header>

        <div class="nav-bar">
            <a href="/" class="nav-link active">Dashboard</a>
            <a href="/dashboard?ticker={ticker}" class="nav-link">Analysis</a>
            <a href="/api/query?ticker={ticker}" class="nav-link" target="_blank">API</a>
        </div>

        <div class="sentiment-hero">
            <div class="sentiment-label">Analyst Consensus</div>
            <div class="sentiment-value">{rec_text}</div>
            <div class="upside {upside_class}">
                Potential Upside: {upside}
            </div>

            <div class="price-comparison">
                <div class="price-item">
                    <div class="price-label">Current Price</div>
                    <div class="price-value">${current_price}</div>
                </div>
                <div class="price-item">
                    <div class="price-label">Analyst Target</div>
                    <div class="price-value" style="color: var(--green);">${target_price}</div>
                </div>
                <div class="price-item">
                    <div class="price-label">Coverage</div>
                    <div class="price-value">{analyst_count} analysts</div>
                </div>
            </div>

            <div class="analyst-count">
                <span class="analyst-icon">👥</span>
                Based on {analyst_count} analyst ratings
            </div>
        </div>

        <div class="quick-stats">
            <div class="quick-stat">
                <div class="quick-stat-value" style="color: {health_color};">{health_score}/6</div>
                <div class="quick-stat-label">Health Score</div>
            </div>
            <div class="quick-stat">
                <div class="quick-stat-value" style="color: {val_color};">{val_label}</div>
                <div class="quick-stat-label">Valuation</div>
            </div>
            <div class="quick-stat">
                <div class="quick-stat-value" style="color: {growth_color};">{growth_label}</div>
                <div class="quick-stat-label">Growth</div>
            </div>
            <div class="quick-stat">
                <div class="quick-stat-value">{roe:.1}%</div>
                <div class="quick-stat-label">ROE</div>
            </div>
        </div>

        <div class="metrics-grid">
            <div class="card">
                <div class="card-header">
                    <span class="card-title">Valuation Ratios</span>
                    <span class="card-badge badge-{val_badge}">{val_badge_label}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">P/E Ratio</span>
                    <span class="metric-value">{pe_ratio}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Forward P/E</span>
                    <span class="metric-value">{forward_pe}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">EV/EBITDA</span>
                    <span class="metric-value">{ev_ebitda}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">P/B Ratio</span>
                    <span class="metric-value">{pb_ratio}</span>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span class="card-title">Financial Health</span>
                    <span class="card-badge badge-{health_badge}">{health_label_text}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Return on Equity</span>
                    <span class="metric-value">{roe}%</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Debt-to-Equity</span>
                    <span class="metric-value">{d_to_e}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Free Cash Flow</span>
                    <span class="metric-value">{fcf_formatted}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Current Ratio</span>
                    <span class="metric-value">{current_ratio}</span>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span class="card-title">Growth Metrics</span>
                    <span class="card-badge badge-{growth_badge}">{growth_badge_label}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Revenue Growth (3Y)</span>
                    <span class="metric-value {rev_growth_color}">{revenue_growth}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">EPS Growth (3Y)</span>
                    <span class="metric-value {eps_growth_color}">{eps_growth}</span>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span class="card-title">Market Data</span>
                    <span class="card-badge badge-blue">LIVE</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Last Updated</span>
                    <span class="metric-value">{last_updated}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Data Source</span>
                    <span class="metric-value">Mock Provider</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Recommendation</span>
                    <span class="metric-value" style="color: {rec_color};">{rec_text}</span>
                </div>
            </div>
        </div>

        <footer>
            <p>StockRating Dashboard v1.0 • Mock data provider • Not financial advice</p>
        </footer>
    </div>
</body>
</html>"#,
        ticker = data.ticker,
        company_name = data.company_name,
        rec_text = rec_text,
        rec_color = rec_color,
        upside = upside,
        upside_class = if upside_numeric > 0.0 { "positive" } else { "negative" },
        target_price = format_ratio(data.market_sentiment.target_price_consensus),
        current_price = format_ratio(data.market_sentiment.current_price),
        analyst_count = data.market_sentiment.analyst_count.unwrap_or(0),
        health_score = health_label.0,
        health_color = health_label.1,
        val_label = val_assessment.0,
        val_color = val_assessment.1,
        val_badge = if val_assessment.1 == "#10b981" || val_assessment.1 == "#3b82f6" { "green" } else if val_assessment.1 == "#f59e0b" { "yellow" } else { "red" },
        val_badge_label = val_assessment.0,
        growth_label = growth_assessment.0,
        growth_color = growth_assessment.1,
        growth_badge = if growth_assessment.1 == "#10b981" || growth_assessment.1 == "#3b82f6" { "green" } else if growth_assessment.1 == "#f59e0b" { "yellow" } else { "red" },
        growth_badge_label = growth_assessment.0,
        health_label_text = health_label.0,
        health_badge = if health_label.1 == "#10b981" { "green" } else if health_label.1 == "#f59e0b" { "yellow" } else { "red" },
        roe = roe_pct,
        pe_ratio = format_ratio(data.valuation_ratios.pe_ratio),
        forward_pe = format_ratio(data.valuation_ratios.forward_pe_ratio),
        ev_ebitda = format_ratio(data.valuation_ratios.ev_to_ebitda),
        pb_ratio = format_ratio(data.valuation_ratios.pb_ratio),
        d_to_e = format_ratio(Some(d_to_e)),
        fcf_formatted = fcf_formatted,
        current_ratio = format_ratio(Some(current_ratio)),
        revenue_growth = format_growth(data.growth_metrics.revenue_growth_3y),
        rev_growth_color = if let Some(g) = data.growth_metrics.revenue_growth_3y {
            if g > 0.0 { "positive" } else { "negative" }
        } else { "" },
        eps_growth = format_growth(data.growth_metrics.eps_growth_3y),
        eps_growth_color = if let Some(g) = data.growth_metrics.eps_growth_3y {
            if g > 0.0 { "positive" } else { "negative" }
        } else { "" },
        last_updated = data.last_updated.unwrap_or(Utc::now()).format("%Y-%m-%d %H:%M UTC").to_string(),
    )
}

pub fn html_index(tickers: &[String]) -> String {
    let mut ticker_cards = String::new();
    for ticker in tickers {
        ticker_cards.push_str(&format!(
            r#"<a href="/dashboard?ticker={ticker}" class="ticker-card">
                <div class="ticker-card-symbol">{ticker}</div>
                <div class="ticker-card-label">Click to analyze</div>
            </a>"#,
        ));
    }

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>StockRating - Stock Analysis Dashboard</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&display=swap" rel="stylesheet">
    <style>
        :root {{
            --bg-primary: #0b0f19;
            --bg-secondary: #111827;
            --bg-card: #1a2236;
            --bg-card-hover: #1f2a42;
            --border: #2d3a52;
            --border-light: #3b4a66;
            --text-primary: #f1f5f9;
            --text-secondary: #94a3b8;
            --text-muted: #64748b;
            --accent-blue: #3b82f6;
            --accent-blue-glow: rgba(59, 130, 246, 0.3);
        }}

        * {{ margin: 0; padding: 0; box-sizing: border-box; }}

        body {{
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            min-height: 100vh;
        }}

        .bg-grid {{
            position: fixed;
            top: 0; left: 0; right: 0; bottom: 0;
            background-image:
                linear-gradient(rgba(59, 130, 246, 0.03) 1px, transparent 1px),
                linear-gradient(90deg, rgba(59, 130, 246, 0.03) 1px, transparent 1px);
            background-size: 50px 50px;
            pointer-events: none;
            z-index: 0;
        }}

        .bg-glow {{
            position: fixed;
            top: -100px;
            left: 50%;
            transform: translateX(-50%);
            width: 1000px;
            height: 600px;
            background: radial-gradient(circle, rgba(59, 130, 246, 0.1) 0%, transparent 60%);
            pointer-events: none;
            z-index: 0;
        }}

        .container {{
            max-width: 1000px;
            margin: 0 auto;
            padding: 60px 24px;
            position: relative;
            z-index: 1;
        }}

        header {{
            text-align: center;
            margin-bottom: 60px;
        }}

        .logo {{
            display: inline-flex;
            align-items: center;
            gap: 16px;
            margin-bottom: 24px;
        }}

        .logo-icon {{
            width: 56px;
            height: 56px;
            background: linear-gradient(135deg, var(--accent-blue), #8b5cf6);
            border-radius: 16px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: 800;
            font-size: 1.5rem;
            color: white;
        }}

        .logo-text {{
            font-size: 2rem;
            font-weight: 800;
            letter-spacing: -0.03em;
        }}

        .logo-text span {{ color: var(--accent-blue); }}

        .subtitle {{
            font-size: 1.1rem;
            color: var(--text-secondary);
            margin-bottom: 40px;
        }}

        .ticker-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
        }}

        .ticker-card {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 16px;
            padding: 32px 24px;
            text-align: center;
            text-decoration: none;
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            position: relative;
            overflow: hidden;
        }}

        .ticker-card::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 3px;
            background: linear-gradient(90deg, var(--accent-blue), #8b5cf6);
            opacity: 0;
            transition: opacity 0.3s;
        }}

        .ticker-card:hover {{
            border-color: var(--accent-blue);
            background: var(--bg-card-hover);
            transform: translateY(-4px);
            box-shadow: 0 12px 40px rgba(0, 0, 0, 0.3), 0 0 0 1px var(--accent-blue-glow);
        }}

        .ticker-card:hover::before {{ opacity: 1; }}

        .ticker-card-symbol {{
            font-size: 1.5rem;
            font-weight: 700;
            margin-bottom: 8px;
            letter-spacing: 0.05em;
        }}

        .ticker-card-label {{
            font-size: 0.85rem;
            color: var(--text-muted);
        }}

        footer {{
            text-align: center;
            margin-top: 60px;
            padding-top: 32px;
            border-top: 1px solid var(--border);
            color: var(--text-muted);
            font-size: 0.85rem;
        }}
    </style>
</head>
<body>
    <div class="bg-grid"></div>
    <div class="bg-glow"></div>

    <div class="container">
        <header>
            <div class="logo">
                <div class="logo-icon">S</div>
                <div class="logo-text"><span>Stock</span>Rating</div>
            </div>
            <p class="subtitle">Select a stock ticker to view comprehensive analysis</p>
        </header>

        <div class="ticker-grid">
            {tickers}
        </div>

        <footer>
            <p>StockRating Dashboard v1.0 • Mock data provider • Not financial advice</p>
        </footer>
    </div>
</body>
</html>"#,
        tickers = ticker_cards,
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

async fn index_handler() -> Html<String> {
    let tickers: Vec<String> = vec![
        "AAPL".to_string(),
        "MSFT".to_string(),
        "GOOGL".to_string(),
        "TSLA".to_string(),
        "AMZN".to_string(),
    ];
    Html(html_index(&tickers))
}

async fn dashboard_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Html<String> {
    let ticker = params.ticker.to_uppercase();
    let data_provider = state.data_provider.read().unwrap();
    let result = match data_provider.get_stock_data(&ticker) {
        Some(data) => Html(html_page(&data)),
        None => {
            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{ticker} - StockRating</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap" rel="stylesheet">
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: 'Inter', sans-serif; background: #0b0f19; color: #f1f5f9; min-height: 100vh; display: flex; align-items: center; justify-content: center; }}
        .container {{ text-align: center; padding: 40px; }}
        .error-icon {{ font-size: 4rem; margin-bottom: 20px; }}
        h2 {{ font-size: 2rem; margin-bottom: 12px; }}
        p {{ color: #94a3b8; margin-bottom: 8px; font-size: 1.1rem; }}
        a {{ color: #3b82f6; text-decoration: none; font-weight: 600; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error-icon">⚠️</div>
        <h2>Ticker Not Found</h2>
        <p>Stock data for "{ticker}" is not available.</p>
        <p><a href="/">← Back to Dashboard</a></p>
    </div>
</body>
</html>"#,
                ticker = ticker,
            );
            Html(html)
        }
    };
    result
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
