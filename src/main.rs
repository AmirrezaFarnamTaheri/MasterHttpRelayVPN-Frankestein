use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

use mhrv_jni::branding::PRODUCT_NAME;
use mhrv_jni::cert_installer::{install_ca, is_ca_trusted};
use mhrv_jni::config::Config;
use mhrv_jni::mitm::{MitmCertManager, CA_CERT_FILE};
use mhrv_jni::proxy_server::ProxyServer;
use mhrv_jni::{scan_ips, scan_sni, test_cmd};

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Args {
    config_path: Option<PathBuf>,
    install_cert: bool,
    no_cert_check: bool,
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
}

fn print_help() {
    println!(
        "mhrv-f {} — {} (Apps Script client)

USAGE:
    mhrv-f [OPTIONS]                  Start the proxy server (default)
    mhrv-f test [OPTIONS]             Probe the Apps Script relay end-to-end
    mhrv-f doctor [OPTIONS]           Guided diagnostics (first-run fix assistant)
    mhrv-f doctor-fix [OPTIONS]       Doctor + apply one-click fixes (best-effort)
    mhrv-f support-bundle [OPTIONS]   Export an anonymized diagnostics bundle
    mhrv-f rollback-config            Restore last-known-good config (best-effort)
    mhrv-f scan-ips [OPTIONS]         Scan Google frontend IPs for reachability + latency
    mhrv-f scan-sni         Scan Google SNI name using Google frontend IPs found in 'scan-ips' command
    mhrv-f test-sni [OPTIONS]         Probe each SNI name in the rotation pool against google_ip

OPTIONS:
    -c, --config PATH    Path to config.json (default: ./config.json)
    --install-cert       Install the MITM CA certificate and exit
    --no-cert-check      Skip the auto-install-if-untrusted check on startup
    -h, --help           Show this message
    -V, --version        Show version

ENV:
    RUST_LOG             Override log level (e.g. info, debug)
",
        VERSION,
        PRODUCT_NAME
    );
}

fn parse_args() -> Result<Args, String> {
    let mut config_path: Option<PathBuf> = None;
    let mut install_cert = false;
    let mut no_cert_check = false;
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
            "--no-cert-check" => no_cert_check = true,
            other => return Err(format!("unknown argument: {}", other)),
        }
    }
    Ok(Args {
        config_path,
        install_cert,
        no_cert_check,
        command,
    })
}

fn init_logging(level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

#[tokio::main]
async fn main() -> ExitCode {
    // Install default rustls crypto provider (ring).
    let _ = rustls::crypto::ring::default_provider().install_default();

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

    let config_path = mhrv_jni::data_dir::resolve_config_path(args.config_path.as_deref());
    let config = match Config::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!(
                "No valid config found. Copy config.example.json to either:\n  {}\nor run with --config <path>.",
                config_path.display()
            );
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
            let report = mhrv_jni::doctor::run(&config).await;
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
            let (before, fixes, after) = mhrv_jni::doctor::run_with_fixes(&config).await;
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

    let socks5_port = config.socks5_port.unwrap_or(config.listen_port + 1);
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
    tracing::info!("SOCKS5 proxy : {}:{}", config.listen_host, socks5_port);
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
        mhrv_jni::config::Mode::GoogleOnly => {
            tracing::warn!(
                "google_only bootstrap: direct SNI-rewrite tunnel to {} only. \
                 Open https://script.google.com in your browser (proxy set to \
                 {}:{}), deploy Code.gs, then switch to apps_script mode.",
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
