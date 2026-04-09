use chrono::Local;
use scraper::{Html, Selector};
use serde_json::json;
use std::env;

const OBSIDIAN_API_PORT: u16 = 27123;

fn fetch_page(url: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let res = ureq::get(url)
        .set("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&res);

    let title_sel = Selector::parse("title").unwrap();
    let title = doc
        .select(&title_sel)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Untitled".to_string());

    let selectors = ["article p", "main p", "body p"];
    let mut paragraphs = String::new();

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
            paragraphs = text;
            break;
        }
    }

    Ok((title, paragraphs))
}

fn summarize_with_gemini(
    api_key: &str,
    title: &str,
    content: &str,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let prompt = format!(
        "以下のウェブページの内容を日本語で要約してください。\nタイトル: {}\nURL: {}\n本文:\n{}\n\n以下の形式で出力してください：\n## 要約\n（3〜5文で簡潔に）\n\n## ポイント\n- （箇条書きで3〜5点）\n\n## タグ\n内容に合ったObsidianタグを3〜5個、#付きで提案してください（例: #rust #プログラミング #Web開発）",
        title, url, content
    );

    let body = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let endpoint = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let mut last_err = String::new();
    let mut res_value: Option<serde_json::Value> = None;
    for i in 0..3 {
        if i > 0 {
            eprintln!("  リトライ中... ({}/3)", i + 1);
            std::thread::sleep(std::time::Duration::from_secs(2 * i as u64));
        }
        match ureq::post(&endpoint)
            .set("Content-Type", "application/json")
            .send_json(&body)
        {
            Ok(r) => {
                res_value = Some(r.into_json()?);
                break;
            }
            Err(e) => last_err = e.to_string(),
        }
    }
    let res = res_value.ok_or(last_err)?;

    let summary = res["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("要約に失敗しました")
        .to_string();

    Ok(summary)
}

fn save_to_obsidian(
    api_key: &str,
    filename: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let encoded: String = filename
        .chars()
        .flat_map(|c| {
            if c == ' ' {
                "%20".chars().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect();

    let url = format!(
        "http://127.0.0.1:{}/vault/Bookmarks/{}",
        OBSIDIAN_API_PORT, encoded
    );

    let res = ureq::put(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "text/markdown")
        .send_string(content)?;

    if res.status() < 300 {
        Ok(())
    } else {
        Err(format!("Obsidian API error: {}", res.status()).into())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("使い方: obsidian-clip <URL>");
        eprintln!("環境変数: GEMINI_API_KEY, OBSIDIAN_API_KEY");
        std::process::exit(1);
    }

    let url = &args[1];
    let gemini_key = env::var("GEMINI_API_KEY").unwrap_or_default();
    let obsidian_key = env::var("OBSIDIAN_API_KEY").unwrap_or_default();

    println!("📄 ページを取得中: {}", url);
    let (title, content) = match fetch_page(url) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("❌ ページ取得失敗: {}", e);
            std::process::exit(1);
        }
    };
    println!("✅ タイトル: {}", title);

    let summary = if !gemini_key.is_empty() {
        println!("🤖 Geminiで要約中...");
        match summarize_with_gemini(&gemini_key, &title, &content, url) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("⚠️  要約失敗: {}", e);
                "（要約に失敗しました）".to_string()
            }
        }
    } else {
        "（GEMINI_API_KEYを設定すると自動要約されます）".to_string()
    };

    let date = Local::now().format("%Y-%m-%d").to_string();
    let safe_title: String = title
        .chars()
        .map(|c| if "/:\\*?\"<>|".contains(c) { '-' } else { c })
        .collect();
    let truncated = safe_title.trim();
    let end = truncated.char_indices().nth(60).map(|(i, _)| i).unwrap_or(truncated.len());
    let filename = format!("{}-{}.md", date, &truncated[..end]);

    let note = format!(
        "## 概要\n- **タイトル**: {}\n- **URL**: {}\n- **保存日**: {}\n\n{}\n",
        title, url, date, summary
    );

    if !obsidian_key.is_empty() {
        println!("💾 Obsidianに保存中...");
        match save_to_obsidian(&obsidian_key, &filename, &note) {
            Ok(_) => println!("✅ 保存完了: Bookmarks/{}", filename),
            Err(e) => {
                eprintln!("❌ Obsidian保存失敗: {}", e);
                println!("--- ノート内容 ---\n{}", note);
            }
        }
    } else {
        println!("⚠️  OBSIDIAN_API_KEY未設定\n--- ノート内容 ---\n{}", note);
    }
}