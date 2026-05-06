use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use eframe::egui;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as AsyncMutex;
use tokio::task::JoinHandle;

use mhrv_jni::branding::{GITHUB_REPO_URL, PRODUCT_NAME};
use mhrv_jni::cert_installer::{install_ca, reconcile_sudo_environment, remove_ca};
use mhrv_jni::config::{
    AccountGroup, Config, DomainOverride, FrontingGroup, ScriptId, VercelConfig,
    CURRENT_CONFIG_VERSION,
};
use mhrv_jni::data_dir;
use mhrv_jni::domain_fronter::{DomainFronter, DEFAULT_GOOGLE_SNI_POOL};
use mhrv_jni::lan_utils::{detect_lan_ip, is_loopback_only, is_share_on_lan};
use mhrv_jni::mitm::{MitmCertManager, CA_CERT_FILE};
use mhrv_jni::proxy_server::ProxyServer;
use mhrv_jni::{doctor, scan_ips, scan_sni, test_cmd};

mod ui_doctor;
mod ui_format;
mod ui_fs;
mod ui_help;
mod ui_mode;
mod ui_monitor;
mod ui_setup;
mod ui_style;
mod ui_trust;
mod ui_xhttp;

use ui_doctor::{doctor_level_label, render_doctor_summary_card};
use ui_format::{fmt_bytes, fmt_duration};
use ui_fs::{downloads_dir, reveal_in_file_manager};
use ui_help::{backend_tool_entries, help_walkthrough, render_tool_help_row};
use ui_mode::{ghost_action, info_chip, mode_dashboard_panel, mode_summary, mode_summary_panel};
use ui_monitor::{
    degradation_changes, notable_failure_lines, quota_calls_per_hour, traffic_stat_rows,
};
use ui_setup::show_first_run_wizard;
use ui_style::{
    apply_ui_theme, form_row, help_callout, help_muted, help_subheading, primary_button, section,
    ACCENT, ACCENT_MINT, ACCENT_WARM, CARD_FILL, CARD_STROKE, CARD_STROKE_HI, ERR_RED,
    HEADER_SHADOW, OK_GREEN, SURFACE_SHADOW, TEXT_LABEL, TEXT_MAIN, TEXT_MUTED,
};
use ui_trust::trust_center_tab;
use ui_xhttp::{
    poll_xhttp_cloud_deploy, xhttp_vless_generator, XhttpDeployPipe, XhttpGeneratorForm,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const WIN_WIDTH: f32 = 900.0;
const WIN_HEIGHT: f32 = 968.0;
const LOG_MAX: usize = 200;

fn main() -> eframe::Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    reconcile_sudo_environment();
    mhrv_jni::rlimit::raise_nofile_limit_best_effort();

    let shared = Arc::new(Shared::default());
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<Cmd>();

    // Bridge tracing into the Recent log so proxy output appears in the panel.
    //
    // The env-filter respects RUST_LOG if set, otherwise defaults to info.
    // When the proxy starts and the config is saved, the in-process filter
    // follows the config log level.
    install_ui_tracing(shared.clone());

    let shared_bg = shared.clone();
    std::thread::Builder::new()
        .name("mhrv-bg".into())
        .spawn(move || background_thread(shared_bg, cmd_rx))
        .expect("failed to spawn background thread");

    let (form, load_err) = load_form();
    let last_saved_fingerprint = form_fingerprint(&form);
    let initial_toast = load_err.map(|e| (e, Instant::now()));

    // Default renderer is Glow (OpenGL). If the GPU stack cannot provide
    // OpenGL 2+, set `MHRV_RENDERER=wgpu` to use the wgpu backend
    // (DX12 on Windows, Vulkan on Linux, Metal on macOS):
    //
    //     MHRV_RENDERER=wgpu mhrv-f-ui
    //
    // The launcher scripts (run.bat / run.command / run.sh) honour
    // the same variable and forward it through.
    let use_wgpu = std::env::var("MHRV_RENDERER")
        .map(|v| v.eq_ignore_ascii_case("wgpu"))
        .unwrap_or(false);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WIN_WIDTH, WIN_HEIGHT])
            .with_min_inner_size([560.0, 520.0])
            .with_title(format!("{} v{}", PRODUCT_NAME, VERSION)),
        renderer: if use_wgpu {
            eframe::Renderer::Wgpu
        } else {
            eframe::Renderer::Glow
        },
        ..Default::default()
    };

    eframe::run_native(
        PRODUCT_NAME,
        options,
        Box::new(move |cc| {
            apply_ui_theme(&cc.egui_ctx);
            Ok(Box::new(App {
                shared,
                cmd_tx,
                form,
                active_tab: UiTab::Setup,
                last_poll: Instant::now(),
                toast: initial_toast,
                xhttp_deploy: XhttpDeployPipe::default(),
                last_saved_fingerprint,
            }))
        }),
    )
}

#[derive(Default)]
struct Shared {
    state: Mutex<UiState>,
}

#[derive(Default)]
struct UiState {
    running: bool,
    started_at: Option<Instant>,
    last_stats: Option<mhrv_jni::domain_fronter::StatsSnapshot>,
    last_per_site: Vec<(String, mhrv_jni::domain_fronter::HostStat)>,
    last_doctor_report: Option<doctor::DoctorReport>,
    last_doctor_at: Option<Instant>,
    log: VecDeque<String>,
    /// Result + timestamp for transient status banners (auto-hide after 10s).
    ca_trusted: Option<bool>,
    ca_trusted_at: Option<Instant>,
    last_test_ok: Option<bool>,
    last_test_msg: String,
    last_test_msg_at: Option<Instant>,
    /// Per-SNI probe results, populated by Cmd::TestSni / TestAllSni.
    sni_probe: HashMap<String, SniProbeState>,
    /// Most recent result of the Check-for-updates button.
    /// `None` = never checked this session. `Some(InFlight)` during the
    /// probe, then the resolved outcome.
    last_update_check: Option<UpdateProbeState>,
    last_update_check_at: Option<Instant>,
    /// Set while a download of a release asset is in flight. `None` when
    /// idle or after a completed download has been acknowledged.
    download_in_progress: bool,
    /// Prevent install/remove CA races from back-to-back button clicks.
    cert_op_in_progress: bool,
    /// One-line status of the most recent download (Ok(path) or Err(msg)).
    last_download: Option<Result<std::path::PathBuf, String>>,
    last_download_at: Option<Instant>,

    // Dashboard history: sampled from last_stats over time.
    degrade_history: VecDeque<(Instant, u8, String)>,
}

#[derive(Clone, Debug)]
enum UpdateProbeState {
    InFlight,
    Done(mhrv_jni::update_check::UpdateCheck),
}

#[derive(Clone, Debug)]
enum SniProbeState {
    InFlight,
    Ok(u32),
    Failed(String),
}

pub(crate) enum Cmd {
    Start(Config),
    Stop,
    Test(Config),
    Doctor(Config),
    DoctorFix(Config),
    InstallCa,
    RemoveCa,
    CheckCaTrusted,
    PollStats,
    /// Probe a single SNI against the given google_ip. Result is written
    /// into UiState::sni_probe keyed by the SNI string.
    TestSni {
        google_ip: String,
        sni: String,
    },
    /// Probe a batch of SNI names. Results appear in UiState::sni_probe one
    /// by one as each probe finishes.
    TestAllSni {
        google_ip: String,
        snis: Vec<String>,
    },
    /// Hit github.com + the Releases API and compare the running version
    /// to the latest tag. Result is written to UiState::last_update_check.
    /// `route` controls whether the request goes direct or is tunnelled
    /// through our local HTTP proxy (useful when the user's ISP IP has
    /// exhausted GitHub's unauthenticated rate limit).
    CheckUpdate {
        route: mhrv_jni::update_check::Route,
    },
    /// Download a release asset to ~/Downloads. Fires when the user clicks
    /// the "Download update" button after a successful CheckUpdate surfaces
    /// an UpdateAvailable with a matching platform asset.
    DownloadUpdate {
        route: mhrv_jni::update_check::Route,
        url: String,
        name: String,
    },
}

struct App {
    shared: Arc<Shared>,
    cmd_tx: Sender<Cmd>,
    form: FormState,
    active_tab: UiTab,
    last_poll: Instant,
    toast: Option<(String, Instant)>,
    xhttp_deploy: XhttpDeployPipe,
    last_saved_fingerprint: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiTab {
    Setup,
    Network,
    Advanced,
    Monitor,
    Trust,
    Help,
}

#[derive(Clone)]
pub(crate) struct FormState {
    /// `"apps_script"` (default), `"vercel_edge"`, `"direct"`, or `"full"`.
    /// Controls whether a relay is wired up. `direct` tolerates empty
    /// Apps Script config because it only uses the SNI-rewrite path.
    pub(crate) mode: String,
    pub(crate) google_ip: String,
    pub(crate) front_domain: String,
    pub(crate) listen_host: String,
    pub(crate) listen_port: String,
    pub(crate) socks5_port: String,
    log_level: String,
    verify_ssl: bool,
    enable_batching: bool,
    pub(crate) vercel_base_url: String,
    pub(crate) vercel_relay_path: String,
    pub(crate) vercel_auth_key: String,
    vercel_verify_tls: bool,
    pub(crate) vercel_max_body_mb: u32,
    vercel_enable_batching: bool,
    pub(crate) show_vercel_auth_key: bool,
    pub(crate) show_first_run_wizard: bool,
    pub(crate) wizard_step: usize,
    upstream_socks5: String,
    parallel_relay: u8,
    coalesce_step_ms: u16,
    coalesce_max_ms: u16,
    runtime_auto_tune: bool,
    runtime_profile: String,
    range_parallelism: u8,
    range_chunk_kb: u32,
    relay_request_timeout_secs: u64,
    request_timeout_secs: u64,
    auto_blacklist_strikes: u32,
    auto_blacklist_window_secs: u64,
    auto_blacklist_cooldown_secs: u64,
    /// SNI rotation pool entries. Each item has a sni name + a checkbox
    /// flag indicating whether it's in the active rotation.
    sni_pool: Vec<SniRow>,
    /// Text field buffer for the "+ add custom SNI" input at the bottom of
    /// the SNI editor window.
    sni_custom_input: String,
    /// Whether the floating SNI editor window is open.
    sni_editor_open: bool,
    /// Whether the Recent log panel is shown. User toggles with a checkbox.
    show_log: bool,
    fetch_ips_from_api: bool,
    max_ips_to_scan: usize,
    scan_batch_size: usize,
    google_ip_validation: bool,
    normalize_x_graphql: bool,
    youtube_via_relay: bool,
    passthrough_hosts: Vec<String>,
    block_quic: bool,
    tunnel_doh: bool,
    bypass_doh_hosts: Vec<String>,
    /// Config-only multi-edge SNI fronting groups. The desktop UI does
    /// not edit these yet, but it must round-trip them so Save does not
    /// erase a hand-edited Vercel/Fastly/Netlify catalog.
    fronting_groups: Vec<FrontingGroup>,
    domain_overrides: Vec<DomainOverride>,
    pub(crate) lan_token: Option<String>,
    pub(crate) lan_allowlist: Option<Vec<String>>,
    outage_reset_enabled: Option<bool>,
    outage_reset_failure_threshold: Option<u32>,
    outage_reset_window_ms: Option<u64>,
    outage_reset_cooldown_ms: Option<u64>,
    relay_rate_limit_qps: Option<f64>,
    relay_rate_limit_burst: Option<u32>,
    // Multi-account Apps Script groups (canonical; saved to `account_groups`).
    pub(crate) account_groups: Vec<AccountGroupForm>,
    // Profiles UI
    profile_name: String,
    profiles: Vec<String>,
    // In-app helper for external Xray/V2Ray XHTTP profiles.
    xhttp_generator: XhttpGeneratorForm,
}

#[derive(Clone)]
pub(crate) struct AccountGroupForm {
    pub(crate) label: String,
    pub(crate) enabled: bool,
    pub(crate) weight: u8,
    pub(crate) auth_key: String,
    pub(crate) script_ids: String, // one per line
    pub(crate) show_auth_key: bool,
}

#[derive(Clone, Debug)]
struct SniRow {
    name: String,
    enabled: bool,
}

pub(crate) fn load_form() -> (FormState, Option<String>) {
    // Try the user-data config first, then the cwd fallback. Report WHY load
    // fails so the user isn't silently shown a blank form (issue: user reports
    // 'settings saved to file but not loaded back'). Without this signal the
    // failure is invisible — `.ok()` swallows it and the form looks fresh.
    let path = data_dir::config_path();
    let cwd = PathBuf::from("config.json");

    let (existing, load_err): (Option<Config>, Option<String>) = if path.exists() {
        tracing::info!("config: attempting load from {}", path.display());
        match Config::load(&path) {
            Ok(c) => {
                tracing::info!("config: loaded OK from {}", path.display());
                (Some(c), None)
            }
            Err(e) => {
                let msg = format!("Config at {} failed to load: {}", path.display(), e);
                tracing::warn!("{}", msg);
                (None, Some(msg))
            }
        }
    } else if cwd.exists() {
        tracing::info!("config: attempting fallback load from {}", cwd.display());
        match Config::load(&cwd) {
            Ok(c) => (Some(c), None),
            Err(e) => {
                let msg = format!("Config at {} failed to load: {}", cwd.display(), e);
                tracing::warn!("{}", msg);
                (None, Some(msg))
            }
        }
    } else {
        tracing::info!(
            "config: no config found at {} — starting with defaults",
            path.display()
        );
        (None, None)
    };
    let form = if let Some(c) = existing {
        let sni_pool = sni_pool_for_form(c.sni_hosts.as_deref(), &c.front_domain);
        let account_groups: Vec<AccountGroupForm> = c
            .account_groups
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|g| AccountGroupForm {
                label: g.label.unwrap_or_default(),
                enabled: g.enabled,
                weight: g.weight,
                auth_key: g.auth_key,
                script_ids: g.script_ids.into_vec().join("\n"),
                show_auth_key: false,
            })
            .collect();
        let mode = if c.mode == "google_only" {
            "direct".to_string()
        } else {
            c.mode.clone()
        };
        FormState {
            mode,
            google_ip: c.google_ip,
            front_domain: c.front_domain,
            listen_host: c.listen_host,
            listen_port: c.listen_port.to_string(),
            socks5_port: c.socks5_port.map(|p| p.to_string()).unwrap_or_default(),
            log_level: c.log_level,
            verify_ssl: c.verify_ssl,
            enable_batching: c.enable_batching,
            vercel_base_url: c.vercel.base_url,
            vercel_relay_path: c.vercel.relay_path,
            vercel_auth_key: c.vercel.auth_key,
            vercel_verify_tls: c.vercel.verify_tls,
            vercel_max_body_mb: c.vercel.max_body_bytes.max(1024).div_ceil(1024 * 1024) as u32,
            vercel_enable_batching: c.vercel.enable_batching,
            show_vercel_auth_key: false,
            show_first_run_wizard: false,
            wizard_step: 0,
            upstream_socks5: c.upstream_socks5.unwrap_or_default(),
            parallel_relay: c.parallel_relay,
            coalesce_step_ms: c.coalesce_step_ms,
            coalesce_max_ms: c.coalesce_max_ms,
            runtime_auto_tune: c.runtime_auto_tune,
            runtime_profile: c
                .runtime_profile
                .clone()
                .unwrap_or_else(|| "balanced".into()),
            range_parallelism: c.range_parallelism.unwrap_or(12),
            range_chunk_kb: (c.range_chunk_bytes.unwrap_or(256 * 1024) / 1024) as u32,
            relay_request_timeout_secs: c.relay_request_timeout_secs.unwrap_or(25),
            request_timeout_secs: c.request_timeout_secs.unwrap_or(30),
            auto_blacklist_strikes: c.auto_blacklist_strikes.unwrap_or(3),
            auto_blacklist_window_secs: c.auto_blacklist_window_secs.unwrap_or(30),
            auto_blacklist_cooldown_secs: c.auto_blacklist_cooldown_secs.unwrap_or(120),
            sni_pool,
            sni_custom_input: String::new(),
            sni_editor_open: false,
            show_log: true,
            fetch_ips_from_api: c.fetch_ips_from_api,
            max_ips_to_scan: c.max_ips_to_scan,
            google_ip_validation: c.google_ip_validation,
            scan_batch_size: c.scan_batch_size,
            normalize_x_graphql: c.normalize_x_graphql,
            youtube_via_relay: c.youtube_via_relay,
            passthrough_hosts: c.passthrough_hosts.clone(),
            block_quic: c.block_quic,
            tunnel_doh: c.tunnel_doh,
            bypass_doh_hosts: c.bypass_doh_hosts.clone(),
            fronting_groups: c.fronting_groups.clone(),
            domain_overrides: c.domain_overrides.clone(),
            lan_token: c.lan_token.clone(),
            lan_allowlist: c.lan_allowlist.clone(),
            outage_reset_enabled: c.outage_reset_enabled,
            outage_reset_failure_threshold: c.outage_reset_failure_threshold,
            outage_reset_window_ms: c.outage_reset_window_ms,
            outage_reset_cooldown_ms: c.outage_reset_cooldown_ms,
            relay_rate_limit_qps: c.relay_rate_limit_qps,
            relay_rate_limit_burst: c.relay_rate_limit_burst,
            account_groups,
            profile_name: String::new(),
            profiles: mhrv_jni::profiles::list_profiles().unwrap_or_default(),
            xhttp_generator: XhttpGeneratorForm::default(),
        }
    } else {
        FormState {
            mode: "apps_script".into(),
            google_ip: "216.239.38.120".into(),
            front_domain: "www.google.com".into(),
            listen_host: "127.0.0.1".into(),
            listen_port: "8085".into(),
            socks5_port: "8086".into(),
            log_level: "info".into(),
            verify_ssl: true,
            enable_batching: false,
            vercel_base_url: String::new(),
            vercel_relay_path: "/api/api".into(),
            vercel_auth_key: String::new(),
            vercel_verify_tls: true,
            vercel_max_body_mb: 4,
            vercel_enable_batching: false,
            show_vercel_auth_key: false,
            show_first_run_wizard: true,
            wizard_step: 0,
            upstream_socks5: String::new(),
            parallel_relay: 0,
            coalesce_step_ms: 0,
            coalesce_max_ms: 0,
            runtime_auto_tune: false,
            runtime_profile: "balanced".into(),
            range_parallelism: 12,
            range_chunk_kb: 256,
            relay_request_timeout_secs: 25,
            request_timeout_secs: 30,
            auto_blacklist_strikes: 3,
            auto_blacklist_window_secs: 30,
            auto_blacklist_cooldown_secs: 120,
            sni_pool: sni_pool_for_form(None, "www.google.com"),
            sni_custom_input: String::new(),
            sni_editor_open: false,
            show_log: true,
            fetch_ips_from_api: false,
            max_ips_to_scan: 100,
            google_ip_validation: true,
            scan_batch_size: 500,
            normalize_x_graphql: false,
            youtube_via_relay: false,
            passthrough_hosts: Vec::new(),
            block_quic: false,
            tunnel_doh: true,
            bypass_doh_hosts: Vec::new(),
            fronting_groups: Vec::new(),
            domain_overrides: Vec::new(),
            lan_token: None,
            lan_allowlist: None,
            outage_reset_enabled: Some(true),
            outage_reset_failure_threshold: Some(3),
            outage_reset_window_ms: Some(5000),
            outage_reset_cooldown_ms: Some(15000),
            relay_rate_limit_qps: None,
            relay_rate_limit_burst: None,
            account_groups: Vec::new(),
            profile_name: String::new(),
            profiles: mhrv_jni::profiles::list_profiles().unwrap_or_default(),
            xhttp_generator: XhttpGeneratorForm::default(),
        }
    };
    (form, load_err)
}

