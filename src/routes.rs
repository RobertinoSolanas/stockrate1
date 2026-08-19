use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::get,
    Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::models::{Recommendation, StockRatingData, StockSearchResult};
use crate::providers::StockDataProvider;
use crate::services::stock_aggregation::*;

#[derive(Clone)]
pub struct AppState {
    pub providers: std::sync::Arc<std::sync::RwLock<Vec<Box<dyn StockDataProvider + Send + Sync>>>>,
}

fn extract_provider_from_params(params: &std::collections::HashMap<String, String>) -> &'static str {
    match params.get("provider").map(|s| s.as_str()) {
        Some("second") => "SecondMockProvider",
        Some("finnhub") => "FinnhubDataProvider",
        _ => "MockDataProvider",
    }
}

fn extract_ticker_from_params(params: &std::collections::HashMap<String, String>) -> String {
    params.get("ticker").cloned().unwrap_or_default().to_uppercase()
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

#[allow(dead_code)]
pub fn create_all_stocks_svg_chart(chart_groups: &Vec<ChartGroup>, _tickers: &[String]) -> String {
    if chart_groups.is_empty() {
        return String::new();
    }

    let font = "Inter,sans-serif";
    let txt_fill = "#94a3b8";
    let mut svg_parts: Vec<String> = Vec::new();

    for group in chart_groups {
        if group.entries.is_empty() {
            continue;
        }

        let max_val = group.entries.iter().map(|e| e.value.abs()).fold(0.0f64, f64::max);
        if max_val == 0.0 {
            continue;
        }

        let chart_width = 400;
        let row_height = 44.0;
        let bar_height = 32.0;
        let entries = &group.entries;
        let height = (entries.len() as f64 * row_height) as usize + 80;

        let mut svg = format!("<svg viewBox=\"0 0 {} {}\" width=\"100%\" style=\"max-width:{}px;min-height:{}px\">\n", chart_width, height, chart_width, height);
        svg.push_str(&format!("    <rect x=\"0\" y=\"0\" width=\"{}\" height=\"30\" fill=\"#111827\" rx=\"8\"/>\n", chart_width));
        svg.push_str(&format!("    <text x=\"{}\" y=\"18\" text-anchor=\"middle\" fill=\"{}\" font-size=\"11\" font-weight=\"600\" font-family=\"{}\">{}</text>\n", chart_width/2, txt_fill, font, group.label));

        for (i, entry) in entries.iter().enumerate() {
            let y = 40.0 + i as f64 * row_height;
            let bar_width = ((entry.value.abs() / max_val) * (chart_width - 170) as f64) as f64;
            let display_val = if entry.value.abs() > 1.0 {
                format!("{:.1}", entry.value)
            } else {
                format!("{:.2}", entry.value)
            };
            let clr = &group.color;

            svg.push_str(&format!("\n            <text x=\"0\" y=\"{}\" fill=\"#64748b\" font-size=\"10\" font-family=\"{}\">{}</text>", y, font, entry.ticker));
            svg.push_str(&format!("\n            <rect x=\"75\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"4\" fill=\"{}\" opacity=\"0.85\"/>", y - 8.0, f64::max(bar_width, 2.0), bar_height - 16.0, clr));
            svg.push_str(&format!("\n            <text x=\"{}\" y=\"{}\" fill=\"#94a3b8\" font-size=\"9\" font-family=\"{}\">{}</text>", 79.0 + f64::max(bar_width, 2.0), y, font, display_val));
            svg.push_str(&format!("\n            <text x=\"{}\" y=\"{}\" fill=\"#475569\" font-size=\"8\" font-family=\"{}\">{}</text>", 79.0 + f64::max(bar_width, 2.0) + 70.0, y, font, entry.provider));
        }

        svg.push_str("\n</svg>");
        svg_parts.push(svg);
    }

    svg_parts.join("\n\n")
}

pub fn html_index(chart_groups: &[ChartGroup], tickers: &[String]) -> String {
    let mut ticker_cards = String::new();
    for ticker in tickers {
        ticker_cards.push_str(&format!(
            r#"<a href="/dashboard?ticker={ticker}" class="ticker-card">
                <div class="ticker-card-symbol">{ticker}</div>
                <div class="ticker-card-label">Click to analyze</div>
            </a>"#,
        ));
    }

    let mut charts_html = String::new();
    for group in chart_groups {
        if group.entries.is_empty() {
            continue;
        }
        let entries_html = group.entries.iter().map(|e| {
            format!(
                r#"<div class="chart-row">
                    <span class="chart-ticker">{ticker}</span>
                    <span class="chart-bar" style="width:{width}px;background:{color};"></span>
                    <span class="chart-value">{value}{unit}</span>
                    <span class="chart-provider">{provider}</span>
                </div>"#,
                ticker = e.ticker,
                width = (e.value.abs() * 2.0).min(200.0) as usize,
                color = group.color,
                value = if e.value.abs() > 1.0 { format!("{:.1}", e.value) } else { format!("{:.2}", e.value) },
                unit = group.unit,
                provider = e.provider,
            )
        }).collect::<String>();

        charts_html.push_str(&format!(r#"<div class="chart-section-card">
            <div class="chart-section-title">{label} {unit}</div>
            {entries}
        </div>"#,
            label = group.label,
            unit = if group.unit.is_empty() { "" } else { &group.unit },
            entries = entries_html,
        ));
    }

    let has_charts = !charts_html.is_empty();

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
            --purple: #8b5cf6;
            --purple-soft: rgba(139, 92, 246, 0.15);
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

        .page-header {{
            margin-bottom: 32px;
        }}

        .page-title {{
            font-size: 2rem;
            font-weight: 800;
            margin-bottom: 8px;
        }}

        .page-subtitle {{
            color: var(--text-muted);
            font-size: 0.95rem;
        }}

        .config-bar {{
            display: flex;
            gap: 12px;
            align-items: center;
            margin-bottom: 32px;
            padding: 16px;
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 12px;
            flex-wrap: wrap;
        }}

        .config-label {{
            color: var(--text-muted);
            font-size: 0.85rem;
            font-weight: 600;
            margin-right: 4px;
        }}

        .config-select {{
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 8px 12px;
            color: var(--text-primary);
            font-size: 0.85rem;
            font-family: 'Inter', sans-serif;
            cursor: pointer;
            outline: none;
        }}

        .config-select:focus {{
            border-color: var(--accent-blue);
        }}

        .config-btn {{
            padding: 8px 16px;
            border-radius: 8px;
            border: 1px solid var(--accent-blue);
            background: transparent;
            color: var(--accent-blue);
            font-size: 0.85rem;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s;
            text-decoration: none;
            font-family: 'Inter', sans-serif;
        }}

        .config-btn:hover {{
            background: var(--accent-blue);
            color: white;
        }}

        .metrics-summary {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
            gap: 16px;
            margin-bottom: 32px;
        }}

        .metric-card {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 20px;
            text-align: center;
        }}

        .metric-card-label {{
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.1em;
            color: var(--text-muted);
            margin-bottom: 8px;
        }}

        .metric-card-value {{
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--text-primary);
        }}

        .charts-container {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
            gap: 24px;
            margin-bottom: 48px;
        }}

        @media (max-width: 1100px) {{
            .charts-container {{ grid-template-columns: 1fr; }}
        }}

        .chart-section-card {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 16px;
            padding: 24px;
        }}

        .chart-section-title {{
            font-size: 0.85rem;
            font-weight: 600;
            color: var(--text-muted);
            text-transform: uppercase;
            letter-spacing: 0.08em;
            margin-bottom: 20px;
            padding-bottom: 12px;
            border-bottom: 1px solid var(--border);
        }}

        .chart-row {{
            display: flex;
            align-items: center;
            padding: 6px 0;
            gap: 12px;
            font-size: 0.85rem;
        }}

        .chart-ticker {{
            font-weight: 600;
            color: var(--text-primary);
            min-width: 50px;
        }}

        .chart-bar {{
            height: 20px;
            border-radius: 4px;
            transition: width 0.3s ease;
            min-width: 2px;
        }}

        .chart-value {{
            font-weight: 600;
            color: var(--text-secondary);
            min-width: 80px;
            text-align: right;
            font-variant-numeric: tabular-nums;
        }}

        .chart-provider {{
            color: var(--text-muted);
            font-size: 0.75rem;
            min-width: 100px;
        }}

        .ticker-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 48px;
        }}

        .ticker-card {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 16px;
            padding: 32px 24px;
            text-align: center;
            text-decoration: none;
            transition: all 0.3s;
        }}

        .ticker-card:hover {{
            border-color: var(--accent-blue);
            background: var(--bg-card-hover);
            transform: translateY(-4px);
        }}

        .ticker-card-symbol {{
            font-size: 1.5rem;
            font-weight: 700;
            margin-bottom: 8px;
        }}

        .ticker-card-label {{
            font-size: 0.85rem;
            color: var(--text-muted);
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
            </div>
        </header>

        <div class="nav-bar">
            <a href="/" class="nav-link active">Dashboard</a>
            <a href="/dashboard?ticker=AAPL" class="nav-link">Analysis</a>
            <a href="/compare?ticker=AAPL" class="nav-link">Compare</a>
            <a href="/portfolio" class="nav-link">Portfolio</a>
            <a href="/finnhub" class="nav-link">Finnhub</a>
        </div>

        <div class="page-header">
            <h1 class="page-title">Market Overview</h1>
            <p class="page-subtitle">Aggregated ratings across all data providers</p>
        </div>

        {config_bar}

        {charts_section}

        <div class="ticker-grid">
            {tickers}
        </div>

        <footer>
            <p>StockRating Dashboard v3.0 • Multi-Provider Aggregation • Not financial advice</p>
        </footer>
    </div>
</body>
</html>"#,
        config_bar = if has_charts { format!(r#"<div class="config-bar">
                <span class="config-label">Metrics:</span>
                <select class="config-select" id="metrics-select" onchange="window.location.href='/?metrics='+this.value">
                    <option value="pe,roe,revenue_growth,upside" selected>All Metrics</option>
                    <option value="pe">P/E Only</option>
                    <option value="roe">ROE Only</option>
                    <option value="revenue_growth">Growth Only</option>
                    <option value="upside">Upside Only</option>
                </select>
                <span class="config-label">Chart:</span>
                <select class="config-select" id="chart-select" onchange="window.location.href='/?metrics='+document.getElementById('metrics-select').value+'&chart_type='+this.value">
                    <option value="bar">Bar Chart</option>
                    <option value="horizontal">Horizontal</option>
                </select>
                <a href="/api/all-stocks" class="config-btn" target="_blank">API</a>
                <a href="/api/all-tickers" class="config-btn" target="_blank">Tickers API</a>
            </div>"#) } else { String::new() },
        charts_section = if has_charts { format!(r#"<div class="charts-container">
                {charts_html}
            </div>"#, charts_html = charts_html) } else { String::new() },
        tickers = ticker_cards,
    )
}

pub fn setup_router(
    providers: std::sync::Arc<
        std::sync::RwLock<Vec<Box<dyn StockDataProvider + Send + Sync>>>,
    >,
) -> Router {
    let state = AppState { providers };

    Router::new()
        .route("/", get(index_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/compare", get(compare_handler))
        .route("/portfolio", get(portfolio_handler))
        .route("/finnhub", get(finnhub_handler))
        .route("/api/query", get(api_query_handler))
        .route("/api/all-stocks", get(all_stocks_handler))
        .route("/api/all-tickers", get(all_tickers_handler))
        .route("/api/chart", get(chart_api_handler))
        .route("/api/finnhub/search", get(api_finnhub_search_handler))
        .route("/api/finnhub/tickers", get(api_finnhub_tickers_handler))
        .with_state(state)
}

pub(crate) async fn index_handler(
    State(state): State<AppState>,
) -> Html<String> {
    let service = StockAggregationService::new(state.providers.clone());
    let config = StockAggregationService::default_config();
    let data = service.get_aggregated_data(&config);

    Html(html_index(&data.chart_groups, &data.tickers))
}

async fn dashboard_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let ticker = extract_ticker_from_params(&params);
    let provider_name = extract_provider_from_params(&params);
    let providers = state.providers.read().unwrap();

    let data_provider = providers.iter()
        .find(|p| p.provider_name() == provider_name);

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
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let ticker = extract_ticker_from_params(&params);
    let providers = state.providers.read().unwrap();

    let mock_data = providers.iter()
        .find(|p| p.provider_name() == "MockDataProvider")
        .and_then(|p| (**p).get_stock_data(&ticker));
    let second_data = providers.iter()
        .find(|p| p.provider_name() == "SecondMockProvider")
        .and_then(|p| (**p).get_stock_data(&ticker));
    let finnhub_data = providers.iter()
        .find(|p| p.provider_name() == "FinnhubDataProvider")
        .and_then(|p| (**p).get_stock_data(&ticker));

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

// ---------------------------------------------------------------------------
// Finnhub page: dedicated view for the live Finnhub provider with search,
// a filterable ticker grid, and a full detail panel for a selected ticker.
// ---------------------------------------------------------------------------

async fn finnhub_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let providers = state.providers.read().unwrap();
    let requested = extract_ticker_from_params(&params);

    match providers.iter().find(|p| p.provider_name() == "FinnhubDataProvider") {
        None => Html(html_finnhub_unavailable()),
        Some(fp) => {
            let tickers = fp.list_supported_tickers();
            let selected = if requested.is_empty() {
                None
            } else {
                (**fp).get_stock_data(&requested)
            };
            Html(html_finnhub(&tickers, &requested, selected.as_ref()))
        }
    }
}

// ---------------------------------------------------------------------------
// Finnhub JSON API: symbol search + full alphabetical ticker universe.
// Both return 503 while the Finnhub provider is not configured.
// ---------------------------------------------------------------------------

async fn api_finnhub_search_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<StockSearchResult>>, (StatusCode, String)> {
    let query = params.get("q").cloned().unwrap_or_default();
    let providers = state.providers.read().unwrap();
    match providers.iter().find(|p| p.provider_name() == "FinnhubDataProvider") {
        Some(fp) => Ok(Json(fp.search_symbols(&query))),
        None => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "FinnhubDataProvider is not configured (missing API key)".to_string(),
        )),
    }
}

async fn api_finnhub_tickers_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let providers = state.providers.read().unwrap();
    match providers.iter().find(|p| p.provider_name() == "FinnhubDataProvider") {
        Some(fp) => Ok(Json(fp.list_all_tickers())),
        None => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "FinnhubDataProvider is not configured (missing API key)".to_string(),
        )),
    }
}

