use std::path::{Path, PathBuf};
use std::process::Command;

use crate::mitm::{CA_DIR, CERT_NAME};

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("certificate file not found: {0}")]
    NotFound(String),
    #[error("install failed on this platform")]
    Failed,
    #[error("unsupported platform: {0}")]
    Unsupported(String),
    #[error("io {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("CA still trusted after removal; re-run with admin/sudo")]
    RemovalIncomplete,
}

#[derive(Debug, Clone, Copy)]
pub enum RemovalOutcome {
    Clean,
    NssIncomplete(NssReport),
}

impl RemovalOutcome {
    pub fn summary(&self) -> String {
        match self {
            RemovalOutcome::Clean => "CA removed.".to_string(),
            RemovalOutcome::NssIncomplete(r) if r.tool_missing_with_stores_present => {
                "OS CA removed. NSS cleanup skipped; NSS certutil not found.".to_string()
            }
            RemovalOutcome::NssIncomplete(r) => format!(
                "OS CA removed. NSS cleanup partial: {}/{} browser stores updated.",
                r.ok, r.tried
            ),
        }
    }
}

pub fn reconcile_sudo_environment() {
    #[cfg(unix)]
    unix::reconcile_sudo_home();
}

#[cfg(unix)]
mod unix {
    use super::{should_reconcile_for, sudo_parse_passwd_home};
    use std::path::Path;
    use std::process::Command;

    pub(super) fn reconcile_sudo_home() {
        let euid = unsafe { libc::geteuid() };
        let sudo_user_raw = std::env::var("SUDO_USER").ok();
        let Some(sudo_user) = should_reconcile_for(euid, sudo_user_raw.as_deref()) else {
            return;
        };
        let sudo_user = sudo_user.to_string();
        match resolve_home(&sudo_user) {
            Some(home) => {
                tracing::info!(
                    "Detected sudo invocation (SUDO_USER={}): re-rooting HOME to {} so user-scoped cert paths target the real user.",
                    sudo_user,
                    home
                );
                std::env::set_var("HOME", home);
            }
            None => {
                tracing::warn!(
                    "Running under sudo (SUDO_USER={}), but could not resolve the user's home dir. Cert paths will operate on root's HOME.",
                    sudo_user
                );
            }
        }
    }

    fn resolve_home(sudo_user: &str) -> Option<String> {
        if let Ok(h) = std::env::var("SUDO_HOME") {
            if !h.is_empty() {
                return Some(h);
            }
        }
        if let Ok(out) = Command::new("getent").args(["passwd", sudo_user]).output() {
            if out.status.success() {
                let line = String::from_utf8_lossy(&out.stdout);
                if let Some(h) = sudo_parse_passwd_home(&line) {
                    return Some(h);
                }
            }
        }
        for root in ["/Users", "/home"] {
            let candidate = format!("{}/{}", root, sudo_user);
            if Path::new(&candidate).exists() {
                return Some(candidate);
            }
        }
        None
    }
}

#[cfg_attr(not(unix), allow(dead_code))]
fn should_reconcile_for(euid: u32, sudo_user: Option<&str>) -> Option<&str> {
    if euid != 0 {
        return None;
    }
    let user = sudo_user?;
    if user.is_empty() || user == "root" {
        return None;
    }
    Some(user)
}

#[cfg_attr(not(unix), allow(dead_code))]
fn sudo_parse_passwd_home(content: &str) -> Option<String> {
    let line = content.lines().next()?;
    let fields: Vec<&str> = line.split(':').collect();
    if fields.len() < 7 {
        return None;
    }
    let home = fields[5].trim();
    if home.is_empty() {
        return None;
    }
    Some(home.to_string())
}

/// Install the CA certificate at `path` into the system trust store.
/// Platform-specific — requires admin/sudo on most systems.
pub fn install_ca(path: &Path) -> Result<(), InstallError> {
    if !path.exists() {
        return Err(InstallError::NotFound(path.display().to_string()));
    }

    let path_s = path.to_string_lossy().to_string();

    let os = std::env::consts::OS;
    tracing::info!("Installing CA certificate on {}...", os);

    let ok = match os {
        "macos" => install_macos(&path_s),
        "linux" => install_linux(&path_s),
        "windows" => install_windows(&path_s),
        other => return Err(InstallError::Unsupported(other.to_string())),
    };

    // Best-effort: also install into NSS stores if `certutil` is available.
    // Both Firefox AND Chrome/Chromium on Linux maintain NSS databases that
    // are independent of the OS trust store — which is why running
    // update-ca-certificates alone wasn't enough for a lot of users
    // On some Linux distros, system trust and NSS must both see the cert.
    install_nss_stores(&path_s);

    if ok {
        Ok(())
    } else {
        Err(InstallError::Failed)
    }
}