/// Build the initial `sni_pool` list shown in the editor.
///
/// If the user has explicit `sni_hosts` configured, we show exactly those
/// rows (all enabled). Otherwise we show the default Google pool plus any
/// missing entries, all enabled, with the user's `front_domain` first.
fn sni_pool_for_form(user: Option<&[String]>, front_domain: &str) -> Vec<SniRow> {
    let user_clean: Vec<String> = user
        .unwrap_or(&[])
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if !user_clean.is_empty() {
        return user_clean
            .into_iter()
            .map(|name| SniRow {
                name,
                enabled: true,
            })
            .collect();
    }
    // Default: primary + the other Google-edge subdomains, primary first,
    // all enabled.
    let primary = front_domain.trim().to_string();
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    if !primary.is_empty() {
        seen.insert(primary.clone());
        out.push(SniRow {
            name: primary,
            enabled: true,
        });
    }
    for s in DEFAULT_GOOGLE_SNI_POOL {
        if seen.insert(s.to_string()) {
            out.push(SniRow {
                name: (*s).to_string(),
                enabled: true,
            });
        }
    }
    out
}

impl FormState {
    pub(crate) fn to_config(&self) -> Result<Config, String> {
        let is_direct = self.mode == "direct" || self.mode == "google_only";
        let is_vercel_edge = self.mode == "vercel_edge";
        if !is_direct && !is_vercel_edge {
            let any_enabled = self.account_groups.iter().any(|g| g.enabled);
            if !any_enabled {
                return Err("At least one account group must be enabled".into());
            }
            for (i, g) in self.account_groups.iter().enumerate() {
                if !g.enabled {
                    continue;
                }
                if g.script_ids.trim().is_empty() {
                    return Err(format!(
                        "Account group {}: deployment IDs are required",
                        i + 1
                    ));
                }
                if g.auth_key.trim().is_empty() {
                    return Err(format!("Account group {}: auth key is required", i + 1));
                }
            }
        }
        if is_vercel_edge {
            if self.vercel_base_url.trim().is_empty() {
                return Err("Serverless JSON base URL is required in vercel_edge mode".into());
            }
            if self.vercel_auth_key.trim().is_empty() {
                return Err("Serverless JSON auth key is required in vercel_edge mode".into());
            }
            if !self.vercel_relay_path.trim().starts_with('/') {
                return Err("Serverless JSON relay path must start with /".into());
            }
        }
        let listen_port: u16 = self
            .listen_port
            .parse()
            .map_err(|_| "HTTP port must be a number".to_string())?;
        let socks5_port: Option<u16> = if self.socks5_port.trim().is_empty() {
            None
        } else {
            Some(
                self.socks5_port
                    .parse()
                    .map_err(|_| "SOCKS5 port must be a number".to_string())?,
            )
        };
        if socks5_port == Some(listen_port) {
            return Err("HTTP and SOCKS5 ports must be different".into());
        }
        let account_groups: Option<Vec<AccountGroup>> = if is_direct || is_vercel_edge {
            None
        } else {
            let mut out = Vec::new();
            for g in &self.account_groups {
                let ids: Vec<String> = g
                    .script_ids
                    .split(['\n', ','])
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if ids.is_empty() {
                    continue;
                }
                out.push(AccountGroup {
                    label: if g.label.trim().is_empty() {
                        None
                    } else {
                        Some(g.label.trim().to_string())
                    },
                    auth_key: g.auth_key.clone(),
                    script_ids: if ids.len() == 1 {
                        ScriptId::One(ids[0].clone())
                    } else {
                        ScriptId::Many(ids)
                    },
                    weight: g.weight,
                    enabled: g.enabled,
                });
            }
            Some(out)
        };
        Ok(Config {
            config_version: CURRENT_CONFIG_VERSION,
            mode: self.mode.clone(),
            google_ip: self.google_ip.trim().to_string(),
            front_domain: self.front_domain.trim().to_string(),
            listen_host: self.listen_host.trim().to_string(),
            listen_port,
            socks5_port,
            log_level: self.log_level.trim().to_string(),
            verify_ssl: self.verify_ssl,
            hosts: std::collections::HashMap::new(),
            enable_batching: self.enable_batching,
            upstream_socks5: {
                let v = self.upstream_socks5.trim();
                if v.is_empty() {
                    None
                } else {
                    Some(v.to_string())
                }
            },
            parallel_relay: self.parallel_relay,
            coalesce_step_ms: self.coalesce_step_ms,
            coalesce_max_ms: self.coalesce_max_ms,
            relay_rate_limit_qps: self.relay_rate_limit_qps,
            relay_rate_limit_burst: self.relay_rate_limit_burst,
            runtime_profile: Some(self.runtime_profile.trim().to_string()),
            runtime_auto_tune: self.runtime_auto_tune,
            range_parallelism: Some(self.range_parallelism),
            range_chunk_bytes: Some((self.range_chunk_kb.max(16) as u64) * 1024),
            relay_request_timeout_secs: Some(self.relay_request_timeout_secs.max(5)),
            request_timeout_secs: Some(self.request_timeout_secs.clamp(5, 300)),
            auto_blacklist_strikes: Some(self.auto_blacklist_strikes.clamp(1, 100)),
            auto_blacklist_window_secs: Some(self.auto_blacklist_window_secs.clamp(1, 86_400)),
            auto_blacklist_cooldown_secs: Some(self.auto_blacklist_cooldown_secs.clamp(1, 86_400)),
            sni_hosts: {
                let active: Vec<String> = self
                    .sni_pool
                    .iter()
                    .filter(|r| r.enabled)
                    .map(|r| r.name.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                // None = "use auto-expansion default", Some(list) = explicit.
                // If the user's pool is empty/all-off we still save as None so
                // the backend uses its built-in SNI pool instead of dying on an
                // empty pool.
                if active.is_empty() {
                    None
                } else {
                    Some(active)
                }
            },
            fetch_ips_from_api: self.fetch_ips_from_api,
            max_ips_to_scan: self.max_ips_to_scan,
            google_ip_validation: self.google_ip_validation,
            scan_batch_size: self.scan_batch_size,
            normalize_x_graphql: self.normalize_x_graphql,
            youtube_via_relay: self.youtube_via_relay,
            // Similarly config-only for now; round-trips through the
            // file so the UI doesn't drop the user's entries on save.
            passthrough_hosts: self.passthrough_hosts.clone(),
            block_quic: self.block_quic,
            tunnel_doh: self.tunnel_doh,
            bypass_doh_hosts: self.bypass_doh_hosts.clone(),
            fronting_groups: self.fronting_groups.clone(),
            domain_overrides: self.domain_overrides.clone(),
            account_groups,
            vercel: VercelConfig {
                base_url: self
                    .vercel_base_url
                    .trim()
                    .trim_end_matches('/')
                    .to_string(),
                relay_path: self.vercel_relay_path.trim().to_string(),
                auth_key: self.vercel_auth_key.trim().to_string(),
                verify_tls: self.vercel_verify_tls,
                max_body_bytes: (self.vercel_max_body_mb.max(1) as usize) * 1024 * 1024,
                enable_batching: self.vercel_enable_batching,
            },
            lan_token: self.lan_token.as_deref().and_then(clean_optional_text),
            lan_allowlist: self
                .lan_allowlist
                .as_ref()
                .and_then(|items| clean_optional_list(&items.join("\n"))),
            outage_reset_enabled: self.outage_reset_enabled,
            outage_reset_failure_threshold: self.outage_reset_failure_threshold,
            outage_reset_window_ms: self.outage_reset_window_ms,
            outage_reset_cooldown_ms: self.outage_reset_cooldown_ms,
        })
    }
}

fn save_config(cfg: &Config) -> Result<PathBuf, String> {
    let path = data_dir::config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    // Last-known-good snapshot: keep previous config before overwriting.
    // If snapshot fails, we still proceed with save (best-effort).
    if path.exists() {
        if let Ok(prev) = Config::load(&path) {
            let _ = mhrv_jni::profiles::save_snapshot("last_known_good", &prev);
        }
    }
    let json = serde_json::to_string_pretty(&ConfigWire::from(cfg)).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path)
}

fn config_fingerprint(cfg: &Config) -> Option<String> {
    serde_json::to_string(&ConfigWire::from(cfg)).ok()
}

fn form_fingerprint(form: &FormState) -> Option<String> {
    form.to_config()
        .ok()
        .and_then(|cfg| config_fingerprint(&cfg))
}

#[derive(serde::Serialize)]
struct ConfigWire<'a> {
    config_version: u32,
    mode: &'a str,
    google_ip: &'a str,
    front_domain: &'a str,
    listen_host: &'a str,
    listen_port: u16,
    socks5_port: Option<u16>,
    log_level: &'a str,
    verify_ssl: bool,
    #[serde(skip_serializing_if = "is_false")]
    enable_batching: bool,
    vercel: &'a VercelConfig,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    hosts: &'a std::collections::HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    upstream_socks5: Option<&'a str>,
    #[serde(skip_serializing_if = "is_zero_u8")]
    parallel_relay: u8,
    #[serde(skip_serializing_if = "is_zero_u16")]
    coalesce_step_ms: u16,
    #[serde(skip_serializing_if = "is_zero_u16")]
    coalesce_max_ms: u16,
    #[serde(skip_serializing_if = "is_false")]
    runtime_auto_tune: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_profile: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    range_parallelism: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    range_chunk_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relay_request_timeout_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_timeout_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_blacklist_strikes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_blacklist_window_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_blacklist_cooldown_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sni_hosts: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "is_false")]
    normalize_x_graphql: bool,
    #[serde(skip_serializing_if = "is_false")]
    youtube_via_relay: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    passthrough_hosts: &'a Vec<String>,
    #[serde(skip_serializing_if = "is_false")]
    block_quic: bool,
    #[serde(skip_serializing_if = "is_false")]
    tunnel_doh: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bypass_doh_hosts: &'a Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fronting_groups: &'a Vec<FrontingGroup>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    domain_overrides: &'a Vec<DomainOverride>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lan_token: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lan_allowlist: Option<&'a Vec<String>>,
    // IP-scan knobs. These used to be missing from the wire struct, so
    // every Save-config silently dropped them — the user would toggle
    // "fetch from API" on, save, reopen, and find it off again. Add
    // them here and keep them in sync if Config ever grows more.
    #[serde(skip_serializing_if = "is_false")]
    fetch_ips_from_api: bool,
    max_ips_to_scan: usize,
    scan_batch_size: usize,
    google_ip_validation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    outage_reset_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outage_reset_failure_threshold: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outage_reset_window_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outage_reset_cooldown_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relay_rate_limit_qps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relay_rate_limit_burst: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    account_groups: Option<&'a Vec<AccountGroup>>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn is_zero_u8(v: &u8) -> bool {
    *v == 0
}

fn is_zero_u16(v: &u16) -> bool {
    *v == 0
}

impl<'a> From<&'a Config> for ConfigWire<'a> {
    fn from(c: &'a Config) -> Self {
        ConfigWire {
            config_version: c.config_version,
            mode: c.mode.as_str(),
            google_ip: c.google_ip.as_str(),
            front_domain: c.front_domain.as_str(),
            listen_host: c.listen_host.as_str(),
            listen_port: c.listen_port,
            socks5_port: c.socks5_port,
            log_level: c.log_level.as_str(),
            verify_ssl: c.verify_ssl,
            enable_batching: c.enable_batching,
            vercel: &c.vercel,
            hosts: &c.hosts,
            upstream_socks5: c.upstream_socks5.as_deref(),
            parallel_relay: c.parallel_relay,
            coalesce_step_ms: c.coalesce_step_ms,
            coalesce_max_ms: c.coalesce_max_ms,
            runtime_auto_tune: c.runtime_auto_tune,
            runtime_profile: c.runtime_profile.as_deref(),
            range_parallelism: c.range_parallelism,
            range_chunk_bytes: c.range_chunk_bytes,
            relay_request_timeout_secs: c.relay_request_timeout_secs,
            request_timeout_secs: c.request_timeout_secs,
            auto_blacklist_strikes: c.auto_blacklist_strikes,
            auto_blacklist_window_secs: c.auto_blacklist_window_secs,
            auto_blacklist_cooldown_secs: c.auto_blacklist_cooldown_secs,
            sni_hosts: c
                .sni_hosts
                .as_ref()
                .map(|v| v.iter().map(String::as_str).collect()),
            normalize_x_graphql: c.normalize_x_graphql,
            youtube_via_relay: c.youtube_via_relay,
            passthrough_hosts: &c.passthrough_hosts,
            block_quic: c.block_quic,
            tunnel_doh: c.tunnel_doh,
            bypass_doh_hosts: &c.bypass_doh_hosts,
            fronting_groups: &c.fronting_groups,
            domain_overrides: &c.domain_overrides,
            lan_token: c.lan_token.as_deref(),
            lan_allowlist: c.lan_allowlist.as_ref(),
            fetch_ips_from_api: c.fetch_ips_from_api,
            max_ips_to_scan: c.max_ips_to_scan,
            scan_batch_size: c.scan_batch_size,
            google_ip_validation: c.google_ip_validation,
            outage_reset_enabled: c.outage_reset_enabled,
            outage_reset_failure_threshold: c.outage_reset_failure_threshold,
            outage_reset_window_ms: c.outage_reset_window_ms,
            outage_reset_cooldown_ms: c.outage_reset_cooldown_ms,
            relay_rate_limit_qps: c.relay_rate_limit_qps,
            relay_rate_limit_burst: c.relay_rate_limit_burst,
            account_groups: c.account_groups.as_ref(),
        }
    }
}

