use eframe::egui;

use mhrv_jni::branding::{GITHUB_REPO_URL, PRODUCT_NAME};

use crate::ui_fs::open_local_resource;
use crate::ui_style::{
    help_callout, help_muted, help_subheading, mode_goal_card, ACCENT, ACCENT_MINT, ACCENT_WARM,
};
use crate::ui_trust::trust_center_snapshot_panel;
use crate::FormState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ToolHelpEntry {
    pub name: &'static str,
    pub role: &'static str,
    pub next_step: &'static str,
    pub local_path: Option<&'static str>,
}

const BACKEND_TOOL_ENTRIES: [ToolHelpEntry; 8] = [
    ToolHelpEntry {
        name: "Apps Script Code.gs",
        role: "default backend for apps_script mode.",
        next_step: "Deploy assets/apps_script/Code.gs as a Web app, set AUTH_KEY, then paste deployment IDs into Multi-account pools.",
        local_path: Some("assets/apps_script/Code.gs"),
    },
    ToolHelpEntry {
        name: "Cloudflare Worker exit",
        role: "optional Apps Script-compatible exit path.",
        next_step: "Deploy tools/cloudflare-worker-json-relay, then use assets/apps_script/CodeCloudflareWorker.gs in Apps Script when you want Worker egress.",
        local_path: Some("tools/cloudflare-worker-json-relay"),
    },
    ToolHelpEntry {
        name: "Vercel Edge JSON",
        role: "native vercel_edge-compatible mode with no VPS.",
        next_step: "Deploy tools/vercel-json-relay, set AUTH_KEY, disable Deployment Protection, then paste Base URL and key in this UI.",
        local_path: Some("tools/vercel-json-relay"),
    },
    ToolHelpEntry {
        name: "Netlify Edge JSON",
        role: "native vercel_edge-compatible mode with no VPS.",
        next_step: "Deploy tools/netlify-json-relay, set AUTH_KEY, confirm /api/api returns JSON, then paste the Netlify site URL and key in this UI.",
        local_path: Some("tools/netlify-json-relay"),
    },
    ToolHelpEntry {
        name: "Vercel XHTTP helper",
        role: "external Xray/V2Ray helper for a Vercel front, not a native desktop mode.",
        next_step: "Use tools/vercel-xhttp-relay first, or tools/vercel-xhttp-relay-node when the Edge runtime is not a good fit. Keep Host set to your Vercel project domain.",
        local_path: Some("tools/vercel-xhttp-relay"),
    },
    ToolHelpEntry {
        name: "Netlify XHTTP helper",
        role: "external Xray/V2Ray helper for a Netlify front, not a native desktop mode.",
        next_step: "Use tools/netlify-xhttp-relay with your own XHTTP backend. Start with your deployed domain; for Address/SNI tests load the in-app generator preset and keep Host on your deployed site unless you knowingly accept a mismatched-front profile.",
        local_path: Some("tools/netlify-xhttp-relay"),
    },
    ToolHelpEntry {
        name: "Field notes",
        role: "cleaned edge candidates and external-client caveats.",
        next_step: "See docs/field-notes.md for Google SNI candidates, Vercel Address/SNI names, Netlify/Fastly/CloudFront notes, and rejected risky items.",
        local_path: Some("docs/field-notes.md"),
    },
    ToolHelpEntry {
        name: "tunnel-node",
        role: "server component for full mode.",
        next_step: "Build and run tunnel-node on your VPS, point the full-mode Apps Script channel at it, then verify with an IP-check page.",
        local_path: Some("tunnel-node"),
    },
];

pub(crate) fn backend_tool_entries() -> &'static [ToolHelpEntry] {
    &BACKEND_TOOL_ENTRIES
}