/// Heuristic check: is the CA already in the trust store?
/// Best-effort — on unknown state we return false to always attempt install.
pub fn is_ca_trusted(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    match std::env::consts::OS {
        "windows" => is_trusted_windows(path),
        _ => is_ca_trusted_by_name(),
    }
}

pub fn is_ca_trusted_by_name() -> bool {
    match std::env::consts::OS {
        "macos" => is_trusted_macos(),
        "linux" => is_trusted_linux(),
        "windows" => is_trusted_windows_by_name(),
        _ => false,
    }
}

/// Remove the generated CA from OS/browser trust stores and delete the local
/// `ca/` directory only after OS trust no longer appears active.
pub fn remove_ca(base_dir: &Path) -> Result<RemovalOutcome, InstallError> {
    let os = std::env::consts::OS;
    tracing::info!("Removing CA certificate on {}...", os);

    let platform_ok = match os {
        "macos" => {
            remove_macos();
            true
        }
        "linux" => remove_linux(),
        "windows" => {
            remove_windows();
            true
        }
        other => return Err(InstallError::Unsupported(other.to_string())),
    };

    if !platform_ok || is_ca_trusted_by_name() {
        tracing::error!(
            "MITM CA is still trusted after OS removal attempt (platform_ok={}); refusing to touch browser state or delete on-disk files.",
            platform_ok
        );
        return Err(InstallError::RemovalIncomplete);
    }

    let nss = remove_nss_stores();

    let ca_dir = base_dir.join(CA_DIR);
    if ca_dir.exists() {
        std::fs::remove_dir_all(&ca_dir).map_err(|e| InstallError::Io {
            path: ca_dir.clone(),
            source: e,
        })?;
        tracing::info!("Deleted local CA directory: {}", ca_dir.display());
    }

    if nss.is_clean() {
        Ok(RemovalOutcome::Clean)
    } else {
        Ok(RemovalOutcome::NssIncomplete(nss))
    }
}

// ---------- macOS ----------

fn install_macos(cert_path: &str) -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    let login_kc_db = format!("{}/Library/Keychains/login.keychain-db", home);
    let login_kc = format!("{}/Library/Keychains/login.keychain", home);
    let login_keychain = if Path::new(&login_kc_db).exists() {
        login_kc_db
    } else {
        login_kc
    };

    // Try login keychain first (no sudo).
    let res = Command::new("security")
        .args([
            "add-trusted-cert",
            "-d",
            "-r",
            "trustRoot",
            "-k",
            &login_keychain,
            cert_path,
        ])
        .status();
    if let Ok(s) = res {
        if s.success() {
            tracing::info!("CA installed into login keychain.");
            return true;
        }
    }

    // Fall back to system keychain (needs sudo).
    tracing::warn!("login keychain install failed — trying system keychain (needs sudo).");
    let res = Command::new("sudo")
        .args([
            "security",
            "add-trusted-cert",
            "-d",
            "-r",
            "trustRoot",
            "-k",
            "/Library/Keychains/System.keychain",
            cert_path,
        ])
        .status();
    if let Ok(s) = res {
        if s.success() {
            tracing::info!("CA installed into System keychain.");
            return true;
        }
    }
    tracing::error!("macOS install failed — run with sudo or install manually.");
    false
}

fn is_trusted_macos() -> bool {
    let out = Command::new("security")
        .args(["find-certificate", "-a", "-c", CERT_NAME])
        .output();
    match out {
        Ok(o) => !o.stdout.is_empty() && o.status.success(),
        Err(_) => false,
    }
}

fn remove_macos() -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    let login_kc_db = format!("{}/Library/Keychains/login.keychain-db", home);
    let login_kc = format!("{}/Library/Keychains/login.keychain", home);
    let login_keychain = if Path::new(&login_kc_db).exists() {
        login_kc_db
    } else {
        login_kc
    };
    for args in [
        vec!["delete-certificate", "-c", CERT_NAME, &login_keychain],
        vec![
            "delete-certificate",
            "-c",
            CERT_NAME,
            "/Library/Keychains/System.keychain",
        ],
    ] {
        let _ = Command::new("security").args(&args).status();
    }
    if is_trusted_macos() {
        tracing::warn!("macOS CA removal did not fully clear trust; admin access may be required");
        false
    } else {
        true
    }
}

// ---------- Linux ----------