#[cfg(test)]
mod config_wire_tests {
    use super::*;

    #[test]
    fn config_wire_preserves_domain_overrides_and_batching_flags() {
        let cfg = Config::from_json_str(
            r#"{
                "mode": "vercel_edge",
                "enable_batching": true,
                "vercel": {
                    "base_url": "https://example.vercel.app",
                    "relay_path": "/api/api",
                    "auth_key": "test-auth-key-please-change-32chars",
                    "enable_batching": true
                },
                "domain_overrides": [{
                    "host": ".example.com",
                    "force_route": "relay",
                    "never_chunk": true
                }]
            }"#,
        )
        .expect("config should load");

        let value = serde_json::to_value(ConfigWire::from(&cfg)).expect("wire should serialize");

        assert_eq!(value["enable_batching"], true);
        assert_eq!(value["vercel"]["enable_batching"], true);
        assert_eq!(value["domain_overrides"][0]["host"], ".example.com");
        assert_eq!(value["domain_overrides"][0]["force_route"], "relay");
        assert_eq!(value["domain_overrides"][0]["never_chunk"], true);
    }

    #[test]
    fn config_wire_serializes_all_current_config_fields_when_present() {
        let cfg = Config::from_json_str(
            r#"{
                "config_version": 1,
                "mode": "apps_script",
                "google_ip": "216.239.38.120",
                "front_domain": "www.google.com",
                "listen_host": "127.0.0.1",
                "listen_port": 8085,
                "socks5_port": 8086,
                "log_level": "debug",
                "verify_ssl": true,
                "hosts": {"example.local": "127.0.0.1"},
                "enable_batching": true,
                "upstream_socks5": "127.0.0.1:50529",
                "parallel_relay": 3,
                "coalesce_step_ms": 50,
                "coalesce_max_ms": 900,
                "relay_rate_limit_qps": 2.5,
                "relay_rate_limit_burst": 5,
                "runtime_profile": "max_speed",
                "runtime_auto_tune": true,
                "range_parallelism": 8,
                "range_chunk_bytes": 262144,
                "relay_request_timeout_secs": 25,
                "request_timeout_secs": 45,
                "auto_blacklist_strikes": 4,
                "auto_blacklist_window_secs": 60,
                "auto_blacklist_cooldown_secs": 180,
                "sni_hosts": ["www.google.com", "drive.google.com"],
                "fetch_ips_from_api": true,
                "max_ips_to_scan": 42,
                "scan_batch_size": 64,
                "google_ip_validation": true,
                "normalize_x_graphql": true,
                "youtube_via_relay": true,
                "passthrough_hosts": ["localhost", ".internal.example"],
                "block_quic": true,
                "tunnel_doh": true,
                "bypass_doh_hosts": ["dns.google"],
                "fronting_groups": [{
                    "name": "vercel",
                    "ip": "76.76.21.21",
                    "sni": "react.dev",
                    "domains": ["vercel.com"]
                }],
                "domain_overrides": [{
                    "host": ".example.com",
                    "force_route": "relay",
                    "never_chunk": true
                }],
                "account_groups": [{
                    "label": "primary",
                    "auth_key": "test-auth-key-please-change-32chars",
                    "script_ids": ["AKfycb_primary", "AKfycb_backup"],
                    "weight": 2,
                    "enabled": true
                }],
                "vercel": {
                    "base_url": "https://example.vercel.app",
                    "relay_path": "/api/api",
                    "auth_key": "test-auth-key-please-change-32chars",
                    "verify_tls": true,
                    "max_body_bytes": 2097152,
                    "enable_batching": true
                },
                "lan_token": "local-test-token",
                "lan_allowlist": ["127.0.0.1/32"],
                "outage_reset_enabled": true,
                "outage_reset_failure_threshold": 5,
                "outage_reset_window_ms": 6000,
                "outage_reset_cooldown_ms": 30000
            }"#,
        )
        .expect("complete config should load");

        let value = serde_json::to_value(ConfigWire::from(&cfg)).expect("wire should serialize");
        let obj = value.as_object().expect("wire output should be an object");
        let missing: Vec<&str> = [
            "config_version",
            "mode",
            "google_ip",
            "front_domain",
            "listen_host",
            "listen_port",
            "socks5_port",
            "log_level",
            "verify_ssl",
            "hosts",
            "enable_batching",
            "upstream_socks5",
            "parallel_relay",
            "coalesce_step_ms",
            "coalesce_max_ms",
            "relay_rate_limit_qps",
            "relay_rate_limit_burst",
            "runtime_profile",
            "runtime_auto_tune",
            "range_parallelism",
            "range_chunk_bytes",
            "relay_request_timeout_secs",
            "request_timeout_secs",
            "auto_blacklist_strikes",
            "auto_blacklist_window_secs",
            "auto_blacklist_cooldown_secs",
            "sni_hosts",
            "fetch_ips_from_api",
            "max_ips_to_scan",
            "scan_batch_size",
            "google_ip_validation",
            "normalize_x_graphql",
            "youtube_via_relay",
            "passthrough_hosts",
            "block_quic",
            "tunnel_doh",
            "bypass_doh_hosts",
            "fronting_groups",
            "domain_overrides",
            "account_groups",
            "vercel",
            "lan_token",
            "lan_allowlist",
            "outage_reset_enabled",
            "outage_reset_failure_threshold",
            "outage_reset_window_ms",
            "outage_reset_cooldown_ms",
        ]
        .into_iter()
        .filter(|key| !obj.contains_key(*key))
        .collect();

        assert!(missing.is_empty(), "ConfigWire dropped fields: {missing:?}");
        assert_eq!(
            value["account_groups"][0]["script_ids"][0],
            "AKfycb_primary"
        );
        assert_eq!(value["vercel"]["max_body_bytes"], 2_097_152);
        assert_eq!(value["fronting_groups"][0]["name"], "vercel");
        assert_eq!(value["domain_overrides"][0]["never_chunk"], true);
    }
}

fn clean_optional_text(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn clean_optional_list(value: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for part in value.lines().flat_map(|line| line.split(',')) {
        let item = part.trim();
        if !item.is_empty() && !out.iter().any(|existing| existing == item) {
            out.push(item.to_string());
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn tab_button(ui: &mut egui::Ui, active: bool, text: &str) -> egui::Response {
    let fill = if active {
        ACCENT.linear_multiply(0.72)
    } else {
        egui::Color32::from_rgb(42, 39, 36)
    };
    let stroke = if active {
        egui::Stroke::new(1.0, ACCENT.linear_multiply(1.05))
    } else {
        egui::Stroke::new(1.0, CARD_STROKE.linear_multiply(0.92))
    };
    ui.add(
        egui::Button::new(egui::RichText::new(text).strong().color(if active {
            egui::Color32::WHITE
        } else {
            TEXT_LABEL
        }))
        .fill(fill)
        .stroke(stroke)
        .min_size(egui::vec2(118.0, 36.0))
        .rounding(9.0),
    )
}

fn tab_bar(ui: &mut egui::Ui, active_tab: &mut UiTab) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(29, 27, 25))
        .stroke(egui::Stroke::new(1.0, CARD_STROKE_HI.linear_multiply(0.42)))
        .rounding(11.0)
        .shadow(egui::Shadow {
            offset: egui::vec2(0.0, 3.0),
            blur: 14.0,
            spread: 0.0,
            color: egui::Color32::from_black_alpha(52),
        })
        .inner_margin(egui::Margin::symmetric(11.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (tab, label) in [
                    (UiTab::Setup, "Setup"),
                    (UiTab::Network, "Network"),
                    (UiTab::Advanced, "Advanced"),
                    (UiTab::Monitor, "Monitor"),
                    (UiTab::Trust, "Trust"),
                    (UiTab::Help, "Help & docs"),
                ] {
                    if tab_button(ui, *active_tab == tab, label).clicked() {
                        *active_tab = tab;
                    }
                }
            });
        });
}

impl App {
    fn save_current_config(&mut self) -> Result<PathBuf, String> {
        let cfg = self.form.to_config()?;
        let fingerprint = config_fingerprint(&cfg);
        let path = save_config(&cfg)?;
        self.last_saved_fingerprint = fingerprint;
        Ok(path)
    }