pub(crate) fn render_tool_help_row(ui: &mut egui::Ui, entry: ToolHelpEntry) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(entry.name)
                .strong()
                .color(egui::Color32::from_gray(220)),
        );
        ui.label(egui::RichText::new("->").color(egui::Color32::from_gray(110)));
        help_muted(ui, entry.role);
        if let Some(path) = entry.local_path {
            if ui
                .small_button("open")
                .on_hover_text(format!("Open {} in the file manager.", path))
                .clicked()
            {
                open_local_resource(path);
            }
        }
    });
    ui.add_space(1.0);
    ui.horizontal_wrapped(|ui| {
        ui.add_space(14.0);
        ui.small(egui::RichText::new(entry.next_step).color(egui::Color32::from_gray(145)));
    });
}

pub(crate) fn help_walkthrough(ui: &mut egui::Ui, form: &FormState) {
    ui.spacing_mut().item_spacing.y = 7.0;
    help_subheading(ui, "Welcome");
    help_muted(
        ui,
        &format!(
            "{} is the desktop control room for the relay engine. It runs a local HTTP + SOCKS5 \
             proxy: browsers and apps talk to localhost, and the selected mode decides where the \
             request goes next: Apps Script, serverless JSON, direct fronting, or full tunnel.",
            PRODUCT_NAME
        ),
    );

    help_subheading(ui, "First-time checklist");
    help_muted(
        ui,
        "1) Choose a mode. Apps Script and serverless JSON are no-VPS relay modes; Full needs a tunnel-node.\n\
         2) Fill the relay credentials for that mode: Apps Script account groups, or Vercel/Netlify Base URL + AUTH_KEY.\n\
         3) Click Install CA once for Apps Script/serverless JSON/direct fronting, then Check CA. Full mode does not need local MITM CA.\n\
         4) Keep front_domain as www.google.com and run Scan IPs / SNI tests if connections time out.\n\
         5) Save config, then Start. Set your browser or system proxy to the HTTP port; SOCKS5 is optional.\n\
         6) Use Test relay and Doctor early. They are faster than guessing.",
    );

    help_subheading(ui, "Trust Center (certificates & signing)");
    help_muted(
        ui,
        "CA install/remove, Firefox NSS vs OS trust, Android user-CA limits, APK signing policy, \
         and diagnostic redaction expectations are summarized in one maintainer-facing hub doc.",
    );
    trust_center_snapshot_panel(ui, form.to_config());
    ui.add_space(4.0);
    ui.hyperlink_to(
        egui::RichText::new("Open docs/trust-center.md")
            .size(12.0)
            .color(ACCENT),
        "docs/trust-center.md",
    );

    help_subheading(ui, "Backend registry (deploy map)");
    help_muted(
        ui,
        "Canonical table of Apps Script helpers, Cloudflare Worker exit, serverless JSON relays, tunnel-node, \
         compat probes, and Doctor/Test wiring — before dedicated Backend Registry UI lands.",
    );
    ui.add_space(4.0);
    ui.hyperlink_to(
        egui::RichText::new("Open docs/backend-registry.md")
            .size(12.0)
            .color(ACCENT),
        "docs/backend-registry.md",
    );

    help_subheading(ui, "Choose by goal");
    ui.horizontal_wrapped(|ui| {
        mode_goal_card(
            ui,
            "Fastest normal setup",
            "Use Apps Script. Deploy Code.gs, add one account group, install the CA, then Start.",
            ACCENT,
        );
        mode_goal_card(
            ui,
            "No Google script quota pool yet",
            "Use Serverless JSON. Deploy Vercel or Netlify JSON relay, set AUTH_KEY, paste Base URL.",
            ACCENT_MINT,
        );
        mode_goal_card(
            ui,
            "Need only setup access",
            "Use Direct to reach script.google.com or tested fronting-group targets without relay credentials.",
            ACCENT_WARM,
        );
        mode_goal_card(
            ui,
            "Need no local CA",
            "Use Full tunnel with tunnel-node. It needs a VPS but avoids local HTTPS interception.",
            egui::Color32::from_rgb(170, 145, 225),
        );
    });

    help_subheading(ui, "Modes - pick the story that matches your network");
    help_muted(
        ui,
        "- Apps Script: classic no-VPS path through your Google Apps Script deployment.\n\
         - Serverless JSON: no-VPS fetch relay; deploy tools/vercel-json-relay or tools/netlify-json-relay.\n\
         - Direct: no-relay SNI rewrite for Google plus configured fronting groups such as Vercel, Fastly, and Netlify/CloudFront.\n\
         - Full tunnel: routes through Apps Script + tunnel-node; no local MITM certificate, but requires server infrastructure.",
    );
    help_callout(
        ui,
        "Mode requirements at a glance",
        "Apps Script needs Code.gs, at least one account group, and local CA trust. Serverless JSON needs a Vercel/Netlify JSON endpoint, AUTH_KEY, and local CA trust. Direct needs only edge/SNI settings but is not a full proxy. Full tunnel needs CodeFull.gs plus tunnel-node on a VPS and does not use the local CA.",
        ACCENT_MINT,
    );

    help_subheading(ui, "Backends, tools, and what they are not");
    help_muted(
        ui,
        "Native desktop modes are Apps Script, serverless JSON, Direct, and Full tunnel. Cloudflare Worker is an optional Apps Script exit. Vercel XHTTP and Netlify XHTTP helpers are for external Xray/V2Ray backends, so they are documented as tools rather than selectable desktop modes. Field notes collect tested edge-name candidates without raw forum noise.",
    );
    help_callout(
        ui,
        "Avoid split-brain setup",
        "Do not mix native Serverless JSON fields with XHTTP helper configs. The desktop UI talks to JSON/base64 fetch relays. XHTTP helpers are for Xray/V2Ray clients and have their own host/path/SNI rules.",
        ACCENT_WARM,
    );
    help_callout(
        ui,
        "Defaults that should usually stay put",
        "Apps Script: front_domain www.google.com, local HTTP/SOCKS on 127.0.0.1, verify SSL on. Serverless JSON: Base URL is only the Vercel/Netlify origin, relay path /api/api, max body 4 MiB, verify TLS on. External XHTTP: use the in-app VLESS generator for Vercel and Netlify presets. Vercel candidates include react.dev, nextjs.org, cursor.com, and Vercel subdomains. Netlify candidates include kubernetes.io, helm.sh, letsencrypt.org, and related Helm/Kubernetes/SIG subdomains. Host should usually remain your own deployed site domain.",
        ACCENT_MINT,
    );

    help_subheading(ui, "Sharing and per-app routing");
    help_muted(
        ui,
        "Desktop per-app routing is app-level: point one browser profile, Telegram, xray, or any app with proxy settings at 127.0.0.1:HTTP/SOCKS while other apps stay direct. To share to other devices, bind to 0.0.0.0 and set Allowed IPs; SOCKS5 cannot carry the LAN token header. Android is different: VPN mode has native app splitting, and Proxy-only mode lets individual apps opt in through their own proxy settings.",
    );

    help_subheading(ui, "If something looks stuck");
    help_muted(
        ui,
        "Timeouts: wrong google_ip, poisoned DNS, blocked SNI, stale Apps Script deployment, or backend relay timeout.\n\
         HTML instead of JSON: Apps Script access is not Anyone, or platform protection/routing is in front of the relay.\n\
         Quota / 504 spikes: add deployment IDs/accounts, lower fan-out, or enable relay_rate_limit_qps.\n\
         Certificate warnings: Install CA again or run Doctor. Firefox may need restart/NSS handling.",
    );

    help_subheading(ui, "Account groups explained");
    help_muted(
        ui,
        "A group is one relay identity, usually one Google account. Inside that group, one AUTH_KEY protects all deployment IDs from that account. Multiple IDs inside the same group help rotation/fallback and can smooth transient deployment failures, but they still share that Google account's daily quota and concurrency limits. Multiple groups are different accounts or deliberately separated quota pools. The engine can pick across groups, respect enabled/disabled state, and use weights so a stronger account carries more load.",
    );
    help_callout(
        ui,
        "Practical group recipe",
        "Start with one group: label it, paste one AUTH_KEY, paste one or more deployment IDs from the same Apps Script account, then Test relay. Add a second group only when you have a second account or want a backup identity. If quota pressure rises, add capacity first; if failures spike, lower fan-out/rate before adding aggressive speed knobs.",
        ACCENT,
    );

    help_subheading(ui, "Advanced tuning recipe");
    help_muted(
        ui,
        "Optimize in this order: 1) verify google_ip/front_domain/SNI first, 2) add account groups or deployment IDs for capacity, 3) enable runtime_auto_tune with balanced profile, 4) tune parallel_relay only if multiple healthy IDs exist, 5) increase range_parallelism only for large downloads, 6) add relay_rate_limit_qps when quotas or 504 storms appear. Never change several knobs at once; the Dashboard should tell you which limit moved.",
    );

    egui::CollapsingHeader::new(
        egui::RichText::new("Tips for each area of this window")
            .strong()
            .color(ACCENT)
            .size(13.0),
    )
    .id_source("help_area_tips")
    .default_open(false)
    .show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        help_subheading(ui, "Mode");
        help_muted(ui, "Changing mode reshapes the whole form. Apps Script uses account groups, serverless JSON uses Base URL + AUTH_KEY, Direct is no-relay fronting, and Full is tunnel-node based.");
        help_subheading(ui, "Apps Script relay / Multi-account pools");
        help_muted(ui, "Each enabled group is one Google account: its own AUTH_KEY and one-or-more deployment IDs. We rotate IDs to spread load. Labels are optional but help you read logs.");
        help_subheading(ui, "Serverless JSON relay");
        help_muted(ui, "Base URL is the Vercel or Netlify app origin, relay path is usually /api/api, and auth key must match the AUTH_KEY environment variable. Protection or routing pages must not sit in front of the relay endpoint.");
        help_subheading(ui, "Backend tools");
        help_muted(ui, "Use the Backend tools section to decide which file or VPS component to deploy: Code.gs for Apps Script, CodeCloudflareWorker.gs plus a Worker when you want Cloudflare egress, Vercel/Netlify JSON for native vercel_edge, separate Vercel XHTTP and Netlify XHTTP helpers for external Xray/V2Ray, and tunnel-node for full mode.");
        help_subheading(ui, "Network");
        help_muted(ui, "google_ip is the IPv4 of a Google edge that accepts TLS with front_domain as SNI. Ports default to 8085/8086 but can move if those are busy. Listen host stays on 127.0.0.1 unless you know you need otherwise.");
        help_subheading(ui, "Sharing");
        help_muted(ui, "Local-only is safest. LAN sharing is useful for another phone/laptop on the same Wi-Fi, but set Allowed IPs before exposing SOCKS5. A token protects HTTP clients that can add X-MHRV-F-Token; it is not a SOCKS5 password.");
        help_subheading(ui, "Profiles");
        help_muted(ui, "Save named snapshots (home / office / experimental) so you can flip between known-good configs without hand-editing JSON.");
        help_subheading(ui, "Traffic + Dashboard");
        help_muted(ui, "Once running, watch relay failures, degrade level, and quota pressure. Spikes usually mean \"add capacity\" (more deployments / accounts) or \"slow down\" (rate limits, smaller parallel_relay, lower range_parallelism / bigger range_chunk_bytes).");
        help_subheading(ui, "Updates");
        help_muted(ui, "Check for updates talks to GitHub Releases. If your ISP rate-limits GitHub, start the proxy first and check again — the UI can route the request through the relay bucket.");
    });

    egui::CollapsingHeader::new(
        egui::RichText::new("Advanced options — what changes when you tweak them")
            .strong()
            .color(ACCENT)
            .size(13.0),
    )
    .id_source("help_advanced_options")
    .default_open(false)
    .show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        help_muted(
            ui,
            "Rule of thumb: increase speed knobs only when you have enough script IDs/accounts; otherwise you often just convert \"slow\" into \"quota exhausted\".",
        );

        help_subheading(ui, "parallel_relay (fan-out per request)");
        help_muted(
            ui,
            "Higher = lower tail latency (less \"one slow script stalls the page\"), but burns quota faster because it launches multiple relay calls for the same request.",
        );

        help_subheading(ui, "relay_rate_limit_qps / burst");
        help_muted(
            ui,
            "A soft governor: lower values smooth spikes and reduce 504 storms, but can make pages feel slower because requests queue instead of bursting.",
        );

        help_subheading(ui, "range_parallelism / range_chunk_bytes");
        help_muted(
            ui,
            "Affects large downloads: higher parallelism is faster but increases in-flight relay calls; larger chunks reduce call count (quota-friendly) but each call runs longer.",
        );

        help_subheading(ui, "runtime_auto_tune + runtime_profile");
        help_muted(
            ui,
            "Auto-picks safe defaults for a few hot knobs. eco = quota-friendly and stable; max_speed = fastest but most quota-hungry.",
        );

        help_subheading(ui, "upstream_socks5");
        help_muted(
            ui,
            "Only affects raw TCP flows that bypass the relay (passthrough / non-HTTP). Useful when you already run xray/sing-box locally; it does not change Apps Script-relayed HTTP/HTTPS.",
        );

        help_subheading(ui, "passthrough_hosts / domain_overrides");
        help_muted(
            ui,
            "Use these to fix one broken site without changing global behavior. passthrough saves quota and avoids MITM for that host; domain_overrides can force direct/relay/sni_rewrite and can disable chunking (never_chunk) for fragile anti-bot flows.",
        );

        help_subheading(ui, "verify_ssl");
        help_muted(
            ui,
            "Keep ON unless you understand the risk. Turning it OFF makes the outer TLS tunnel accept a MITM middlebox — it may 'work' on hostile networks, but you lose certificate validation on the outer hop.",
        );

        help_subheading(ui, "youtube_via_relay");
        help_muted(
            ui,
            "Routes YouTube HTML/API through Apps Script. Can bypass Restricted-Mode/SafeSearch-on-SNI issues, but costs quota and uses the fixed Apps Script User-Agent. Thumbnails/assets stay on SNI rewrite; googlevideo.com is not forced onto the normal Google frontend IP.",
        );

        ui.add_space(4.0);
        ui.hyperlink_to(
            egui::RichText::new("Open full advanced reference (docs/advanced-options.md)")
                .size(12.0)
                .color(ACCENT),
            "docs/advanced-options.md",
        );
    });

    egui::CollapsingHeader::new(
        egui::RichText::new("Privacy & trust — plain language")
            .strong()
            .color(ACCENT)
            .size(13.0),
    )
    .id_source("help_privacy")
    .default_open(false)
    .show(ui, |ui| {
        help_muted(
            ui,
            "Your traffic touches Google's network and your own Apps Script code. MITM mode can read HTTPS on this machine exactly like any debugging proxy — only install the CA on devices you control. \
             Full tunnel shifts trust to whatever tunnel node you operate. When in doubt, read the Security section in the README.",
        );
    });

    help_subheading(ui, "Android companion");
    help_muted(
        ui,
        "The mobile build wraps the same Rust engine with a VPN/proxy UI. Install the APK from the maintainer releases page, walk through the in-app Help section there, then mirror the deployment IDs + keys you use on desktop.",
    );

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        help_muted(ui, "Maintainer repository:");
        ui.add_space(4.0);
        ui.hyperlink_to(
            egui::RichText::new(GITHUB_REPO_URL)
                .size(12.0)
                .color(ACCENT),
            GITHUB_REPO_URL,
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_tool_entries_cover_expected_catalog() {
        let names: Vec<&str> = backend_tool_entries()
            .iter()
            .map(|entry| entry.name)
            .collect();
        assert_eq!(
            names,
            vec![
                "Apps Script Code.gs",
                "Cloudflare Worker exit",
                "Vercel Edge JSON",
                "Netlify Edge JSON",
                "Vercel XHTTP helper",
                "Netlify XHTTP helper",
                "Field notes",
                "tunnel-node",
            ]
        );
    }

    #[test]
    fn backend_tool_entries_keep_local_paths() {
        assert!(backend_tool_entries()
            .iter()
            .all(|entry| entry.local_path.is_some()));
    }
}