fn install_linux(cert_path: &str) -> bool {
    let distro = detect_linux_distro();
    tracing::info!("Detected Linux distro family: {}", distro);
    let safe_name = CERT_NAME.replace(' ', "_");

    match distro.as_str() {
        "debian" => {
            let dest = format!("/usr/local/share/ca-certificates/{}.crt", safe_name);
            try_copy_and_run(cert_path, &dest, &[&["update-ca-certificates"]])
        }
        "rhel" => {
            let dest = format!("/etc/pki/ca-trust/source/anchors/{}.crt", safe_name);
            try_copy_and_run(cert_path, &dest, &[&["update-ca-trust", "extract"]])
        }
        "arch" => {
            let dest = format!(
                "/etc/ca-certificates/trust-source/anchors/{}.crt",
                safe_name
            );
            try_copy_and_run(cert_path, &dest, &[&["trust", "extract-compat"]])
        }
        "openwrt" => {
            // OpenWRT itself doesn't open HTTPS connections through the proxy —
            // LAN clients do. The CA needs to be trusted on the CLIENTS, not on
            // the router. So this is a no-op success with guidance rather than
            // an error.
            tracing::info!(
                "OpenWRT detected: the router doesn't need to trust the MITM CA. \
                 Copy {} to each LAN client (browser / OS trust store) instead. \
                 Example: scp root@<router>:{} ./ and import from there.",
                cert_path,
                cert_path
            );
            true
        }
        _ => {
            tracing::warn!(
                "Unknown Linux distro — CA file is at {}. Copy it into your system's \
                 trust anchors dir (e.g. /usr/local/share/ca-certificates/ for \
                 Debian-like, /etc/pki/ca-trust/source/anchors/ for RHEL-like) and \
                 run the corresponding refresh command.",
                cert_path
            );
            false
        }
    }
}

fn try_copy_and_run(src: &str, dest: &str, cmds: &[&[&str]]) -> bool {
    // First try without sudo.
    let mut ok = true;
    if let Some(parent) = Path::new(dest).parent() {
        if std::fs::create_dir_all(parent).is_err() {
            ok = false;
        }
    }
    if ok && std::fs::copy(src, dest).is_err() {
        ok = false;
    }
    if ok {
        for cmd in cmds {
            if !run_cmd(cmd) {
                ok = false;
                break;
            }
        }
    }
    if ok {
        tracing::info!("CA installed via {}.", cmds[0].join(" "));
        return true;
    }

    // Retry with sudo.
    tracing::warn!("direct install failed — retrying with sudo.");
    if !run_cmd(&["sudo", "cp", src, dest]) {
        return false;
    }
    for cmd in cmds {
        let mut full: Vec<&str> = vec!["sudo"];
        full.extend_from_slice(cmd);
        if !run_cmd(&full) {
            return false;
        }
    }
    tracing::info!("CA installed via sudo.");
    true
}

fn run_cmd(args: &[&str]) -> bool {
    if args.is_empty() {
        return false;
    }
    let out = Command::new(args[0]).args(&args[1..]).status();
    matches!(out, Ok(s) if s.success())
}

fn detect_linux_distro() -> String {
    // Marker-file shortcuts (most reliable).
    if Path::new("/etc/openwrt_release").exists() {
        return "openwrt".into();
    }
    if Path::new("/etc/debian_version").exists() {
        return "debian".into();
    }
    if Path::new("/etc/redhat-release").exists() || Path::new("/etc/fedora-release").exists() {
        return "rhel".into();
    }
    if Path::new("/etc/arch-release").exists() {
        return "arch".into();
    }
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        return classify_os_release(&content);
    }
    "unknown".into()
}

/// Parse /etc/os-release content and return a distro family.
///
/// We specifically look at the `ID` and `ID_LIKE` fields (not a substring
/// search over the whole file) because random other fields like
/// `OPENWRT_DEVICE_ARCH=x86_64` contain substrings that false-positive on
/// "arch". Exposed for unit testing.
fn classify_os_release(content: &str) -> String {
    let mut id = String::new();
    let mut id_like = String::new();
    for line in content.lines() {
        let (k, v) = match line.split_once('=') {
            Some(x) => x,
            None => continue,
        };
        let v = v
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_ascii_lowercase();
        match k.trim() {
            "ID" => id = v,
            "ID_LIKE" => id_like = v,
            _ => {}
        }
    }
    let tokens: Vec<&str> = id
        .split(|c: char| c.is_whitespace() || c == ',')
        .chain(id_like.split(|c: char| c.is_whitespace() || c == ','))
        .filter(|t| !t.is_empty())
        .collect();
    let has = |needle: &str| tokens.contains(&needle);
    if has("openwrt") {
        return "openwrt".into();
    }
    if has("debian") || has("ubuntu") || has("mint") || has("raspbian") {
        return "debian".into();
    }
    if has("fedora") || has("rhel") || has("centos") || has("rocky") || has("almalinux") {
        return "rhel".into();
    }
    if has("arch") || has("manjaro") || has("endeavouros") {
        return "arch".into();
    }
    "unknown".into()
}

