use serde_json::json;

use crate::config::Config;
use crate::note::render_template;
use crate::Result;

pub fn summarize(config: &Config, title: &str, content: &str, url: &str) -> Result<String> {
    let api_key = config.gemini_api_key.as_deref().unwrap_or_default();

    let prompt = render_template(
        &config.prompt_template,
        &[("title", title), ("url", url), ("content", content)],
    );

    let body = json!({
        "contents": [{ "parts": [{ "text": prompt }] }]
    });

    let endpoint = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        config.gemini_model, api_key
    );

    let res = call_with_retry(&endpoint, &body, config.max_retries)?;

    let summary = res["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("要約に失敗しました")
        .to_string();

    Ok(summary)
}

fn call_with_retry(
    endpoint: &str,
    body: &serde_json::Value,
    max_retries: u32,
) -> Result<serde_json::Value> {
    let mut last_err = String::new();

    for attempt in 1..=max_retries {
        if attempt > 1 {
            eprintln!("  リトライ中... ({}/{})", attempt, max_retries);
            std::thread::sleep(std::time::Duration::from_secs(2 * (attempt - 1) as u64));
        }

        match ureq::post(endpoint)
            .set("Content-Type", "application/json")
            .send_json(body)
        {
            Ok(res) => return Ok(res.into_json()?),
            Err(e) => last_err = e.to_string(),
        }
    }

    Err(last_err.into())
}
