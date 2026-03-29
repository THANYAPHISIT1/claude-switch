use std::path::PathBuf;

use chrono::{DateTime, Local};
use serde_json::Value;

pub fn run_usage(profile: &str) {
    if let Some((pct, label)) = get_usage(profile) {
        println!("session: {}% resets {}", pct, label);
    }
}

fn get_usage(profile: &str) -> Option<(u32, String)> {
    if let Some(cache) = load_cache(profile) {
        return Some((cache.pct, cache.label));
    }

    let (session_key, cf_clearance) = load_cookies(profile)?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;
    let result = rt.block_on(fetch_usage(&session_key, &cf_clearance))?;

    save_cache(profile, result.0, &result.1);
    Some(result)
}

async fn fetch_usage(session_key: &str, cf_clearance: &str) -> Option<(u32, String)> {
    let cookie = format!("sessionKey={}; cf_clearance={}", session_key, cf_clearance);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .ok()?;

    let resp = client
        .get("https://claude.ai/api/organizations")
        .header("Cookie", cookie.as_str())
        .send()
        .await
        .ok()?;

    let orgs: Value = resp.json().await.ok()?;
    let orgs = orgs.as_array()?;

    for org in orgs {
        let Some(uuid) = org["uuid"].as_str() else { continue };
        let usage_url = format!("https://claude.ai/api/organizations/{}/usage", uuid);

        let Ok(usage_resp) = client
            .get(&usage_url)
            .header("Cookie", cookie.as_str())
            .send()
            .await
        else {
            continue;
        };

        if !usage_resp.status().is_success() {
            continue;
        }

        let Ok(usage) = usage_resp.json::<Value>().await else { continue };

        let Some(utilization) = usage["five_hour"]["utilization"].as_f64() else { continue };
        let Some(resets_at) = usage["five_hour"]["resets_at"].as_str() else { continue };

        let pct = (utilization * 100.0).round() as u32;
        let label = reset_time_label(resets_at);

        return Some((pct, label));
    }

    None
}

fn load_cookies(profile: &str) -> Option<(String, String)> {
    let content = std::fs::read_to_string(cookie_file(profile)).ok()?;
    let v: Value = serde_json::from_str(&content).ok()?;
    let session_key = v["sessionKey"].as_str()?.to_string();
    let cf_clearance = v["cf_clearance"].as_str()?.to_string();
    Some((session_key, cf_clearance))
}

struct UsageCache {
    pct: u32,
    label: String,
}

fn load_cache(profile: &str) -> Option<UsageCache> {
    let content = std::fs::read_to_string(cache_file(profile)).ok()?;
    let v: Value = serde_json::from_str(&content).ok()?;
    let pct = v["pct"].as_u64()? as u32;
    let label = v["label"].as_str()?.to_string();
    let fetched_at = v["fetched_at"].as_i64()?;

    let now = chrono::Utc::now().timestamp();
    if now - fetched_at > 300 {
        return None;
    }

    Some(UsageCache { pct, label })
}

fn save_cache(profile: &str, pct: u32, label: &str) {
    let now = chrono::Utc::now().timestamp();
    let v = serde_json::json!({
        "pct": pct,
        "label": label,
        "fetched_at": now,
    });
    let _ = std::fs::write(cache_file(profile), v.to_string());
}

fn reset_time_label(iso_str: &str) -> String {
    DateTime::parse_from_rfc3339(iso_str)
        .ok()
        .map(|dt| {
            let local: DateTime<Local> = dt.with_timezone(&Local);
            let hour = local.format("%I").to_string();
            let hour = hour.trim_start_matches('0');
            let ampm = local.format("%p").to_string().to_lowercase();
            format!("{}{}", hour, ampm)
        })
        .unwrap_or_default()
}

fn cache_file(profile: &str) -> PathBuf {
    std::env::temp_dir().join(format!("claude-usage-cache-{}.json", profile))
}

pub(crate) fn cookie_file(profile: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(format!(".claude-{}", profile))
        .join(".claude-cookies.json")
}
