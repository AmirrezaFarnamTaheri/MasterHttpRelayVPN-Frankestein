use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use rand::{distributions::Alphanumeric, Rng};
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

use mhrv_jni::branding::PRODUCT_NAME;
use mhrv_jni::cert_installer::{install_ca, is_ca_trusted, reconcile_sudo_environment, remove_ca};
use mhrv_jni::config::Config;
use mhrv_jni::mitm::{MitmCertManager, CA_CERT_FILE};
use mhrv_jni::proxy_server::ProxyServer;
use mhrv_jni::{scan_ips, scan_sni, test_cmd};

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Args {
    config_path: Option<PathBuf>,
    install_cert: bool,
    remove_cert: bool,
    no_cert_check: bool,
    tunnel_node_url: Option<String>,
    command: Command,
}

enum Command {
    Serve,
    Test,
    Doctor,
    DoctorFix,
    SupportBundle,
    ScanIps,
    TestSni,
    ScanSni,
    RollbackConfig,
    InitConfig,
}

fn print_help() {
    println!(
        "mhrv-f {} — {} (relay client)

USAGE:
    mhrv-f [OPTIONS]                  Start the proxy server (default)
    mhrv-f test [OPTIONS]             Probe apps_script/vercel_edge relay end-to-end
    mhrv-f doctor [OPTIONS]           Guided diagnostics (first-run fix assistant)
    mhrv-f doctor-fix [OPTIONS]       Doctor + apply one-click fixes (best-effort)
    mhrv-f init-config [OPTIONS]      Write config.json interactively
    mhrv-f support-bundle [OPTIONS]   Export an anonymized diagnostics bundle
    mhrv-f rollback-config            Restore last-known-good config (best-effort)
    mhrv-f scan-ips [OPTIONS]         Scan Google frontend IPs for reachability + latency
    mhrv-f scan-sni         Scan Google SNI name using Google frontend IPs found in 'scan-ips' command
    mhrv-f test-sni [OPTIONS]         Probe each SNI name in the rotation pool against google_ip

OPTIONS:
    -c, --config PATH    Path to config.json (default: ./config.json)
    --install-cert       Install the MITM CA certificate and exit
    --remove-cert        Remove the MITM CA from trust stores and delete local ca/
    --no-cert-check      Skip the auto-install-if-untrusted check on startup
    --tunnel-node-url URL
                         Full-mode Doctor: probe URL/health/details live
    -h, --help           Show this message
    -V, --version        Show version

ENV:
    RUST_LOG             Override tracing filter (e.g. info, debug)
    NO_COLOR             Disable ANSI colors in CLI logs
    FORCE_COLOR          Force ANSI colors in CLI logs
",
        VERSION,
        PRODUCT_NAME
    );
}

fn parse_args() -> Result<Args, String> {
    let mut config_path: Option<PathBuf> = None;
    let mut install_cert = false;
    let mut remove_cert = false;
    let mut no_cert_check = false;
    let mut tunnel_node_url: Option<String> = None;
    let mut command = Command::Serve;

    let mut raw: Vec<String> = std::env::args().skip(1).collect();
    if let Some(first) = raw.first() {
        match first.as_str() {
            "test" => {
                command = Command::Test;
                raw.remove(0);
            }
            "doctor" => {
                command = Command::Doctor;
                raw.remove(0);
            }
            "doctor-fix" => {
                command = Command::DoctorFix;
                raw.remove(0);
            }
            "init-config" => {
                command = Command::InitConfig;
                raw.remove(0);
            }
            "support-bundle" => {
                command = Command::SupportBundle;
                raw.remove(0);
            }
            "rollback-config" => {
                command = Command::RollbackConfig;
                raw.remove(0);
            }
            "scan-ips" => {
                command = Command::ScanIps;
                raw.remove(0);
            }
            "scan-sni" => {
                command = Command::ScanSni;
                raw.remove(0);
            }
            "test-sni" => {
                command = Command::TestSni;
                raw.remove(0);
            }
            _ => {}
        }
    }

    let mut it = raw.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("mhrv-f {}", VERSION);
                std::process::exit(0);
            }
            "-c" | "--config" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--config needs a path".to_string())?;
                config_path = Some(PathBuf::from(v));
            }
            "--install-cert" => install_cert = true,
            "--remove-cert" => remove_cert = true,
            "--no-cert-check" => no_cert_check = true,
            "--tunnel-node-url" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--tunnel-node-url needs a URL".to_string())?;
                tunnel_node_url = Some(v);
            }
            other => return Err(format!("unknown argument: {}", other)),
        }
    }
    if install_cert && remove_cert {
        return Err("--install-cert and --remove-cert cannot be used together".into());
    }
    Ok(Args {
        config_path,
        install_cert,
        remove_cert,
        no_cert_check,
        tunnel_node_url,
        command,
    })
}

