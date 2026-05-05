use std::path::Path;

use serde::Serialize;

use crate::cert_installer::{browser_trust_probe, is_ca_trusted};
use crate::config::{Config, Mode};
use crate::mitm::{CA_CERT_FILE, CA_KEY_FILE};

const SNAPSHOT_VERSION: u32 = 2;

#[derive(Clone, Debug, Serialize)]
pub struct TrustSnapshot {
    pub schema_version: u32,
    pub platform: &'static str,
    pub arch: &'static str,
    pub mode: String,
    pub ca_required: bool,
    pub ca: CaSnapshot,
    pub browser: BrowserTrustSnapshot,
    pub android: AndroidTrustSnapshot,
    pub signing: SigningSnapshot,
}

#[derive(Clone, Debug, Serialize)]
pub struct CaSnapshot {
    pub cert_path: String,
    pub key_path: String,
    pub cert_exists: bool,
    pub key_exists: bool,
    pub trusted_by_platform_probe: Option<bool>,
    pub status: TrustStatus,
    pub next_action: Option<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BrowserTrustSnapshot {
    pub firefox_nss_live_probe: TrustProbeAvailability,
    pub certutil_available: bool,
    pub firefox_profile_count: usize,
    pub firefox_profiles_with_cert_db: usize,
    pub firefox_profiles_with_enterprise_roots_marker: usize,
    pub firefox_profiles_with_user_owned_enterprise_roots: usize,
    pub firefox_profiles_with_nss_cert: Option<usize>,
    pub firefox_profiles: Vec<BrowserProfileTrustSnapshot>,
    pub chrome_nssdb_present: bool,
    pub chrome_nssdb_has_cert: Option<bool>,
    pub note: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct BrowserProfileTrustSnapshot {
    pub profile_label: String,
    pub has_cert_db: bool,
    pub nss_has_cert: Option<bool>,
    pub enterprise_roots_marker: bool,
    pub enterprise_roots_user_owned: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AndroidTrustSnapshot {
    pub user_ca_limitations_apply: bool,
    pub note: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct SigningSnapshot {
    pub android_release_keystore_policy: &'static str,
    pub ci_release_source_of_truth: &'static str,
    pub docs: &'static str,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustStatus {
    NotRequired,
    Missing,
    PresentTrusted,
    PresentUntrusted,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustProbeAvailability {
    ReadOnly,
}

pub fn snapshot(config: &Config) -> TrustSnapshot {
    let mode = config.mode_kind().ok();
    let ca_required = mode.is_none_or(ca_required_for_mode);
    let base = crate::data_dir::data_dir();
    let ca = ca_snapshot(&base, ca_required);
    let browser_probe = browser_trust_probe();
    let firefox_profiles_with_enterprise_roots_marker = browser_probe
        .firefox_profiles
        .iter()
        .filter(|p| p.enterprise_roots_marker)
        .count();
    let firefox_profiles_with_user_owned_enterprise_roots = browser_probe
        .firefox_profiles
        .iter()
        .filter(|p| p.enterprise_roots_user_owned)
        .count();
    let firefox_profiles_with_cert_db = browser_probe
        .firefox_profiles
        .iter()
        .filter(|p| p.has_cert_db)
        .count();
    let firefox_profiles_with_nss_cert = browser_probe.certutil_available.then(|| {
        browser_probe
            .firefox_profiles
            .iter()
            .filter(|p| p.nss_has_cert == Some(true))
            .count()
    });
    let firefox_profiles = browser_probe
        .firefox_profiles
        .iter()
        .map(|profile| BrowserProfileTrustSnapshot {
            profile_label: redacted_profile_label(&profile.path),
            has_cert_db: profile.has_cert_db,
            nss_has_cert: profile.nss_has_cert,
            enterprise_roots_marker: profile.enterprise_roots_marker,
            enterprise_roots_user_owned: profile.enterprise_roots_user_owned,
        })
        .collect();

    TrustSnapshot {
        schema_version: SNAPSHOT_VERSION,
        platform: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        mode: config.mode.clone(),
        ca_required,
        ca,
        browser: BrowserTrustSnapshot {
            firefox_nss_live_probe: TrustProbeAvailability::ReadOnly,
            certutil_available: browser_probe.certutil_available,
            firefox_profile_count: browser_probe.firefox_profiles.len(),
            firefox_profiles_with_cert_db,
            firefox_profiles_with_enterprise_roots_marker,
            firefox_profiles_with_user_owned_enterprise_roots,
            firefox_profiles_with_nss_cert,
            firefox_profiles,
            chrome_nssdb_present: browser_probe.chrome_nssdb_present,
            chrome_nssdb_has_cert: browser_probe.chrome_nssdb_has_cert,
            note: "Read-only browser probe reports Firefox profile/NSS DB presence, certutil availability, app CA presence when certutil can query it, and enterprise_roots state; install/remove paths still own any mutations.",
        },
        android: AndroidTrustSnapshot {
            user_ca_limitations_apply: true,
            note: "Android apps may opt out of user-installed CAs; browser behavior does not prove every app will trust the local CA.",
        },
        signing: SigningSnapshot {
            android_release_keystore_policy: "committed_keystore",
            ci_release_source_of_truth: ".github/workflows/release.yml",
            docs: "docs/android-signing.md",
        },
    }
}

fn ca_required_for_mode(mode: Mode) -> bool {
    matches!(mode, Mode::AppsScript | Mode::VercelEdge | Mode::Direct)
}

fn ca_snapshot(base: &Path, ca_required: bool) -> CaSnapshot {
    let cert_path = base.join(CA_CERT_FILE);
    let key_path = base.join(CA_KEY_FILE);
    let cert_exists = cert_path.exists();
    let key_exists = key_path.exists();
    let trusted_by_platform_probe = cert_exists.then(|| is_ca_trusted(&cert_path));
    let status = match (ca_required, cert_exists, trusted_by_platform_probe) {
        (false, _, _) => TrustStatus::NotRequired,
        (true, false, _) => TrustStatus::Missing,
        (true, true, Some(true)) => TrustStatus::PresentTrusted,
        (true, true, _) => TrustStatus::PresentUntrusted,
    };
    let next_action = match status {
        TrustStatus::NotRequired => None,
        TrustStatus::Missing => Some("Start once or run doctor-fix to generate the local CA, then install it if this mode intercepts HTTPS."),
        TrustStatus::PresentTrusted => None,
        TrustStatus::PresentUntrusted => Some("Run mhrv-f --install-cert or import ca/ca.crt into the relevant OS/browser trust store."),
    };

    CaSnapshot {
        cert_path: display_path(&cert_path),
        key_path: display_path(&key_path),
        cert_exists,
        key_exists,
        trusted_by_platform_probe,
        status,
        next_action,
    }
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

fn redacted_profile_label(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("profile")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_with_mode(mode: &str) -> Config {
        Config::from_json_str(&format!(
            r#"{{
                "mode": "{mode}",
                "account_groups": [{{
                    "name": "test",
                    "auth_key": "test-auth-key-please-change-32chars",
                    "script_ids": ["AKfycbxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"]
                }}],
                "vercel": {{
                    "base_url": "https://example.net",
                    "auth_key": "serverless-auth-key"
                }},
                "google_ip": "216.239.38.120",
                "front_domain": "www.google.com"
            }}"#
        ))
        .expect("test config")
    }

    #[test]
    fn full_mode_does_not_require_local_ca() {
        let snap = snapshot(&cfg_with_mode("full"));
        assert!(!snap.ca_required);
        assert_eq!(snap.ca.status, TrustStatus::NotRequired);
    }

    #[test]
    fn direct_mode_requires_local_ca_snapshot() {
        let snap = snapshot(&cfg_with_mode("direct"));
        assert!(snap.ca_required);
        assert!(matches!(
            snap.ca.status,
            TrustStatus::Missing | TrustStatus::PresentTrusted | TrustStatus::PresentUntrusted
        ));
    }

    #[test]
    fn browser_probe_is_read_only_snapshot() {
        let snap = snapshot(&cfg_with_mode("direct"));
        assert_eq!(
            snap.browser.firefox_nss_live_probe,
            TrustProbeAvailability::ReadOnly
        );
    }

    #[test]
    fn profile_labels_do_not_expose_parent_paths() {
        assert_eq!(
            redacted_profile_label(
                r"C:\Users\alice\AppData\Roaming\Mozilla\Firefox\Profiles\abc.default-release"
            ),
            "abc.default-release"
        );
        assert_eq!(
            redacted_profile_label("/home/alice/.mozilla/firefox/xyz.default"),
            "xyz.default"
        );
    }
}
