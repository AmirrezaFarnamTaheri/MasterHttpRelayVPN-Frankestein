use crate::config::Config;
use url::Url;

pub const REDACTED_AUTH_KEY: &str = "<redacted-auth-key>";
pub const REDACTED_SERVERLESS_AUTH_KEY: &str = "<redacted-serverless-auth-key>";
pub const REDACTED_DEPLOYMENT_ID: &str = "<redacted-deployment-id>";
pub const REDACTED_LAN_TOKEN: &str = "<redacted-lan-token>";

pub fn mask_deployment_id(id: &str) -> String {
    let trimmed = id.trim();
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= 10 {
        REDACTED_DEPLOYMENT_ID.into()
    } else {
        let prefix: String = chars.iter().take(6).collect();
        let suffix: String = chars
            .iter()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("{prefix}...{suffix}")
    }
}

pub fn redact_url_credentials(url: &Url) -> String {
    let mut display = url.clone();
    let _ = display.set_username("");
    let _ = display.set_password(None);
    display.to_string()
}

pub fn redact_config_secrets_in_text(text: &str, cfg: &Config) -> String {
    let mut out = text.to_string();
    if let Some(groups) = cfg.account_groups.as_ref() {
        for group in groups {
            let auth = group.auth_key.trim();
            if !auth.is_empty() {
                out = out.replace(auth, REDACTED_AUTH_KEY);
            }
            for id in group.script_ids.clone().into_vec() {
                let trimmed = id.trim();
                if trimmed.is_empty() {
                    continue;
                }
                out = out.replace(trimmed, &mask_deployment_id(trimmed));
            }
        }
    }
    let vercel_auth = cfg.vercel.auth_key.trim();
    if !vercel_auth.is_empty() {
        out = out.replace(vercel_auth, REDACTED_SERVERLESS_AUTH_KEY);
    }
    if let Some(token) = cfg.lan_token.as_deref().map(str::trim) {
        if !token.is_empty() {
            out = out.replace(token, REDACTED_LAN_TOKEN);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_deployment_id_keeps_only_prefix_and_suffix() {
        assert_eq!(
            mask_deployment_id("AKfycb1234567890abcdef"),
            "AKfycb...cdef"
        );
        assert_eq!(mask_deployment_id("short"), REDACTED_DEPLOYMENT_ID);
    }

    #[test]
    fn redact_url_credentials_removes_username_and_password() {
        let url = Url::parse("https://user:pass@example.com/health/details").unwrap();
        let redacted = redact_url_credentials(&url);
        assert_eq!(redacted, "https://example.com/health/details");
    }

    #[test]
    fn redact_config_secrets_in_text_masks_known_values() {
        let cfg = Config::from_json_str(
            r#"{
                "mode": "apps_script",
                "account_groups": [{
                    "auth_key": "secret-auth-key",
                    "script_ids": ["AKfycb1234567890abcdef"]
                }],
                "vercel": { "auth_key": "serverless-secret" },
                "lan_token": "lan-secret-token"
            }"#,
        )
        .expect("config");
        let text = "secret-auth-key AKfycb1234567890abcdef serverless-secret lan-secret-token";
        let redacted = redact_config_secrets_in_text(text, &cfg);
        assert!(!redacted.contains("secret-auth-key"));
        assert!(!redacted.contains("AKfycb1234567890abcdef"));
        assert!(!redacted.contains("serverless-secret"));
        assert!(!redacted.contains("lan-secret-token"));
        assert!(redacted.contains(REDACTED_AUTH_KEY));
        assert!(redacted.contains("AKfycb...cdef"));
        assert!(redacted.contains(REDACTED_SERVERLESS_AUTH_KEY));
        assert!(redacted.contains(REDACTED_LAN_TOKEN));
    }
}