fn is_trusted_linux() -> bool {
    let anchor_dirs = [
        "/usr/local/share/ca-certificates",
        "/etc/pki/ca-trust/source/anchors",
        "/etc/ca-certificates/trust-source/anchors",
    ];
    for d in anchor_dirs {
        if let Ok(entries) = std::fs::read_dir(d) {
            for e in entries.flatten() {
                let name = e.file_name();
                let s = name.to_string_lossy().to_lowercase();
                if s.contains("masterhttprelayvpn") || s.contains("mhrv") {
                    return true;
                }
            }
        }
    }
    false
}

fn remove_linux() -> bool {
    let safe_name = CERT_NAME.replace(' ', "_");
    let candidates = [
        format!("/usr/local/share/ca-certificates/{}.crt", safe_name),
        format!("/usr/local/share/ca-certificates/{}.crt", CERT_NAME),
        format!("/etc/pki/ca-trust/source/anchors/{}.crt", safe_name),
        format!("/etc/pki/ca-trust/source/anchors/{}.crt", CERT_NAME),
        format!(
            "/etc/ca-certificates/trust-source/anchors/{}.crt",
            safe_name
        ),
        format!(
            "/etc/ca-certificates/trust-source/anchors/{}.crt",
            CERT_NAME
        ),
    ];
    let mut removed = false;
    for path in candidates {
        let p = Path::new(&path);
        if p.exists() {
            match std::fs::remove_file(p) {
                Ok(()) => removed = true,
                Err(e) => tracing::warn!("could not remove CA anchor {}: {}", p.display(), e),
            }
        }
    }
    for cmd in [
        &["update-ca-certificates"][..],
        &["update-ca-trust", "extract"][..],
        &["trust", "extract-compat"][..],
    ] {
        let _ = run_cmd(cmd);
    }
    removed || !is_trusted_linux()
}

// ---------- Windows ----------

/// Check whether our exact CA is present in the Windows Trusted Root store.
///
/// This compares SHA-1 thumbprints instead of subject names. Subject/name
/// matching can false-positive if an old or unrelated CA shares `CERT_NAME`.
fn is_trusted_windows(path: &Path) -> bool {
    let Some(want) = windows_cert_thumbprint(path) else {
        tracing::debug!(
            "Windows CA trust check: could not compute thumbprint for {}",
            path.display()
        );
        return false;
    };

    for args in [vec!["-user", "-store", "Root"], vec!["-store", "Root"]] {
        let out = Command::new("certutil").args(&args).output();
        if let Ok(o) = out {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if o.status.success() && normalized_hex_contains(&stdout, &want) {
                return true;
            }
        }
    }
    false
}

fn is_trusted_windows_by_name() -> bool {
    windows_store_has(true) || windows_store_has(false)
}

fn windows_store_has(user: bool) -> bool {
    let mut args: Vec<&str> = Vec::new();
    if user {
        args.push("-user");
    }
    args.extend(["-store", "Root", CERT_NAME]);
    let out = Command::new("certutil").args(&args).output();
    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            o.status.success()
                && stdout
                    .to_ascii_lowercase()
                    .contains(&CERT_NAME.to_ascii_lowercase())
        }
        Err(_) => false,
    }
}

fn windows_cert_thumbprint(path: &Path) -> Option<String> {
    let out = Command::new("certutil")
        .args(["-hashfile", path.to_string_lossy().as_ref(), "SHA1"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
        if compact.len() >= 40 && compact.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(compact[..40].to_ascii_lowercase());
        }
    }
    None
}

fn normalized_hex_contains(haystack: &str, needle_hex: &str) -> bool {
    only_hex(haystack).contains(&needle_hex.to_ascii_lowercase())
}

fn only_hex(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_hexdigit())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn install_windows(cert_path: &str) -> bool {
    // Per-user Root store (no admin required).
    let res = Command::new("certutil")
        .args(["-addstore", "-user", "Root", cert_path])
        .status();
    if let Ok(s) = res {
        if s.success() {
            tracing::info!("CA installed in Windows user Trusted Root store.");
            return true;
        }
    }
    // System store (admin).
    let res = Command::new("certutil")
        .args(["-addstore", "Root", cert_path])
        .status();
    if let Ok(s) = res {
        if s.success() {
            tracing::info!("CA installed in Windows system Trusted Root store.");
            return true;
        }
    }
    tracing::error!("Windows install failed — run as administrator or install manually.");
    false
}

fn remove_windows() {
    let mut any = false;

    if windows_store_has(true) {
        let res = Command::new("certutil")
            .args(["-delstore", "-user", "Root", CERT_NAME])
            .status();
        if matches!(res, Ok(s) if s.success()) {
            tracing::info!("Removed CA from Windows user Trusted Root store.");
            any = true;
        } else {
            tracing::warn!("failed to remove CA from Windows user Trusted Root store");
        }
    }

    if windows_store_has(false) {
        let res = Command::new("certutil")
            .args(["-delstore", "Root", CERT_NAME])
            .status();
        if matches!(res, Ok(s) if s.success()) {
            tracing::info!("Removed CA from Windows machine Trusted Root store.");
            any = true;
        } else {
            tracing::warn!(
                "failed to remove CA from Windows machine Trusted Root store (run as administrator to complete)"
            );
        }
    }

    if !any {
        tracing::info!("No MITM CA found in Windows Trusted Root stores.");
    }
}