    fn start_current_config(&mut self, save_first: bool) {
        match self.form.to_config() {
            Ok(cfg) => {
                if save_first {
                    let fingerprint = config_fingerprint(&cfg);
                    match save_config(&cfg) {
                        Ok(path) => {
                            self.last_saved_fingerprint = fingerprint;
                            self.toast = Some((
                                format!("Saved to {}; starting proxy.", path.display()),
                                Instant::now(),
                            ));
                            let _ = self.cmd_tx.send(Cmd::Start(cfg));
                        }
                        Err(e) => {
                            self.toast = Some((format!("Save failed: {}", e), Instant::now()));
                        }
                    }
                } else {
                    let _ = self.cmd_tx.send(Cmd::Start(cfg));
                }
            }
            Err(e) => {
                self.toast = Some((format!("Cannot start: {}", e), Instant::now()));
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Some(msg) =
            poll_xhttp_cloud_deploy(&mut self.form.xhttp_generator, &mut self.xhttp_deploy, ctx)
        {
            self.toast = Some((msg, Instant::now()));
            ctx.request_repaint();
        }
        if self.last_poll.elapsed() > Duration::from_millis(700) {
            let _ = self.cmd_tx.send(Cmd::PollStats);
            self.last_poll = Instant::now();
        }
        ctx.request_repaint_after(Duration::from_millis(500));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(10.0, 9.0);

            // Wrap the whole central panel in a vertical scroll area so the
            // form + stats + log panel stay accessible on short screens
            // (~13" laptops at default scaling). Nested scroll areas still
            // work fine within this outer scroller.
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {

            // ── Header: full product name, repo link, version tag, status pill ─
            let running = self.shared.state.lock().unwrap().running;
            let start_error = self.form.to_config().err();
            let can_start = start_error.is_none();
            let current_fingerprint = form_fingerprint(&self.form);
            let has_unsaved_changes = current_fingerprint
                .as_ref()
                .is_some_and(|fp| self.last_saved_fingerprint.as_ref() != Some(fp));
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(26, 25, 24))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(
                        (CARD_STROKE.r() + CARD_STROKE_HI.r()) / 2,
                        (CARD_STROKE.g() + CARD_STROKE_HI.g()) / 2,
                        (CARD_STROKE.b() + CARD_STROKE_HI.b()) / 2,
                    ),
                ))
                .rounding(12.0)
                .shadow(HEADER_SHADOW)
                .inner_margin(egui::Margin::symmetric(23.0, 19.0))
                .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(PRODUCT_NAME)
                            .size(23.0)
                            .strong()
                            .color(TEXT_MAIN),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(
                            "Desktop relay control, diagnostics, profiles, and sharing",
                        )
                        .size(13.0)
                        .color(TEXT_MUTED),
                    );
                });
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.hyperlink_to(
                    egui::RichText::new("Source & releases")
                        .size(13.0)
                        .color(ACCENT)
                        .strong(),
                    GITHUB_REPO_URL,
                );
                ui.label(egui::RichText::new("·").color(egui::Color32::from_gray(90)));
                ui.hyperlink_to(
                    egui::RichText::new(format!("v{}", VERSION))
                        .color(egui::Color32::from_gray(150))
                        .monospace(),
                    format!("{}/releases/tag/v{}", GITHUB_REPO_URL, VERSION),
                );
                ui.label(egui::RichText::new("·").color(egui::Color32::from_gray(90)));
                ui.label(
                    egui::RichText::new("Short name: mhrv-f (CLI)")
                        .size(12.0)
                        .color(egui::Color32::from_gray(140)),
                );
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                let (mode_title, _, _, _) = mode_summary(&self.form.mode);
                info_chip(ui, format!("mode: {}", mode_title), ACCENT);
                let socks_label = if self.form.socks5_port.trim().is_empty() {
                    "SOCKS5 off".to_string()
                } else {
                    format!("SOCKS5 {}", self.form.socks5_port.trim())
                };
                info_chip(
                    ui,
                    format!("HTTP {} / {}", self.form.listen_port.trim(), socks_label),
                    egui::Color32::from_rgb(120, 185, 150),
                );
                let lan_bound = matches!(self.form.listen_host.trim(), "0.0.0.0" | "::");
                info_chip(
                    ui,
                    if lan_bound { "LAN sharing on" } else { "local-only" },
                    if lan_bound {
                        egui::Color32::from_rgb(235, 155, 95)
                    } else {
                        egui::Color32::from_rgb(120, 185, 150)
                    },
                );
                info_chip(
                    ui,
                    format!("profile: {}", self.form.runtime_profile.trim()),
                    egui::Color32::from_rgb(170, 145, 225),
                );
                info_chip(
                    ui,
                    if running { "running" } else { "stopped" },
                    if running { OK_GREEN } else { ERR_RED },
                );
                if has_unsaved_changes {
                    info_chip(ui, "unsaved edits", ACCENT_WARM);
                }
            });
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                if running {
                    let btn = egui::Button::new(
                        egui::RichText::new("Stop")
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(ERR_RED)
                    .min_size(egui::vec2(104.0, 30.0))
                    .rounding(6.0);
                    if ui
                        .add(btn)
                        .on_hover_text("Stop the local HTTP/SOCKS proxy listeners.")
                        .clicked()
                    {
                        let _ = self.cmd_tx.send(Cmd::Stop);
                    }
                } else if ui
                    .add_enabled(
                        can_start,
                        primary_button(if has_unsaved_changes {
                            "Save and start"
                        } else {
                            "Start"
                        }),
                    )
                    .on_hover_text(if has_unsaved_changes {
                        "Save the current form first, then start the local HTTP/SOCKS proxy listeners."
                    } else {
                        "Start the local HTTP/SOCKS proxy listeners."
                    })
                    .clicked()
                {
                    self.start_current_config(has_unsaved_changes);
                }

                if ui
                    .add(ghost_action("Test relay"))
                    .on_hover_text("Run one end-to-end relay request and show the result below.")
                    .clicked()
                {
                    match self.form.to_config() {
                        Ok(cfg) => {
                            let _ = self.cmd_tx.send(Cmd::Test(cfg));
                        }
                        Err(e) => {
                            self.toast = Some((format!("Cannot test: {}", e), Instant::now()));
                        }
                    }
                }
                if ui
                    .add(ghost_action("Doctor"))
                    .on_hover_text("Check config, connectivity, CA status, and relay health.")
                    .clicked()
                {
                    match self.form.to_config() {
                        Ok(cfg) => {
                            let _ = self.cmd_tx.send(Cmd::Doctor(cfg));
                        }
                        Err(e) => {
                            self.toast =
                                Some((format!("Cannot run doctor: {}", e), Instant::now()));
                        }
                    }
                }
                if ui
                    .add(ghost_action("Save config"))
                    .on_hover_text("Write the current form to the app config file.")
                    .clicked()
                {
                    match self.save_current_config() {
                        Ok(_) => self.toast = Some(("Config saved.".into(), Instant::now())),
                        Err(e) => self.toast = Some((format!("Save failed: {}", e), Instant::now())),
                    }
                }
                if ui
                    .add(ghost_action("Walkthrough"))
                    .on_hover_text("Re-open the first-run setup guide.")
                    .clicked()
                {
                    self.form.show_first_run_wizard = true;
                    self.form.wizard_step = 0;
                }
            });
                });
            ui.add_space(12.0);
            tab_bar(ui, &mut self.active_tab);
            ui.add_space(8.0);
            if self.form.show_first_run_wizard {
                show_first_run_wizard(ui, &mut self.form, &self.cmd_tx, &mut self.toast);
            }

            if self.active_tab == UiTab::Help {
                section(ui, "Help & walkthrough", |ui| {
                    help_walkthrough(ui, &self.form);
                });
            }
            if self.active_tab == UiTab::Trust {
                section(ui, "Trust Center", |ui| {
                    trust_center_tab(ui, &self.form, &self.cmd_tx);
                });
            }

            if self.active_tab == UiTab::Setup {
            ui.add_space(2.0);

            // ── Section: Mode ─────────────────────────────────────────────
            // Surfacing the mode at the top of the form because it changes
            // which of the sections below are actually used. `direct` is
            // a no-relay SNI-rewrite mode: Google edge by default, plus
            // to deploy Code.gs — once deployed, they switch back to
            // apps_script when you need the Apps Script relay.
            section(ui, "Mode", |ui| {
                help_muted(ui, "Start here: the mode decides whether relay credentials are required and whether you need the MITM certificate on this computer.");
                form_row(ui, "Mode", Some(
                    "apps_script — Full DPI bypass via Apps Script + local MITM (needs trusted CA).\n\
                     vercel_edge — Serverless JSON fetch relay on Vercel/Netlify (needs trusted CA).\n\
                     full — Everything tunnels via Apps Script + your tunnel node (no local cert).\n\
                     direct - No relay: Google SNI rewrite plus configured fronting_groups."
                ), |ui| {
                    egui::ComboBox::from_id_source("mode")
                        .selected_text(match self.form.mode.as_str() {
                            "direct" | "google_only" => "Direct fronting (no relay)",
                            "full" => "Full tunnel (no cert)",
                            "vercel_edge" => "Serverless JSON (no VPS)",
                            _ => "Apps Script (MITM)",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.form.mode,
                                "apps_script".into(),
                                "Apps Script (MITM)",
                            );
                            ui.selectable_value(
                                &mut self.form.mode,
                                "full".into(),
                                "Full tunnel (no cert)",
                            );
                            ui.selectable_value(
                                &mut self.form.mode,
                                "vercel_edge".into(),
                                "Serverless JSON (no VPS)",
                            );
                            ui.selectable_value(
                                &mut self.form.mode,
                                "direct".into(),
                                "Direct fronting (no relay)",
                            );
                        });
                });
                mode_summary_panel(ui, &self.form.mode);
                ui.add_space(10.0);
                mode_dashboard_panel(
                    ui,
                    &self.form,
                    running,
                    has_unsaved_changes,
                    &mut self.active_tab,
                );
                if self.form.mode == "direct" {
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.small(egui::RichText::new(
                            "Bootstrap mode — reach script.google.com to deploy Code.gs, then switch back to Apps Script.",
                        )
                        .color(OK_GREEN));
                    });
                }
                if self.form.mode == "full" {
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.small(egui::RichText::new(
                            "Full tunnel — all traffic tunneled end-to-end via Apps Script + remote tunnel node. No certificate needed.",
                        )
                        .color(OK_GREEN));
                    });
                }
                if self.form.mode == "vercel_edge" {
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.small(egui::RichText::new(
                            "Serverless JSON — no VPS backend; deploy tools/vercel-json-relay or tools/netlify-json-relay and trust the local CA.",
                        )
                        .color(OK_GREEN));
                    });
                }
            });

            let direct_mode = self.form.mode == "direct" || self.form.mode == "google_only";
            let vercel_edge = self.form.mode == "vercel_edge";
            let _using_groups = !self.form.account_groups.is_empty();

            // ── Section: Apps Script relay ────────────────────────────────
            section(ui, "Apps Script relay", |ui| {
                ui.add_enabled_ui(!direct_mode && !vercel_edge, |ui| {
                    help_muted(
                        ui,
                        "Paste the same deployment URLs / IDs and AUTH_KEY values you configured in Google Apps Script. \
                         Everything now lives under Advanced → Multi-account pools (each row is one Google account with its own secret). \
                         Single-field tutorials still work conceptually — just translate them into one pool.",
                    );
                });
            });

            // ── Section: Network ──────────────────────────────────────────
            section(ui, "Serverless JSON relay", |ui| {
                ui.add_enabled_ui(vercel_edge, |ui| {
                    help_muted(
                        ui,
                        "Deploy tools/vercel-json-relay to Vercel or tools/netlify-json-relay to Netlify, set AUTH_KEY, then paste the deployment URL and the same key here. If the client receives HTML instead of JSON, remove platform protection or fix the /api/api route.",
                    );
                    form_row(ui, "Base URL", Some("Example: https://my-relay.vercel.app or https://my-relay.netlify.app"), |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.form.vercel_base_url)
                                .hint_text("https://your-project.vercel.app or https://your-site.netlify.app")
                                .desired_width(f32::INFINITY),
                        );
                    });
                    form_row(ui, "Relay path", Some("Default for the bundled Edge function is /api/api."), |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.form.vercel_relay_path)
                                .desired_width(f32::INFINITY),
                        );
                    });
                    form_row(ui, "Auth key", Some("Must match AUTH_KEY in the Vercel/Netlify project."), |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.form.vercel_auth_key)
                                .password(!self.form.show_vercel_auth_key)
                                .desired_width(f32::INFINITY),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.checkbox(&mut self.form.show_vercel_auth_key, "Show auth key");
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.checkbox(&mut self.form.vercel_verify_tls, "Verify relay TLS certificate");
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.checkbox(
                            &mut self.form.vercel_enable_batching,
                            "Enable JSON batch envelope",
                        )
                        .on_hover_text(
                            "Experimental serverless relay batching. Keep off unless your deployed \
                             JSON relay supports the {k,q:[...]} batch request envelope.",
                        );
                    });
                    form_row(ui, "Max body MB", Some("Guardrail for one client request sent through JSON/base64."), |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.form.vercel_max_body_mb)
                                .speed(1)
                                .range(1..=32),
                        );
                    });
                });
            });

            section(ui, "Backend tools and deployment recipes", |ui| {
                help_muted(
                    ui,
                    "Native modes are selected above. The helpers below are deployment recipes or companion relays; use the row that matches the mode or external client you actually run.",
                );
                ui.add_space(4.0);
                for (idx, entry) in backend_tool_entries().iter().copied().enumerate() {
                    render_tool_help_row(ui, entry);
                    if idx + 1 != backend_tool_entries().len() {
                        ui.separator();
                    }
                }
                ui.add_space(8.0);
                help_subheading(ui, "XHTTP VLESS generator");
                if let Some(msg) =
                    xhttp_vless_generator(ui, &mut self.form.xhttp_generator, &mut self.xhttp_deploy)
                {
                    self.toast = Some((msg, Instant::now()));
                }
            });

            }
            if self.active_tab == UiTab::Network {
            section(ui, "Network", |ui| {
                help_muted(
                    ui,
                    "These knobs describe how we reach Google’s edge. If anything feels flaky, re-check google_ip first, then front_domain, then the SNI pool.",
                );
                form_row(ui, "Google IP", Some(
                    "IPv4 address of a Google frontend that answers TLS when your front_domain is sent as the SNI. \
                     Wrong or poisoned values cause instant timeouts — use Scan IPs or an IP you’ve verified manually."
                ), |ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.form.google_ip)
                        .desired_width(f32::INFINITY));
                });
                ui.horizontal(|ui| {
                    ui.add_space(120.0 + 8.0);
                    if ui.small_button("scan IPs")
                        .on_hover_text(
                            "Probe known Google frontend IPs; report which are reachable \
                             (results go to the log panel)."
                        )
                        .clicked()
                    {
                        if let Ok(cfg) = self.form.to_config() {
                            let _ = self.cmd_tx.send(Cmd::Test(cfg.clone()));
                            self.toast = Some((
                                "Scan started — check the Recent log below.".into(),
                                Instant::now(),
                            ));
                        }
                    }
                    let active_sni = self.form.sni_pool.iter().filter(|r| r.enabled).count();
                    let total_sni = self.form.sni_pool.len();
                    let sni_btn = egui::Button::new(
                        egui::RichText::new(format!("SNI pool… ({}/{})", active_sni, total_sni))
                            .color(egui::Color32::WHITE),
                    )
                    .fill(ACCENT)
                    .rounding(6.0);
                    if ui.add(sni_btn)
                        .on_hover_text(
                            "Open the SNI rotation pool editor. Test which front-domain \
                             names get through your network's DPI."
                        )
                        .clicked()
                    {
                        self.form.sni_editor_open = true;
                    }
                });

                form_row(ui, "Front domain", Some(
                    "Hostname for the outer TLS SNI (not the HTTP Host header). Must stay a name, never a raw IP. \
                     www.google.com is the usual default; some networks respond better to other Google hostnames — use the SNI tester."
                ), |ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.form.front_domain)
                        .desired_width(f32::INFINITY));
                });

                form_row(ui, "Listen host", Some(
                    "Interface to bind the local HTTP/SOCKS listeners. 127.0.0.1 keeps traffic on this machine; \
                     0.0.0.0 exposes them to your LAN (only if you understand the security trade-offs)."
                ), |ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.form.listen_host)
                        .desired_width(f32::INFINITY));
                });

                ui.horizontal(|ui| {
                    let pr = ui.add_sized(
                        [120.0, 20.0],
                        egui::Label::new(egui::RichText::new("Ports")
                            .color(egui::Color32::from_gray(200))),
                    );
                    pr.on_hover_text(
                        "HTTP is the primary proxy port. SOCKS5 is optional but handy for apps that only speak SOCKS. \
                         Leave SOCKS blank to disable — they must not be the same number.",
                    );
                    ui.label(egui::RichText::new("HTTP").small());
                    ui.add(egui::TextEdit::singleline(&mut self.form.listen_port).desired_width(70.0));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("SOCKS5").small());
                    ui.add(egui::TextEdit::singleline(&mut self.form.socks5_port).desired_width(70.0));
                });
            });

            section(ui, "Sharing and per-app routing", |ui| {
                help_muted(
                    ui,
                    "Desktop exposes HTTP/SOCKS proxy listeners. Apps that let you choose a proxy can opt in per app. Full transparent desktop per-app capture needs OS-specific packet filtering and is not exposed as a fake toggle here.",
                );
                let listen_host_snapshot = self.form.listen_host.trim().to_string();
                let lower_snapshot = listen_host_snapshot.to_ascii_lowercase();
                let custom_bind = !listen_host_snapshot.is_empty()
                    && !is_share_on_lan(&listen_host_snapshot)
                    && !is_loopback_only(&lower_snapshot);
                let lan_bound = is_share_on_lan(&listen_host_snapshot);
                let detected_lan_ip = if lan_bound { detect_lan_ip() } else { None };
                ui.horizontal_wrapped(|ui| {
                    if custom_bind {
                        ui.label(
                            egui::RichText::new(format!("Custom bind: {listen_host_snapshot}"))
                                .strong()
                                .color(egui::Color32::from_rgb(240, 190, 90)),
                        )
                        .on_hover_text(
                            "A specific bind address is set in Listen host. The LAN toggle is locked so Save cannot replace that custom interface.",
                        );
                        ui.small("Edit Listen host above to switch back to local-only or all-interface sharing.");
                    } else {
                        let mut share = lan_bound;
                        if ui
                            .checkbox(&mut share, "Share with other devices on my Wi-Fi / network")
                            .on_hover_text(
                                "When enabled, HTTP/SOCKS bind to 0.0.0.0 so trusted LAN devices can connect. macOS may show a Firewall prompt the first time.",
                            )
                            .changed()
                        {
                            self.form.listen_host = if share {
                                "0.0.0.0".into()
                            } else {
                                "127.0.0.1".into()
                            };
                        }
                        let (color, label) = if share {
                            (ERR_RED, "LAN exposed")
                        } else {
                            (OK_GREEN, "local-only")
                        };
                        ui.label(egui::RichText::new(label).strong().color(color));
                    }
                });
                let lan_bound = is_share_on_lan(self.form.listen_host.trim());
                let detected_lan_ip = if lan_bound { detected_lan_ip } else { None };
                let client_host = if lan_bound {
                    detected_lan_ip
                        .map(|ip| ip.to_string())
                        .unwrap_or_else(|| "this-device-LAN-IP".into())
                } else {
                    "127.0.0.1".into()
                };
                let http_endpoint = format!("http://{}:{}", client_host, self.form.listen_port.trim());
                let socks_endpoint = self.form.socks5_port.trim();
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(120.0 + 8.0);
                    ui.label(
                        egui::RichText::new(&http_endpoint)
                            .monospace()
                            .color(egui::Color32::from_gray(185)),
                    );
                    if ui
                        .small_button("copy HTTP")
                        .on_hover_text("Copy the HTTP proxy endpoint for browser/app proxy settings.")
                        .clicked()
                    {
                        ui.ctx().copy_text(http_endpoint.clone());
                        self.toast = Some(("HTTP proxy endpoint copied.".into(), Instant::now()));
                    }
                    if !socks_endpoint.is_empty() {
                        let socks_url = format!("socks5://{}:{}", client_host, socks_endpoint);
                        ui.label(
                            egui::RichText::new(&socks_url)
                                .monospace()
                                .color(egui::Color32::from_gray(185)),
                        );
                        if ui
                            .small_button("copy SOCKS")
                            .on_hover_text("Copy the SOCKS5 endpoint for apps that support SOCKS.")
                            .clicked()
                        {
                            ui.ctx().copy_text(socks_url);
                            self.toast = Some(("SOCKS5 endpoint copied.".into(), Instant::now()));
                        }
                    }
                });
                if lan_bound {
                    let msg = if detected_lan_ip.is_some() {
                        "LAN address was detected from the OS route table; no probe packet is sent."
                    } else {
                        "LAN address could not be detected. Use the active Wi-Fi/Ethernet IPv4 from system network settings."
                    };
                    ui.horizontal_wrapped(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.small(egui::RichText::new(msg).color(egui::Color32::from_gray(145)));
                    });
                }

                let mut lan_token = self.form.lan_token.clone().unwrap_or_default();
                form_row(
                    ui,
                    "LAN token",
                    Some("Optional HTTP proxy guard. Clients that support custom proxy headers must send X-MHRV-F-Token. SOCKS5 cannot send this header, so use allowed IPs for SOCKS5 over LAN."),
                    |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut lan_token)
                                .hint_text("blank = no token")
                                .desired_width(f32::INFINITY),
                        );
                    },
                );
                self.form.lan_token = clean_optional_text(&lan_token);

                let mut lan_allowlist = self
                    .form
                    .lan_allowlist
                    .clone()
                    .unwrap_or_default()
                    .join("\n");
                form_row(
                    ui,
                    "Allowed IPs",
                    Some("Optional LAN clients allowed to connect. Supports one IP or CIDR per line, for example 192.168.1.42 or 192.168.1.0/24. Required for safe SOCKS5 sharing."),
                    |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut lan_allowlist)
                                .hint_text("192.168.1.42\n192.168.1.0/24")
                                .desired_rows(2)
                                .desired_width(f32::INFINITY),
                        );
                    },
                );
                self.form.lan_allowlist = clean_optional_list(&lan_allowlist);

                let lan_bound = is_share_on_lan(self.form.listen_host.trim());
                let token_set = self
                    .form
                    .lan_token
                    .as_deref()
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
                let allowlist_set = self
                    .form
                    .lan_allowlist
                    .as_ref()
                    .map(|v| !v.is_empty())
                    .unwrap_or(false);
                if lan_bound && !token_set && !allowlist_set {
                    ui.small(egui::RichText::new(
                        "Before sharing on LAN, add a token for HTTP clients or allowed IPs for SOCKS5/HTTP clients.",
                    ).color(ERR_RED));
                } else if lan_bound && token_set && !allowlist_set {
                    ui.small(egui::RichText::new(
                        "HTTP can use the token; SOCKS5 over LAN will fail closed until Allowed IPs is set.",
                    ).color(egui::Color32::from_rgb(240, 190, 90)));
                } else {
                    ui.small(egui::RichText::new(
                        "Android VPN mode has native app splitting. Desktop per-app routing is app-level proxy opt-in unless you add an external OS routing tool.",
                    ).color(egui::Color32::from_gray(145)));
                }
            });

            // ── Section: Advanced (collapsed by default) ──────────────────
            }
            if self.active_tab == UiTab::Advanced {
            ui.add_space(6.0);
            egui::CollapsingHeader::new(
                egui::RichText::new("Advanced")
                    .size(13.0)
                    .color(ACCENT.linear_multiply(0.75))
                    .strong(),
            )
            .id_source("panel_advanced")
            .default_open(false)
            .show(ui, |ui| {
                let frame = egui::Frame::none()
                    .fill(CARD_FILL)
                    .stroke(egui::Stroke::new(1.0, CARD_STROKE))
                    .rounding(10.0)
                    .inner_margin(egui::Margin::same(12.0));
                frame.show(ui, |ui| {
                    help_callout(
                        ui,
                        "How to tune without making things worse",
                        "If pages are slow but reliable, try balanced auto-tune first. If pages fail with quota/504 messages, add account groups or lower rate/fan-out instead of raising speed knobs. If large downloads stall, adjust range settings. If non-HTTP apps fail, use upstream SOCKS5 or Full mode rather than forcing Apps Script to carry raw TCP.",
                        ACCENT_WARM,
                    );
                    help_muted(ui, "Power-user levers: SOCKS chaining, parallelism, runtime profiles, and per-account pools. Expand each subsection when you need it — defaults are safe for everyday use.");
                    form_row(ui, "Upstream SOCKS5", Some(
                        "Optional. host:port of a local xray / v2ray / sing-box SOCKS5 inbound. \
                         When set, non-HTTP / raw-TCP traffic (Telegram MTProto, IMAP, SSH, …) \
                         is chained through it instead of direct. HTTP/HTTPS still go through \
                         the Apps Script relay."
                    ), |ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.form.upstream_socks5)
                            .hint_text("empty = direct; 127.0.0.1:50529 for local xray")
                            .desired_width(f32::INFINITY));
                    });

                    form_row(ui, "Parallel dispatch", Some(
                        "Fire N Apps Script IDs in parallel per request and take the first \
                         response. 0/1 = off. 2-3 kills long-tail latency at N× quota cost. \
                         Only effective with multiple IDs configured."
                    ), |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.parallel_relay)
                            .speed(1)
                            .range(0..=8));
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.checkbox(
                            &mut self.form.youtube_via_relay,
                            "Send YouTube HTML/API through relay",
                        )
                        .on_hover_text(
                            "Helps with YouTube Restricted Mode/SNI policy issues by relaying \
                             youtube.com/youtu.be/youtubei.googleapis.com. Thumbnails stay on \
                             SNI rewrite, and googlevideo.com is not forced onto the normal \
                             Google frontend IP.",
                        );
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.checkbox(&mut self.form.runtime_auto_tune, "Adaptive runtime profile (auto-tune)")
                            .on_hover_text(
                                "When enabled, mhrv-f applies a profile (eco/balanced/max_speed) to a few hot-path knobs: \
                                 parallel_relay (if set to 0/1), range-parallelism, and relay timeouts.",
                            );
                    });

                    form_row(ui, "Profile", Some("eco / balanced / max_speed"), |ui| {
                        egui::ComboBox::from_id_source("runtime_profile")
                            .selected_text(&self.form.runtime_profile)
                            .show_ui(ui, |ui| {
                                for p in ["eco", "balanced", "max_speed"] {
                                    ui.selectable_value(&mut self.form.runtime_profile, p.into(), p);
                                }
                            });
                    });

                    form_row(ui, "Range parallelism", Some("Max concurrent chunk fetches for large downloads (relay_parallel_range)."), |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.range_parallelism)
                            .speed(1)
                            .range(1..=32));
                    });

                    form_row(ui, "Range chunk (KB)", Some("Chunk size for range-parallel downloads."), |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.range_chunk_kb)
                            .speed(16)
                            .range(16..=2048));
                    });

                    form_row(ui, "Relay timeout (s)", Some("Timeout for one Apps Script relay round trip."), |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.relay_request_timeout_secs)
                            .speed(1)
                            .range(5..=120));
                    });

                    form_row(ui, "Full batch timeout (s)", Some("Full mode sends many socket operations in one Apps Script + tunnel-node batch. Higher values help slow-but-working networks finish; lower values expose dead batches sooner. If you have only one deployment, lowering this can turn recoverable slowness into errors. If you have several healthy deployments, a moderate lower value can fail over faster."), |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.request_timeout_secs)
                            .speed(1)
                            .range(5..=300));
                    });

                    form_row(ui, "Timeout strikes", Some("How much proof is needed before a deployment is treated as bad. A strike is one timed-out Full-mode batch. More strikes are patient and protect single-deployment users from false lockouts. Fewer strikes are aggressive and help multi-ID pools stop wasting retries on a bad deployment."), |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.auto_blacklist_strikes)
                            .speed(1)
                            .range(1..=100));
                    });

                    form_row(ui, "Strike window (s)", Some("How close together strikes must be to count as the same incident. Short windows catch immediate outages but forgive scattered mobile-network blips. Long windows are stricter because failures several seconds apart can still accumulate."), |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.auto_blacklist_window_secs)
                            .speed(1)
                            .range(1..=86_400));
                    });

                    form_row(ui, "Cooldown (s)", Some("How long the deployment stays out of rotation after enough strikes. Short cooldown retries soon, useful when that deployment may be your only path. Long cooldown is better when other IDs can carry traffic, because it prevents repeatedly paying latency/quota for a known-bad ID."), |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.auto_blacklist_cooldown_secs)
                            .speed(1)
                            .range(1..=86_400));
                    });

                    form_row(ui, "Log level", None, |ui| {
                        egui::ComboBox::from_id_source("loglevel")
                            .selected_text(&self.form.log_level)
                            .show_ui(ui, |ui| {
                                for lvl in ["warn", "info", "debug", "trace"] {
                                    ui.selectable_value(&mut self.form.log_level, lvl.into(), lvl);
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.checkbox(&mut self.form.verify_ssl, "Verify TLS server certificate (recommended)");
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.checkbox(
                            &mut self.form.enable_batching,
                            "Preserve legacy Apps Script batching flag",
                        )
                        .on_hover_text(
                            "Advanced compatibility flag for old or hand-edited configs. Runtime \
                             batching for Serverless JSON is controlled by the JSON batch envelope \
                             option in the Serverless JSON relay section.",
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        // Per-group toggles exist inside the multi-account editor.
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.checkbox(&mut self.form.normalize_x_graphql, "Normalize X/Twitter GraphQL URLs")
                            .on_hover_text(
                                "Trim the `features` / `fieldToggles` query params from x.com/i/api/graphql/… \
                                 requests before relaying. Massively improves cache hit rate when browsing \
                                 Twitter/X. Off by default — some endpoints may reject trimmed requests. \
                                 Based on the community X GraphQL cache pattern.",
                            );
                    });

                    ui.add_space(6.0);
                    egui::CollapsingHeader::new("Multi-account pools (backup Google accounts)")
                        .default_open(false)
                        .show(ui, |ui| {
                            help_callout(
                                ui,
                                "What is a group?",
                                "Treat one group as one Google account or quota pool. One AUTH_KEY protects that group's Apps Script deployments. Multiple deployment IDs inside the group are rotation/fallback endpoints for the same account. Multiple groups mean multiple accounts or intentionally separate pools; weights decide how much traffic each group should receive.",
                                ACCENT_MINT,
                            );
                            ui.small(
                                "Optional. Configure multiple Apps Script account pools (each with its own AUTH_KEY + deployment IDs). \
                                 When at least one group is set here, use only these rows for relay identity — not separate one-line deployment/auth fields from single-field tutorials.",
                            );
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                if ui.button("+ Add group").clicked() {
                                    self.form.account_groups.push(AccountGroupForm {
                                        label: String::new(),
                                        enabled: true,
                                        weight: 1,
                                        auth_key: String::new(),
                                        script_ids: String::new(),
                                        show_auth_key: false,
                                    });
                                }
                                if ui.button("Clear groups").clicked() {
                                    self.form.account_groups.clear();
                                }
                                if !self.form.account_groups.is_empty() {
                                    ui.small(egui::RichText::new(format!(
                                        "{} group(s) configured",
                                        self.form.account_groups.len()
                                    )).color(OK_GREEN));
                                }
                            });

                            let mut remove_idx: Option<usize> = None;
                            for (i, g) in self.form.account_groups.iter_mut().enumerate() {
                                ui.add_space(4.0);
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut g.enabled, "");
                                    ui.label(egui::RichText::new(format!("Group {}", i + 1)).strong());
                                    ui.add_space(6.0);
                                    ui.label("weight");
                                    ui.add(
                                        egui::DragValue::new(&mut g.weight)
                                            .speed(1)
                                            .range(1..=10),
                                    );
                                    if ui.small_button("remove").clicked() {
                                        remove_idx = Some(i);
                                    }
                                });

                                form_row(ui, "Label", Some("Optional label shown in UI/logs."), |ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut g.label)
                                            .hint_text("e.g. personal / backup1")
                                            .desired_width(f32::INFINITY),
                                    );
                                });

                                form_row(ui, "IDs", Some("One deployment ID per line."), |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut g.script_ids)
                                            .hint_text("one deployment ID per line")
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(2),
                                    );
                                });

                                form_row(ui, "Auth key", Some("Must match AUTH_KEY inside that account's Code.gs."), |ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut g.auth_key)
                                            .password(!g.show_auth_key)
                                            .desired_width(f32::INFINITY),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.add_space(120.0 + 8.0);
                                    ui.checkbox(&mut g.show_auth_key, "Show auth key");
                                });
                            }
                            if let Some(i) = remove_idx {
                                self.form.account_groups.remove(i);
                            }
                        });
                });
            });

            // ── Bottom of form: Save + config-path hint ───────────────────
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.add(primary_button("Save config")).clicked() {
                    match self.save_current_config() {
                        Ok(p) => self.toast = Some((format!("Saved to {}", p.display()), Instant::now())),
                        Err(e) => self.toast = Some((format!("Save failed: {}", e), Instant::now())),
                    }
                }
                ui.small(egui::RichText::new(format!("→ {}", data_dir::config_path().display()))
                    .color(egui::Color32::from_gray(130)));
            });
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                help_muted(ui, "Tip: if fields changed, the header button becomes Save and start so the running proxy uses the form you are looking at.");
            });

            // Profiles: save/load named configs in user-data dir.
            ui.add_space(6.0);
            section(ui, "Profiles", |ui| {
                help_muted(ui, "Named snapshots of your whole form — handy when you switch networks or experiment with SNI lists.");
                form_row(ui, "Name", Some("Use only letters/numbers/_-"), |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.form.profile_name)
                            .hint_text("e.g. home / backup / gaming")
                            .desired_width(f32::INFINITY),
                    );
                });
                ui.horizontal(|ui| {
                    ui.add_space(120.0 + 8.0);
                    if ui.button("Refresh list").clicked() {
                        self.form.profiles = mhrv_jni::profiles::list_profiles().unwrap_or_default();
                    }
                    if ui.button("Save as profile").clicked() {
                        match self.form.to_config() {
                            Ok(cfg) => match mhrv_jni::profiles::save_profile(&self.form.profile_name, &cfg) {
                                Ok(p) => {
                                    self.toast = Some((format!("Profile saved: {}", p.display()), Instant::now()));
                                    self.form.profiles = mhrv_jni::profiles::list_profiles().unwrap_or_default();
                                }
                                Err(e) => self.toast = Some((format!("Profile save failed: {}", e), Instant::now())),
                            },
                            Err(e) => self.toast = Some((format!("Cannot save profile: {}", e), Instant::now())),
                        }
                    }
                });

                if self.form.profiles.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        ui.small(egui::RichText::new("No saved profiles yet.").color(egui::Color32::from_gray(140)));
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.add_space(120.0 + 8.0);
                        egui::ComboBox::from_id_source("profiles_list")
                            .selected_text(if self.form.profile_name.trim().is_empty() {
                                "(select…)".to_string()
                            } else {
                                self.form.profile_name.trim().to_string()
                            })
                            .show_ui(ui, |ui| {
                                for p in self.form.profiles.clone() {
                                    ui.selectable_value(&mut self.form.profile_name, p.clone(), p);
                                }
                            });
                        if ui.button("Load").clicked() {
                            match mhrv_jni::profiles::load_profile(&self.form.profile_name) {
                                Ok(cfg) => {
                                    // Replace form fields with loaded config.
                                    // Reuse the existing Config->Form mapping by writing
                                    // a minimal inline conversion here.
                                    self.form.mode = cfg.mode.clone();
                                    self.form.google_ip = cfg.google_ip.clone();
                                    self.form.front_domain = cfg.front_domain.clone();
                                    self.form.listen_host = cfg.listen_host.clone();
                                    self.form.listen_port = cfg.listen_port.to_string();
                                    self.form.socks5_port = cfg.socks5_port.map(|p| p.to_string()).unwrap_or_default();
                                    self.form.log_level = cfg.log_level.clone();
                                    self.form.verify_ssl = cfg.verify_ssl;
                                    self.form.enable_batching = cfg.enable_batching;
                                    self.form.vercel_base_url = cfg.vercel.base_url.clone();
                                    self.form.vercel_relay_path = cfg.vercel.relay_path.clone();
                                    self.form.vercel_auth_key = cfg.vercel.auth_key.clone();
                                    self.form.vercel_verify_tls = cfg.vercel.verify_tls;
                                    self.form.vercel_max_body_mb = cfg
                                        .vercel
                                        .max_body_bytes
                                        .max(1024)
                                        .div_ceil(1024 * 1024)
                                        as u32;
                                    self.form.vercel_enable_batching = cfg.vercel.enable_batching;
                                    self.form.upstream_socks5 = cfg.upstream_socks5.clone().unwrap_or_default();
                                    self.form.parallel_relay = cfg.parallel_relay;
                                    self.form.coalesce_step_ms = cfg.coalesce_step_ms;
                                    self.form.coalesce_max_ms = cfg.coalesce_max_ms;
                                    self.form.runtime_auto_tune = cfg.runtime_auto_tune;
                                    self.form.runtime_profile = cfg
                                        .runtime_profile
                                        .clone()
                                        .unwrap_or_else(|| "balanced".into());
                                    self.form.range_parallelism = cfg.range_parallelism.unwrap_or(12);
                                    self.form.range_chunk_kb =
                                        (cfg.range_chunk_bytes.unwrap_or(256 * 1024) / 1024) as u32;
                                    self.form.relay_request_timeout_secs =
                                        cfg.relay_request_timeout_secs.unwrap_or(25);
                                    self.form.request_timeout_secs =
                                        cfg.request_timeout_secs.unwrap_or(30);
                                    self.form.auto_blacklist_strikes =
                                        cfg.auto_blacklist_strikes.unwrap_or(3);
                                    self.form.auto_blacklist_window_secs =
                                        cfg.auto_blacklist_window_secs.unwrap_or(30);
                                    self.form.auto_blacklist_cooldown_secs =
                                        cfg.auto_blacklist_cooldown_secs.unwrap_or(120);
                                    self.form.normalize_x_graphql = cfg.normalize_x_graphql;
                                    self.form.youtube_via_relay = cfg.youtube_via_relay;
                                    self.form.passthrough_hosts = cfg.passthrough_hosts.clone();
                                    self.form.block_quic = cfg.block_quic;
                                    self.form.tunnel_doh = cfg.tunnel_doh;
                                    self.form.bypass_doh_hosts = cfg.bypass_doh_hosts.clone();
                                    self.form.domain_overrides = cfg.domain_overrides.clone();
                                    self.form.lan_token = cfg.lan_token.clone();
                                    self.form.lan_allowlist = cfg.lan_allowlist.clone();
                                    self.form.outage_reset_enabled = cfg.outage_reset_enabled;
                                    self.form.outage_reset_failure_threshold =
                                        cfg.outage_reset_failure_threshold;
                                    self.form.outage_reset_window_ms = cfg.outage_reset_window_ms;
                                    self.form.outage_reset_cooldown_ms = cfg.outage_reset_cooldown_ms;
                                    self.form.relay_rate_limit_qps = cfg.relay_rate_limit_qps;
                                    self.form.relay_rate_limit_burst = cfg.relay_rate_limit_burst;
                                    self.form.fetch_ips_from_api = cfg.fetch_ips_from_api;
                                    self.form.max_ips_to_scan = cfg.max_ips_to_scan;
                                    self.form.google_ip_validation = cfg.google_ip_validation;
                                    self.form.scan_batch_size = cfg.scan_batch_size;
                                    // Account groups.
                                    self.form.account_groups = cfg.account_groups.clone().unwrap_or_default().into_iter().map(|g| {
                                        AccountGroupForm {
                                            label: g.label.unwrap_or_default(),
                                            enabled: g.enabled,
                                            weight: g.weight,
                                            auth_key: g.auth_key,
                                            script_ids: g.script_ids.into_vec().join("\n"),
                                            show_auth_key: false,
                                        }
                                    }).collect();
                                    // SNI pool UI
                                    self.form.sni_pool = sni_pool_for_form(cfg.sni_hosts.as_deref(), &cfg.front_domain);
                                    self.toast = Some((format!("Profile loaded: {}", self.form.profile_name), Instant::now()));
                                }
                                Err(e) => self.toast = Some((format!("Profile load failed: {}", e), Instant::now())),
                            }
                        }
                    });
                }
            });

            }
            // Non-fatal warnings about unsafe settings. These are advisory only.
            if let Ok(cfg) = self.form.to_config() {
                let warns = cfg.unsafe_warnings();
                if !warns.is_empty() {
                    ui.add_space(4.0);
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(60, 50, 30))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(140, 120, 70)))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Warnings").strong().color(egui::Color32::from_rgb(240, 210, 120)));
                            for w in warns.iter().take(3) {
                                ui.small(egui::RichText::new(format!("• {}", w)).color(egui::Color32::from_gray(230)));
                            }
                            if warns.len() > 3 {
                                ui.small(egui::RichText::new(format!("…and {} more", warns.len() - 3)).color(egui::Color32::from_gray(200)));
                            }
                        });
                }
            }

            // Floating SNI editor window. Rendered here so it's inside the
            // same egui context but visually pops out with its own title bar.
            self.show_sni_editor(ctx);

            ui.add_space(8.0);

            // ── Status + stats card ────────────────────────────────────────
            if self.active_tab == UiTab::Monitor {
            let (
                running,
                started_at,
                stats,
                ca_trusted,
                last_test_msg,
                per_site,
                doctor_report,
                doctor_at,
            ) = {
                let s = self.shared.state.lock().unwrap();
                (
                    s.running,
                    s.started_at,
                    s.last_stats,
                    s.ca_trusted,
                    s.last_test_msg.clone(),
                    s.last_per_site.clone(),
                    s.last_doctor_report.clone(),
                    s.last_doctor_at,
                )
            };

            let status_title = if running {
                let up = started_at.map(|t| t.elapsed()).unwrap_or_default();
                format!("Traffic  ·  uptime {}", fmt_duration(up))
            } else {
                "Traffic  ·  (not running)".to_string()
            };
            section(ui, &status_title, |ui| {
                if let Some(s) = stats {
                    // Compact two-column layout so all core metrics fit in ~6
                    // rows instead of a tall vertical strip.
                    let rows = traffic_stat_rows(&s);
                    egui::Grid::new("stats")
                        .num_columns(4)
                        .spacing([16.0, 4.0])
                        .show(ui, |ui| {
                            for chunk in rows.chunks(2) {
                                for (label, value) in chunk.iter() {
                                    ui.add_sized(
                                        [110.0, 18.0],
                                        egui::Label::new(
                                            egui::RichText::new(*label)
                                                .color(egui::Color32::from_gray(150)),
                                        ),
                                    );
                                    ui.add_sized(
                                        [140.0, 18.0],
                                        egui::Label::new(
                                            egui::RichText::new(value).monospace(),
                                        ),
                                    );
                                }
                                // Pad the final short row so grid columns stay aligned.
                                if chunk.len() == 1 {
                                    ui.label("");
                                    ui.label("");
                                }
                                ui.end_row();
                            }
                        });
                } else {
                    ui.label(
                        egui::RichText::new("No traffic yet — click Start and send a request.")
                            .color(egui::Color32::from_gray(150))
                            .italics(),
                    );
                }
            });

            ui.add_space(4.0);
            egui::CollapsingHeader::new(
                egui::RichText::new("What the traffic numbers mean")
                    .strong()
                    .color(ACCENT)
                    .size(12.5),
            )
            .id_source("help_traffic_stats")
            .default_open(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 5.0;
                help_muted(ui, "relay calls / failures — volume through Apps Script vs errors (timeouts, HTTP 5xx, etc.).");
                help_muted(ui, "coalesced — duplicate in-flight requests merged to protect quota.");
                help_muted(ui, "today calls / reset in — Apps Script daily counter window (UTC midnight reset).");
                help_muted(ui, "cache hits — responses served from the local cache instead of another relay hop.");
                help_muted(ui, "bytes relayed — payload volume observed by the proxy since start.");
                help_muted(ui, "degrade — automatic backoff level when the engine detects overload or repeated failures.");
                help_muted(ui, "active scripts — deployments still participating vs temporarily blacklisted.");
            });

            ui.add_space(8.0);
            render_doctor_summary_card(ui, doctor_report.as_ref(), doctor_at);

            // ── Dashboard widgets ───────────────────────────────────────────
            let (degrade_history, recent_log) = {
                let s = self.shared.state.lock().unwrap();
                (s.degrade_history.clone(), s.log.iter().cloned().collect::<Vec<String>>())
            };
            section(ui, "Dashboard", |ui| {
                if let Some(s) = stats {
                    // Quota pressure (very rough): show current call rate since UTC midnight.
                    let calls_per_hour = quota_calls_per_hour(s.today_calls, s.today_reset_secs);
                    egui::Frame::none()
                        .fill(CARD_FILL)
                        .stroke(egui::Stroke::new(1.0, CARD_STROKE))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Quota pressure").strong());
                            ui.small(format!(
                                "today_calls={}  reset_in={}  approx_rate={:.1} calls/hour",
                                s.today_calls,
                                fmt_duration(Duration::from_secs(s.today_reset_secs)),
                                calls_per_hour
                            ));
                            ui.small("Tip: add more account_groups/script_ids, reduce fanout, or enable relay_rate_limit_qps if quota spikes cause failures.");
                        });

                    ui.add_space(6.0);

                    // Degradation timeline: show only changes (level/reason).
                    let changes = degradation_changes(&degrade_history);
                    egui::Frame::none()
                        .fill(CARD_FILL)
                        .stroke(egui::Stroke::new(1.0, CARD_STROKE))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Degradation timeline").strong());
                            if changes.is_empty() {
                                ui.small("No degradation changes recorded yet.");
                            } else {
                                for (ago, lvl, reason) in changes {
                                    ui.small(format!("{} ago: L{} ({})", fmt_duration(ago), lvl, reason));
                                }
                            }
                        });

                    ui.add_space(6.0);

                    // Recent failures / notable events: mine the Recent log for high-signal lines.
                    let notable = notable_failure_lines(recent_log);
                    egui::Frame::none()
                        .fill(CARD_FILL)
                        .stroke(egui::Stroke::new(1.0, CARD_STROKE))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Recent failures (best-effort)").strong());
                            if notable.is_empty() {
                                ui.small("No recent failure-like events in this session's log.");
                            } else {
                                for line in notable {
                                    ui.small(line);
                                }
                            }
                            ui.small("Actions: add backup deployments/accounts, enable relay_rate_limit_qps, or switch problematic domains to direct via domain_overrides.");
                        });
                } else {
                    ui.small("Start the proxy to populate dashboard widgets.");
                }
            });

            if !per_site.is_empty() {
                ui.add_space(2.0);
                egui::CollapsingHeader::new(format!("Per-site ({} hosts)", per_site.len()))
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(140.0)
                            .show(ui, |ui| {
                                egui::Grid::new("per_site")
                                    .num_columns(5)
                                    .spacing([8.0, 2.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label(egui::RichText::new("host").strong());
                                        ui.label(egui::RichText::new("req").strong());
                                        ui.label(egui::RichText::new("hit%").strong());
                                        ui.label(egui::RichText::new("bytes").strong());
                                        ui.label(egui::RichText::new("avg ms").strong());
                                        ui.end_row();
                                        for (host, st) in per_site.iter().take(60) {
                                            let hit_pct = if st.requests > 0 {
                                                (st.cache_hits as f64 / st.requests as f64) * 100.0
                                            } else { 0.0 };
                                            ui.label(egui::RichText::new(host).monospace());
                                            ui.label(egui::RichText::new(st.requests.to_string()).monospace());
                                            ui.label(egui::RichText::new(format!("{:.0}%", hit_pct)).monospace());
                                            ui.label(egui::RichText::new(fmt_bytes(st.bytes)).monospace());
                                            ui.label(egui::RichText::new(format!("{:.0}", st.avg_latency_ms())).monospace());
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
            }

            ui.add_space(8.0);

            // ── Primary action: Start / Stop is the headline; others smaller ──
            ui.horizontal(|ui| {
                if !running {
                    let btn = egui::Button::new(
                        egui::RichText::new("▶  Start").color(egui::Color32::WHITE).strong(),
                    )
                    .fill(OK_GREEN.linear_multiply(0.92))
                    .stroke(egui::Stroke::new(1.0, OK_GREEN.linear_multiply(1.2)))
                    .min_size(egui::vec2(130.0, 36.0))
                    .rounding(8.0);
                    let enabled = can_start;
                    if ui.add_enabled(enabled, btn).clicked() {
                        self.start_current_config(has_unsaved_changes);
                    }
                } else {
                    let btn = egui::Button::new(
                        egui::RichText::new("■  Stop").color(egui::Color32::WHITE).strong(),
                    )
                    .fill(ERR_RED.linear_multiply(0.9))
                    .stroke(egui::Stroke::new(1.0, ERR_RED.linear_multiply(1.15)))
                    .min_size(egui::vec2(130.0, 36.0))
                    .rounding(8.0);
                    if ui.add(btn).clicked() {
                        let _ = self.cmd_tx.send(Cmd::Stop);
                    }
                }

                if ui.add(
                    egui::Button::new("Test relay")
                        .min_size(egui::vec2(0.0, 32.0))
                        .rounding(8.0),
                    ).on_hover_text("Send one request through the Apps Script relay end-to-end and report the result.").clicked() {
                    match self.form.to_config() {
                        Ok(cfg) => {
                            let _ = self.cmd_tx.send(Cmd::Test(cfg));
                        }
                        Err(e) => {
                            self.toast = Some((format!("Cannot test: {}", e), Instant::now()));
                        }
                    }
                }

                if ui.add(
                    egui::Button::new("Doctor")
                        .min_size(egui::vec2(0.0, 32.0))
                        .rounding(8.0),
                )
                .on_hover_text("Run guided diagnostics and print actionable fixes into the Recent log.")
                .clicked()
                {
                    match self.form.to_config() {
                        Ok(cfg) => {
                            let _ = self.cmd_tx.send(Cmd::Doctor(cfg));
                        }
                        Err(e) => {
                            self.toast = Some((format!("Cannot run doctor: {}", e), Instant::now()));
                        }
                    }
                }

                if ui.add(
                    egui::Button::new("Doctor + Fix")
                        .min_size(egui::vec2(0.0, 32.0))
                        .rounding(8.0),
                )
                .on_hover_text("Run doctor, apply safe one-click fixes (best-effort), then re-run doctor.")
                .clicked()
                {
                    match self.form.to_config() {
                        Ok(cfg) => {
                            let _ = self.cmd_tx.send(Cmd::DoctorFix(cfg));
                        }
                        Err(e) => {
                            self.toast = Some((format!("Cannot run doctor: {}", e), Instant::now()));
                        }
                    }
                }
            });
            if let Some(e) = self.form.to_config().err() {
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(format!("To start, fix: {e}"))
                        .color(ERR_RED)
                        .size(12.0),
                );
            }

            // Secondary actions — smaller, grouped together on their own line.
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let cert_op = self.shared.state.lock().unwrap().cert_op_in_progress;
                ui.add_enabled_ui(!cert_op, |ui| {
                    if ui
                        .small_button("Install CA")
                        .on_hover_text("Install or repair the local MITM CA trust.")
                        .clicked()
                    {
                        let _ = self.cmd_tx.send(Cmd::InstallCa);
                    }
                });
                ui.add_enabled_ui(!running && !cert_op, |ui| {
                    if ui
                        .small_button("Remove CA")
                        .on_hover_text(
                            "Remove the local MITM CA from OS/browser trust stores and delete ca/. Stop the proxy first.",
                        )
                        .clicked()
                    {
                        let _ = self.cmd_tx.send(Cmd::RemoveCa);
                    }
                });
                if ui.small_button("Check CA").clicked() {
                    let _ = self.cmd_tx.send(Cmd::CheckCaTrusted);
                }
                if ui.small_button("Check for updates")
                    .on_hover_text(
                        "Ask GitHub's Releases API for the latest tag and compare against this \
                         running version. When the proxy is running, the request is tunnelled \
                         through it — so GitHub sees an Apps Script IP instead of your ISP IP \
                         (different rate-limit bucket, and works even if GitHub is blocked on \
                         your network). No background polling — only fires when you click."
                    )
                    .clicked()
                {
                    let route = self.update_check_route();
                    let _ = self.cmd_tx.send(Cmd::CheckUpdate { route });
                }
            });

            // ── Transient status line ─────────────────────────────────────
            // One compact line at most. Everything auto-hides after 10s so
            // stale messages don't keep pushing the log panel off-screen.
            // Priority: update-check in flight > fresh test msg > fresh CA
            // result > update-check result. Old/expired entries are dropped.
            const TRANSIENT_TTL: Duration = Duration::from_secs(10);
            let (test_msg_fresh, ca_trusted_fresh, update_check_fresh, download_fresh) = {
                let s = self.shared.state.lock().unwrap();
                (
                    s.last_test_msg_at
                        .is_some_and(|t| t.elapsed() < TRANSIENT_TTL),
                    s.ca_trusted_at
                        .is_some_and(|t| t.elapsed() < TRANSIENT_TTL),
                    s.last_update_check_at
                        .is_some_and(|t| t.elapsed() < TRANSIENT_TTL),
                    s.last_download_at
                        .is_some_and(|t| t.elapsed() < TRANSIENT_TTL),
                )
            };

            let mut shown_any = false;
            let update_is_inflight = matches!(
                self.shared.state.lock().unwrap().last_update_check,
                Some(UpdateProbeState::InFlight)
            );
            if update_is_inflight {
                ui.small(
                    egui::RichText::new("Checking for updates…")
                        .color(egui::Color32::GRAY),
                );
                shown_any = true;
            } else if update_check_fresh {
                let done = self.shared.state.lock().unwrap().last_update_check.clone();
                if let Some(UpdateProbeState::Done(r)) = done {
                    use mhrv_jni::update_check::UpdateCheck;
                    let color = match &r {
                        UpdateCheck::UpToDate { .. } => OK_GREEN,
                        UpdateCheck::UpdateAvailable { .. } => {
                            egui::Color32::from_rgb(220, 170, 80)
                        }
                        _ => ERR_RED,
                    };
                    ui.horizontal(|ui| {
                        ui.small(egui::RichText::new(r.summary()).color(color));
                        if let UpdateCheck::UpdateAvailable {
                            release_url, asset, ..
                        } = &r
                        {
                            ui.hyperlink_to("open release", release_url);
                            if let Some(a) = asset {
                                let dl_in_flight = self.shared.state.lock().unwrap().download_in_progress;
                                if dl_in_flight {
                                    ui.small(
                                        egui::RichText::new("downloading…")
                                            .color(egui::Color32::GRAY),
                                    );
                                } else {
                                    let btn = egui::Button::new(
                                        egui::RichText::new(format!(
                                            "⤓ Download {} ({:.1} MB)",
                                            a.name,
                                            a.size_bytes as f64 / 1_048_576.0
                                        ))
                                        .color(egui::Color32::WHITE),
                                    )
                                    .fill(ACCENT)
                                    .rounding(8.0);
                                    if ui.add(btn).clicked() {
                                        let route = self.update_check_route();
                                        let _ = self.cmd_tx.send(Cmd::DownloadUpdate {
                                            route,
                                            url: a.download_url.clone(),
                                            name: a.name.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    });
                    shown_any = true;
                }
            } else if test_msg_fresh && !last_test_msg.is_empty() {
                let color = if last_test_msg.starts_with("Test passed") {
                    OK_GREEN
                } else {
                    ERR_RED
                };
                ui.small(egui::RichText::new(last_test_msg).color(color));
                shown_any = true;
            } else if download_fresh {
                let dl = self.shared.state.lock().unwrap().last_download.clone();
                match dl {
                    Some(Ok(path)) => {
                        ui.horizontal(|ui| {
                            ui.small(
                                egui::RichText::new(format!("Downloaded → {}", path.display()))
                                    .color(OK_GREEN),
                            );
                            if ui.small_button("show in folder").clicked() {
                                reveal_in_file_manager(&path);
                            }
                        });
                    }
                    Some(Err(msg)) => {
                        ui.small(
                            egui::RichText::new(format!("Download failed: {}", msg))
                                .color(ERR_RED),
                        );
                    }
                    None => {
                        ui.small(
                            egui::RichText::new("Downloading…")
                                .color(egui::Color32::GRAY),
                        );
                    }
                }
                shown_any = true;
            } else if ca_trusted_fresh {
                match ca_trusted {
                    Some(true) => {
                        ui.small(
                            egui::RichText::new("CA appears trusted on this machine.")
                                .color(OK_GREEN),
                        );
                    }
                    Some(false) => {
                        ui.small(
                            egui::RichText::new(
                                "CA is NOT trusted in the system store. Click Install CA.",
                            )
                            .color(ERR_RED),
                        );
                    }
                    None => {}
                }
                shown_any = true;
            }
            // Reserve a line of space even when empty so the log below doesn't
            // jump when a transient message appears / disappears.
            if !shown_any {
                ui.small(" ");
            }

            ui.add_space(4.0);

            // ── Recent log ────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Recent log").strong());
                ui.checkbox(&mut self.form.show_log, "Show panel")
                    .on_hover_text(
                        "Streams tracing output from the running proxy. Leave it on while testing; \
                         hide it to reclaim vertical space. Use Save to export a timestamped log for bug reports.",
                    );
                if ui.small_button("save…")
                    .on_hover_text(
                        "Write every line in the log panel to a timestamped file in the \
                         user-data dir. Useful for filing bug reports."
                    )
                    .clicked()
                {
                    let log = self.shared.state.lock().unwrap().log.clone();
                    let fname = format!(
                        "log-{}.txt",
                        time::OffsetDateTime::now_utc()
                            .format(&time::macros::format_description!(
                                "[year][month][day]-[hour][minute][second]"
                            ))
                            .unwrap_or_default(),
                    );
                    let path = data_dir::data_dir().join(&fname);
                    let body: String = log.iter().cloned().collect::<Vec<_>>().join("\n");
                    match std::fs::write(&path, body) {
                        Ok(_) => self.toast = Some((
                            format!("Log saved to {}", path.display()),
                            Instant::now(),
                        )),
                        Err(e) => self.toast = Some((
                            format!("Log save failed: {}", e),
                            Instant::now(),
                        )),
                    }
                }
                if ui.small_button("clear").clicked() {
                    self.shared.state.lock().unwrap().log.clear();
                }
            });
            if self.form.show_log {
                egui::Frame::none()
                    .fill(CARD_FILL.linear_multiply(0.92))
                    .stroke(egui::Stroke::new(1.0, CARD_STROKE))
                    .shadow(SURFACE_SHADOW)
                    .rounding(10.0)
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(220.0)
                            .min_scrolled_height(220.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                let log = self.shared.state.lock().unwrap().log.clone();
                                if log.is_empty() {
                                    ui.small(
                                        egui::RichText::new("(empty — run some traffic or click Test)")
                                            .color(egui::Color32::from_gray(120))
                                            .italics(),
                                    );
                                }
                                for line in log.iter() {
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(line).monospace().size(11.0),
                                        )
                                        .wrap(),
                                    );
                                }
                            });
                    });
            }

            }
            // Transient toast at the bottom. Config-load failures stick for
            // 30s instead of 5 because they explain why the form looks empty.
            if let Some((msg, t)) = &self.toast {
                let ttl = if msg.contains("failed to load") {
                    Duration::from_secs(30)
                } else {
                    Duration::from_secs(5)
                };
                if t.elapsed() < ttl {
                    ui.add_space(6.0);
                    let lower = msg.to_ascii_lowercase();
                    let (accent, dot) = if lower.contains("fail")
                        || lower.contains("cannot ")
                        || lower.contains("error")
                    {
                        (ERR_RED.linear_multiply(0.85), ERR_RED)
                    } else if lower.contains("saved")
                        || lower.contains("copied")
                        || lower.contains("finished")
                        || lower.contains("succeeded")
                        || lower.contains("passed")
                        || lower.contains("loaded:")
                        || lower.contains("downloaded →")
                    {
                        (OK_GREEN.linear_multiply(0.82), OK_GREEN)
                    } else {
                        (
                            ACCENT_WARM.linear_multiply(0.88),
                            ACCENT_WARM.linear_multiply(1.05),
                        )
                    };
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(38, 36, 33))
                        .stroke(egui::Stroke::new(1.0, accent.linear_multiply(0.65)))
                        .rounding(10.0)
                        .shadow(egui::Shadow {
                            offset: egui::vec2(0.0, 3.0),
                            blur: 14.0,
                            spread: 0.0,
                            color: egui::Color32::from_black_alpha(56),
                        })
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("●")
                                        .size(13.0)
                                        .color(dot),
                                );
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(msg.as_str())
                                        .size(13.0)
                                        .color(TEXT_MAIN),
                                );
                            });
                        });
                } else {
                    self.toast = None;
                }
            }
                }); // end ScrollArea
        });
    }
}

