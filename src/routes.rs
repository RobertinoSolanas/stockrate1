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
    pub providers: std::sync::Arc<std::sync::RwLock<Vec<Box<dyn StockDataProvider + Send + Sync>>>>,
}

#[derive(Deserialize)]
struct QueryParams {
    #[serde(rename = "ticker")]
    ticker: String,
    #[serde(rename = "provider")]
    provider: Option<String>,
}

#[derive(Deserialize)]
struct CompareParams {
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

pub fn health_score_label(score: f64, d_to_e: f64) -> (&'static str, &'static str) {
    let score = if score > 0.3 { 3.0 } else if score > 0.1 { 2.0 } else { 1.0 };
    let debt_score = if d_to_e < 0.5 { 3.0 } else if d_to_e < 1.0 { 2.0 } else { 1.0 };
    let total = score + debt_score;
    match total {
        5.0..=6.0 => ("Excellent", "#10b981"),
        3.0..=4.0 => ("Moderate", "#f59e0b"),
        _ => ("Weak", "#ef4444"),
    }
}

pub fn valuation_assessment(pe_ratio: Option<f64>, ev_ebitda: Option<f64>) -> (&'static str, &'static str) {
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

pub fn growth_assessment(rev: Option<f64>, eps: Option<f64>) -> (&'static str, &'static str) {
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

pub fn create_bar_chart(data_points: &[(String, f64, String)], colors: &[&str], title: &str) -> String {
    let max_val = data_points.iter().map(|(_, v, _)| v.abs()).fold(0.0f64, f64::max);
    if max_val == 0.0 {
        return String::new();
    }
    let chart_width = 400;
    let bar_height = 32.0;
    let row_height = 44.0;
    let height = (data_points.len() as f64 * row_height) as usize + 60;
    
    let font = "Inter,sans-serif";
    let txt_fill = "#94a3b8";
    
    for (i, (label, value, _color)) in data_points.iter().enumerate() {
        let y = 35.0 + i as f64 * row_height;
        let bar_width = ((*value / max_val) * (chart_width - 140) as f64) as f64;
        let display_val = if *value > 1.0 { format!("{:.1}", value) } else { format!("{:.2}", value) };
        let clr = colors.get(i).copied().unwrap_or("#3b82f6");
        
        let _svg_line = format!("<text x=\"0\" y=\"{}\" fill=\"#64748b\" font-size=\"11\" font-family=\"{}\">{}</text>\n            <rect x=\"80\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"4\" fill=\"{}\" opacity=\"0.85\"/>\n            <text x=\"{}\" y=\"{}\" fill=\"{}\" font-size=\"10\" font-family=\"{}\">{}</text>",
            y, font, label,
            y - 8.0,
            f64::max(bar_width, 2.0),
            bar_height - 16.0,
            clr,
            84.0 + f64::max(bar_width, 2.0),
            y,
            txt_fill, font, display_val);
        
        // SVG lines appended
    }
    
    let mut svg_result = format!("<svg viewBox=\"0 0 {chart_width} {height}\" width=\"100%\" style=\"max-width:{chart_width}px;min-height:{height}px\">\n    <text x=\"{}\" y=\"20\" text-anchor=\"middle\" fill=\"{}\" font-size=\"13\" font-weight=\"600\" font-family=\"{}\">{}</text>",
        chart_width/2, txt_fill, font, title);
    
    for (i, (label, value, _color)) in data_points.iter().enumerate() {
        let y = 35.0 + i as f64 * row_height;
        let bar_width = ((*value / max_val) * (chart_width - 140) as f64) as f64;
        let display_val = if *value > 1.0 { format!("{:.1}", value) } else { format!("{:.2}", value) };
        let clr = colors.get(i).copied().unwrap_or("#3b82f6");
        let txt_fine = "#94a3b8";
        svg_result.push_str(&format!("\n            <text x=\"0\" y=\"{}\" fill=\"#64748b\" font-size=\"11\" font-family=\"{}\">{}</text>", y, font, label));
        svg_result.push_str(&format!("\n            <rect x=\"80\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"4\" fill=\"{}\" opacity=\"0.85\"/>", y - 8.0, f64::max(bar_width, 2.0), bar_height - 16.0, clr));
        svg_result.push_str(&format!("\n            <text x=\"{}\" y=\"{}\" fill=\"{}\" font-size=\"10\" font-family=\"{}\">{}</text>", 84.0 + f64::max(bar_width, 2.0), y, txt_fine, font, display_val));
    }
    
    svg_result.push_str("</svg>");
    svg_result
}

pub fn create_comparison_chart(data_a: &[f64], data_b: &[f64], labels: &[&str], titles: &[&str]) -> String {
    let max_val = data_a.iter().chain(data_b.iter()).cloned().fold(0.0f64, f64::max);
    if max_val == 0.0 {
        return String::new();
    }
    let chart_width = 500;
    let bar_height = 18;
    let group_width = chart_width / labels.len();
    let height = labels.len() * 55 + 70;
    
    let font = "Inter,sans-serif";
    let mut svg_result = String::new();
    svg_result.push_str(&format!("<svg viewBox=\"0 0 {chart_width} {height}\" width=\"100%\" style=\"max-width:{chart_width}px;min-height:{height}px\">"));
    svg_result.push_str(&format!("\n    <rect x=\"0\" y=\"0\" width=\"{}\" height=\"28\" fill=\"#1a2236\" rx=\"8\"/>", chart_width));
    svg_result.push_str(&format!("\n    <circle cx=\"20\" cy=\"14\" r=\"6\" fill=\"#3b82f6\"/><text x=\"32\" y=\"18\" fill=\"#94a3b8\" font-size=\"10\" font-family=\"{}\">{}</text>", font, titles[0]));
    svg_result.push_str(&format!("\n    <circle cx=\"170\" cy=\"14\" r=\"6\" fill=\"#f59e0b\"/><text x=\"182\" y=\"18\" fill=\"#94a3b8\" font-size=\"10\" font-family=\"{}\">{}</text>", font, titles[1]));
    
    for (i, label) in labels.iter().enumerate() {
        let val_a = data_a.get(i).copied().unwrap_or(0.0);
        let val_b = data_b.get(i).copied().unwrap_or(0.0);
        let x_offset = i * group_width + group_width / 2;
        let y = 45.0 + i as f64 * 55.0;
        let bar_a_width = ((val_a / max_val) * (group_width / 2 - 10) as f64) as f64;
        let bar_b_width = ((val_b / max_val) * (group_width / 2 - 10) as f64) as f64;
        let clr_a = "#3b82f6";
        let clr_b = "#f59e0b";
        let clr_txt = "#cbd5e1";
        let yyy = y + 14.0;
        let yy22 = y + 18.0;
        let yy33 = y + 31.0;
        let xxo = x_offset as f64;
        let xxa = xxo - group_width as f64 / 4.0 + 5.0;
        let xxb = xxo + 5.0;
        let tt1 = xxo - group_width as f64 / 4.0 + 9.0 + f64::max(bar_a_width, 1.0);
        let tt2 = xxo + 9.0 + f64::max(bar_b_width, 1.0);
        svg_result.push_str(&format!("\n            <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" fill=\"#64748b\" font-size=\"10\" font-weight=\"500\" font-family=\"{}\">{}</text>", xxo, yyy, font, label));
        svg_result.push_str(&format!("\n            <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"{}\" opacity=\"0.8\"/>", xxa, yy22, f64::max(bar_a_width, 1.0), bar_height as f64, clr_a));
        svg_result.push_str(&format!("\n            <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"{}\" opacity=\"0.8\"/>", xxb, yy22, f64::max(bar_b_width, 1.0), bar_height as f64, clr_b));
        svg_result.push_str(&format!("\n            <text x=\"{}\" y=\"{}\" fill=\"{}\" font-size=\"9\" font-family=\"{}\">{}</text>", tt1, yy33, clr_txt, font, format!("{:.2}", val_a)));
        svg_result.push_str(&format!("\n            <text x=\"{}\" y=\"{}\" fill=\"{}\" font-size=\"9\" font-family=\"{}\">{}</text>", tt2, yy33, clr_txt, font, format!("{:.2}", val_b)));
    }
    
    svg_result.push_str("</svg>");
    svg_result
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
    let pe = data.valuation_ratios.pe_ratio.unwrap_or(0.0);
    let ev = data.valuation_ratios.ev_to_ebitda.unwrap_or(0.0);
    let rev_growth = data.growth_metrics.revenue_growth_3y.unwrap_or(0.0);
    let eps_growth = data.growth_metrics.eps_growth_3y.unwrap_or(0.0);

    let health_label = health_score_label(roe, d_to_e);
    let val_assessment = valuation_assessment(Some(pe), Some(ev));
    let growth_assessment = growth_assessment(Some(rev_growth), Some(eps_growth));

    let roe_pct = roe * 100.0;
    let fcf_formatted = format_market_cap(Some(fcf));

    let rec_bar_data = vec![
        ("P/E".to_string(), pe, "#3b82f6".to_string()),
        ("EV/EBITDA".to_string(), ev, "#8b5cf6".to_string()),
        ("P/B".to_string(), data.valuation_ratios.pb_ratio.unwrap_or(0.0), "#ec4899".to_string()),
    ];
    let health_bar_data = vec![
        ("ROE".to_string(), roe * 100.0, "#10b981".to_string()),
        ("Debt/Equity".to_string(), d_to_e, "#f59e0b".to_string()),
        ("Current Ratio".to_string(), current_ratio, "#3b82f6".to_string()),
    ];
    let growth_bar_data = vec![
        ("Revenue 3Y".to_string(), rev_growth * 100.0, "#22c55e".to_string()),
        ("EPS 3Y".to_string(), eps_growth * 100.0, "#3b82f6".to_string()),
    ];

    let rec_svg = create_bar_chart(&rec_bar_data, &["#3b82f6", "#8b5cf6", "#ec4899"], "Valuation Metrics");
    let health_svg = create_bar_chart(&health_bar_data, &["#10b981", "#f59e0b", "#3b82f6"], "Financial Health Metrics");
    let growth_svg = create_bar_chart(&growth_bar_data, &["#22c55e", "#3b82f6"], "Growth Metrics (3-Year Annualized)");

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{ticker} - StockRating</title>
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
            --orange: #f97316;
            --purple: #8b5cf6;
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

        header {{
            padding: 24px 0;
            border-bottom: 1px solid var(--border);
            margin-bottom: 32px;
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
            background: linear-gradient(135deg, var(--accent-blue), var(--purple));
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
        }}

        .ticker-company {{
            color: var(--text-secondary);
            font-size: 0.9rem;
        }}

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

        .provider-selector {{
            display: flex;
            gap: 8px;
            margin-bottom: 32px;
            align-items: center;
        }}

        .provider-label {{
            color: var(--text-muted);
            font-size: 0.85rem;
            font-weight: 500;
            margin-right: 8px;
        }}

        .provider-btn {{
            padding: 8px 16px;
            border-radius: 8px;
            border: 1px solid var(--border);
            background: var(--bg-card);
            color: var(--text-secondary);
            font-size: 0.85rem;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s;
            text-decoration: none;
        }}

        .provider-btn:hover {{
            border-color: var(--accent-blue);
            color: var(--text-primary);
        }}

        .provider-btn.active {{
            background: var(--accent-blue);
            border-color: var(--accent-blue);
            color: white;
        }}

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
            background: linear-gradient(90deg, var(--green), var(--accent-blue), var(--purple));
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

        .chart-section {{
            margin-bottom: 32px;
        }}

        .chart-grid {{
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 24px;
        }}

        @media (max-width: 1100px) {{
            .chart-grid {{ grid-template-columns: 1fr; }}
        }}

        .chart-card {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 16px;
            padding: 24px;
        }}

        .chart-title {{
            font-size: 0.9rem;
            font-weight: 600;
            color: var(--text-muted);
            text-transform: uppercase;
            letter-spacing: 0.08em;
            margin-bottom: 16px;
        }}

        footer {{
            text-align: center;
            padding: 40px 0;
            color: var(--text-muted);
            font-size: 0.85rem;
            border-top: 1px solid var(--border);
            margin-top: 40px;
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
            <a href="/" class="nav-link">Dashboard</a>
            <a href="/dashboard?ticker={ticker}" class="nav-link active">Analysis</a>
            <a href="/compare?ticker={ticker}" class="nav-link">Compare</a>
            <a href="/api/query?ticker={ticker}" class="nav-link" target="_blank">API</a>
        </div>

        <div class="provider-selector">
            <span class="provider-label">Provider:</span>
            <a href="/dashboard?ticker={ticker}" class="provider-btn active">MockDataProvider</a>
            <a href="/dashboard?ticker={ticker}&provider=second" class="provider-btn">SecondMockProvider</a>
            <a href="/dashboard?ticker={ticker}&provider=finnhub" class="provider-btn">FinnhubDataProvider</a>
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
                Provider: {provider_name} • Based on {analyst_count} analyst ratings
            </div>
        </div>

        <div class="card" style="margin-bottom:32px;display:flex;justify-content:space-between;align-items:center;">
            <div>
                <div class="card-title" style="margin-bottom:8px;">Health Score</div>
                <div style="font-size:2rem;font-weight:800;color:{health_color};">{health_score}/6</div>
            </div>
            <div>
                <div class="card-title" style="margin-bottom:8px;">Valuation</div>
                <div style="font-size:1.3rem;font-weight:700;color:{val_color};">{val_label}</div>
            </div>
            <div>
                <div class="card-title" style="margin-bottom:8px;">Growth</div>
                <div style="font-size:1.3rem;font-weight:700;color:{growth_color};">{growth_label}</div>
            </div>
            <div>
                <div class="card-title" style="margin-bottom:8px;">ROE</div>
                <div style="font-size:1.3rem;font-weight:700;">{roe:.1}%</div>
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
                    <span class="metric-value">{provider_name}</span>
                </div>
                <div class="metric-row">
                    <span class="metric-label">Recommendation</span>
                    <span class="metric-value" style="color: {rec_color};">{rec_text}</span>
                </div>
            </div>
        </div>

        <div class="chart-section">
            <div class="chart-grid">
                <div class="chart-card">
                    <div class="chart-title">Valuation Metrics</div>
                    {rec_svg}
                </div>
                <div class="chart-card">
                    <div class="chart-title">Financial Health Metrics</div>
                    {health_svg}
                </div>
                <div class="chart-card">
                    <div class="chart-title">Growth Metrics (3-Year Annualized)</div>
                    {growth_svg}
                </div>
            </div>
        </div>

        <footer>
            <p>StockRating Dashboard v2.0 • Provider: {provider_name} • Not financial advice</p>
        </footer>
    </div>
</body>
</html>"#,
        ticker = data.ticker,
        company_name = data.company_name,
        provider_name = &data.provider,
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
        pe_ratio = format_ratio(Some(pe)),
        forward_pe = format_ratio(data.valuation_ratios.forward_pe_ratio),
        ev_ebitda = format_ratio(Some(ev)),
        pb_ratio = format_ratio(data.valuation_ratios.pb_ratio),
        d_to_e = format_ratio(Some(d_to_e)),
        fcf_formatted = fcf_formatted,
        current_ratio = format_ratio(Some(current_ratio)),
        revenue_growth = format_growth(Some(rev_growth)),
        rev_growth_color = if rev_growth > 0.0 { "positive" } else { "negative" },
        eps_growth = format_growth(Some(eps_growth)),
        eps_growth_color = if eps_growth > 0.0 { "positive" } else { "negative" },
        last_updated = data.last_updated.unwrap_or(Utc::now()).format("%Y-%m-%d %H:%M UTC").to_string(),
        rec_svg = rec_svg,
        health_svg = health_svg,
        growth_svg = growth_svg,
    )
}

pub fn html_compare(data_a: &StockRatingData, data_b: &StockRatingData) -> String {
    let ticker = &data_a.ticker;
    let company_a = &data_a.company_name;
    let company_b = &data_b.company_name;

    let pe_a = data_a.valuation_ratios.pe_ratio.unwrap_or(0.0);
    let pe_b = data_b.valuation_ratios.pe_ratio.unwrap_or(0.0);
    let ev_a = data_a.valuation_ratios.ev_to_ebitda.unwrap_or(0.0);
    let ev_b = data_b.valuation_ratios.ev_to_ebitda.unwrap_or(0.0);
    let pb_a = data_a.valuation_ratios.pb_ratio.unwrap_or(0.0);
    let pb_b = data_b.valuation_ratios.pb_ratio.unwrap_or(0.0);
    
    let roe_a = data_a.financial_health.return_on_equity.unwrap_or(0.0) * 100.0;
    let roe_b = data_b.financial_health.return_on_equity.unwrap_or(0.0) * 100.0;
    let de_a = data_a.financial_health.debt_to_equity.unwrap_or(0.0);
    let de_b = data_b.financial_health.debt_to_equity.unwrap_or(0.0);
    let cr_a = data_a.financial_health.current_ratio.unwrap_or(0.0);
    let cr_b = data_b.financial_health.current_ratio.unwrap_or(0.0);
    
    let rg_a = data_a.growth_metrics.revenue_growth_3y.unwrap_or(0.0) * 100.0;
    let rg_b = data_b.growth_metrics.revenue_growth_3y.unwrap_or(0.0) * 100.0;
    let eg_a = data_a.growth_metrics.eps_growth_3y.unwrap_or(0.0) * 100.0;
    let eg_b = data_b.growth_metrics.eps_growth_3y.unwrap_or(0.0) * 100.0;

    let target_a = data_a.market_sentiment.target_price_consensus.unwrap_or(0.0);
    let target_b = data_b.market_sentiment.target_price_consensus.unwrap_or(0.0);
    let current_a = data_a.market_sentiment.current_price.unwrap_or(0.0);
    let current_b = data_b.market_sentiment.current_price.unwrap_or(0.0);

    let up_a = if current_a > 0.0 { ((target_a - current_a) / current_a) * 100.0 } else { 0.0 };
    let up_b = if current_b > 0.0 { ((target_b - current_b) / current_b) * 100.0 } else { 0.0 };

    let rec_a = format_recommendation(&data_a.market_sentiment.recommendation_consensus.clone().unwrap_or(Recommendation::Hold));
    let rec_b = format_recommendation(&data_b.market_sentiment.recommendation_consensus.clone().unwrap_or(Recommendation::Hold));

    let chart = create_comparison_chart(
        &[pe_a, roe_a, rg_a, target_a / if current_a > 0.0 { current_a } else { 1.0 }],
        &[pe_b, roe_b, rg_b, target_b / if current_b > 0.0 { current_b } else { 1.0 }],
        &["P/E", "ROE %", "Rev Growth %", "Target/Price"],
        &[&data_a.provider, &data_b.provider],
    );

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{ticker} - Compare Providers</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&display=swap" rel="stylesheet">
    <style>
        :root {{
            --bg-primary: #0b0f19;
            --bg-card: #1a2236;
            --border: #2d3a52;
            --text-primary: #f1f5f9;
            --text-secondary: #94a3b8;
            --text-muted: #64748b;
            --accent-blue: #3b82f6;
            --blue-soft: rgba(59, 130, 246, 0.15);
            --green-soft: rgba(16, 185, 129, 0.15);
            --yellow-soft: rgba(245, 158, 11, 0.15);
            --green: #10b981;
            --yellow: #f59e0b;
        }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: 'Inter', sans-serif; background: var(--bg-primary); color: var(--text-primary); min-height: 100vh; }}
        .container {{ max-width: 1200px; margin: 0 auto; padding: 24px; }}
        header {{ padding: 24px 0; border-bottom: 1px solid var(--border); margin-bottom: 32px; }}
        .logo {{ display: flex; align-items: center; gap: 12px; margin-bottom: 16px; }}
        .logo-icon {{ width: 40px; height: 40px; background: linear-gradient(135deg, var(--accent-blue), #8b5cf6); border-radius: 10px; display: flex; align-items: center; justify-content: center; font-weight: 800; font-size: 1.1rem; color: white; }}
        .logo-text {{ font-size: 1.3rem; font-weight: 700; }}
        .logo-text span {{ color: var(--accent-blue); }}
        .ticker-badge {{ display: inline-flex; align-items: center; gap: 12px; padding: 10px 20px; background: var(--bg-card); border: 1px solid var(--border); border-radius: 12px; }}
        .nav-bar {{ display: flex; gap: 8px; margin-bottom: 32px; padding: 6px; background: #111827; border-radius: 14px; border: 1px solid var(--border); width: fit-content; }}
        .nav-link {{ padding: 10px 20px; border-radius: 10px; text-decoration: none; color: var(--text-secondary); font-weight: 500; font-size: 0.9rem; }}
        .nav-link:hover {{ color: var(--text-primary); background: var(--bg-card); }}
        .nav-link.active {{ background: var(--accent-blue); color: white; }}
        .compare-header {{ text-align: center; margin-bottom: 40px; }}
        .compare-ticker {{ font-size: 2.5rem; font-weight: 800; margin-bottom: 8px; }}
        .compare-companies {{ color: var(--text-muted); font-size: 0.95rem; }}
        .compare-table {{ width: 100%; border-collapse: separate; border-spacing: 0; margin-bottom: 40px; }}
        .compare-table th {{ background: var(--bg-card); padding: 16px; text-align: left; font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.08em; color: var(--text-muted); border-bottom: 1px solid var(--border); }}
        .compare-table th:first-child {{ border-radius: 12px 0 0 0; }}
        .compare-table th:last-child {{ border-radius: 0 12px 0 0; }}
        .compare-table td {{ padding: 16px; border-bottom: 1px solid var(--border); font-size: 0.95rem; }}
        .compare-table tr:last-child td {{ border-bottom: none; }}
        .compare-table td:first-child {{ color: var(--text-muted); font-weight: 500; }}
        .provider-badge {{ display: inline-flex; align-items: center; gap: 8px; padding: 6px 14px; background: var(--bg-card); border: 1px solid var(--border); border-radius: 8px; font-weight: 600; font-size: 0.9rem; }}
        .provider-badge.blue {{ border-color: var(--accent-blue); color: var(--accent-blue); }}
        .provider-badge.yellow {{ border-color: var(--yellow); color: var(--yellow); }}
        .chart-section {{ margin-bottom: 40px; }}
        .chart-card {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: 16px; padding: 24px; }}
        .chart-title {{ font-size: 1rem; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.08em; margin-bottom: 20px; }}
        .footer {{ text-align: center; padding: 40px 0; color: var(--text-muted); font-size: 0.85rem; border-top: 1px solid var(--border); margin-top: 40px; }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <div class="logo">
                <div class="logo-icon">S</div>
                <div class="logo-text"><span>Stock</span>Rating</div>
            </div>
            <div class="ticker-badge">
                <span style="font-weight:700;font-size:1.1rem;">{ticker}</span>
            </div>
        </header>

        <div class="nav-bar">
            <a href="/" class="nav-link">Dashboard</a>
            <a href="/dashboard?ticker={ticker}" class="nav-link">Analysis</a>
            <a href="/compare?ticker={ticker}" class="nav-link active">Compare</a>
        </div>

        <div class="compare-header">
            <div class="compare-ticker">{ticker}</div>
            <div class="compare-companies">{company_a} ({provider_a}) vs {company_b} ({provider_b})</div>
        </div>

        <table class="compare-table">
            <thead>
                <tr>
                    <th>Metric</th>
                    <th><span class="provider-badge blue">{provider_a}</span></th>
                    <th><span class="provider-badge yellow">{provider_b}</span></th>
                </tr>
            </thead>
            <tbody>
                <tr><td>Recommendation</td><td>{rec_a}</td><td>{rec_b}</td></tr>
                <tr><td>Current Price</td><td>${current_a:.2}</td><td>${current_b:.2}</td></tr>
                <tr><td>Target Price</td><td>${target_a:.2}</td><td>${target_b:.2}</td></tr>
                <tr><td>Upside/Downside</td><td style="color:{up_a_color_a}">{up_a:+.1}%</td><td style="color:{up_a_color_b}">{up_b:+.1}%</td></tr>
                <tr><td>P/E Ratio</td><td>{pe_a:.2}</td><td>{pe_b:.2}</td></tr>
                <tr><td>EV/EBITDA</td><td>{ev_a:.2}</td><td>{ev_b:.2}</td></tr>
                <tr><td>P/B Ratio</td><td>{pb_a:.2}</td><td>{pb_b:.2}</td></tr>
                <tr><td>ROE</td><td>{roe_a:.1}%</td><td>{roe_b:.1}%</td></tr>
                <tr><td>Debt-to-Equity</td><td>{de_a:.2}</td><td>{de_b:.2}</td></tr>
                <tr><td>Current Ratio</td><td>{cr_a:.2}</td><td>{cr_b:.2}</td></tr>
                <tr><td>Revenue Growth (3Y)</td><td>{rg_a:+.1}%</td><td>{rg_b:+.1}%</td></tr>
                <tr><td>EPS Growth (3Y)</td><td>{eg_a:+.1}%</td><td>{eg_b:+.1}%</td></tr>
            </tbody>
        </table>

        <div class="chart-section">
            <div class="chart-card">
                <div class="chart-title">Side-by-Side Comparison</div>
                {chart}
            </div>
        </div>

        <div class="footer">
            <p>StockRating Compare View v2.0 • Provider comparison • Not financial advice</p>
        </div>
    </div>
</body>
</html>"#,
        ticker = ticker,
        provider_a = &data_a.provider,
        company_a = company_a,
        provider_b = &data_b.provider,
        company_b = company_b,
        rec_a = rec_a,
        rec_b = rec_b,
        current_a = current_a,
        current_b = current_b,
        target_a = target_a,
        target_b = target_b,
        up_a = up_a,
        up_b = up_b,
        up_a_color_a = if up_a >= 0.0 { "var(--green)" } else { "#ef4444" },
        up_a_color_b = if up_b >= 0.0 { "var(--green)" } else { "#ef4444" },
        pe_a = pe_a,
        pe_b = pe_b,
        ev_a = ev_a,
        ev_b = ev_b,
        pb_a = pb_a,
        pb_b = pb_b,
        roe_a = roe_a,
        roe_b = roe_b,
        de_a = de_a,
        de_b = de_b,
        cr_a = cr_a,
        cr_b = cr_b,
        rg_a = rg_a,
        rg_b = rg_b,
        eg_a = eg_a,
        eg_b = eg_b,
        chart = chart,
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
    <title>StockRating - Dashboard</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&display=swap" rel="stylesheet">
    <style>
        :root {{
            --bg-primary: #0b0f19;
            --bg-card: #1a2236;
            --border: #2d3a52;
            --text-primary: #f1f5f9;
            --text-muted: #64748b;
            --accent-blue: #3b82f6;
        }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: 'Inter', sans-serif; background: var(--bg-primary); color: var(--text-primary); min-height: 100vh; }}
        .container {{ max-width: 1000px; margin: 0 auto; padding: 60px 24px; }}
        header {{ text-align: center; margin-bottom: 60px; }}
        .logo {{ display: inline-flex; align-items: center; gap: 16px; margin-bottom: 24px; }}
        .logo-icon {{ width: 56px; height: 56px; background: linear-gradient(135deg, var(--accent-blue), #8b5cf6); border-radius: 16px; display: flex; align-items: center; justify-content: center; font-weight: 800; font-size: 1.5rem; color: white; }}
        .logo-text {{ font-size: 2rem; font-weight: 800; letter-spacing: -0.03em; }}
        .logo-text span {{ color: var(--accent-blue); }}
        .subtitle {{ color: var(--text-muted); margin-bottom: 40px; font-size: 1.1rem; }}
        .ticker-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; }}
        .ticker-card {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: 16px; padding: 32px 24px; text-align: center; text-decoration: none; transition: all 0.3s; }}
        .ticker-card:hover {{ border-color: var(--accent-blue); background: #1f2a42; transform: translateY(-4px); }}
        .ticker-card-symbol {{ font-size: 1.5rem; font-weight: 700; margin-bottom: 8px; }}
        .ticker-card-label {{ font-size: 0.85rem; color: var(--text-muted); }}
        footer {{ text-align: center; margin-top: 60px; padding-top: 32px; border-top: 1px solid var(--border); color: var(--text-muted); font-size: 0.85rem; }}
    </style>
</head>
<body>
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
            <p>StockRating Dashboard v2.0 • Multi-Provider • Not financial advice</p>
        </footer>
    </div>
</body>
</html>"#,
        tickers = ticker_cards,
    )
}

pub fn setup_router(providers: Vec<Box<dyn StockDataProvider + Send + Sync>>) -> Router {
    let state = AppState {
        providers: std::sync::Arc::new(std::sync::RwLock::new(providers)),
    };

    Router::new()
        .route("/", get(index_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/compare", get(compare_handler))
        .route("/api/query", get(api_query_handler))
        .with_state(state)
}

async fn index_handler() -> Html<String> {
    let tickers: Vec<String> = vec![
        "AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string(), "TSLA".to_string(),
        "AMZN".to_string(), "NVDA".to_string(), "META".to_string(), "AMD".to_string(),
    ];
    Html(html_index(&tickers))
}

async fn dashboard_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Html<String> {
    let ticker = params.ticker.to_uppercase();
    let provider_name = params.provider.as_deref().unwrap_or("mock");
    let providers = state.providers.read().unwrap();
    
    let data_provider = match provider_name {
        "second" => providers.iter().find(|p| p.provider_name() == "SecondMockDataProvider"),
        "finnhub" => providers.iter().find(|p| p.provider_name() == "FinnhubDataProvider"),
        _ => providers.iter().find(|p| p.provider_name() == "MockDataProvider"),
    };

    let result = match data_provider {
        Some(dp) => {
            let data = (**dp).get_stock_data(&ticker);
            match data {
                Some(stock_data) => Html(html_page(&stock_data)),
                None => Html(format!(
                    r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>{ticker} - StockRating</title><link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap" rel="stylesheet"><style>*{{margin:0;padding:0;box-sizing:border-box}}body{{font-family:'Inter',sans-serif;background:#0b0f19;color:#f1f5f9;min-height:100vh;display:flex;align-items:center;justify-content:center}}.container{{text-align:center;padding:40px}}h2{{font-size:2rem;margin-bottom:12px}}p{{color:#94a3b8;margin-bottom:8px;font-size:1.1rem}}a{{color:#3b82f6;text-decoration:none;font-weight:600}}a:hover{{text-decoration:underline}}</style></head><body><div class="container"><div style="font-size:4rem;margin-bottom:20px">&#9888;</div><h2>Ticker Not Found</h2><p>Stock data for "{ticker}" is not available in {provider_name} provider.</p><p><a href="/">&#8592; Back to Dashboard</a></p></div></body></html>"#,
                    ticker = ticker,
                    provider_name = dp.provider_name(),
                )),
            }
        }
        None => Html(format!(
            r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>StockRating</title><link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap" rel="stylesheet"><style>*{{margin:0;padding:0;box-sizing:border-box}}body{{font-family:'Inter',sans-serif;background:#0b0f19;color:#f1f5f9;min-height:100vh;display:flex;align-items:center;justify-content:center}}.container{{text-align:center;padding:40px}}h2{{font-size:2rem;margin-bottom:12px}}p{{color:#94a3b8;margin-bottom:8px;font-size:1.1rem}}a{{color:#3b82f6;text-decoration:none;font-weight:600}}</style></head><body><div class="container"><h2>No Provider Found</h2><p>Provider "{provider_name}" not available.</p><p><a href="/">&#8592; Back to Dashboard</a></p></div></body></html>"#,
            provider_name = provider_name,
        )),
    };
    result
}

async fn compare_handler(
    State(state): State<AppState>,
    Query(params): Query<CompareParams>,
) -> Html<String> {
    let ticker = params.ticker.to_uppercase();
    let providers = state.providers.read().unwrap();
    
    let mock_data = providers.iter().find(|p| p.provider_name() == "MockDataProvider")
        .map(|p| (**p).get_stock_data(&ticker))
        .flatten();
    let second_data = providers.iter().find(|p| p.provider_name() == "SecondMockDataProvider")
        .map(|p| (**p).get_stock_data(&ticker))
        .flatten();
    let finnhub_data = providers.iter().find(|p| p.provider_name() == "FinnhubDataProvider")
        .map(|p| (**p).get_stock_data(&ticker))
        .flatten();

    // Prefer MockDataProvider vs FinnhubDataProvider if Finnhub available, otherwise Mock vs Second
    if let (Some(a), Some(b)) = (mock_data.clone(), finnhub_data) {
        Html(html_compare(&a, &b))
    } else if let (Some(a), Some(b)) = (mock_data, second_data) {
        Html(html_compare(&a, &b))
    } else {
        Html(format!(
            r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>{ticker} - Compare</title><link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap" rel="stylesheet"><style>*{{margin:0;padding:0;box-sizing:border-box}}body{{font-family:'Inter',sans-serif;background:#0b0f19;color:#f1f5f9;min-height:100vh;display:flex;align-items:center;justify-content:center}}.container{{text-align:center;padding:40px}}h2{{font-size:2rem;margin-bottom:12px}}p{{color:#94a3b8;margin-bottom:8px;font-size:1.1rem}}a{{color:#3b82f6;text-decoration:none;font-weight:600}}</style></head><body><div class="container"><h2>Compare Unavailable</h2><p>Both providers must have data for "{ticker}" to compare.</p><p><a href="/dashboard?ticker={ticker}">&#8592; View Single Provider</a></p></div></body></html>"#,
            ticker = ticker,
        ))
    }
}

async fn api_query_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<StockRatingData>, (StatusCode, String)> {
    let ticker = params.ticker;
    let provider_name = params.provider.as_deref().unwrap_or("mock");
    let providers = state.providers.read().unwrap();
    
    let data_provider = match provider_name {
        "second" => providers.iter().find(|p| p.provider_name() == "SecondMockDataProvider"),
        "finnhub" => providers.iter().find(|p| p.provider_name() == "FinnhubDataProvider"),
        _ => providers.iter().find(|p| p.provider_name() == "MockDataProvider"),
    };

    match data_provider {
        Some(dp) => match (**dp).get_stock_data(&ticker) {
            Some(data) => Ok(Json(data)),
            None => Err((StatusCode::NOT_FOUND, format!("Ticker '{}' not found", ticker))),
        },
        None => Err((StatusCode::NOT_FOUND, format!("Provider '{}' not found", provider_name))),
    }
}
