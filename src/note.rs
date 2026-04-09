pub fn build_filename(title: &str, date: &str) -> String {
    let safe: String = title
        .chars()
        .map(|c| if "/:\\*?\"<>|".contains(c) { '-' } else { c })
        .collect();
    let trimmed = safe.trim();
    let end = trimmed
        .char_indices()
        .nth(60)
        .map(|(i, _)| i)
        .unwrap_or(trimmed.len());
    format!("{}-{}.md", date, &trimmed[..end])
}

pub fn render_template(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (key, val) in vars {
        out = out.replace(&format!("{{{{{}}}}}", key), val);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_basic() {
        assert_eq!(
            build_filename("Hello World", "2026-04-10"),
            "2026-04-10-Hello World.md"
        );
    }

    #[test]
    fn filename_sanitizes_special_chars() {
        assert_eq!(
            build_filename("foo/bar:baz*qux", "2026-01-01"),
            "2026-01-01-foo-bar-baz-qux.md"
        );
    }

    #[test]
    fn filename_truncates_long_title() {
        let long = "a".repeat(100);
        let result = build_filename(&long, "2026-01-01");
        assert_eq!(result, format!("2026-01-01-{}.md", "a".repeat(60)));
    }

    #[test]
    fn filename_with_japanese() {
        let title = "Rustプログラミング言語入門ガイド";
        let result = build_filename(title, "2026-04-10");
        assert_eq!(result, "2026-04-10-Rustプログラミング言語入門ガイド.md");
    }

    #[test]
    fn render_replaces_placeholders() {
        let tmpl = "Title: {{title}}, URL: {{url}}";
        let result = render_template(tmpl, &[("title", "Test"), ("url", "https://example.com")]);
        assert_eq!(result, "Title: Test, URL: https://example.com");
    }

    #[test]
    fn render_leaves_unknown_placeholders() {
        let tmpl = "{{known}} and {{unknown}}";
        let result = render_template(tmpl, &[("known", "value")]);
        assert_eq!(result, "value and {{unknown}}");
    }

    #[test]
    fn render_handles_empty_template() {
        assert_eq!(render_template("", &[("title", "test")]), "");
    }

    #[test]
    fn render_multiple_occurrences() {
        let tmpl = "{{x}} + {{x}}";
        let result = render_template(tmpl, &[("x", "1")]);
        assert_eq!(result, "1 + 1");
    }
}