// ---------- NSS (Firefox + Chrome/Chromium on Linux) ----------

/// Best-effort install of the CA into all discovered NSS stores:
///   1. Every Firefox profile (each has its own cert9.db).
///   2. On Linux, the shared Chrome/Chromium NSS DB at ~/.pki/nssdb —
///      this is the one update-ca-certificates does NOT populate, and
///      missing it was the real blocker for Chrome users who'd installed
///      the OS-level CA and still see TLS errors in the browser.
///
/// Silently no-ops if `certutil` (from libnss3-tools) isn't on PATH.
/// Browsers must be closed during install for changes to take effect.
fn install_nss_stores(cert_path: &str) {
    // First, try to make Firefox pick up the OS-level CA automatically by
    // flipping the `security.enterprise_roots.enabled` pref in user.js of
    // every Firefox profile we find. This is the cleanest cross-platform
    // fix because it doesn't depend on whether NSS certutil is installed
    // — Firefox just starts trusting whatever the OS trusts. Especially
    // important on Windows where NSS certutil isn't on PATH.
    enable_firefox_enterprise_roots();

    if !has_nss_certutil() {
        tracing::debug!(
            "NSS certutil not found — Firefox will still trust the CA via the \
             `security.enterprise_roots.enabled` user.js pref (flipped above). \
             For Chrome/Chromium on Linux, install `libnss3-tools` (Debian/Ubuntu) \
             or `nss-tools` (Fedora/RHEL), or import ca.crt manually via \
             chrome://settings/certificates → Authorities."
        );
        return;
    }

    let mut ok = 0;
    let mut tried = 0;

    // 1. Firefox profiles.
    for p in firefox_profile_dirs() {
        tried += 1;
        if install_nss_in_profile(&p, cert_path) {
            ok += 1;
        }
    }

    // 2. Chrome/Chromium shared NSS DB (Linux only).
    #[cfg(target_os = "linux")]
    {
        if let Some(nssdb) = chrome_nssdb_path() {
            // Ensure the DB exists. certutil -N creates an empty cert9.db in
            // the directory if none is there. An empty passphrase is fine
            // for a user-local DB.
            let dir_arg = format!("sql:{}", nssdb.display());
            if !nssdb.join("cert9.db").exists() && !nssdb.join("cert8.db").exists() {
                let _ = std::fs::create_dir_all(&nssdb);
                let _ = Command::new("certutil")
                    .args(["-N", "-d", &dir_arg, "--empty-password"])
                    .output();
            }
            tried += 1;
            if install_nss_in_dir(&dir_arg, cert_path) {
                ok += 1;
                tracing::info!(
                    "CA installed in Chrome/Chromium NSS DB: {}",
                    nssdb.display()
                );
            }
        }
    }

    if ok > 0 {
        tracing::info!("CA installed in {}/{} NSS store(s).", ok, tried);
    } else if tried > 0 {
        tracing::warn!(
            "NSS install: 0/{} stores updated. If Firefox/Chrome was running, close \
             them and retry. Otherwise, import ca.crt manually via browser settings.",
            tried
        );
    }
}

/// Write `user_pref("security.enterprise_roots.enabled", true);` to every
/// discovered Firefox profile's user.js. This makes Firefox trust the OS
/// trust store on next startup — so our already-successful system-level
/// CA install automatically propagates. Critical on Windows where Firefox
/// keeps its own NSS DB independent of Windows cert store, and NSS
/// certutil isn't typically installed so the certutil-based path doesn't
/// fire there.
///
/// Existing user.js entries for other prefs are preserved by appending
/// rather than truncating. Idempotent.
fn enable_firefox_enterprise_roots() {
    let mut touched = 0;
    for profile in firefox_profile_dirs() {
        let user_js = profile.join("user.js");
        let existing = std::fs::read_to_string(&user_js).unwrap_or_default();
        match add_enterprise_roots_block(&existing) {
            EnterpriseRootsEdit::AddedBlock(new) => {
                if let Err(e) = std::fs::write(&user_js, new) {
                    tracing::debug!(
                        "firefox profile {}: user.js write failed: {}",
                        profile.display(),
                        e
                    );
                    continue;
                }
                touched += 1;
            }
            EnterpriseRootsEdit::AlreadyOurs => {}
            EnterpriseRootsEdit::UserOwned => {
                tracing::debug!(
                    "firefox profile {} already has a user-owned enterprise_roots pref; leaving alone",
                    profile.display()
                );
            }
        }
    }
    if touched > 0 {
        tracing::info!(
            "enabled Firefox enterprise_roots in {} profile(s) — restart Firefox for it to take effect",
            touched
        );
    }
}