fn html_finnhub_unavailable() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Finnhub - StockRating</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap" rel="stylesheet">
    <style>*{{margin:0;padding:0;box-sizing:border-box}}body{{font-family:'Inter',sans-serif;background:#0b0f19;color:#f1f5f9;min-height:100vh;display:flex;align-items:center;justify-content:center}}.container{{text-align:center;padding:40px;max-width:560px}}h2{{font-size:1.8rem;margin-bottom:12px}}p{{color:#94a3b8;margin-bottom:10px;line-height:1.6}}a{{color:#3b82f6;text-decoration:none;font-weight:600}}code{{background:#1a2236;padding:2px 8px;border-radius:6px;color:#e2e8f0}}</style>
</head>
<body><div class="container"><div style="font-size:3rem;margin-bottom:16px">&#128202;</div><h2>Finnhub not configured</h2><p>The live <strong>FinnhubDataProvider</strong> is disabled because no API key was found.</p><p>Set the <code>FINNHUB_API_KEY</code> environment variable, or add <code>FINNHUB_API_KEY=your_key</code> to <code>resources/credentials.txt</code>, then restart the app.</p><p><a href="/">&#8592; Back to Dashboard</a></p></div></body>
</html>"#.to_string()
}

fn render_finnhub_detail(data: &StockRatingData) -> String {
    let rec = data
        .market_sentiment
        .recommendation_consensus
        .clone()
        .unwrap_or(Recommendation::Hold);
    let rec_text = format_recommendation(&rec);
    let rec_color = match rec {
        Recommendation::StrongBuy => "#10b981",
        Recommendation::Buy => "#22c55e",
        Recommendation::Hold => "#f59e0b",
        Recommendation::Sell => "#f97316",
        Recommendation::StrongSell => "#ef4444",
    };

    let price = data
        .market_sentiment
        .current_price
        .map(|p| format!("${:.2}", p))
        .unwrap_or_else(|| "N/A".to_string());
    let target = data
        .market_sentiment
        .target_price_consensus
        .map(|p| format!("${:.2}", p))
        .unwrap_or_else(|| "N/A".to_string());
    let (upside, upside_class) = match (data.market_sentiment.target_price_consensus, data.market_sentiment.current_price) {
        (Some(t), Some(c)) if c > 0.0 => {
            let u = ((t - c) / c) * 100.0;
            (format!("{:+.1}%", u), if u >= 0.0 { "up" } else { "down" })
        }
        _ => ("N/A".to_string(), "muted"),
    };
    let analysts = data
        .market_sentiment
        .analyst_count
        .map(|a| a.to_string())
        .unwrap_or_else(|| "N/A".to_string());
    let last_updated = data
        .last_updated
        .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "N/A".to_string());

    let pe = data.valuation_ratios.pe_ratio.unwrap_or(0.0);
    let fpe = data.valuation_ratios.forward_pe_ratio.unwrap_or(0.0);
    let ev = data.valuation_ratios.ev_to_ebitda.unwrap_or(0.0);
    let pb = data.valuation_ratios.pb_ratio.unwrap_or(0.0);
    let roe = data.financial_health.return_on_equity.unwrap_or(0.0) * 100.0;
    let d2e = data.financial_health.debt_to_equity.unwrap_or(0.0);
    let cr = data.financial_health.current_ratio.unwrap_or(0.0);
    let rev = data.growth_metrics.revenue_growth_3y.unwrap_or(0.0) * 100.0;
    let eps = data.growth_metrics.eps_growth_3y.unwrap_or(0.0) * 100.0;

    let val_chart = create_bar_chart(
        &[("P/E".to_string(), pe, "#3b82f6".to_string()),
          ("Fwd P/E".to_string(), fpe, "#8b5cf6".to_string()),
          ("EV/EBITDA".to_string(), ev, "#ec4899".to_string()),
          ("P/B".to_string(), pb, "#06b6d4".to_string())],
        &["#3b82f6", "#8b5cf6", "#ec4899", "#06b6d4"],
        "Valuation Metrics",
    );
    let health_chart = create_bar_chart(
        &[("ROE".to_string(), roe, "#10b981".to_string()),
          ("Debt/Equity".to_string(), d2e, "#f59e0b".to_string()),
          ("Current Ratio".to_string(), cr, "#3b82f6".to_string())],
        &["#10b981", "#f59e0b", "#3b82f6"],
        "Financial Health",
    );
    let growth_chart = create_bar_chart(
        &[("Revenue 3Y".to_string(), rev, "#22c55e".to_string()),
          ("EPS 3Y".to_string(), eps, "#3b82f6".to_string())],
        &["#22c55e", "#3b82f6"],
        "Growth (3-Year Annualized)",
    );

    let (val_label, val_color) = valuation_assessment(
        data.valuation_ratios.pe_ratio,
        data.valuation_ratios.ev_to_ebitda,
    );
    let (health_label, health_color) = health_score_label(
        data.financial_health.return_on_equity.unwrap_or(0.0),
        data.financial_health.debt_to_equity.unwrap_or(0.0),
    );
    let (growth_label, growth_color) = growth_assessment(
        data.growth_metrics.revenue_growth_3y,
        data.growth_metrics.eps_growth_3y,
    );

    format!(
        r#"<div class="detail">
            <div class="detail-head">
                <div>
                    <div class="detail-symbol">{ticker} <span class="provider-pill">FinnhubDataProvider</span></div>
                    <div class="detail-name">{company_name}</div>
                </div>
                <div class="detail-updated">Updated: {last_updated}</div>
            </div>
            <div class="stat-row">
                <div class="stat"><div class="stat-label">Current Price</div><div class="stat-value">{price}</div></div>
                <div class="stat"><div class="stat-label">Target Price</div><div class="stat-value">{target}</div></div>
                <div class="stat"><div class="stat-label">Upside</div><div class="stat-value {upside_class}">{upside}</div></div>
                <div class="stat"><div class="stat-label">Recommendation</div><div class="stat-value"><span class="rec-badge" style="background:{rec_color}">{rec_text}</span></div></div>
                <div class="stat"><div class="stat-label">Analysts</div><div class="stat-value">{analysts}</div></div>
            </div>
            <div class="charts">
                <div class="chart-card">{val_chart}<div class="assess" style="color:{val_color}">&#9679; Valuation: {val_label}</div></div>
                <div class="chart-card">{health_chart}<div class="assess" style="color:{health_color}">&#9679; Health: {health_label}</div></div>
                <div class="chart-card">{growth_chart}<div class="assess" style="color:{growth_color}">&#9679; Growth: {growth_label}</div></div>
            </div>
        </div>"#,
        ticker = data.ticker,
        company_name = data.company_name,
        last_updated = last_updated,
        price = price,
        target = target,
        upside = upside,
        upside_class = upside_class,
        rec_text = rec_text,
        rec_color = rec_color,
        analysts = analysts,
        val_chart = val_chart,
        health_chart = health_chart,
        growth_chart = growth_chart,
        val_color = val_color,
        val_label = val_label,
        health_color = health_color,
        health_label = health_label,
        growth_color = growth_color,
        growth_label = growth_label,
    )
}