impl App {
    /// Pick the route for an update-check or download request: if the
    /// proxy is running and we have a local HTTP listen_port, tunnel
    /// through it (GitHub sees Apps Script's IP instead of the user's
    /// rate-limited ISP IP). Otherwise go direct.
    fn update_check_route(&self) -> mhrv_jni::update_check::Route {
        let running = self.shared.state.lock().unwrap().running;
        if running {
            if let Ok(port) = self.form.listen_port.trim().parse::<u16>() {
                let host = if self.form.listen_host.trim().is_empty() {
                    "127.0.0.1".to_string()
                } else {
                    self.form.listen_host.trim().to_string()
                };
                return mhrv_jni::update_check::Route::Proxy { host, port };
            }
        }
        mhrv_jni::update_check::Route::Direct
    }

    /// Floating editor window for the SNI rotation pool. Opens from the
    /// **SNI pool…** button in the main form. The list is live-editable
    /// (reorder / toggle / add / remove); changes only persist when the user
    /// hits **Save config** in the main window. Probe results are cached in
    /// `UiState::sni_probe` so they survive opening and closing the editor.
    fn show_sni_editor(&mut self, ctx: &egui::Context) {
        if !self.form.sni_editor_open {
            return;
        }
        let mut keep_open = true;
        egui::Window::new("SNI rotation pool")
            .open(&mut keep_open)
            .resizable(true)
            .default_size(egui::vec2(520.0, 420.0))
            .min_width(460.0)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(
                        "Which SNI names to rotate through when opening TLS connections \
                         to your Google IP. Some names may be locally blocked (Iran has \
                         dropped mail.google.com at times, for example); use the Test \
                         buttons to check — TLS handshake + HTTP HEAD against the \
                         configured google_ip, per name.",
                    )
                    .small(),
                );
                ui.add_space(4.0);