fn init_logging(level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(ansi_logs_enabled())
        .try_init();
}

fn ansi_logs_enabled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if std::env::var_os("FORCE_COLOR").is_some() {
        return true;
    }
    io::stderr().is_terminal()
}

fn random_auth_key() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

fn prompt_line(question: &str, default: Option<&str>) -> Result<String, String> {
    let suffix = default.map(|d| format!(" [{d}]")).unwrap_or_default();
    loop {
        print!("? {question}{suffix}: ");
        io::stdout()
            .flush()
            .map_err(|e| format!("failed to flush stdout: {e}"))?;
        let mut raw = String::new();
        io::stdin()
            .read_line(&mut raw)
            .map_err(|e| format!("failed to read stdin: {e}"))?;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            if let Some(d) = default {
                return Ok(d.to_string());
            }
            println!("  value required");
            continue;
        }
        return Ok(trimmed.to_string());
    }
}

fn prompt_yes_no(question: &str, default: bool) -> Result<bool, String> {
    let hint = if default { "Y/n" } else { "y/N" };
    loop {
        print!("? {question} [{hint}]: ");
        io::stdout()
            .flush()
            .map_err(|e| format!("failed to flush stdout: {e}"))?;
        let mut raw = String::new();
        io::stdin()
            .read_line(&mut raw)
            .map_err(|e| format!("failed to read stdin: {e}"))?;
        let answer = raw.trim().to_ascii_lowercase();
        if answer.is_empty() {
            return Ok(default);
        }
        match answer.as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("  answer y or n"),
        }
    }
}

fn prompt_u16(question: &str, default: u16) -> Result<u16, String> {
    loop {
        let answer = prompt_line(question, Some(&default.to_string()))?;
        match answer.parse::<u16>() {
            Ok(v) => return Ok(v),
            Err(_) => println!("  enter a valid TCP port"),
        }
    }
}

fn first_account_group_mut(
    value: &mut serde_json::Value,
) -> Result<&mut serde_json::Map<String, serde_json::Value>, String> {
    value
        .get_mut("account_groups")
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|groups| groups.first_mut())
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| "config template is missing account_groups[0]".to_string())
}