fn html_finnhub(tickers: &[String], requested: &str, selected: Option<&StockRatingData>) -> String {
    let nav_ticker = if requested.is_empty() { "AAPL".to_string() } else { requested.to_string() };

    let mut cards = String::new();
    for t in tickers {
        let active = if *t == requested { " active" } else { "" };
        cards.push_str(&format!(
            r#"<a href="/finnhub?ticker={}" class="fh-card{}" data-ticker="{}"><div class="fh-card-symbol">{}</div><div class="fh-card-label">View details</div></a>"#,
            t, active, t, t
        ));
    }

    let detail_html = match selected {
        Some(data) => render_finnhub_detail(data),
        None if !requested.is_empty() => format!(
            r#"<div class="detail notfound"><div class="nf-icon">&#9888;</div><h3>No Finnhub data for "{requested}"</h3><p>The Finnhub API returned no data for this ticker. Use the search box to find the correct symbol (it searches the full US universe by name or ticker), or pick a popular ticker below.</p></div>"#
        ),
        None => r#"<div class="detail hint"><p>Type a company name or ticker in the search box above (e.g. "apple" or "NVDA"), or click a ticker below, to see live Finnhub details.</p></div>"#.to_string(),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Finnhub - StockRating</title>
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
            --green: #10b981;
            --red: #ef4444;
            --purple: #8b5cf6;
        }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif; background: var(--bg-primary); color: var(--text-primary); min-height: 100vh; }}
        .bg-grid {{ position: fixed; inset: 0; background-image: linear-gradient(rgba(59,130,246,0.03) 1px, transparent 1px), linear-gradient(90deg, rgba(59,130,246,0.03) 1px, transparent 1px); background-size: 50px 50px; pointer-events: none; z-index: 0; }}
        .bg-glow {{ position: fixed; top: -200px; left: 50%; transform: translateX(-50%); width: 800px; height: 600px; background: radial-gradient(circle, rgba(59,130,246,0.08) 0%, transparent 70%); pointer-events: none; z-index: 0; }}
        .container {{ max-width: 1200px; margin: 0 auto; padding: 0 24px 48px; position: relative; z-index: 1; }}
        header {{ padding: 24px 0; border-bottom: 1px solid var(--border); margin-bottom: 24px; }}
        .header-inner {{ display: flex; justify-content: space-between; align-items: center; }}
        .logo {{ display: flex; align-items: center; gap: 12px; }}
        .logo-icon {{ width: 40px; height: 40px; background: linear-gradient(135deg, var(--accent-blue), var(--purple)); border-radius: 10px; display: flex; align-items: center; justify-content: center; font-weight: 800; font-size: 1.1rem; color: white; }}
        .logo-text {{ font-size: 1.3rem; font-weight: 700; }}
        .logo-text span {{ color: var(--accent-blue); }}
        .provider-tag {{ font-size: 0.85rem; color: var(--text-secondary); border: 1px solid var(--border); padding: 6px 14px; border-radius: 999px; }}
        .nav-bar {{ display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 28px; padding: 6px; background: var(--bg-secondary); border-radius: 14px; border: 1px solid var(--border); width: fit-content; }}
        .nav-link {{ padding: 10px 18px; border-radius: 10px; text-decoration: none; color: var(--text-secondary); font-weight: 500; font-size: 0.9rem; }}
        .nav-link:hover {{ color: var(--text-primary); background: var(--bg-card); }}
        .nav-link.active {{ background: var(--accent-blue); color: white; }}
        .page-head {{ margin-bottom: 20px; }}
        .page-head h1 {{ font-size: 1.8rem; font-weight: 700; margin-bottom: 6px; }}
        .page-head .subtitle {{ color: var(--text-secondary); font-size: 0.95rem; }}
        .search-bar {{ display: flex; gap: 10px; margin-bottom: 28px; }}
        .search-bar input {{ flex: 1; padding: 14px 16px; border-radius: 12px; border: 1px solid var(--border); background: var(--bg-card); color: var(--text-primary); font-size: 1rem; font-family: inherit; }}
        .search-bar input:focus {{ outline: none; border-color: var(--accent-blue); box-shadow: 0 0 0 3px rgba(59,130,246,0.15); }}
        .search-bar button {{ padding: 14px 26px; border-radius: 12px; border: none; background: var(--accent-blue); color: white; font-size: 1rem; font-weight: 600; cursor: pointer; font-family: inherit; }}
        .search-bar button:hover {{ background: #2563eb; }}
        .detail {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: 16px; padding: 24px; margin-bottom: 32px; }}
        .detail.hint {{ text-align: center; color: var(--text-secondary); padding: 40px; }}
        .detail.notfound {{ text-align: center; }}
        .detail.notfound .nf-icon {{ font-size: 3rem; margin-bottom: 12px; }}
        .detail.notfound h3 {{ font-size: 1.3rem; margin-bottom: 8px; }}
        .detail.notfound p {{ color: var(--text-secondary); }}
        .detail-head {{ display: flex; justify-content: space-between; align-items: flex-start; flex-wrap: wrap; gap: 8px; margin-bottom: 20px; }}
        .detail-symbol {{ font-size: 1.6rem; font-weight: 800; display: flex; align-items: center; gap: 10px; }}
        .provider-pill {{ font-size: 0.7rem; font-weight: 600; color: var(--accent-blue); background: rgba(59,130,246,0.15); border: 1px solid rgba(59,130,246,0.3); padding: 3px 10px; border-radius: 999px; }}
        .detail-name {{ color: var(--text-secondary); margin-top: 4px; }}
        .detail-updated {{ color: var(--text-muted); font-size: 0.8rem; }}
        .stat-row {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 14px; margin-bottom: 24px; }}
        .stat {{ background: var(--bg-secondary); border: 1px solid var(--border); border-radius: 12px; padding: 16px; }}
        .stat-label {{ color: var(--text-muted); font-size: 0.78rem; text-transform: uppercase; letter-spacing: 0.04em; margin-bottom: 6px; }}
        .stat-value {{ font-size: 1.4rem; font-weight: 700; }}
        .stat-value.up {{ color: var(--green); }}
        .stat-value.down {{ color: var(--red); }}
        .stat-value.muted {{ color: var(--text-secondary); }}
        .rec-badge {{ color: white; font-size: 0.85rem; font-weight: 700; padding: 6px 14px; border-radius: 999px; }}
        .charts {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; }}
        .chart-card {{ background: var(--bg-secondary); border: 1px solid var(--border); border-radius: 12px; padding: 16px; }}
        .assess {{ margin-top: 10px; font-size: 0.85rem; font-weight: 600; text-align: center; }}
        .section-title {{ font-size: 1.1rem; font-weight: 700; margin-bottom: 16px; }}
        .section-title .count {{ color: var(--text-muted); font-weight: 500; font-size: 0.9rem; }}
        .fh-grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 12px; }}
        .fh-card {{ display: block; background: var(--bg-card); border: 1px solid var(--border); border-radius: 12px; padding: 18px 16px; text-decoration: none; color: var(--text-primary); transition: all 0.15s; }}
        .fh-card:hover {{ background: var(--bg-card-hover); border-color: var(--accent-blue); transform: translateY(-2px); }}
        .fh-card.active {{ border-color: var(--accent-blue); box-shadow: 0 0 0 2px rgba(59,130,246,0.3); }}
        .fh-card-symbol {{ font-size: 1.2rem; font-weight: 800; }}
        .fh-card-label {{ color: var(--text-muted); font-size: 0.78rem; margin-top: 4px; }}
        .search-wrap {{ position: relative; flex: 1; }}
        .search-wrap input {{ width: 100%; }}
        .fh-suggest {{ position: absolute; top: calc(100% + 6px); left: 0; right: 0; background: var(--bg-card); border: 1px solid var(--border-light); border-radius: 12px; box-shadow: 0 16px 40px rgba(0,0,0,0.55); max-height: 320px; overflow-y: auto; z-index: 60; }}
        .fh-suggest-item {{ display: flex; justify-content: space-between; align-items: center; gap: 16px; padding: 12px 16px; cursor: pointer; border-bottom: 1px solid var(--border); }}
        .fh-suggest-item:last-child {{ border-bottom: none; }}
        .fh-suggest-item:hover, .fh-suggest-item.active {{ background: var(--bg-card-hover); }}
        .fh-suggest .fh-sym {{ font-weight: 700; color: var(--accent-blue); white-space: nowrap; }}
        .fh-suggest .fh-desc {{ color: var(--text-secondary); font-size: 0.88rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }}
        .fh-suggest-empty {{ padding: 14px 16px; color: var(--text-muted); font-size: 0.9rem; }}
        .letter-nav {{ display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 14px; }}
        .letter-btn {{ width: 34px; height: 34px; display: flex; align-items: center; justify-content: center; border-radius: 9px; background: var(--bg-card); border: 1px solid var(--border); color: var(--text-secondary); text-decoration: none; font-size: 0.85rem; font-weight: 600; }}
        .letter-btn:hover {{ background: var(--accent-blue); border-color: var(--accent-blue); color: white; }}
        .fh-alpha {{ display: flex; flex-direction: column; gap: 8px; }}
        .alpha-loading {{ color: var(--text-muted); padding: 12px 0; }}
        .alpha-group {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: 12px; overflow: hidden; }}
        .alpha-group summary {{ list-style: none; cursor: pointer; display: flex; align-items: center; gap: 12px; padding: 12px 18px; font-weight: 700; font-size: 1.05rem; }}
        .alpha-group summary::-webkit-details-marker {{ display: none; }}
        .alpha-group summary::before {{ content: "\25B8"; color: var(--text-muted); font-size: 0.8rem; transition: transform 0.15s; }}
        .alpha-group[open] summary::before {{ transform: rotate(90deg); }}
        .alpha-count {{ color: var(--text-muted); font-weight: 500; font-size: 0.8rem; }}
        .alpha-list {{ display: flex; flex-wrap: wrap; gap: 8px; padding: 6px 18px 16px; }}
        .alpha-link {{ padding: 5px 12px; border-radius: 999px; background: var(--bg-secondary); border: 1px solid var(--border); color: var(--text-secondary); text-decoration: none; font-size: 0.85rem; font-weight: 600; }}
        .alpha-link:hover {{ color: var(--text-primary); border-color: var(--accent-blue); }}
        .alpha-link.active {{ background: var(--accent-blue); border-color: var(--accent-blue); color: white; }}
        .foot-note {{ margin-top: 32px; color: var(--text-muted); font-size: 0.8rem; text-align: center; }}
    </style>
