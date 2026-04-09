use serde::Deserialize;
use std::env;
use std::fs;

const DEFAULT_MODEL: &str = "gemini-2.5-flash";
const DEFAULT_PORT: u16 = 27123;
const DEFAULT_FOLDER: &str = "Bookmarks";
const DEFAULT_RETRIES: u32 = 3;

pub const DEFAULT_NOTE_TEMPLATE: &str = "\
## 概要
- **タイトル**: {{title}}
- **URL**: {{url}}
- **保存日**: {{date}}

{{summary}}
";

pub const DEFAULT_PROMPT_TEMPLATE: &str = "\
以下のウェブページの内容を日本語で要約してください。
タイトル: {{title}}
URL: {{url}}
本文:
{{content}}

以下の形式で出力してください：
## 要約
（3〜5文で簡潔に）

## ポイント
- （箇条書きで3〜5点）

## タグ
内容に合ったObsidianタグを3〜5個、#付きで提案してください（例: #rust #プログラミング #Web開発）";

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub gemini_api_key: Option<String>,
    pub obsidian_api_key: Option<String>,
    pub gemini_model: String,
    pub obsidian_api_port: u16,
    pub obsidian_folder: String,
    pub max_retries: u32,
    pub user_agent: String,
    pub note_template: String,
    pub prompt_template: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gemini_api_key: None,
            obsidian_api_key: None,
            gemini_model: DEFAULT_MODEL.to_string(),
            obsidian_api_port: DEFAULT_PORT,
            obsidian_folder: DEFAULT_FOLDER.to_string(),
            max_retries: DEFAULT_RETRIES,
            user_agent: default_user_agent(),
            note_template: DEFAULT_NOTE_TEMPLATE.to_string(),
            prompt_template: DEFAULT_PROMPT_TEMPLATE.to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut config = load_from_file().unwrap_or_default();

        if let Ok(key) = env::var("GEMINI_API_KEY") {
            if !key.is_empty() {
                config.gemini_api_key = Some(key);
            }
        }
        if let Ok(key) = env::var("OBSIDIAN_API_KEY") {
            if !key.is_empty() {
                config.obsidian_api_key = Some(key);
            }
        }

        config
    }

    pub fn has_gemini_key(&self) -> bool {
        self.gemini_api_key.as_ref().is_some_and(|k| !k.is_empty())
    }

    pub fn has_obsidian_key(&self) -> bool {
        self.obsidian_api_key
            .as_ref()
            .is_some_and(|k| !k.is_empty())
    }
}

fn load_from_file() -> Option<Config> {
    let path = dirs::config_dir()?.join("obsidian-clip/config.toml");
    let content = fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

fn default_user_agent() -> String {
    let os = if cfg!(target_os = "macos") {
        "Macintosh; Intel Mac OS X 10_15_7"
    } else if cfg!(target_os = "windows") {
        "Windows NT 10.0; Win64; x64"
    } else {
        "X11; Linux x86_64"
    };
    format!("Mozilla/5.0 ({})", os)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.gemini_model, "gemini-2.5-flash");
        assert_eq!(config.obsidian_api_port, 27123);
        assert_eq!(config.obsidian_folder, "Bookmarks");
        assert_eq!(config.max_retries, 3);
        assert!(config.gemini_api_key.is_none());
        assert!(config.obsidian_api_key.is_none());
    }

    #[test]
    fn has_key_checks() {
        let mut config = Config::default();
        assert!(!config.has_gemini_key());
        assert!(!config.has_obsidian_key());

        config.gemini_api_key = Some("key123".to_string());
        assert!(config.has_gemini_key());

        config.gemini_api_key = Some("".to_string());
        assert!(!config.has_gemini_key());
    }

    #[test]
    fn parse_partial_toml() {
        let toml_str = r#"
            gemini_model = "gemini-2.0-pro"
            obsidian_folder = "Clips"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.gemini_model, "gemini-2.0-pro");
        assert_eq!(config.obsidian_folder, "Clips");
        // 未指定のフィールドはデフォルト値
        assert_eq!(config.obsidian_api_port, 27123);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn user_agent_contains_mozilla() {
        let ua = default_user_agent();
        assert!(ua.starts_with("Mozilla/5.0"));
    }

    #[test]
    fn default_templates_contain_placeholders() {
        assert!(DEFAULT_NOTE_TEMPLATE.contains("{{title}}"));
        assert!(DEFAULT_NOTE_TEMPLATE.contains("{{url}}"));
        assert!(DEFAULT_NOTE_TEMPLATE.contains("{{date}}"));
        assert!(DEFAULT_NOTE_TEMPLATE.contains("{{summary}}"));
        assert!(DEFAULT_PROMPT_TEMPLATE.contains("{{title}}"));
        assert!(DEFAULT_PROMPT_TEMPLATE.contains("{{content}}"));
    }
}
