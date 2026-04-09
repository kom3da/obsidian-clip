mod config;
mod fetch;
mod gemini;
mod note;
mod obsidian;

use chrono::Local;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Args {
    urls: Vec<String>,
    dry_run: bool,
}

fn parse_args() -> Args {
    let mut urls = Vec::new();
    let mut dry_run = false;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("obsidian-clip {}", VERSION);
                std::process::exit(0);
            }
            "-n" | "--dry-run" => dry_run = true,
            s if s.starts_with('-') => {
                eprintln!("不明なオプション: {}", s);
                eprintln!("--help でヘルプを表示");
                std::process::exit(1);
            }
            _ => urls.push(arg),
        }
    }

    if urls.is_empty() {
        eprintln!("エラー: URLを指定してください");
        eprintln!("--help でヘルプを表示");
        std::process::exit(1);
    }

    Args { urls, dry_run }
}

fn print_help() {
    println!(
        "\
obsidian-clip {}
Webページを取得・要約してObsidianに保存するCLIツール

使い方:
    obsidian-clip [オプション] <URL>...

オプション:
    -n, --dry-run    保存せずにノート内容をプレビュー
    -h, --help       ヘルプを表示
    -V, --version    バージョンを表示

環境変数:
    GEMINI_API_KEY    Gemini APIキー
    OBSIDIAN_API_KEY  Obsidian Local REST APIキー

設定ファイル:
    ~/.config/obsidian-clip/config.toml",
        VERSION
    );
}

fn process_url(url: &str, config: &config::Config, dry_run: bool) {
    println!("\n📄 ページを取得中: {}", url);
    let (title, content) = match fetch::fetch_page(url, &config.user_agent) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("❌ ページ取得失敗: {}", e);
            return;
        }
    };
    println!("✅ タイトル: {}", title);

    let summary = if config.has_gemini_key() {
        println!("🤖 Geminiで要約中...");
        match gemini::summarize(config, &title, &content, url) {
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
            ("url", url),
            ("date", date.as_str()),
            ("summary", summary.as_str()),
        ],
    );

    if dry_run {
        println!("--- プレビュー: {} ---\n{}", filename, note_content);
        return;
    }

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

fn main() {
    let args = parse_args();
    let config = config::Config::load();

    for url in &args.urls {
        process_url(url, &config, args.dry_run);
    }
}