</head>
<body>
    <div class="bg-grid"></div>
    <div class="bg-glow"></div>
    <div class="container">
        <header>
            <div class="header-inner">
                <div class="logo"><div class="logo-icon">SR</div><div class="logo-text">Stock<span>Rating</span></div></div>
                <div class="provider-tag">Live &middot; FinnhubDataProvider</div>
            </div>
        </header>
        <div class="nav-bar">
            <a href="/" class="nav-link">Dashboard</a>
            <a href="/dashboard?ticker={nav_ticker}" class="nav-link">Analysis</a>
            <a href="/compare?ticker={nav_ticker}" class="nav-link">Compare</a>
            <a href="/portfolio" class="nav-link">Portfolio</a>
            <a href="/finnhub" class="nav-link active">Finnhub</a>
        </div>
        <div class="page-head">
            <h1>Finnhub Live Data</h1>
            <p class="subtitle">Real-time quotes, valuation, financial health, growth and analyst sentiment from the Finnhub API.</p>
        </div>
        <form action="/finnhub" method="get" class="search-bar">
            <div class="search-wrap">
                <input id="fh-search" name="ticker" type="text" placeholder="Search by name or ticker (e.g. AAPL, apple, NVDA, nvidia)" autocomplete="off" spellcheck="false" value="{requested}">
                <div id="fh-suggest" class="fh-suggest" hidden></div>
            </div>
            <button type="submit">Search</button>
        </form>
        {detail_html}
        <div class="section-title">Popular Tickers <span class="count">({ticker_count})</span></div>
        <div class="fh-grid">{ticker_cards}</div>
        <div class="section-title">Browse All Stocks <span class="count" id="fh-total-count"></span></div>
        <div class="letter-nav" id="fh-letter-nav"></div>
        <div id="fh-alpha" class="fh-alpha"><div class="alpha-loading">Loading the full US ticker list from Finnhub&hellip;</div></div>
        <div class="foot-note">Data served by FinnhubDataProvider, cached in-memory (TTL 300s). Live values are refreshed in the background while the app runs. The full ticker list is fetched from the Finnhub screener API, sorted alphabetically and cached for one hour.</div>
    </div>
    <script>
        (function () {{
            var input = document.getElementById('fh-search');
            var box = document.getElementById('fh-suggest');
            var cards = document.querySelectorAll('.fh-card');
            if (!input || !box) return;

            var items = [];
            var activeIdx = -1;
            var timer = null;

            function closeSuggest() {{
                box.hidden = true;
                box.innerHTML = '';
                items = [];
                activeIdx = -1;
            }}

            function highlight(i) {{
                activeIdx = i;
                Array.prototype.forEach.call(box.children, function (el, idx) {{
                    el.classList.toggle('active', idx === i);
                }});
            }}

            function selectTicker(sym) {{
                window.location.href = '/finnhub?ticker=' + encodeURIComponent(sym);
            }}

            function renderSuggest(results) {{
                items = results || [];
                activeIdx = -1;
                box.innerHTML = '';
                if (!items.length) {{
                    var empty = document.createElement('div');
                    empty.className = 'fh-suggest-empty';
                    empty.textContent = 'No matches — press Enter to look up the exact ticker';
                    box.appendChild(empty);
                    box.hidden = false;
                    return;
                }}
                items.forEach(function (r) {{
                    var el = document.createElement('div');
                    el.className = 'fh-suggest-item';
                    var sym = document.createElement('span');
                    sym.className = 'fh-sym';
                    sym.textContent = r.symbol;
                    var desc = document.createElement('span');
                    desc.className = 'fh-desc';
                    desc.textContent = r.description || r.display || '';
                    el.appendChild(sym);
                    el.appendChild(desc);
                    el.addEventListener('mousedown', function (e) {{
                        e.preventDefault();
                        selectTicker(r.symbol);
                    }});
                    box.appendChild(el);
                }});
                box.hidden = false;
            }}

            function filterCards() {{
                var term = input.value.trim().toUpperCase();
                cards.forEach(function (c) {{
                    var t = c.getAttribute('data-ticker');
                    c.style.display = (t.indexOf(term) === 0) ? '' : 'none';
                }});
            }}

            input.addEventListener('input', function () {{
                filterCards();
                var q = input.value.trim();
                clearTimeout(timer);
                if (q.length < 1) {{
                    closeSuggest();
                    return;
                }}
                timer = setTimeout(function () {{
                    fetch('/api/finnhub/search?q=' + encodeURIComponent(q))
                        .then(function (res) {{
                            if (!res.ok) throw new Error('search failed');
                            return res.json();
                        }})
                        .then(renderSuggest)
                        .catch(closeSuggest);
                }}, 250);
            }});

            input.addEventListener('keydown', function (e) {{
                if (box.hidden || !items.length) return;
                if (e.key === 'ArrowDown') {{
                    e.preventDefault();
                    highlight((activeIdx + 1) % items.length);
                }} else if (e.key === 'ArrowUp') {{
                    e.preventDefault();
                    highlight((activeIdx - 1 + items.length) % items.length);
                }} else if (e.key === 'Enter') {{
                    if (activeIdx >= 0 && items[activeIdx]) {{
                        e.preventDefault();
                        selectTicker(items[activeIdx].symbol);
                    }}
                }} else if (e.key === 'Escape') {{
                    closeSuggest();
                }}
            }});

            document.addEventListener('click', function (e) {{
                if (e.target !== input && !box.contains(e.target)) closeSuggest();
            }});

            // ------------------------------------------------------------------
            // Alphabetical universe: fetch the full sorted ticker list and
            // render it grouped by first letter with a quick-jump letter nav.
            // ------------------------------------------------------------------
            var alpha = document.getElementById('fh-alpha');
            var letterNav = document.getElementById('fh-letter-nav');
            var totalCount = document.getElementById('fh-total-count');
            var activeTicker = '{nav_ticker}';
            if (!alpha || !letterNav) return;

            fetch('/api/finnhub/tickers')
                .then(function (res) {{
                    if (!res.ok) throw new Error('ticker list unavailable');
                    return res.json();
                }})
                .then(function (tickers) {{
                    if (!Array.isArray(tickers) || !tickers.length) {{
                        alpha.innerHTML = '<div class="alpha-loading">Ticker list unavailable right now.</div>';
                        return;
                    }}
                    if (totalCount) totalCount.textContent = '(' + tickers.length + ')';
                    alpha.innerHTML = '';
                    var letters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');
                    letters.forEach(function (ch) {{
                        var group = tickers.filter(function (t) {{ return t.charAt(0) === ch; }});
                        if (!group.length) return;
                        var det = document.createElement('details');
                        det.className = 'alpha-group';
                        det.id = 'fh-alpha-' + ch;
                        if (activeTicker.charAt(0) === ch) det.open = true;
                        var sum = document.createElement('summary');
                        sum.textContent = ch;
                        var cnt = document.createElement('span');
                        cnt.className = 'alpha-count';
                        cnt.textContent = '(' + group.length + ')';
                        sum.appendChild(cnt);
                        det.appendChild(sum);
                        var list = document.createElement('div');
                        list.className = 'alpha-list';
                        group.forEach(function (t) {{
                            var a = document.createElement('a');
                            a.className = 'alpha-link' + (t === activeTicker ? ' active' : '');
                            a.href = '/finnhub?ticker=' + encodeURIComponent(t);
                            a.textContent = t;
                            list.appendChild(a);
                        }});
                        det.appendChild(list);
                        alpha.appendChild(det);
                        var btn = document.createElement('a');
                        btn.className = 'letter-btn';
                        btn.href = '#fh-alpha-' + ch;
                        btn.textContent = ch;
                        btn.addEventListener('click', function (e) {{
                            e.preventDefault();
                            det.open = true;
                            det.scrollIntoView({{ behavior: 'smooth', block: 'start' }});
                        }});
                        letterNav.appendChild(btn);
                    }});
                }})
                .catch(function () {{
                    alpha.innerHTML = '<div class="alpha-loading">Ticker list unavailable right now (Finnhub API not reachable).</div>';
                }});
        }})();
    </script>