fn run_init_config(config_arg: Option<&Path>) -> ExitCode {
    if !io::stdin().is_terminal() {
        eprintln!("init-config is interactive; run it from a terminal.");
        return ExitCode::FAILURE;
    }

    let path = mhrv_jni::data_dir::resolve_config_path(config_arg);
    if path.exists() {
        match prompt_yes_no(
            &format!("{} already exists. Overwrite?", path.display()),
            false,
        ) {
            Ok(true) => {}
            Ok(false) => {
                println!("Nothing changed.");
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        }
    }

    let mut value: serde_json::Value =
        match serde_json::from_str(include_str!("../config.example.json")) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to parse bundled config template: {e}");
                return ExitCode::FAILURE;
            }
        };

    println!("MasterHttpRelayVPN - setup");
    println!("Paste the Apps Script deployment ID(s) after deploying assets/apps_script/Code.gs.");

    let key_default = random_auth_key();
    let auth_key = match prompt_line("auth_key", Some(&key_default)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let ids_raw = match prompt_line("Deployment ID(s), comma-separated", None) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let script_ids: Vec<serde_json::Value> = ids_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| serde_json::Value::String(s.to_string()))
        .collect();
    if script_ids.is_empty() {
        eprintln!("at least one deployment ID is required");
        return ExitCode::FAILURE;
    }

    let lan = match prompt_yes_no("Enable LAN sharing?", false) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let listen_host_default = if lan { "0.0.0.0" } else { "127.0.0.1" };
    let listen_host = match prompt_line("Listen host", Some(listen_host_default)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let listen_port = match prompt_u16("HTTP proxy port", 8085) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let enable_socks = match prompt_yes_no("Enable SOCKS5 proxy?", true) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    value["mode"] = serde_json::Value::String("apps_script".into());
    value["listen_host"] = serde_json::Value::String(listen_host);
    value["listen_port"] = serde_json::Value::Number(u64::from(listen_port).into());
    if enable_socks {
        let socks5_port = match prompt_u16("SOCKS5 port", 8086) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        };
        value["socks5_port"] = serde_json::Value::Number(u64::from(socks5_port).into());
    } else {
        value["socks5_port"] = serde_json::Value::Null;
    }

    match first_account_group_mut(&mut value) {
        Ok(group) => {
            group.insert("auth_key".into(), serde_json::Value::String(auth_key));
            group.insert("script_ids".into(), serde_json::Value::Array(script_ids));
        }
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    }

    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("failed to create {}: {e}", parent.display());
            return ExitCode::FAILURE;
        }
    }
    if path.exists() {
        let backup = path.with_extension("json.bak");
        if let Err(e) = std::fs::copy(&path, &backup) {
            eprintln!(
                "failed to backup existing config to {}: {e}",
                backup.display()
            );
            return ExitCode::FAILURE;
        }
        println!("Backed up existing config to {}", backup.display());
    }

    let json = match serde_json::to_string_pretty(&value) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to render config: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = std::fs::write(&path, format!("{json}\n")) {
        eprintln!("failed to write {}: {e}", path.display());
        return ExitCode::FAILURE;
    }
    println!("Wrote {}", path.display());
    println!(
        "Remember to set the same AUTH_KEY inside assets/apps_script/Code.gs before deploying."
    );
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    // Windows in particular can run into stack overflows on the main thread
    // under some async + TLS IO paths. Run the async entrypoint on a thread
    // with a larger stack to make behavior consistent across environments.
    let th = std::thread::Builder::new()
        .name("mhrv-f-main".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(async_main())
        });
    match th {
        Ok(h) => h.join().unwrap_or(ExitCode::FAILURE),
        Err(_) => ExitCode::FAILURE,
    }
}