                // Action row.
                let google_ip = self.form.google_ip.trim().to_string();
                let probe_map = self.shared.state.lock().unwrap().sni_probe.clone();
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Test all").on_hover_text(
                        "Probe every SNI in the list against the configured google_ip in parallel."
                    ).clicked() {
                        let snis: Vec<String> = self
                            .form
                            .sni_pool
                            .iter()
                            .map(|r| r.name.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if !snis.is_empty() && !google_ip.is_empty() {
                            let _ = self.cmd_tx.send(Cmd::TestAllSni {
                                google_ip: google_ip.clone(),
                                snis,
                            });
                        }
                    }
                    if ui
                        .button("Keep working only")
                        .on_hover_text("Uncheck every SNI that didn't pass the last probe.")
                        .clicked()
                    {
                        for row in &mut self.form.sni_pool {
                            let ok = matches!(probe_map.get(&row.name), Some(SniProbeState::Ok(_)));
                            row.enabled = ok;
                        }
                    }
                    if ui.button("Enable all").clicked() {
                        for row in &mut self.form.sni_pool {
                            row.enabled = true;
                        }
                    }
                    if ui.button("Clear status").clicked() {
                        self.shared.state.lock().unwrap().sni_probe.clear();
                    }
                    if ui
                        .button("Reset to defaults")
                        .on_hover_text(
                            "Replace the list with the built-in Google SNI pool. Custom entries \
                         are dropped.",
                        )
                        .clicked()
                    {
                        self.form.sni_pool = DEFAULT_GOOGLE_SNI_POOL
                            .iter()
                            .map(|s| SniRow {
                                name: (*s).to_string(),
                                enabled: true,
                            })
                            .collect();
                        self.shared.state.lock().unwrap().sni_probe.clear();
                    }
                });
                ui.separator();

                // Main list — one horizontal row per SNI, explicit widths so
                // the domain text field gets the room it needs.
                let mut to_remove: Option<usize> = None;
                let mut test_name: Option<String> = None;
                const STATUS_W: f32 = 150.0;
                const NAME_W: f32 = 230.0;
                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .show(ui, |ui| {
                        for (i, row) in self.form.sni_pool.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut row.enabled, "");
                                ui.add(
                                    egui::TextEdit::singleline(&mut row.name)
                                        .desired_width(NAME_W)
                                        .font(egui::TextStyle::Monospace),
                                );
                                let status_txt = match probe_map.get(&row.name) {
                                    Some(SniProbeState::Ok(ms)) => {
                                        egui::RichText::new(format!("ok  {} ms", ms))
                                            .color(egui::Color32::from_rgb(80, 180, 100))
                                            .monospace()
                                    }
                                    Some(SniProbeState::Failed(e)) => {
                                        let short = if e.len() > 22 { &e[..22] } else { e };
                                        egui::RichText::new(format!("fail {}", short))
                                            .color(egui::Color32::from_rgb(220, 110, 110))
                                            .monospace()
                                    }
                                    Some(SniProbeState::InFlight) => {
                                        egui::RichText::new("testing…")
                                            .color(egui::Color32::GRAY)
                                            .monospace()
                                    }
                                    None => egui::RichText::new("untested")
                                        .color(egui::Color32::GRAY)
                                        .monospace(),
                                };
                                ui.add_sized(
                                    [STATUS_W, 18.0],
                                    egui::Label::new(status_txt).truncate(),
                                );
                                if ui.small_button("Test").clicked() {
                                    test_name = Some(row.name.clone());
                                }
                                if ui
                                    .small_button("remove")
                                    .on_hover_text("Remove this row")
                                    .clicked()
                                {
                                    to_remove = Some(i);
                                }
                            });
                        }
                    });

                if let Some(name) = test_name {
                    let name = name.trim().to_string();
                    if !name.is_empty() && !google_ip.is_empty() {
                        let _ = self.cmd_tx.send(Cmd::TestSni {
                            google_ip: google_ip.clone(),
                            sni: name,
                        });
                    }
                }
                if let Some(i) = to_remove {
                    self.form.sni_pool.remove(i);
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.form.sni_custom_input)
                            .hint_text("add a custom SNI (e.g. translate.google.com)")
                            .desired_width(280.0),
                    );
                    let add_clicked = ui.button("+ Add").clicked();
                    if add_clicked {
                        let new_name = self.form.sni_custom_input.trim().to_string();
                        if !new_name.is_empty()
                            && !self.form.sni_pool.iter().any(|r| r.name == new_name)
                        {
                            self.form.sni_pool.push(SniRow {
                                name: new_name.clone(),
                                enabled: true,
                            });
                            self.form.sni_custom_input.clear();
                            // Auto-probe the freshly added name so the user gets
                            // immediate feedback instead of a silent "untested"
                            // row. Needs a non-empty google_ip to have meaning.
                            if !google_ip.is_empty() {
                                let _ = self.cmd_tx.send(Cmd::TestSni {
                                    google_ip: google_ip.clone(),
                                    sni: new_name,
                                });
                            }
                        }
                    }
                });

                ui.add_space(6.0);
                ui.separator();
                ui.small(
                    "Changes take effect on the next Start of the proxy. \
                     Don't forget to press Save config in the main window to persist.",
                );
            });
        self.form.sni_editor_open = keep_open;
    }
}

