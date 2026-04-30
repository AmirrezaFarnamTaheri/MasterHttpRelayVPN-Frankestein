//! Small, bounded response classifiers for support logs.
//!
//! These helpers never log full bodies. They inspect a short prefix and emit
//! hints for common relay failure signatures: HTML auth/protection pages,
//! Cloudflare/Turnstile challenges, and quota/limit pages.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QualityHint {
    HtmlInsteadOfJson,
    CloudflareChallenge,
    QuotaOrLimit,
}

impl QualityHint {
    pub fn message(self) -> &'static str {
        match self {
            QualityHint::HtmlInsteadOfJson => {
                "HTML returned where JSON was expected; check relay auth, platform protection/routing, or Apps Script sharing"
            }
            QualityHint::CloudflareChallenge => {
                "Cloudflare/Turnstile challenge marker detected; the upstream may be serving an anti-bot page"
            }
            QualityHint::QuotaOrLimit => {
                "Quota or rate-limit marker detected; add capacity or slow relay concurrency"
            }
        }
    }
}

pub fn classify(status: u16, content_type: Option<&str>, body: &[u8]) -> Option<QualityHint> {
    let prefix = String::from_utf8_lossy(&body[..body.len().min(2048)]);
    let lower = prefix.to_ascii_lowercase();
    if lower.contains("turnstile")
        || lower.contains("cf-chl")
        || lower.contains("cloudflare")
        || lower.contains("checking your browser")
    {
        return Some(QualityHint::CloudflareChallenge);
    }
    if status == 429
        || lower.contains("quota")
        || lower.contains("rate limit")
        || lower.contains("daily limit")
        || lower.contains("too many requests")
        || lower.contains("too many times")
        || lower.contains("service invoked")
        || lower.contains("resource exhausted")
    {
        return Some(QualityHint::QuotaOrLimit);
    }
    let ct = content_type.unwrap_or("").to_ascii_lowercase();
    if ct.contains("text/html")
        || lower.trim_start().starts_with("<!doctype")
        || lower.trim_start().starts_with("<html")
        || lower.contains("<title>")
    {
        return Some(QualityHint::HtmlInsteadOfJson);
    }
    None
}

pub fn log_hint(context: &str, status: u16, content_type: Option<&str>, body: &[u8]) {
    if let Some(hint) = classify(status, content_type, body) {
        tracing::warn!(
            "{}: {} (status={}, content_type={}, prefix=\"{}\")",
            context,
            hint.message(),
            status,
            content_type.unwrap_or("<none>"),
            safe_prefix(body)
        );
    }
}

fn safe_prefix(body: &[u8]) -> String {
    let raw = String::from_utf8_lossy(&body[..body.len().min(180)]);
    let mut out = String::new();
    let mut last_ws = false;
    for c in raw.chars() {
        if c.is_ascii_control() || c.is_whitespace() {
            if !last_ws {
                out.push(' ');
                last_ws = true;
            }
            continue;
        }
        last_ws = false;
        match c {
            '"' | '\'' | '`' => out.push('_'),
            _ if c.is_ascii() => out.push(c),
            _ => out.push('?'),
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_html_auth_pages() {
        assert_eq!(
            classify(
                200,
                Some("text/html"),
                b"<!doctype html><title>Login</title>"
            ),
            Some(QualityHint::HtmlInsteadOfJson)
        );
    }

    #[test]
    fn classifies_turnstile() {
        assert_eq!(
            classify(403, Some("text/html"), b"<html>Cloudflare Turnstile</html>"),
            Some(QualityHint::CloudflareChallenge)
        );
    }

    #[test]
    fn classifies_quota() {
        assert_eq!(
            classify(429, Some("text/plain"), b"Too many requests"),
            Some(QualityHint::QuotaOrLimit)
        );
    }
}