async fn async_main() -> ExitCode {
    // Install default rustls crypto provider (ring).
    let _ = rustls::crypto::ring::default_provider().install_default();
    reconcile_sudo_environment();

    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            print_help();
            return ExitCode::from(2);
        }
    };

    // --install-cert can run without a valid config — only needs the CA file.
    if args.install_cert {
        init_logging("info");
        let base = mhrv_jni::data_dir::data_dir();
        if let Err(e) = MitmCertManager::new_in(&base) {
            eprintln!("failed to initialize CA: {}", e);
            return ExitCode::FAILURE;
        }
        let ca_path = base.join(CA_CERT_FILE);
        match install_ca(&ca_path) {
            Ok(()) => {
                tracing::info!("CA installed. You may need to restart your browser.");
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("install failed: {}", e);
                return ExitCode::FAILURE;
            }
        }
    }

    if args.remove_cert {
        init_logging("info");
        let base = mhrv_jni::data_dir::data_dir();
        match remove_ca(&base) {
            Ok(outcome) => {
                tracing::info!("{}", outcome.summary());
                tracing::info!("A fresh CA will be generated on next non-full start.");
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("remove failed: {}", e);
                return ExitCode::FAILURE;
            }
        }
    }

    if matches!(args.command, Command::InitConfig) {
        return run_init_config(args.config_path.as_deref());
    }

    let config_path = mhrv_jni::data_dir::resolve_config_path(args.config_path.as_deref());
    let config = match Config::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!(
                "No valid config found. Copy config.example.json to either:\n  {}\nor run with --config <path>.",
                config_path.display()
            );
            eprintln!("For an interactive setup, run: mhrv-f init-config");
            return ExitCode::FAILURE;
        }
    };

    init_logging(&config.log_level);

    // Non-fatal safety warnings (shown once on startup).
    for w in config.unsafe_warnings() {
        tracing::warn!("config warning: {}", w);
    }

    // Bump RLIMIT_NOFILE now that tracing is live — minimal Linux images
    // often ship a default so low that we run out
    // of fds under normal proxy load. This logs the before/after values
    // at info level so field reports tell us whether the kernel cap is
    // the real culprit.
    mhrv_jni::rlimit::raise_nofile_limit_best_effort();

    match args.command {
        Command::Test => {
            let ok = test_cmd::run(&config).await;
            return if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
        Command::Doctor => {
            let doctor_options = mhrv_jni::doctor::DoctorOptions {
                tunnel_node_url: args.tunnel_node_url.clone(),
            };
            let report = mhrv_jni::doctor::run_with_options(&config, &doctor_options).await;
            for it in &report.items {
                let level = match it.level {
                    mhrv_jni::doctor::DoctorLevel::Ok => "OK",
                    mhrv_jni::doctor::DoctorLevel::Warn => "WARN",
                    mhrv_jni::doctor::DoctorLevel::Fail => "FAIL",
                };
                println!("[{level}] {} — {}", it.id, it.title);
                if !it.detail.trim().is_empty() {
                    println!("{}", it.detail);
                }
                if let Some(fix) = &it.fix {
                    println!("Fix: {}", fix);
                }
                println!();
            }
            return if report.ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
        Command::DoctorFix => {
            let doctor_options = mhrv_jni::doctor::DoctorOptions {
                tunnel_node_url: args.tunnel_node_url.clone(),
            };
            let (before, fixes, after) =
                mhrv_jni::doctor::run_with_fixes_and_options(&config, &doctor_options).await;
            println!("== Doctor (before) ==");
            for it in &before.items {
                let level = match it.level {
                    mhrv_jni::doctor::DoctorLevel::Ok => "OK",
                    mhrv_jni::doctor::DoctorLevel::Warn => "WARN",
                    mhrv_jni::doctor::DoctorLevel::Fail => "FAIL",
                };
                println!("[{level}] {} — {}", it.id, it.title);
                if !it.detail.trim().is_empty() {
                    println!("{}", it.detail);
                }
                println!();
            }
            println!("== One-click fixes ==");
            if fixes.is_empty() {
                println!("(no automatic fixes available for this config)");
            } else {
                for f in fixes {
                    println!(
                        "[{}] {} — {}",
                        if f.ok { "OK" } else { "FAIL" },
                        f.id,
                        f.detail
                    );
                }
            }
            println!();
            println!("== Doctor (after) ==");
            for it in &after.items {
                let level = match it.level {
                    mhrv_jni::doctor::DoctorLevel::Ok => "OK",
                    mhrv_jni::doctor::DoctorLevel::Warn => "WARN",
                    mhrv_jni::doctor::DoctorLevel::Fail => "FAIL",
                };
                println!("[{level}] {} — {}", it.id, it.title);
                if !it.detail.trim().is_empty() {
                    println!("{}", it.detail);
                }
                if let Some(fix) = &it.fix {
                    println!("Fix: {}", fix);
                }
                println!();
            }
            return if after.ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
        Command::InitConfig => unreachable!("handled before config load"),
        Command::SupportBundle => {
            match mhrv_jni::support_bundle::export_support_bundle(&config).await {
                Ok(dir) => {
                    println!("Support bundle exported to: {}", dir.display());
                    return ExitCode::SUCCESS;
                }
                Err(e) => {
                    eprintln!("Support bundle export failed: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        }
        Command::RollbackConfig => {
            init_logging("info");
            let path = mhrv_jni::data_dir::config_path();
            match mhrv_jni::profiles::load_profile("last_known_good") {
                Ok(cfg) => {
                    let json = serde_json::to_string_pretty(&cfg).unwrap_or_else(|_| "{}".into());
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    match std::fs::write(&path, json) {
                        Ok(()) => {
                            println!("Restored last-known-good config to {}", path.display());
                            return ExitCode::SUCCESS;
                        }
                        Err(e) => {
                            eprintln!("Rollback write failed: {}", e);
                            return ExitCode::FAILURE;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("No last-known-good snapshot found: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        }
        Command::ScanIps => {
            let ok = scan_ips::run(&config).await;
            return if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
        Command::ScanSni => {
            let ok = scan_sni::discover_snis_from_google_ips(&config).await;
            return if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }

        Command::TestSni => {
            let ok = scan_sni::run(&config).await;
            return if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
        Command::Serve => {}
    }

    let mode = match config.mode_kind() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("config: {}", e);
            return ExitCode::FAILURE;
        }
    };
    tracing::warn!("mhrv-f {} starting (mode: {})", VERSION, mode.as_str());
    tracing::info!(
        "HTTP proxy   : {}:{}",
        config.listen_host,
        config.listen_port
    );
    if let Some(socks5_port) = config.socks5_port {
        tracing::info!("SOCKS5 proxy : {}:{}", config.listen_host, socks5_port);
    } else {
        tracing::info!("SOCKS5 proxy : disabled");
    }
    match mode {
        mhrv_jni::config::Mode::AppsScript => {
            tracing::info!(
                "Apps Script relay: SNI={} -> script.google.com (via {})",
                config.front_domain,
                config.google_ip
            );
            let groups = config.account_groups_resolved();
            let total_ids: usize = groups
                .iter()
                .map(|g| g.script_ids.clone().into_vec().len())
                .sum();
            tracing::info!("Accounts: {}  Deployments: {}", groups.len(), total_ids);
        }
        mhrv_jni::config::Mode::VercelEdge => {
            tracing::info!(
                "Serverless JSON relay: {}{}",
                config.vercel.base_url.trim_end_matches('/'),
                config.vercel.relay_path
            );
            tracing::warn!(
                "Serverless JSON mode still uses local MITM for HTTPS. Install and trust the local CA on this device."
            );
        }
        mhrv_jni::config::Mode::Direct => {
            tracing::warn!(
                "direct mode: SNI-rewrite tunnel only (Google edge {} + any \
                 configured fronting_groups). Open https://script.google.com \
                 in your browser (proxy set to {}:{}), deploy Code.gs, then \
                 switch to apps_script mode for full DPI bypass.",
                config.google_ip,
                config.listen_host,
                config.listen_port
            );
        }
        mhrv_jni::config::Mode::Full => {
            tracing::info!(
                "Full tunnel: SNI={} -> script.google.com (via {})",
                config.front_domain,
                config.google_ip
            );
            let groups = config.account_groups_resolved();
            let total_ids: usize = groups
                .iter()
                .map(|g| g.script_ids.clone().into_vec().len())
                .sum();
            tracing::info!("Accounts: {}  Deployments: {}", groups.len(), total_ids);
            tracing::warn!(
                "Full tunnel mode: NO certificate installation needed. \
                 ALL traffic is tunneled end-to-end through Apps Script + tunnel node."
            );
        }
    }

    // Initialize MITM manager (generates CA on first run).
    let base = mhrv_jni::data_dir::data_dir();
    let mitm = match MitmCertManager::new_in(&base) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("failed to init MITM CA: {}", e);
            return ExitCode::FAILURE;
        }
    };
    let ca_path = base.join(CA_CERT_FILE);

    if !args.no_cert_check && mode != mhrv_jni::config::Mode::Full {
        if !is_ca_trusted(&ca_path) {
            tracing::warn!("MITM CA is not (obviously) trusted — attempting install...");
            match install_ca(&ca_path) {
                Ok(()) => tracing::info!("CA installed."),
                Err(e) => tracing::error!(
                    "Auto-install failed ({}). Run with --install-cert (may need sudo) \
                     or install ca/ca.crt manually as a trusted root.",
                    e
                ),
            }
        } else {
            tracing::info!("MITM CA appears to be trusted.");
        }
    }

    let mitm = Arc::new(Mutex::new(mitm));
    let server = match ProxyServer::new(&config, mitm) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to build proxy server: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let run = server.run(shutdown_rx);
    tokio::select! {
        r = run => {
            if let Err(e) = r {
                eprintln!("server error: {}", e);
                return ExitCode::FAILURE;
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::warn!("Ctrl+C — shutting down.");
            let _ = shutdown_tx.send(());
        }
    }
    ExitCode::SUCCESS
}
