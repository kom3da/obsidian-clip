use crate::Result;

pub fn save(api_key: &str, port: u16, folder: &str, filename: &str, content: &str) -> Result<()> {
    let encoded = encode_path(filename);
    let url = format!("http://127.0.0.1:{}/vault/{}/{}", port, folder, encoded);

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

fn encode_path(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            ' ' => "%20".chars().collect::<Vec<_>>(),
            '#' => "%23".chars().collect::<Vec<_>>(),
            '?' => "%3F".chars().collect::<Vec<_>>(),
            '&' => "%26".chars().collect::<Vec<_>>(),
            _ => vec![c],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_spaces() {
        assert_eq!(encode_path("hello world"), "hello%20world");
    }

    #[test]
    fn encode_special_chars() {
        assert_eq!(encode_path("a#b?c&d"), "a%23b%3Fc%26d");
    }

    #[test]
    fn encode_no_change() {
        assert_eq!(encode_path("simple.md"), "simple.md");
    }

    #[test]
    fn encode_japanese() {
        let input = "2026-04-10-Rust入門.md";
        assert_eq!(encode_path(input), input);
    }
}
