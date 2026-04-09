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
