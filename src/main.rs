mod config;
mod fetch;
mod gemini;
mod note;
mod obsidian;

use chrono::Local;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("使い方: obsidian-clip <URL>");
        eprintln!("環境変数: GEMINI_API_KEY, OBSIDIAN_API_KEY");
        eprintln!("設定ファイル: ~/.config/obsidian-clip/config.toml");
        std::process::exit(1);
    }

    let url = &args[1];
    let config = config::Config::load();

    println!("📄 ページを取得中: {}", url);
    let (title, content) = match fetch::fetch_page(url, &config.user_agent) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("❌ ページ取得失敗: {}", e);
            std::process::exit(1);
        }
    };
    println!("✅ タイトル: {}", title);

    let summary = if config.has_gemini_key() {
        println!("🤖 Geminiで要約中...");
        match gemini::summarize(&config, &title, &content, url) {
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
    let filename = note::build_filename(&title, &date);
    let note_content = note::render_template(
        &config.note_template,
        &[
            ("title", title.as_str()),
            ("url", url.as_str()),
            ("date", date.as_str()),
            ("summary", summary.as_str()),
        ],
    );

    if config.has_obsidian_key() {
        let api_key = config.obsidian_api_key.as_deref().unwrap_or_default();
        println!("💾 Obsidianに保存中...");
        match obsidian::save(
            api_key,
            config.obsidian_api_port,
            &config.obsidian_folder,
            &filename,
            &note_content,
        ) {
            Ok(_) => println!("✅ 保存完了: {}/{}", config.obsidian_folder, filename),
            Err(e) => {
                eprintln!("❌ Obsidian保存失敗: {}", e);
                println!("--- ノート内容 ---\n{}", note_content);
            }
        }
    } else {
        println!(
            "⚠️  OBSIDIAN_API_KEY未設定\n--- ノート内容 ---\n{}",
            note_content
        );
    }
}