</body>
</html>"#,
        nav_ticker = nav_ticker,
        requested = requested,
        detail_html = detail_html,
        ticker_count = tickers.len(),
        ticker_cards = cards,
    )
}

async fn api_query_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<StockRatingData>, (StatusCode, String)> {
    let ticker = extract_ticker_from_params(&params);
    let provider_name = extract_provider_from_params(&params);
    let providers = state.providers.read().unwrap();

    match providers.iter().find(|p| p.provider_name() == provider_name) {
        Some(dp) => match (**dp).get_stock_data(&ticker) {
            Some(data) => Ok(Json(data)),
            None => Err((StatusCode::NOT_FOUND, format!("Ticker '{}' not found", ticker))),
        },
        None => Err((StatusCode::NOT_FOUND, format!("Provider '{}' not found", provider_name))),
    }
}

async fn portfolio_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let service = StockAggregationService::new(state.providers.clone());
    let config = StockAggregationService::default_config();
    let aggregated = service.get_aggregated_data(&config);
    let providers = state.providers.read().unwrap();

    let provider_name_filter = extract_provider_from_params(&params);
    let all_tickers = aggregated.tickers;

    let active_provider_name = providers.iter()
        .find(|p| p.provider_name() == provider_name_filter)
        .map(|p| p.provider_name())
        .unwrap_or("MockDataProvider");

    let mut stock_records: Vec<(String, StockRatingData)> = Vec::new();

    if let Some(provider) = providers.iter().find(|p| p.provider_name() == provider_name_filter) {
        for ticker in &all_tickers {
            if let Some(data) = provider.get_stock_data(ticker) {
                stock_records.push((ticker.clone(), data));
            }
        }
    }
    
    let mut valuation_data: Vec<(String, f64, String)> = Vec::new();
    let mut growth_data: Vec<(String, f64, String)> = Vec::new();
    let mut health_data: Vec<(String, f64, String)> = Vec::new();
    let mut sentiment_data: Vec<(String, f64, String)> = Vec::new();
    
    let colors = ["#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6", "#ec4899", "#f97316", "#06b6d4"];
    
    for (ticker, data) in &stock_records {
        let pe = data.valuation_ratios.pe_ratio.unwrap_or(0.0);
        let roe = data.financial_health.return_on_equity.unwrap_or(0.0) * 100.0;
        let rev_growth = data.growth_metrics.revenue_growth_3y.unwrap_or(0.0) * 100.0;
        let upside = match (data.market_sentiment.target_price_consensus, data.market_sentiment.current_price) {
            (Some(t), Some(c)) if c > 0.0 => ((t - c) / c) * 100.0,
            _ => 0.0,
        };
        
        valuation_data.push((ticker.clone(), pe, "#3b82f6".to_string()));
        growth_data.push((ticker.clone(), rev_growth, "#10b981".to_string()));
        health_data.push((ticker.clone(), roe, "#f59e0b".to_string()));
        sentiment_data.push((ticker.clone(), upside, "#8b5cf6".to_string()));
    }
    
    let val_svg = create_bar_chart(&valuation_data, &colors.iter().take(8).cloned().collect::<Vec<&str>>(), "P/E Ratio Comparison (Lower is Better)");
    let growth_svg = create_bar_chart(&growth_data, &colors.iter().take(8).cloned().collect::<Vec<&str>>(), "3-Year Revenue Growth Comparison");
    let health_svg = create_bar_chart(&health_data, &colors.iter().take(8).cloned().collect::<Vec<&str>>(), "Return on Equity (ROE) Comparison");
    let sentiment_svg = create_bar_chart(&sentiment_data, &colors.iter().take(8).cloned().collect::<Vec<&str>>(), "Analyst Upside/Downside % (Price Target vs Current)");
    
    let mut ticker_rows = String::new();
    let mut ticker_cards = String::new();
    
    for (ticker, data) in &stock_records {
        let rec = data.market_sentiment.recommendation_consensus.clone().unwrap_or(Recommendation::Hold);
        let rec_text = format_recommendation(&rec);
        let rec_color = match rec {
            Recommendation::StrongBuy => "#10b981",
            Recommendation::Buy => "#22c55e",
            Recommendation::Hold => "#f59e0b",
            Recommendation::Sell => "#f97316",
            Recommendation::StrongSell => "#ef4444",
        };
        let pe = data.valuation_ratios.pe_ratio.unwrap_or(0.0);
        let roe = data.financial_health.return_on_equity.unwrap_or(0.0) * 100.0;
        let rev_growth = data.growth_metrics.revenue_growth_3y.unwrap_or(0.0);
        let upside = match (data.market_sentiment.target_price_consensus, data.market_sentiment.current_price) {
            (Some(t), Some(c)) if c > 0.0 => ((t - c) / c) * 100.0,
            _ => 0.0,
        };
        
        let rev_pct = rev_growth * 100.0;
        ticker_rows.push_str(&format!(
            "<tr><td><a href=\"/dashboard?ticker={ticker}\" style=\"color:var(--accent-blue);text-decoration:none;font-weight:600;\">{ticker}</a></td><td style=\"color:{rec_color};font-weight:600;\">{rec_text}</td><td>{pe:.1}</td><td>{roe:.1}%</td><td>{rev_pct:+.1}%</td><td style=\"color:{upside_color};font-weight:600;\">{upside:+.1}%</td></tr>",
            rec_color = rec_color,
            upside_color = if upside >= 0.0 { "#10b981" } else { "#ef4444" }
        ));
        
        ticker_cards.push_str(&format!(
            "<a href=\"/dashboard?ticker={ticker}\" class=\"ticker-card\">\n                <div class=\"ticker-card-symbol\" style=\"color:{rec_color};\">{ticker}</div>\n                <div class=\"ticker-card-label\">{rec_text}</div>\n                <div style=\"font-size:0.75rem;color:var(--text-muted);margin-top:4px;\">P/E: {pe:.1} | ROE: {roe:.1}%</div>\n            </a>",
            rec_color = rec_color
        ));
    }
    
    let provider_btn_active = |name: &str, selected: &str| -> String {
        if name == selected {
            " class=\"provider-btn active\"".to_string()
        } else {
            String::new()
        }
    };
    
    let mock_active = provider_btn_active("MockDataProvider", active_provider_name);
    let second_active = provider_btn_active("SecondMockProvider", active_provider_name);
    let finnhub_active = provider_btn_active("FinnhubDataProvider", active_provider_name);
    
    let has_finnhub = providers.iter().any(|p| p.provider_name() == "FinnhubDataProvider");
    
    let provider_selector = format!(r#"<div class="provider-selector">
            <span class="provider-label">Data Provider:</span>
            <a href="/portfolio?provider=mock" class="provider-btn{}">MockDataProvider</a>
            <a href="/portfolio?provider=second" class="provider-btn{}">SecondMockProvider</a>{}
        </div>"#, mock_active, second_active,
        if has_finnhub {
            format!("<a href=\"/portfolio?provider=finnhub\" class=\"provider-btn{}\">FinnhubDataProvider</a>", finnhub_active)
        } else {
            String::new()
        }
    );
    
    let has_data = !stock_records.is_empty();
    
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Portfolio - StockRating</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&display=swap" rel="stylesheet">
    <style>
        :root {{
            --bg-primary: #0b0f19;
            --bg-card: #1a2236;
            --bg-card-hover: #1f2a42;
            --border: #2d3a52;
            --text-primary: #f1f5f9;
            --text-secondary: #94a3b8;
            --text-muted: #64748b;
            --accent-blue: #3b82f6;
            --green: #10b981;
            --yellow: #f59e0b;
            --red: #ef4444;
        }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: 'Inter', sans-serif; background: var(--bg-primary); color: var(--text-primary); min-height: 100vh; }}
        .container {{ max-width: 1400px; margin: 0 auto; padding: 0 24px; position: relative; z-index: 1; }}
        header {{ padding: 24px 0; border-bottom: 1px solid var(--border); margin-bottom: 32px; }}
        .header-inner {{ display: flex; justify-content: space-between; align-items: center; }}
        .logo {{ display: flex; align-items: center; gap: 12px; }}
        .logo-icon {{ width: 40px; height: 40px; background: linear-gradient(135deg, var(--accent-blue), #8b5cf6); border-radius: 10px; display: flex; align-items: center; justify-content: center; font-weight: 800; font-size: 1.1rem; color: white; }}
        .logo-text {{ font-size: 1.3rem; font-weight: 700; }}
        .logo-text span {{ color: var(--accent-blue); }}
        .nav-bar {{ display: flex; gap: 8px; margin-bottom: 32px; padding: 6px; background: #111827; border-radius: 14px; border: 1px solid var(--border); width: fit-content; }}
        .nav-link {{ padding: 10px 20px; border-radius: 10px; text-decoration: none; color: var(--text-secondary); font-weight: 500; font-size: 0.9rem; }}
        .nav-link:hover {{ color: var(--text-primary); background: var(--bg-card); }}
        .nav-link.active {{ background: var(--accent-blue); color: white; }}
        .provider-selector {{ display: flex; gap: 8px; margin-bottom: 32px; align-items: center; }}
        .provider-label {{ color: var(--text-muted); font-size: 0.85rem; font-weight: 500; margin-right: 8px; }}
        .provider-btn {{ padding: 8px 16px; border-radius: 8px; border: 1px solid var(--border); background: var(--bg-card); color: var(--text-secondary); font-size: 0.85rem; font-weight: 500; cursor: pointer; transition: all 0.2s; text-decoration: none; }}
        .provider-btn:hover {{ border-color: var(--accent-blue); color: var(--text-primary); }}
        .provider-btn.active {{ background: var(--accent-blue); border-color: var(--accent-blue); color: white; }}
        .page-title {{ font-size: 1.8rem; font-weight: 700; margin-bottom: 8px; }}
        .page-subtitle {{ color: var(--text-muted); margin-bottom: 32px; font-size: 0.95rem; }}
        .provider-badge {{ display: inline-flex; align-items: center; gap: 8px; padding: 6px 14px; background: var(--bg-card); border: 1px solid var(--border); border-radius: 8px; font-weight: 600; font-size: 0.85rem; color: var(--accent-blue); margin-bottom: 32px; }}
        .chart-section {{ margin-bottom: 32px; }}
        .chart-grid {{ display: grid; grid-template-columns: repeat(2, 1fr); gap: 24px; }}
        @media (max-width: 1100px) {{ .chart-grid {{ grid-template-columns: 1fr; }} }}
        .chart-card {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: 16px; padding: 24px; }}
        .chart-title {{ font-size: 0.9rem; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.08em; margin-bottom: 16px; }}
        .data-table {{ width: 100%; border-collapse: separate; border-spacing: 0; margin-bottom: 40px; }}
        .data-table th {{ background: var(--bg-card); padding: 16px; text-align: left; font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.08em; color: var(--text-muted); border-bottom: 1px solid var(--border); }}
        .data-table th:first-child {{ border-radius: 12px 0 0 0; }}
        .data-table th:last-child {{ border-radius: 0 12px 0 0; }}
        .data-table td {{ padding: 16px; border-bottom: 1px solid var(--border); font-size: 0.95rem; }}
        .data-table tr:last-child td {{ border-bottom: none; }}
        .ticker-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 40px; }}
        .ticker-card {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: 16px; padding: 24px; text-align: center; text-decoration: none; transition: all 0.3s; }}
        .ticker-card:hover {{ border-color: var(--accent-blue); background: var(--bg-card-hover); transform: translateY(-4px); }}
        .ticker-card-symbol {{ font-size: 1.5rem; font-weight: 700; margin-bottom: 8px; }}
        .ticker-card-label {{ font-size: 0.85rem; color: var(--text-muted); }}
        section.card {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: 16px; padding: 28px; margin-bottom: 32px; }}
        section.card h2 {{ font-size: 1.1rem; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.08em; margin-bottom: 20px; }}
        .no-data {{ text-align: center; padding: 60px 20px; color: var(--text-muted); }}
        .no-data h2 {{ font-size: 1.5rem; color: var(--text-primary); margin-bottom: 12px; }}
        footer {{ text-align: center; padding: 40px 0; color: var(--text-muted); font-size: 0.85rem; border-top: 1px solid var(--border); margin-top: 40px; }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <div class="header-inner">
                <div class="logo">
                    <div class="logo-icon">S</div>
                    <div class="logo-text"><span>Stock</span>Rating</div>
                </div>
            </div>
        </header>

        <div class="nav-bar">
            <a href="/" class="nav-link">Dashboard</a>
            <a href="/dashboard?ticker=AAPL" class="nav-link">Analysis</a>
            <a href="/compare?ticker=AAPL" class="nav-link">Compare</a>
            <a href="/portfolio" class="nav-link active">Portfolio View</a>
        </div>
        
        {provider_selector}

        <h1 class="page-title">Portfolio Overview</h1>
        <div class="provider-badge">
            <span>●</span>
            <span>Viewing data from: <strong>{active_provider_name}</strong></span>
        </div>
        <p class="page-subtitle">Comparing all available stocks side-by-side across key metrics</p>
        
        {no_data_message}
        
        {main_content}

        <footer>
            <p>StockRating Portfolio View • {active_provider_name} • Not financial advice</p>
        </footer>
    </div>
</body>
</html>"#,
        provider_selector = provider_selector,
        active_provider_name = active_provider_name,
        no_data_message = if has_data { String::new() } else {
            "<div class=\"no-data\"><h2>No Data Available</h2><p>No stock data found for provider \"{active_provider_name}\".</p></div>".to_string()
        },
        main_content = if has_data {
            format!(r#"<section class="card">
            <h2>Quick Select</h2>
            <div class="ticker-grid">
                {ticker_cards}
            </div>
        </section>

        <section class="card">
            <h2>All Stocks Comparison Table</h2>
            <table class="data-table">
                <thead>
                    <tr>
                        <th>Ticker</th>
                        <th>Recommendation</th>
                        <th>P/E Ratio</th>
                        <th>ROE</th>
                        <th>Rev Growth 3Y</th>
                        <th>Upside %</th>
                    </tr>
                </thead>
                <tbody>
                    {ticker_rows}
                </tbody>
            </table>
        </section>

        <div class="chart-section">
            <div class="chart-grid">
                <div class="chart-card">
                    <div class="chart-title">Valuation</div>
                    {val_svg}
                </div>
                <div class="chart-card">
                    <div class="chart-title">Growth</div>
                    {growth_svg}
                </div>
                <div class="chart-card">
                    <div class="chart-title">Financial Health</div>
                    {health_svg}
                </div>
                <div class="chart-card">
                    <div class="chart-title">Market Sentiment</div>
                    {sentiment_svg}
                </div>
            </div>
        </div>"#,
            ticker_cards = ticker_cards,
            ticker_rows = ticker_rows,
            val_svg = val_svg,
            growth_svg = growth_svg,
            health_svg = health_svg,
            sentiment_svg = sentiment_svg,
        )} else {
            String::new()
        },
    ))
}