// ---------- Background thread: owns the tokio runtime + proxy lifecycle ----------

fn background_thread(shared: Arc<Shared>, rx: Receiver<Cmd>) {
    let rt = Runtime::new().expect("failed to create tokio runtime");

    type ActiveProxy = (
        JoinHandle<()>,
        Arc<AsyncMutex<Option<Arc<DomainFronter>>>>,
        tokio::sync::oneshot::Sender<()>,
    );
    let mut active: Option<ActiveProxy> = None;

    loop {
        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(Cmd::PollStats) => {
                if let Some((_, fronter_slot, _)) = &active {
                    let slot = fronter_slot.clone();
                    let shared = shared.clone();
                    rt.spawn(async move {
                        let f = slot.lock().await;
                        if let Some(fronter) = f.as_ref() {
                            let s = fronter.snapshot_stats();
                            let per_site = fronter.snapshot_per_site();
                            let mut st = shared.state.lock().unwrap();
                            st.last_stats = Some(s);
                            st.last_per_site = per_site;

                            // Dashboard: keep a small degradation history buffer.
                            let reason = String::from_utf8_lossy(&s.degrade_reason)
                                .trim_matches(char::from(0))
                                .trim()
                                .to_string();
                            st.degrade_history
                                .push_back((Instant::now(), s.degrade_level, reason));
                            while st.degrade_history.len() > 120 {
                                st.degrade_history.pop_front();
                            }
                        }
                    });
                }
            }
            // In background_thread function, modify the Cmd::Start handler:
            Ok(Cmd::Start(cfg)) => {
                if active.is_some() {
                    push_log(&shared, "[ui] already running");
                    continue;
                }
                push_log(&shared, "[ui] starting proxy...");
                let shared2 = shared.clone();
                let fronter_slot: Arc<AsyncMutex<Option<Arc<DomainFronter>>>> =
                    Arc::new(AsyncMutex::new(None));
                let fronter_slot2 = fronter_slot.clone();

                let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

                let handle = rt.spawn(async move {
                    let base = data_dir::data_dir();
                    let mitm = match MitmCertManager::new_in(&base) {
                        Ok(m) => m,
                        Err(e) => {
                            push_log(&shared2, &format!("[ui] MITM init failed: {}", e));
                            shared2.state.lock().unwrap().running = false;
                            return;
                        }
                    };
                    let mitm = Arc::new(AsyncMutex::new(mitm));
                    let server = match ProxyServer::new(&cfg, mitm) {
                        Ok(s) => s,
                        Err(e) => {
                            push_log(&shared2, &format!("[ui] proxy build failed: {}", e));
                            shared2.state.lock().unwrap().running = false;
                            return;
                        }
                    };
                    // `fronter()` is `None` in direct mode; the status panel's
                    // relay stats simply show no data in that case.
                    *fronter_slot2.lock().await = server.fronter();
                    {
                        let mut s = shared2.state.lock().unwrap();
                        s.running = true;
                        s.started_at = Some(Instant::now());
                    }
                    let socks_log = cfg
                        .socks5_port
                        .map(|p| format!("{}:{}", cfg.listen_host, p))
                        .unwrap_or_else(|| "disabled".into());
                    push_log(
                        &shared2,
                        &format!(
                            "[ui] listening HTTP {}:{} SOCKS5 {}",
                            cfg.listen_host, cfg.listen_port, socks_log
                        ),
                    );

                    if let Err(e) = server.run(shutdown_rx).await {
                        push_log(&shared2, &format!("[ui] proxy error: {}", e));
                    }

                    shared2.state.lock().unwrap().running = false;
                    shared2.state.lock().unwrap().started_at = None;
                    push_log(&shared2, "[ui] proxy stopped");
                });

                active = Some((handle, fronter_slot, shutdown_tx));
            }

            Ok(Cmd::Stop) => {
                if let Some((mut handle, _, shutdown_tx)) = active.take() {
                    push_log(&shared, "[ui] stop requested");
                    let _ = shutdown_tx.send(());

                    // Give the proxy 2 seconds to shut down gracefully
                    rt.block_on(async {
                        tokio::select! {
                            _ = &mut handle => {
                                push_log(&shared, "[ui] proxy stopped gracefully");
                            }
                            _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
                                handle.abort();
                                let _ = handle.await;
                                push_log(&shared, "[ui] shutdown timeout, forced abort");
                            }
                        }
                    });

                    shared.state.lock().unwrap().running = false;
                    shared.state.lock().unwrap().started_at = None;
                }
            }

            Ok(Cmd::Test(cfg)) => {
                let shared2 = shared.clone();
                let mode_explainer = match cfg.mode_kind().ok() {
                    Some(mhrv_jni::config::Mode::Full) => Some(
                        "Test Relay is wired only for apps_script mode. In full mode the data plane is tunnel-node. To verify it end-to-end, start the proxy and load https://whatismyipaddress.com in your browser through 127.0.0.1:8085; the IP shown should be your tunnel-node VPS IP.",
                    ),
                    Some(mhrv_jni::config::Mode::Direct) => Some(
                        "Test Relay is wired only for apps_script mode. In direct mode there is no Apps Script relay; requests go through the SNI-rewrite tunnel directly to the selected edge. Verify by loading https://www.google.com through the proxy.",
                    ),
                    _ => None,
                };
                if let Some(msg) = mode_explainer {
                    {
                        let mut st = shared.state.lock().unwrap();
                        st.last_test_ok = None;
                        st.last_test_msg = msg.into();
                        st.last_test_msg_at = Some(Instant::now());
                    }
                    push_log(&shared, &format!("[ui] test skipped: {}", msg));
                    continue;
                }
                push_log(&shared, "[ui] running test...");
                rt.spawn(async move {
                    let ok = test_cmd::run(&cfg).await;
                    {
                        let mut st = shared2.state.lock().unwrap();
                        st.last_test_ok = Some(ok);
                        st.last_test_msg = if ok {
                            "Test passed — relay is working.".into()
                        } else {
                            "Test failed — see Recent log below for details.".into()
                        };
                        st.last_test_msg_at = Some(Instant::now());
                    }
                    push_log(
                        &shared2,
                        &format!("[ui] test result: {}", if ok { "pass" } else { "fail" }),
                    );
                    // Also run ip scan on demand (cheap).
                    let _ = scan_ips::run(&cfg).await;
                });
            }
            Ok(Cmd::Doctor(cfg)) => {
                let shared2 = shared.clone();
                push_log(&shared, "[ui] running doctor...");
                rt.spawn(async move {
                    let report = doctor::run(&cfg).await;
                    for it in &report.items {
                        push_log(
                            &shared2,
                            &format!(
                                "[doctor] [{}] {} — {}",
                                doctor_level_label(&it.level),
                                it.id,
                                it.title
                            ),
                        );
                        if !it.detail.trim().is_empty() {
                            push_log(&shared2, &format!("[doctor] {}", it.detail));
                        }
                        if let Some(fix) = &it.fix {
                            push_log(&shared2, &format!("[doctor] fix: {}", fix));
                        }
                    }
                    push_log(
                        &shared2,
                        &format!(
                            "[doctor] done: {}",
                            if report.ok() { "OK" } else { "needs attention" }
                        ),
                    );
                    {
                        let mut st = shared2.state.lock().unwrap();
                        st.last_doctor_report = Some(report.clone());
                        st.last_doctor_at = Some(Instant::now());
                    }
                });
            }
            Ok(Cmd::DoctorFix(cfg)) => {
                let shared2 = shared.clone();
                push_log(&shared, "[ui] running doctor + fixes...");
                rt.spawn(async move {
                    let (before, fixes, after) = doctor::run_with_fixes(&cfg).await;
                    push_log(&shared2, "[doctor] BEFORE:");
                    for it in &before.items {
                        push_log(
                            &shared2,
                            &format!(
                                "[doctor] [{}] {} — {}",
                                doctor_level_label(&it.level),
                                it.id,
                                it.title
                            ),
                        );
                        if !it.detail.trim().is_empty() {
                            push_log(&shared2, &format!("[doctor] {}", it.detail));
                        }
                    }
                    if fixes.is_empty() {
                        push_log(&shared2, "[doctor] fixes: (none available)");
                    } else {
                        for f in fixes {
                            push_log(
                                &shared2,
                                &format!(
                                    "[doctor] fix {}: {}",
                                    if f.ok { "OK" } else { "FAIL" },
                                    f.detail
                                ),
                            );
                        }
                    }
                    push_log(&shared2, "[doctor] AFTER:");
                    for it in &after.items {
                        push_log(
                            &shared2,
                            &format!(
                                "[doctor] [{}] {} — {}",
                                doctor_level_label(&it.level),
                                it.id,
                                it.title
                            ),
                        );
                        if !it.detail.trim().is_empty() {
                            push_log(&shared2, &format!("[doctor] {}", it.detail));
                        }
                        if let Some(fix) = &it.fix {
                            push_log(&shared2, &format!("[doctor] fix: {}", fix));
                        }
                    }
                    push_log(&shared2, "[doctor] done");
                    {
                        let mut st = shared2.state.lock().unwrap();
                        st.last_doctor_report = Some(after.clone());
                        st.last_doctor_at = Some(Instant::now());
                    }
                });
            }
            Ok(Cmd::InstallCa) => {
                let shared2 = shared.clone();
                {
                    let mut st = shared2.state.lock().unwrap();
                    if st.cert_op_in_progress {
                        drop(st);
                        push_log(&shared2, "[ui] CA operation already in progress");
                        continue;
                    }
                    st.cert_op_in_progress = true;
                }
                std::thread::spawn(move || {
                    push_log(&shared2, "[ui] installing CA...");
                    let base = data_dir::data_dir();
                    if let Err(e) = MitmCertManager::new_in(&base) {
                        push_log(&shared2, &format!("[ui] CA init failed: {}", e));
                        shared2.state.lock().unwrap().cert_op_in_progress = false;
                        return;
                    }
                    let ca = base.join(CA_CERT_FILE);
                    match install_ca(&ca) {
                        Ok(()) => {
                            push_log(&shared2, "[ui] CA install ok");
                            let mut st = shared2.state.lock().unwrap();
                            st.ca_trusted = Some(true);
                            st.ca_trusted_at = Some(Instant::now());
                        }
                        Err(e) => {
                            push_log(&shared2, &format!("[ui] CA install failed: {}", e));
                            push_log(&shared2, "[ui] hint: run the terminal binary with sudo/admin: mhrv-f --install-cert");
                        }
                    }
                    shared2.state.lock().unwrap().cert_op_in_progress = false;
                });
            }
            Ok(Cmd::RemoveCa) => {
                if active.is_some() || shared.state.lock().unwrap().running {
                    push_log(&shared, "[ui] stop the proxy before removing the CA");
                    continue;
                }
                let shared2 = shared.clone();
                {
                    let mut st = shared2.state.lock().unwrap();
                    if st.cert_op_in_progress {
                        drop(st);
                        push_log(&shared2, "[ui] CA operation already in progress");
                        continue;
                    }
                    st.cert_op_in_progress = true;
                }
                std::thread::spawn(move || {
                    push_log(&shared2, "[ui] removing CA...");
                    let base = data_dir::data_dir();
                    match remove_ca(&base) {
                        Ok(outcome) => {
                            push_log(&shared2, &format!("[ui] {}", outcome.summary()));
                            let mut st = shared2.state.lock().unwrap();
                            st.ca_trusted = Some(false);
                            st.ca_trusted_at = Some(Instant::now());
                        }
                        Err(e) => {
                            push_log(&shared2, &format!("[ui] CA remove failed: {}", e));
                            push_log(&shared2, "[ui] hint: rerun elevated: mhrv-f --remove-cert");
                        }
                    }
                    shared2.state.lock().unwrap().cert_op_in_progress = false;
                });
            }
            Ok(Cmd::TestSni { google_ip, sni }) => {
                let shared2 = shared.clone();
                {
                    let mut st = shared2.state.lock().unwrap();
                    st.sni_probe.insert(sni.clone(), SniProbeState::InFlight);
                }
                rt.spawn(async move {
                    let result = scan_sni::probe_one(&google_ip, &sni).await;
                    let state = match result.latency_ms {
                        Some(ms) => SniProbeState::Ok(ms),
                        None => {
                            SniProbeState::Failed(result.error.unwrap_or_else(|| "failed".into()))
                        }
                    };
                    shared2.state.lock().unwrap().sni_probe.insert(sni, state);
                });
            }
            Ok(Cmd::TestAllSni { google_ip, snis }) => {
                let shared2 = shared.clone();
                {
                    let mut st = shared2.state.lock().unwrap();
                    for s in &snis {
                        st.sni_probe.insert(s.clone(), SniProbeState::InFlight);
                    }
                }
                rt.spawn(async move {
                    let results = scan_sni::probe_all(&google_ip, snis).await;
                    let mut st = shared2.state.lock().unwrap();
                    for (sni, r) in results {
                        let state = match r.latency_ms {
                            Some(ms) => SniProbeState::Ok(ms),
                            None => {
                                SniProbeState::Failed(r.error.unwrap_or_else(|| "failed".into()))
                            }
                        };
                        st.sni_probe.insert(sni, state);
                    }
                });
            }
            Ok(Cmd::CheckCaTrusted) => {
                let shared2 = shared.clone();
                std::thread::spawn(move || {
                    let base = data_dir::data_dir();
                    let ca = base.join(CA_CERT_FILE);
                    let trusted = mhrv_jni::cert_installer::is_ca_trusted(&ca);
                    let mut st = shared2.state.lock().unwrap();
                    st.ca_trusted = Some(trusted);
                    st.ca_trusted_at = Some(Instant::now());
                });
            }
            Ok(Cmd::CheckUpdate { route }) => {
                let shared2 = shared.clone();
                {
                    let mut st = shared2.state.lock().unwrap();
                    st.last_update_check = Some(UpdateProbeState::InFlight);
                    st.last_update_check_at = Some(Instant::now());
                }
                rt.spawn(async move {
                    let result = mhrv_jni::update_check::check(route).await;
                    push_log(
                        &shared2,
                        &format!("[ui] update check: {}", result.summary()),
                    );
                    {
                        let mut st = shared2.state.lock().unwrap();
                        st.last_update_check = Some(UpdateProbeState::Done(result));
                        st.last_update_check_at = Some(Instant::now());
                    }
                });
            }
            Ok(Cmd::DownloadUpdate { route, url, name }) => {
                let shared2 = shared.clone();
                {
                    let mut st = shared2.state.lock().unwrap();
                    st.download_in_progress = true;
                    st.last_download = None;
                }
                push_log(&shared, &format!("[ui] downloading {}", name));
                rt.spawn(async move {
                    let dir = downloads_dir();
                    let out = dir.join(&name);
                    let result = mhrv_jni::update_check::download_asset(route, &url, &out).await;
                    let mut st = shared2.state.lock().unwrap();
                    st.download_in_progress = false;
                    st.last_download_at = Some(Instant::now());
                    match result {
                        Ok(bytes) => {
                            push_log(
                                &shared2,
                                &format!(
                                    "[ui] download ok: {} ({} bytes) -> {}",
                                    name,
                                    bytes,
                                    out.display()
                                ),
                            );
                            st.last_download = Some(Ok(out));
                        }
                        Err(e) => {
                            push_log(&shared2, &format!("[ui] download failed: {}", e));
                            st.last_download = Some(Err(e));
                        }
                    }
                });
            }
            Err(_) => {}
        }

        // Clean up finished task.
        if let Some((handle, _, _)) = &active {
            if handle.is_finished() {
                active = None;
                shared.state.lock().unwrap().running = false;
                shared.state.lock().unwrap().started_at = None;
            }
        }
    }
}

