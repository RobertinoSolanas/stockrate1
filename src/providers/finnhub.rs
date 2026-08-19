use crate::models::*;
use chrono::Utc;
use crate::providers::StockDataProvider;
use reqwest;
use serde::Deserialize;
use serde_json;
use std::time::{Duration, Instant};

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
struct FinnhubSearchResponse {
    #[serde(default)]
    #[allow(dead_code)]
    count: Option<usize>,
    #[serde(default)]
    result: Vec<FinnhubSearchEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FinnhubSearchEntry {
    #[serde(default)]
    description: Option<String>,
    #[serde(rename = "displaySymbol", default)]
    display_symbol: Option<String>,
    #[serde(default)]
    display: Option<String>,
    #[serde(default)]
    symbol: Option<String>,
    #[serde(rename = "type", default)]
    kind: Option<String>,
}

impl FinnhubSearchEntry {
    fn resolved_display(&self) -> String {
        self.display_symbol
            .clone()
            .or_else(|| self.display.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
struct FinnhubScreenerResponse {
    #[serde(default)]
    data: Vec<FinnhubScreenerRow>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FinnhubScreenerRow {
    #[serde(default)]
    ticker: Option<String>,
}

/// Percent-encode a string so it can be interpolated safely into a URL.
pub(crate) fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Map raw Finnhub `/search` entries into deduplicated, alphabetized results.
/// Exchange suffixes ("AAPL:US") are stripped.
pub(crate) fn map_search_entries(entries: Vec<FinnhubSearchEntry>) -> Vec<StockSearchResult> {
    let mut results: Vec<StockSearchResult> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for entry in entries {
        let display = entry.resolved_display();
        let raw_symbol = entry.symbol.unwrap_or_default();
        let base = raw_symbol.split(':').next().unwrap_or("").to_uppercase();
        if base.is_empty() || base.len() > 10 || !seen.insert(base.clone()) {
            continue;
        }
        results.push(StockSearchResult {
            symbol: base,
            description: entry.description.unwrap_or_default(),
            display,
            kind: entry.kind,
        });
    }
    results.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    results
}

/// Extract, deduplicate and alphabetize ticker symbols from the Finnhub
/// `/stock/screener` response (BTreeSet gives us the sort for free).
pub(crate) fn map_screener_rows(rows: Vec<FinnhubScreenerRow>) -> Vec<String> {
    rows.into_iter()
        .filter_map(|r| r.ticker)
        .map(|t| t.split(':').next().unwrap_or("").to_uppercase())
        .filter(|t| !t.is_empty() && t.chars().all(|c| c.is_ascii_alphanumeric()))
        .collect::<std::collections::BTreeSet<String>>()
        .into_iter()
        .collect()
}

// ---------------------------------------------------------------------------
// Fuzzy search helpers (pure functions, unit-tested)
// ---------------------------------------------------------------------------

/// Lowercase a query and split it into alphanumeric words (length >= 2),
/// dropping generic company suffixes ("Apple Inc." -> ["apple"]).
pub(crate) fn tokenize_search_query(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2 && !SEARCH_STOPWORDS.contains(t))
        .map(|t| t.to_string())
        .collect()
}

/// Classic Levenshtein edit distance (two-row dynamic programming).
pub(crate) fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// True when every char of `needle` appears in `hay` in order.
pub(crate) fn is_subsequence(needle: &str, hay: &str) -> bool {
    let mut it = hay.chars();
    needle.chars().all(|n| it.any(|c| c == n))
}

/// Look up a nickname/legacy name: exact match or 4+ char prefix in either
/// direction ("goog" -> google -> GOOGL).
pub(crate) fn alias_lookup(word: &str) -> Option<(&'static str, &'static str)> {
    ALIASES.iter()
        .find(|(alias, _, _)| {
            let a = *alias;
            word == a
                || (word.len() >= 4 && a.starts_with(word))
                || (a.len() >= 4 && word.starts_with(a))
        })
        .map(|(_, sym, name)| (*sym, *name))
}

/// Fuzzy-match one query word against a ticker symbol.
pub(crate) fn score_symbol(word: &str, symbol: &str) -> u32 {
    let s = symbol.to_lowercase();
    if word == s {
        return 90;
    }
    if word.len() >= 3 && s.starts_with(word) {
        return 75;
    }
    let d = levenshtein(word, &s);
    // lev-1 needs 4+ chars on both sides: on 3-char symbols one substitution
    // means a third of the symbol differs ("ibm" ~ IRM) — too noisy.
    if d == 1 && word.len() >= 4 && s.len() >= 4 {
        return 70;
    }
    // lev-2 is only trusted for longer strings: on 4-char symbols it would
    // match roughly half of the whole universe ("aapl" ~ PYPL/BALL/...).
    if d == 2 && word.len() >= 5 && s.len() >= 5 {
        return 55;
    }
    // The symbol is just the (short) start of a longer word: weak signal.
    if s.len() >= 3 && s.len() < word.len() && word.starts_with(&s) {
        return 45;
    }
    // Subsequence in either direction ("nvdia" ~ NVDA).
    if s.len() >= 3 && word.len() >= 3 && is_subsequence(&s, word) {
        return 40;
    }
    if s.len() >= 4 && word.len() >= 3 && is_subsequence(word, &s) {
        return 40;
    }
    0
}

/// Fuzzy-match all query words against a symbol + company name.
/// Returns (score, number_of_query_words_matched) — the match count is used
/// as a tie-breaker so "Bank of America" (2 words matched) outranks
/// "Wilson Bank Holding" (1 word) at equal score.
pub(crate) fn score_candidate(symbol: &str, description: &str, words: &[String]) -> (u32, u32) {
    let name_tokens: Vec<String> = description
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3 && !SEARCH_STOPWORDS.contains(t))
        .map(|t| t.to_string())
        .collect();

    let mut best: u32 = 0;
    let mut matched: u32 = 0;
    for w in words {
        let sym_score = score_symbol(w, symbol);
        if sym_score >= SEARCH_MIN_SCORE {
            matched += 1;
        }
        best = best.max(sym_score);
        // "Apple Inc" -> single meaningful token equal to the word: the user
        // almost certainly meant this exact company. (5+ chars: short generic
        // tokens like "bank" would otherwise boost "M&T Bank" over BAC.)
        if name_tokens.len() == 1 && name_tokens[0] == *w && w.len() >= 5 {
            matched += 1;
            best = best.max(95);
        }
        let mut word_matched_name = false;
        for t in &name_tokens {
            if w == t {
                word_matched_name = true;
                best = best.max(85);
            } else if w.len() >= 3 && t.starts_with(w) {
                word_matched_name = true;
                best = best.max(70);
            } else if t.len() >= 3 && w.starts_with(t) {
                word_matched_name = true;
                best = best.max(50);
            } else {
                let d = levenshtein(w, t);
                if d == 1 && t.len() >= 3 {
                    word_matched_name = true;
                    best = best.max(60);
                } else if d == 2 && t.len() >= 5 && w.len() >= 5 {
                    word_matched_name = true;
                    best = best.max(50);
                }
            }
        }
        if word_matched_name && sym_score < SEARCH_MIN_SCORE {
            matched += 1;
        }
    }
    (best, matched)
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

/// How long the full alphabetical ticker list is kept before re-fetching
/// from the screener endpoint (the universe of listed symbols barely moves).
const TICKER_LIST_TTL: Duration = Duration::from_secs(3600);

/// How long a search response is cached (protects the free-tier rate limit
/// against rapid typing).
const SEARCH_TTL: Duration = Duration::from_secs(120);

/// Minimum score for a candidate to be returned by the fuzzy search.
const SEARCH_MIN_SCORE: u32 = 40;

/// Generic company suffixes dropped from search queries ("Apple Inc." -> [apple]).
const SEARCH_STOPWORDS: &[&str] = &[
    "inc", "corp", "co", "ltd", "group", "groups", "holdings", "holding", "plc",
    "technologies", "technology", "tech", "systems", "system", "international",
    "companies", "company", "class", "the", "and",
];

/// Common nicknames / legacy names -> (ticker, display name).
/// Lets users find rebranded companies ("google" -> GOOGL/Alphabet) and
/// well-known companies by their plain name even when the live search API
/// only knows the formal registration name.
const ALIASES: &[(&str, &str, &str)] = &[
    ("google", "GOOGL", "Alphabet Inc (Google)"),
    ("alphabet", "GOOGL", "Alphabet Inc"),
    ("facebook", "META", "Meta Platforms Inc"),
    ("meta", "META", "Meta Platforms Inc"),
    ("apple", "AAPL", "Apple Inc"),
    ("microsoft", "MSFT", "Microsoft Corp"),
    ("amazon", "AMZN", "Amazon.com Inc"),
    ("tesla", "TSLA", "Tesla Inc"),
    ("nvidia", "NVDA", "NVIDIA Corp"),
    ("intel", "INTC", "Intel Corp"),
    ("netflix", "NFLX", "Netflix Inc"),
    ("disney", "DIS", "Walt Disney Co"),
    ("cisco", "CSCO", "Cisco Systems Inc"),
    ("oracle", "ORCL", "Oracle Corp"),
    ("adobe", "ADBE", "Adobe Inc"),
    ("airbnb", "ABNB", "Airbnb Inc"),
    ("spotify", "SPOT", "Spotify Technology SA"),
    ("palantir", "PLTR", "Palantir Technologies Inc"),
    ("coinbase", "COIN", "Coinbase Global Inc"),
    ("nike", "NKE", "Nike Inc"),
    ("starbucks", "SBUX", "Starbucks Corp"),
    ("walmart", "WMT", "Walmart Inc"),
    ("costco", "COST", "Costco Wholesale Corp"),
    ("berkshire", "BRK.B", "Berkshire Hathaway Inc"),
    ("goldman", "GS", "Goldman Sachs Group Inc"),
    ("jpmorgan", "JPM", "JPMorgan Chase & Co"),
    ("america", "BAC", "Bank of America Corp"),
    ("chase", "JPM", "JPMorgan Chase & Co"),
    ("visa", "V", "Visa Inc"),
    ("mastercard", "MA", "Mastercard Inc"),
    ("intuit", "INTU", "Intuit Inc"),
    ("salesforce", "CRM", "Salesforce Inc"),
    ("coca", "KO", "Coca-Cola Co"),
    ("pepsi", "PEP", "PepsiCo Inc"),
    ("boeing", "BA", "Boeing Co"),
    ("chevron", "CVX", "Chevron Corp"),
    ("exxon", "XOM", "Exxon Mobil Corp"),
    ("depot", "HD", "Home Depot Inc"),
    ("stanley", "MS", "Morgan Stanley"),
    ("paypal", "PYPL", "PayPal Inc"),
    ("amd", "AMD", "Advanced Micro Devices Inc"),
    ("qualcomm", "QCOM", "Qualcomm Inc"),
    ("verizon", "VZ", "Verizon Communications Inc"),
];

pub struct FinnhubDataProvider {
    api_key: String,
    #[allow(dead_code)]
    client: reqwest::Client,
    ticker_list_cache: std::sync::Mutex<Option<(Instant, Vec<String>)>>,
    search_cache: std::sync::Mutex<std::collections::HashMap<String, (Instant, Vec<StockSearchResult>)>>,
}

impl FinnhubDataProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            ticker_list_cache: std::sync::Mutex::new(None),
            search_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    fn new_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime")
    }

    async fn fetch<T: serde::de::DeserializeOwned>(_api_key: &str, url: &str) -> Result<T, String> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await.map_err(|e| e.to_string())?;
        let body = response.text().await.map_err(|e| e.to_string())?;
        serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {}", e))
    }