pub(crate) async fn all_stocks_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let service = StockAggregationService::new(state.providers.clone());
    let config = StockAggregationService::default_config();
    let data = service.get_aggregated_data(&config);

    let mut chart_groups_json = Vec::new();
    for group in &data.chart_groups {
        let mut entries_json = Vec::new();
        for entry in &group.entries {
            entries_json.push(json!({
                "ticker": entry.ticker,
                "company_name": entry.company_name,
                "provider": entry.provider,
                "value": entry.value
            }));
        }
        chart_groups_json.push(json!({
            "metric": group.metric.label(),
            "unit": group.unit,
            "color": group.color,
            "entries": entries_json
        }));
    }

    Json(json!({
        "tickers": data.tickers,
        "providers": data.providers,
        "chart_groups": chart_groups_json,
        "metrics": config.metrics.iter().map(|m| m.label()).collect::<Vec<&str>>(),
        "chart_type": format!("{:?}", config.chart_type),
    }))
}

pub(crate) async fn all_tickers_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let service = StockAggregationService::new(state.providers.clone());
    let tickers = service.get_all_tickers();

    let mut result = Vec::new();
    for t in tickers {
        result.push(json!({
            "ticker": t.ticker,
            "providers": t.providers,
        }));
    }

    Json(json!({
        "tickers": result,
        "total": result.len(),
    }))
}