/// Install a tracing subscriber that mirrors every log event into the UI's
/// Recent log panel.
///
/// Respects `RUST_LOG` if set. Otherwise defaults to `info` — which is what
/// users mean when they pick a non-default log level in the form. (trace /
/// debug flip too much noise for a local GUI, so the combo-box changes level
/// live via the `reload` handle that `with_env_filter` gives us but we keep
/// the default boot-time level at info so first-run behavior is sensible.)
fn install_ui_tracing(shared: Arc<Shared>) {
    use tracing_subscriber::fmt::MakeWriter;
    use tracing_subscriber::EnvFilter;

    /// A MakeWriter that pushes each line into the shared log panel.
    struct UiLogWriter {
        shared: Arc<Shared>,
    }

    struct UiWriterInst {
        shared: Arc<Shared>,
        buf: Vec<u8>,
    }

    impl<'a> MakeWriter<'a> for UiLogWriter {
        type Writer = UiWriterInst;
        fn make_writer(&'a self) -> Self::Writer {
            UiWriterInst {
                shared: self.shared.clone(),
                buf: Vec::with_capacity(128),
            }
        }
    }

    impl std::io::Write for UiWriterInst {
        fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
            self.buf.extend_from_slice(data);
            Ok(data.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            if self.buf.is_empty() {
                return Ok(());
            }
            let text = String::from_utf8_lossy(&self.buf).trim_end().to_string();
            self.buf.clear();
            // Split on newlines in case multiple events got buffered.
            for line in text.lines() {
                if line.is_empty() {
                    continue;
                }
                let mut s = self.shared.state.lock().unwrap();
                s.log.push_back(line.to_string());
                while s.log.len() > LOG_MAX {
                    s.log.pop_front();
                }
            }
            Ok(())
        }
    }

    impl Drop for UiWriterInst {
        fn drop(&mut self) {
            let _ = std::io::Write::flush(self);
        }
    }

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,hyper=warn"));

    let writer = UiLogWriter { shared };

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(false)
        .with_writer(writer)
        .try_init();
}

fn push_log(shared: &Shared, msg: &str) {
    let line = format!(
        "{}  {}",
        time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_default(),
        msg
    );
    let mut s = shared.state.lock().unwrap();
    s.log.push_back(line);
    while s.log.len() > LOG_MAX {
        s.log.pop_front();
    }
}
