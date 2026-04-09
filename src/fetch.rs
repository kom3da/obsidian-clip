use scraper::{Html, Selector};

use crate::Result;

pub fn fetch_page(url: &str, user_agent: &str) -> Result<(String, String)> {
    let html = ureq::get(url)
        .set("User-Agent", user_agent)
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&html);
    let title = extract_title(&doc);
    let body = extract_body(&doc);

    Ok((title, body))
}

fn extract_title(doc: &Html) -> String {
    let sel = Selector::parse("title").unwrap();
    doc.select(&sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Untitled".to_string())
}

fn extract_body(doc: &Html) -> String {
    let selectors = ["article p", "main p", "body p"];

    for sel_str in &selectors {
        let sel = Selector::parse(sel_str).unwrap();
        let text: String = doc
            .select(&sel)
            .map(|e| e.text().collect::<String>())
            .filter(|t| t.trim().len() > 30)
            .take(20)
            .collect::<Vec<_>>()
            .join("\n\n");

        if !text.is_empty() {
            return text;
        }
    }

    String::new()
}