#[derive(Deserialize)]
pub struct ConfigurableParams {
    pub metrics: Option<String>,
    pub providers: Option<String>,
    pub chart_type: Option<String>,
}

async fn chart_api_handler(
    State(state): State<AppState>,
    Query(params): Query<ConfigurableParams>,
) -> Json<serde_json::Value> {
    let mut map = std::collections::HashMap::new();
    if let Some(ref m) = params.metrics {
        map.insert("metrics".to_string(), m.clone());
    }
    if let Some(ref p) = params.providers {
        map.insert("providers".to_string(), p.clone());
    }
    if let Some(ref c) = params.chart_type {
        map.insert("chart_type".to_string(), c.clone());
    }

    let service = StockAggregationService::new(state.providers.clone());
    let config = StockAggregationService::parse_query_params(&map);
    let data = service.get_aggregated_data(&config);

    let mut chart_groups_json = Vec::new();
    for group in &data.chart_groups {
        let mut entries_json = Vec::new();
        for entry in &group.entries {
            entries_json.push(json!({
                "ticker": entry.ticker,
                "company_name": entry.company_name,
                "provider": entry.provider,
                "value": entry.value
            }));
        }
        chart_groups_json.push(json!({
            "metric": group.metric.label(),
            "unit": group.unit,
            "color": group.color,
            "entries": entries_json
        }));
    }

    Json(json!({
        "tickers": data.tickers,
        "providers": data.providers,
        "chart_groups": chart_groups_json,
        "metrics": config.metrics.iter().map(|m| m.label()).collect::<Vec<&str>>(),
        "chart_type": format!("{:?}", config.chart_type),
    }))
}
