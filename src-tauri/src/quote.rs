use encoding_rs::GBK;
use serde::Serialize;

/// Whole-market turnover snapshot from 同花顺 (10jqka).
/// Values are in yuan; the frontend converts to 亿.
#[derive(Debug, Clone, Serialize)]
pub struct MarketTurnover {
    /// 今日实时成交额 (当日成交额), in yuan.
    pub turnover: f64,
    /// 较昨日同期变动, in yuan (can be negative).
    pub turnover_change: f64,
}

/// Fetch whole-market turnover ("市场成交额分时") from 同花顺.
///
/// The response is JSON; the `data.charts.header` array carries the latest
/// summary values keyed by `turnover` (当日成交额) and `turnover_change`
/// (较昨日变动). We read those directly rather than the minute-by-minute
/// `point_list`.
pub async fn fetch_market_turnover() -> Result<MarketTurnover, String> {
    let url = "https://dq.10jqka.com.cn/fuyao/market_analysis_api/chart/v1/get_chart_data?chart_key=turnover_minute";

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .header("Referer", "https://dq.10jqka.com.cn/")
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let header = json["data"]["charts"]["header"]
        .as_array()
        .ok_or("unexpected response shape: missing data.charts.header")?;

    let mut turnover = 0.0;
    let mut turnover_change = 0.0;
    for item in header {
        match item["key"].as_str() {
            Some("turnover") => turnover = item["val"].as_f64().unwrap_or(0.0),
            Some("turnover_change") => turnover_change = item["val"].as_f64().unwrap_or(0.0),
            _ => {}
        }
    }

    Ok(MarketTurnover {
        turnover,
        turnover_change,
    })
}

/// A single stock/index quote parsed from Tencent Finance API.
#[derive(Debug, Clone, Serialize)]
pub struct Quote {
    pub code: String,
    pub name: String,
    pub price: String,
    pub yesterday_close: String,
    pub change: String,
    pub change_pct: String,
    pub high: String,
    pub low: String,
    pub time: String,
}

/// Fetch quotes from Tencent Finance API for the given codes.
/// Codes should be like: "sh000001", "sz399001", "sh600000" etc.
pub async fn fetch_quotes(codes: Vec<String>) -> Result<Vec<Quote>, String> {
    if codes.is_empty() {
        return Ok(vec![]);
    }

    let joined = codes.join(",");
    let url = format!("http://qt.gtimg.cn/q={}", joined);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    // Decode GBK to UTF-8
    let (decoded, _, had_errors) = GBK.decode(&bytes);
    if had_errors {
        // Non-fatal: some chars might fail, continue anyway
    }
    let text = decoded.to_string();

    let mut quotes = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(q) = parse_line(line) {
            quotes.push(q);
        }
    }

    Ok(quotes)
}

/// Parse one line like: v_sh600000="1~浦发银行~600000~10.50~...";
/// Fields are tilde-separated, index from 0.
fn parse_line(line: &str) -> Option<Quote> {
    // Split "v_xxx=" from the quoted payload.
    let eq = line.find('=')?;
    let var_name = &line[..eq]; // e.g. "v_sh600000"

    // Extract the full code with market prefix from the variable name.
    // var_name is like "v_sh600000" or "v_sz000001"
    let full_code = var_name.strip_prefix("v_")?.to_string();
    if full_code.is_empty() {
        return None;
    }

    let payload = &line[eq + 1..];
    // Strip surrounding quotes and trailing ';'
    let payload = payload.trim().trim_end_matches(';').trim_matches('"');
    if payload.is_empty() {
        return None;
    }

    let f: Vec<&str> = payload.split('~').collect();
    let get = |i: usize| f.get(i).map(|s| s.to_string()).unwrap_or_default();

    // Skip if no name field
    let name = get(1);
    if name.is_empty() {
        return None;
    }

    Some(Quote {
        code: full_code,
        name,
        price: get(3),
        yesterday_close: get(4),
        change: get(31),
        change_pct: get(32),
        high: get(33),
        low: get(34),
        time: get(30),
    })
}