    /// One Finnhub `/search` call (US exchange).
    async fn fetch_search(api_key: &str, probe: &str) -> Option<Vec<FinnhubSearchEntry>> {
        let url = format!(
            "https://finnhub.io/api/v1/search?q={}&exchange=US&token={}",
            url_encode(probe),
            api_key
        );
        let resp: FinnhubSearchResponse = Self::fetch(api_key, &url).await.ok()?;
        Some(resp.result)
    }

    /// Fuzzy free-text search.
    ///
    /// Methodology (layered, so typos and nicknames still resolve):
    /// 1. Probe the Finnhub search API with the raw query; if that is thin,
    ///    probe each word and, for words >= 5 chars, walk a prefix ladder
    ///    ("infion" -> "infio" -> "infi") until the API returns anything.
    /// 2. Add local candidates: the nickname alias map ("google" -> GOOGL)
    ///    and Levenshtein/prefix/subsequence matches against the cached
    ///    alphabetical ticker universe ("apal" -> AAPL).
    /// 3. Score every candidate against the query words, deduplicate across
    ///    exchanges (US listing preferred) and return the top 20.
    async fn search_symbols_inner(
        api_key: &str,
        query: &str,
        universe: &[String],
    ) -> Option<Vec<StockSearchResult>> {
        let q = query.trim();
        if q.is_empty() {
            return Some(Vec::new());
        }
        let words = tokenize_search_query(q);

        // 1) API probes
        let mut entries = Self::fetch_search(api_key, q).await.unwrap_or_default();
        if entries.len() < 3 {
            for w in words.iter().take(2) {
                if entries.len() >= 8 {
                    break;
                }
                // Prefix ladder: full word down to 4 chars, stop at first hit.
                for l in (4..=w.len()).rev() {
                    let probe = match w.get(..l) {
                        Some(p) if p.chars().all(|c| c.is_ascii()) => p,
                        _ => continue,
                    };
                    if l == w.len() && probe == q {
                        continue; // already probed as raw query
                    }
                    let res = Self::fetch_search(api_key, probe).await.unwrap_or_default();
                    if !res.is_empty() {
                        entries.extend(res);
                        break;
                    }
                }
            }
        }
        let mut pool = map_search_entries(entries);

        // 2a) Alias map (nicknames / legacy company names)
        for w in &words {
            if let Some((sym, name)) = alias_lookup(w) {
                if !pool.iter().any(|p| p.symbol == sym) {
                    pool.push(StockSearchResult {
                        symbol: sym.to_string(),
                        description: name.to_string(),
                        display: sym.to_string(),
                        kind: Some("stock".to_string()),
                    });
                }
            }
        }

        // 2b) Local fuzzy match of the words against the ticker universe
        for w in &words {
            if w.len() < 3 {
                continue;
            }
            for sym in universe {
                if pool.iter().any(|p| p.symbol == *sym) {
                    continue;
                }
                if score_symbol(w, sym) >= SEARCH_MIN_SCORE {
                    pool.push(StockSearchResult {
                        symbol: sym.clone(),
                        description: String::new(),
                        display: sym.clone(),
                        kind: None,
                    });
                }
            }
        }

        // 3) Score, dedupe across exchanges (US listing preferred), rank.
        //    Sort: score desc -> query words matched desc -> symbol asc.
        let mut best: std::collections::HashMap<String, (u32, u32, StockSearchResult)> =
            std::collections::HashMap::new();
        for p in pool {
            let (score, matched) = score_candidate(&p.symbol, &p.description, &words);
            if score < SEARCH_MIN_SCORE {
                continue;
            }
            let base = p.symbol.split('.').next().unwrap_or(&p.symbol).to_string();
            let is_us = !p.symbol.contains('.');
            match best.get(&base) {
                // A US listing beats a foreign one for the same base symbol.
                Some((_, _, existing)) if !is_us && !existing.symbol.contains('.') => {}
                Some((existing_score, _, _)) if score < *existing_score => {}
                _ => {
                    best.insert(base, (score, matched, p));
                }
            }
        }
        let mut ranked: Vec<(u32, u32, &StockSearchResult)> = best
            .values()
            .map(|(s, m, p)| (*s, *m, p))
            .collect();
        ranked.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| b.1.cmp(&a.1))
                .then_with(|| a.2.symbol.cmp(&b.2.symbol))
        });
        Some(ranked.into_iter().take(20).map(|(_, _, p)| p.clone()).collect())
    }

    /// Fetch the full US ticker universe from the Finnhub screener endpoint.
    /// (Screener requires a paid Finnhub plan; returns `None` on free tiers.)
    async fn screener_tickers(api_key: &str) -> Option<Vec<String>> {
        let url = format!(
            "https://finnhub.io/api/v1/stock/screener?exchange=US&token={}",
            api_key
        );
        let resp: FinnhubScreenerResponse = Self::fetch(api_key, &url).await.ok()?;
        let list = map_screener_rows(resp.data);
        if list.is_empty() {
            None
        } else {
            Some(list)
        }
    }

    /// Fallback universe for free API keys: the S&P 500 constituents CSV from
    /// the public `datasets` GitHub repo (no API key, static CDN, stable URL).
    async fn sp500_tickers() -> Option<Vec<String>> {
        let url = "https://raw.githubusercontent.com/datasets/s-and-p-500-companies/main/data/constituents.csv";
        let client = reqwest::Client::new();
        let body = client.get(url).send().await.ok()?.text().await.ok()?;
        let symbols: Vec<String> = body
            .lines()
            .skip(1) // header row
            .filter_map(|line| line.split(',').next())
            .map(|s| s.trim().trim_matches('"').to_uppercase())
            .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '.'))
            .collect::<std::collections::BTreeSet<String>>()
            .into_iter()
            .collect();
        if symbols.is_empty() {
            None
        } else {
            Some(symbols)
        }
    }

    /// Full alphabetized ticker list, layered API methodology:
    /// 1. Finnhub `/stock/screener` (full US universe, paid plans)
    /// 2. S&P 500 constituents CSV (keyless public dataset, free-tier fallback)
    /// 3. curated in-app ticker list (offline fallback, handled by caller)
    async fn all_tickers_inner(api_key: &str) -> Option<Vec<String>> {
        if let Some(list) = Self::screener_tickers(api_key).await {
            return Some(list);
        }
        Self::sp500_tickers().await
    }

    /// Synchronous fuzzy free-text search (blocking; runs the HTTP calls on a
    /// worker thread, same pattern as `get_stock_data`). Results are cached
    /// per query for `SEARCH_TTL` to be gentle with the API rate limit.
    pub fn search(&self, query: &str) -> Vec<StockSearchResult> {
        let q = query.trim().to_string();
        if q.is_empty() {
            return Vec::new();
        }
        let key = q.to_lowercase();
        if let Some((fetched_at, results)) = self.search_cache.lock().unwrap().get(&key).cloned() {
            if fetched_at.elapsed() <= SEARCH_TTL {
                return results;
            }
        }
        let api_key = self.api_key.clone();
        let universe = self.all_tickers();
        let results = std::thread::spawn(move || {
            let rt = Self::new_runtime();
            rt.block_on(Self::search_symbols_inner(&api_key, &q, &universe))
        })
        .join()
        .unwrap_or(None)
        .unwrap_or_default();
        self.search_cache
            .lock()
            .unwrap()
            .insert(key, (Instant::now(), results.clone()));
        results
    }

    /// Alphabetical list of every available US ticker.
    ///
    /// Methodology: `all_tickers_inner` tries the Finnhub screener API first
    /// (paid plans), then the keyless S&P 500 constituents CSV. Symbols are
    /// deduplicated, upper-cased and sorted with a BTreeSet, then cached
    /// in-process for `TICKER_LIST_TTL`. On failure a stale cache is used,
    /// falling back to the curated supported list.
    pub fn all_tickers(&self) -> Vec<String> {
        if let Some((fetched_at, list)) = self.ticker_list_cache.lock().unwrap().clone() {
            if fetched_at.elapsed() <= TICKER_LIST_TTL {
                return list;
            }
        }
        let api_key = self.api_key.clone();
        let fetched = std::thread::spawn(move || {
            let rt = Self::new_runtime();
            rt.block_on(Self::all_tickers_inner(&api_key))
        })
        .join()
        .unwrap_or(None);
        match fetched {
            Some(list) => {
                *self.ticker_list_cache.lock().unwrap() = Some((Instant::now(), list.clone()));
                list
            }
            // A stale cache beats nothing; the curated list beats an empty one.
            None => self
                .ticker_list_cache
                .lock()
                .unwrap()
                .clone()
                .map(|(_, list)| list)
                .unwrap_or_else(|| self.list_supported_tickers()),
        }
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

    fn search_symbols(&self, query: &str) -> Vec<StockSearchResult> {
        self.search(query)
    }

    fn list_all_tickers(&self) -> Vec<String> {
        self.all_tickers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode_keeps_unreserved_chars() {
        assert_eq!(url_encode("AAPL"), "AAPL");
        assert_eq!(url_encode("a.b-c_d~e"), "a.b-c_d~e");
    }

    #[test]
    fn test_url_encode_encodes_special_chars() {
        assert_eq!(url_encode("big tech"), "big%20tech");
        assert_eq!(url_encode("a&b=c?d/e"), "a%26b%3Dc%3Fd%2Fe");
    }

    #[test]
    fn test_map_search_entries_dedupes_and_strips_exchange_suffix() {
        let entries = vec![
            FinnhubSearchEntry {
                display_symbol: None,
                description: Some("Apple Inc.".into()),
                display: Some("AAPL:US".into()),
                symbol: Some("AAPL:US".into()),
                kind: Some("stock".into()),
            },
            FinnhubSearchEntry {
                display_symbol: None,
                description: Some("Apple Inc. (duplicate)".into()),
                display: Some("AAPL".into()),
                symbol: Some("AAPL".into()),
                kind: Some("stock".into()),
            },
            FinnhubSearchEntry {
                display_symbol: None,
                description: Some("".into()),
                display: None,
                symbol: None,
                kind: None,
            },
        ];
        let results = map_search_entries(entries);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol, "AAPL");
        assert_eq!(results[0].description, "Apple Inc.");
    }

    #[test]
    fn test_map_search_entries_sorts_alphabetically() {
        let entries = vec![
            FinnhubSearchEntry {
                display_symbol: None,
                description: None,
                display: None,
                symbol: Some("MSFT".into()),
                kind: None,
            },
            FinnhubSearchEntry {
                display_symbol: None,
                description: None,
                display: None,
                symbol: Some("AAPL".into()),
                kind: None,
            },
        ];
        let results = map_search_entries(entries);
        let symbols: Vec<_> = results.iter().map(|r| r.symbol.as_str()).collect();
        assert_eq!(symbols, vec!["AAPL", "MSFT"]);
    }

    #[test]
    fn test_map_screener_rows_dedupes_and_sorts() {
        let rows = vec![
            FinnhubScreenerRow {
                ticker: Some("msft".into()),
            },
            FinnhubScreenerRow {
                ticker: Some("AAPL:US".into()),
            },
            FinnhubScreenerRow {
                ticker: Some("msft".into()),
            },
            FinnhubScreenerRow {
                ticker: Some("M7".into()),
            },
            FinnhubScreenerRow {
                ticker: None,
            },
            FinnhubScreenerRow {
                ticker: Some("".into()),
            },
        ];
        let list = map_screener_rows(rows);
        assert_eq!(list, vec!["AAPL", "M7", "MSFT"]);
    }

    #[test]
    fn test_screener_response_parsing() {
        let json = r#"{"data":[{"ticker":"BBAI"},{"ticker":"AA"},{"foo":1}]}"#;
        let resp: FinnhubScreenerResponse = serde_json::from_str(json).unwrap();
        assert_eq!(map_screener_rows(resp.data), vec!["AA", "BBAI"]);
    }

    #[test]
    fn test_all_tickers_falls_back_to_curated_list_offline() {
        // Invalid key + no usable network result: must not panic and must
        // return the curated supported-ticker list instead of an empty vec.
        let provider = FinnhubDataProvider::new("invalid-key-for-offline-test".into());
        let list = provider.all_tickers();
        assert!(!list.is_empty());
        assert!(list.contains(&"AAPL".to_string()));
    }

    #[test]
    fn test_search_empty_query_short_circuits() {
        let provider = FinnhubDataProvider::new("whatever".into());
        assert!(provider.search("   ").is_empty());
    }

    #[test]
    fn test_levenshtein_basics() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("infion", "infineon"), 2);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn test_is_subsequence() {
        assert!(is_subsequence("nvda", "nvdia"));
        assert!(is_subsequence("infi", "infion"));
        assert!(!is_subsequence("abcd", "abxd"));
    }

    #[test]
    fn test_tokenize_drops_stopwords() {
        assert_eq!(tokenize_search_query("Apple Inc."), vec!["apple"]);
        assert_eq!(
            tokenize_search_query("Bank of America Corp"),
            vec!["bank", "of", "america"]
        );
        assert_eq!(tokenize_search_query("Google Technologies, Inc."), vec!["google"]);
    }

    #[test]
    fn test_alias_lookup_nickname_and_prefix() {
        assert_eq!(alias_lookup("google"), Some(("GOOGL", "Alphabet Inc (Google)")));
        assert_eq!(alias_lookup("goog"), Some(("GOOGL", "Alphabet Inc (Google)")));
        assert_eq!(alias_lookup("facebook"), Some(("META", "Meta Platforms Inc")));
        assert_eq!(alias_lookup("zzzz"), None);
    }

    #[test]
    fn test_score_symbol_fuzzy_cases() {
        assert_eq!(score_symbol("aapl", "AAPL"), 90);
        // "apple" is the company name, not the symbol: no symbol-level match
        // (the name itself scores 95 in score_candidate via the single-token
        // exact rule for "Apple Inc").
        assert_eq!(score_symbol("apple", "AAPL"), 0);
        // lev-2 on 4-char symbols is too noisy ("aapl" ~ PYPL etc.) -> 0
        assert_eq!(score_symbol("apal", "AAPL"), 0);
        assert_eq!(score_symbol("aapls", "AAPL"), 70, "lev-1 typo (5 chars)");
        // "aaplle" = full symbol as word prefix: weak-but-real signal.
        assert!(score_symbol("aaplle", "AAPL") >= SEARCH_MIN_SCORE, "prefix of word");
        assert_eq!(score_symbol("nvd", "NVDA"), 75, "prefix");
        // "nvid" ~ NVDA: lev-2 on 4 chars is not trusted and no subsequence
        // (no 'a' in "nvid"); the company NAME "NVIDIA" still matches it via
        // score_candidate (token prefix -> 70).
        assert_eq!(score_symbol("nvid", "NVDA"), 0);
        assert_eq!(score_symbol("nvdia", "NVDA"), 70, "lev-1 typo");
        assert_eq!(score_symbol("nvidiaa", "NVDA"), 40, "subsequence");
        assert_eq!(score_symbol("zzzz", "AAPL"), 0);
    }

    #[test]
    fn test_score_candidate_name_typo() {
        // "infion" is a typo of Infineon: name-token lev-2 should score.
        let (score, matched) = score_candidate("IFX.DE", "INFINEON TECHNOLOGIES AG", &["infion".into()]);
        assert!(score >= SEARCH_MIN_SCORE, "got {}", score);
        assert_eq!(matched, 1);
        // Unrelated name scores nothing.
        let (none, none_matched) = score_candidate("INFY", "INFOSYS LTD", &["infion".into()]);
        assert!(none < SEARCH_MIN_SCORE, "got {}", none);
        assert_eq!(none_matched, 0);
        // Multi-word match count: BAC matches both "bank" and "america".
        let (bac, bac_matched) = score_candidate("BAC", "BANK OF AMERICA CORP", &["bank".into(), "of".into(), "america".into()]);
        assert_eq!(bac_matched, 2);
        let (wbhc, wbhc_matched) = score_candidate("WBHC", "WILSON BANK HOLDING CO", &["bank".into(), "of".into(), "america".into()]);
        assert_eq!(wbhc_matched, 1);
        assert!(bac >= wbhc);
    }

    #[test]
    fn test_search_google_resolves_offline_via_alias() {
        // No network: Finnhub probes fail, but the alias map must still
        // resolve the rebranded "google" -> GOOGL.
        let provider = FinnhubDataProvider::new("invalid-key-for-offline-test".into());
        let results = provider.search("google");
        assert!(!results.is_empty());
        assert_eq!(results[0].symbol, "GOOGL");
        assert!(results[0].description.contains("Alphabet"));
    }

    #[test]
    fn test_search_symbol_typo_resolves_offline_via_universe() {
        // "aapls" is a typo of AAPL: matched against the ticker universe
        // (curated fallback list contains AAPL, so this works offline).
        let provider = FinnhubDataProvider::new("invalid-key-for-offline-test".into());
        let results = provider.search("aapls");
        assert!(results.iter().any(|r| r.symbol == "AAPL"));
    }

    #[test]
    fn test_search_results_are_cached() {
        let provider = FinnhubDataProvider::new("invalid-key-for-offline-test".into());
        let first = provider.search("google");
        let entries = provider.search_cache.lock().unwrap().len();
        assert_eq!(entries, 1);
        let second = provider.search("GOOGLE"); // case-insensitive cache key
        assert_eq!(second, first);
        assert_eq!(provider.search_cache.lock().unwrap().len(), 1);
    }
}