const FX_MARKER: &str = "// mhrv-f: auto-added, safe to strip with --remove-cert";
const FX_PREF: &str = r#"user_pref("security.enterprise_roots.enabled", true);"#;

#[derive(Debug, PartialEq, Eq)]
enum EnterpriseRootsEdit {
    AddedBlock(String),
    AlreadyOurs,
    UserOwned,
}

fn add_enterprise_roots_block(existing: &str) -> EnterpriseRootsEdit {
    if contains_our_block(existing) {
        return EnterpriseRootsEdit::AlreadyOurs;
    }
    if existing.contains("security.enterprise_roots.enabled") {
        return EnterpriseRootsEdit::UserOwned;
    }
    let mut out = existing.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(FX_MARKER);
    out.push('\n');
    out.push_str(FX_PREF);
    out.push('\n');
    EnterpriseRootsEdit::AddedBlock(out)
}

fn strip_enterprise_roots_block(existing: &str) -> Option<String> {
    if !contains_our_block(existing) {
        return None;
    }
    let lines: Vec<&str> = existing.lines().collect();
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let is_marker = lines[i].trim() == FX_MARKER;
        let next_is_our_pref = lines.get(i + 1).is_some_and(|l| l.trim() == FX_PREF);
        if is_marker && next_is_our_pref {
            i += 2;
            continue;
        }
        out.push(lines[i]);
        i += 1;
    }
    let mut joined = out.join("\n");
    if existing.ends_with('\n') && !joined.is_empty() {
        joined.push('\n');
    }
    Some(joined)
}

fn contains_our_block(existing: &str) -> bool {
    let mut prev: Option<&str> = None;
    for line in existing.lines() {
        if prev.map(str::trim) == Some(FX_MARKER) && line.trim() == FX_PREF {
            return true;
        }
        prev = Some(line);
    }
    false
}

fn has_bare_enterprise_roots(existing: &str) -> bool {
    if contains_our_block(existing) {
        return false;
    }
    existing.lines().any(|l| l.trim() == FX_PREF)
}

fn has_nss_certutil() -> bool {
    Command::new("certutil")
        .arg("--help")
        .output()
        .ok()
        .map(|o| {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&o.stderr),
                String::from_utf8_lossy(&o.stdout)
            );
            combined.to_ascii_lowercase().contains("nickname")
        })
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn chrome_nssdb_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(std::path::PathBuf::from(format!("{}/.pki/nssdb", home)))
}

/// Install into a given sql: or classic NSS DB path. Factored out so both
/// Firefox-per-profile and Chrome-shared paths share one code path.
fn install_nss_in_dir(dir_arg: &str, cert_path: &str) -> bool {
    // Delete any stale entry first (ignore errors).
    let _ = Command::new("certutil")
        .args(["-D", "-n", CERT_NAME, "-d", dir_arg])
        .output();

    let res = Command::new("certutil")
        .args([
            "-A", "-n", CERT_NAME, "-t", "C,,", "-d", dir_arg, "-i", cert_path,
        ])
        .output();
    match res {
        Ok(o) if o.status.success() => {
            if nss_cert_present(dir_arg) {
                tracing::debug!("NSS install verified: {}", dir_arg);
                true
            } else {
                tracing::debug!("NSS install completed but verification failed: {}", dir_arg);
                false
            }
        }
        Ok(o) => {
            tracing::debug!(
                "NSS install failed for {}: {}",
                dir_arg,
                String::from_utf8_lossy(&o.stderr).trim()
            );
            false
        }
        Err(e) => {
            tracing::debug!("NSS certutil exec failed for {}: {}", dir_arg, e);
            false
        }
    }
}

fn nss_cert_present(dir_arg: &str) -> bool {
    Command::new("certutil")
        .args(["-L", "-n", CERT_NAME, "-d", dir_arg])
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NssReport {
    pub tried: usize,
    pub ok: usize,
    pub tool_missing_with_stores_present: bool,
}

impl NssReport {
    pub fn is_clean(&self) -> bool {
        !self.tool_missing_with_stores_present && self.tried == self.ok
    }
}

fn remove_nss_stores() -> NssReport {
    disable_firefox_enterprise_roots();

    if !has_nss_certutil() {
        let profiles = firefox_profile_dirs();
        let chrome_present: bool;
        #[cfg(target_os = "linux")]
        {
            chrome_present = chrome_nssdb_path()
                .map(|p| p.join("cert9.db").exists() || p.join("cert8.db").exists())
                .unwrap_or(false);
        }
        #[cfg(not(target_os = "linux"))]
        {
            chrome_present = false;
        }
        let stores_present = !profiles.is_empty() || chrome_present;
        if stores_present {
            tracing::warn!(
                "NSS certutil not found; cannot automatically remove '{}' from Firefox/Chrome NSS stores.",
                CERT_NAME
            );
        }
        return NssReport {
            tried: 0,
            ok: 0,
            tool_missing_with_stores_present: stores_present,
        };
    }
    let mut report = NssReport::default();
    for p in firefox_profile_dirs() {
        report.tried += 1;
        let prefix = if p.join("cert9.db").exists() {
            "sql:"
        } else if p.join("cert8.db").exists() {
            ""
        } else {
            continue;
        };
        let dir_arg = format!("{}{}", prefix, p.display());
        if remove_nss_in_dir(&dir_arg) {
            report.ok += 1;
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(nssdb) = chrome_nssdb_path() {
            if nssdb.join("cert9.db").exists() || nssdb.join("cert8.db").exists() {
                report.tried += 1;
                let dir_arg = format!("sql:{}", nssdb.display());
                if remove_nss_in_dir(&dir_arg) {
                    report.ok += 1;
                }
            }
        }
    }
    if report.tried > 0 && report.ok != report.tried {
        tracing::warn!(
            "NSS cleanup partial: {}/{} stores updated. Close Firefox/Chrome and rerun --remove-cert if needed.",
            report.ok,
            report.tried
        );
    }
    report
}

fn disable_firefox_enterprise_roots() {
    for profile in firefox_profile_dirs() {
        let user_js = profile.join("user.js");
        let Ok(existing) = std::fs::read_to_string(&user_js) else {
            continue;
        };
        if let Some(new) = strip_enterprise_roots_block(&existing) {
            let _ = std::fs::write(&user_js, new);
            continue;
        }
        if has_bare_enterprise_roots(&existing) {
            tracing::info!(
                "Firefox profile {}: security.enterprise_roots.enabled is present without our marker; leaving it in place",
                profile.display()
            );
        }
    }
}

fn remove_nss_in_dir(dir_arg: &str) -> bool {
    match Command::new("certutil")
        .args(["-D", "-n", CERT_NAME, "-d", dir_arg])
        .output()
    {
        Ok(o) if o.status.success() => {
            tracing::debug!("NSS CA removed from {}", dir_arg);
            true
        }
        Ok(o) => {
            let msg = String::from_utf8_lossy(&o.stderr);
            if msg.to_ascii_lowercase().contains("could not find")
                || msg.to_ascii_lowercase().contains("not found")
            {
                true
            } else {
                tracing::debug!("NSS CA remove failed for {}: {}", dir_arg, msg.trim());
                false
            }
        }
        Err(e) => {
            tracing::debug!("NSS certutil remove failed for {}: {}", dir_arg, e);
            false
        }
    }
}

fn install_nss_in_profile(profile: &Path, cert_path: &str) -> bool {
    let prefix = if profile.join("cert9.db").exists() {
        "sql:"
    } else if profile.join("cert8.db").exists() {
        ""
    } else {
        return false;
    };
    let dir_arg = format!("{}{}", prefix, profile.display());
    install_nss_in_dir(&dir_arg, cert_path)
}

fn firefox_profile_dirs() -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;
    let mut roots: Vec<PathBuf> = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();
    match std::env::consts::OS {
        "macos" => {
            roots.push(PathBuf::from(format!(
                "{}/Library/Application Support/Firefox/Profiles",
                home
            )));
        }
        "linux" => {
            roots.push(PathBuf::from(format!("{}/.mozilla/firefox", home)));
            roots.push(PathBuf::from(format!(
                "{}/snap/firefox/common/.mozilla/firefox",
                home
            )));
        }
        "windows" => {
            if let Ok(appdata) = std::env::var("APPDATA") {
                roots.push(PathBuf::from(format!(
                    "{}\\Mozilla\\Firefox\\Profiles",
                    appdata
                )));
            }
        }
        _ => {}
    }

    let mut out: Vec<PathBuf> = Vec::new();
    for root in &roots {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        for ent in entries.flatten() {
            let p = ent.path();
            if !p.is_dir() {
                continue;
            }
            // A profile has cert9.db or cert8.db.
            if p.join("cert9.db").exists() || p.join("cert8.db").exists() {
                out.push(p);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openwrt_os_release_is_not_arch() {
        // Real OpenWRT 23.05 /etc/os-release. Contains OPENWRT_DEVICE_ARCH
        // which substring-matches "arch" — the old detector would mis-classify
        // this as Arch Linux.
        let content = r#"
NAME="OpenWrt"
VERSION="23.05.3"
ID="openwrt"
ID_LIKE="lede openwrt"
PRETTY_NAME="OpenWrt 23.05.3"
VERSION_ID="23.05.3"
HOME_URL="https://openwrt.org/"
BUG_URL="https://bugs.openwrt.org/"
SUPPORT_URL="https://forum.openwrt.org/"
BUILD_ID="r23809-234f1a2efa"
OPENWRT_BOARD="x86/64"
OPENWRT_ARCH="x86_64"
OPENWRT_TAINTS=""
OPENWRT_DEVICE_MANUFACTURER="OpenWrt"
OPENWRT_DEVICE_MANUFACTURER_URL="https://openwrt.org/"
OPENWRT_DEVICE_PRODUCT="Generic"
OPENWRT_DEVICE_REVISION="v0"
OPENWRT_RELEASE="OpenWrt 23.05.3 r23809-234f1a2efa"
"#;
        assert_eq!(classify_os_release(content), "openwrt");
    }

    #[test]
    fn debian_bullseye_classified_as_debian() {
        let content = r#"
PRETTY_NAME="Debian GNU/Linux 11 (bullseye)"
NAME="Debian GNU/Linux"
VERSION_ID="11"
VERSION="11 (bullseye)"
VERSION_CODENAME=bullseye
ID=debian
"#;
        assert_eq!(classify_os_release(content), "debian");
    }

    #[test]
    fn ubuntu_classified_as_debian_via_id_like() {
        let content = r#"
NAME="Ubuntu"
VERSION="22.04.3 LTS (Jammy Jellyfish)"
ID=ubuntu
ID_LIKE=debian
"#;
        assert_eq!(classify_os_release(content), "debian");
    }

    #[test]
    fn fedora_classified_as_rhel() {
        let content = "ID=fedora\nVERSION_ID=39\n";
        assert_eq!(classify_os_release(content), "rhel");
    }

    #[test]
    fn arch_classified_as_arch() {
        let content = "ID=arch\nID_LIKE=\n";
        assert_eq!(classify_os_release(content), "arch");
    }

    #[test]
    fn manjaro_classified_as_arch() {
        let content = "ID=manjaro\nID_LIKE=arch\n";
        assert_eq!(classify_os_release(content), "arch");
    }

    #[test]
    fn empty_os_release_is_unknown() {
        assert_eq!(classify_os_release(""), "unknown");
    }

    #[test]
    fn random_file_with_arch_substring_does_not_match() {
        // Make sure we don't regress to the old substring-match bug.
        let content = "SOMEFIELD=maybearchived\nFOO=bar\n";
        assert_eq!(classify_os_release(content), "unknown");
    }

    #[test]
    fn windows_thumbprint_match_ignores_spaces_and_case() {
        let certutil_output =
            "================ Certificate 0 ================\nCert Hash(sha1): AB CD EF 12\n";
        assert!(normalized_hex_contains(certutil_output, "abcdef12"));
    }

    #[test]
    fn enterprise_roots_block_added_to_empty_userjs() {
        let got = add_enterprise_roots_block("");
        let expected = format!("{}\n{}\n", FX_MARKER, FX_PREF);
        assert_eq!(got, EnterpriseRootsEdit::AddedBlock(expected));
    }

    #[test]
    fn enterprise_roots_block_respects_user_owned_pref() {
        let existing = "user_pref(\"security.enterprise_roots.enabled\", false);\n";
        assert_eq!(
            add_enterprise_roots_block(existing),
            EnterpriseRootsEdit::UserOwned
        );
    }

    #[test]
    fn strip_enterprise_roots_removes_only_our_block() {
        let before = format!(
            "user_pref(\"a\", 1);\n{}\n{}\nuser_pref(\"b\", 2);\n",
            FX_MARKER, FX_PREF
        );
        let after = strip_enterprise_roots_block(&before).expect("should strip");
        assert_eq!(after, "user_pref(\"a\", 1);\nuser_pref(\"b\", 2);\n");
    }

    #[test]
    fn strip_enterprise_roots_refuses_bare_pref() {
        assert!(strip_enterprise_roots_block(FX_PREF).is_none());
    }

    #[test]
    fn sudo_reconcile_requires_root_and_non_root_user() {
        assert_eq!(should_reconcile_for(0, Some("alice")), Some("alice"));
        assert_eq!(should_reconcile_for(1000, Some("alice")), None);
        assert_eq!(should_reconcile_for(0, Some("root")), None);
        assert_eq!(should_reconcile_for(0, None), None);
    }

    #[test]
    fn sudo_parse_passwd_home_extracts_home_field() {
        assert_eq!(
            sudo_parse_passwd_home("alice:x:1000:1000:Alice:/home/alice:/bin/bash\n"),
            Some("/home/alice".to_string())
        );
        assert_eq!(sudo_parse_passwd_home("broken:x:1000"), None);
    }
}
