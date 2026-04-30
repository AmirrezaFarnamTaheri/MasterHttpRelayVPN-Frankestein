package com.farnam.mhrvf
import android.content.Context
import org.json.JSONArray
import org.json.JSONObject
import java.io.File

private const val DEFAULT_ANDROID_GOOGLE_IP = "142.251.36.68"
private const val DEFAULT_RELAY_PATH = "/api/api"

private fun extractScriptId(input: String): String {
    var s = input.trim()
    if (s.isEmpty()) return s
    val marker = "/macros/s/"
    val i = s.indexOf(marker)
    if (i >= 0) s = s.substring(i + marker.length)
    val slash = s.indexOf('/')
    if (slash >= 0) s = s.substring(0, slash)
    val q = s.indexOf('?')
    if (q >= 0) s = s.substring(0, q)
    return s.trim()
}

private fun scriptIdUrl(id: String): String =
    "https://script.google.com/macros/s/${extractScriptId(id)}/exec"

private fun stringList(arr: JSONArray?): List<String> =
    arr?.let {
        buildList {
            for (i in 0 until it.length()) add(it.optString(i))
        }
    }?.map { it.trim() }?.filter { it.isNotEmpty() }.orEmpty()

private fun scriptIdsFromValue(value: Any?): List<String> =
    when (value) {
        is JSONArray -> stringList(value)
        is String -> listOf(value.trim()).filter { it.isNotEmpty() }
        else -> emptyList()
    }.map { extractScriptId(it) }.filter { it.isNotEmpty() }.distinct()

private fun canonicalAccountGroups(
    ids: List<String>,
    authKey: String,
    preservedJson: String,
): JSONArray? {
    if (ids.isEmpty() && authKey.isBlank() && preservedJson.isBlank()) return null
    val groups = if (preservedJson.isNotBlank()) {
        try {
            JSONArray(preservedJson)
        } catch (_: Throwable) {
            JSONArray()
        }
    } else {
        JSONArray()
    }
    val targetIndex = (0 until groups.length())
        .firstOrNull { groups.optJSONObject(it)?.optBoolean("enabled", true) == true }
        ?: 0
    val first = groups.optJSONObject(targetIndex) ?: JSONObject().apply {
        put("label", "primary")
        put("weight", 1)
        put("enabled", true)
    }
    first.put("auth_key", authKey.trim())
    first.put("script_ids", JSONArray().apply { ids.forEach { put(it) } })
    if (groups.length() == 0) {
        groups.put(first)
    } else {
        groups.put(targetIndex, first)
    }
    return groups
}

private data class AccountGroupsProjection(
    val urls: List<String>,
    val authKey: String,
    val preservedJson: String,
)

private fun projectAccountGroups(obj: JSONObject): AccountGroupsProjection {
    val groups = obj.optJSONArray("account_groups")
    if (groups == null || groups.length() == 0) {
        val legacyIds = scriptIdsFromValue(obj.opt("script_ids"))
        return AccountGroupsProjection(
            urls = legacyIds.map { scriptIdUrl(it) },
            authKey = obj.optString("auth_key", ""),
            preservedJson = "",
        )
    }
    val selected = (0 until groups.length())
        .mapNotNull { groups.optJSONObject(it) }
        .firstOrNull { it.optBoolean("enabled", true) }
        ?: groups.optJSONObject(0)
    val ids = scriptIdsFromValue(selected?.opt("script_ids"))
    return AccountGroupsProjection(
        urls = ids.map { scriptIdUrl(it) },
        authKey = selected?.optString("auth_key", "") ?: "",
        preservedJson = groups.toString(),
    )
}

/**
 * Config I/O. The source of truth is a JSON file in the app's files dir —
 * the Rust side parses the same file, so we don't maintain two schemas.
 *
 * What the Android UI exposes is a pragmatic subset of the full mhrv-f
 * config, but we now track parity with the desktop UI on the dimensions
 * that actually matter on a phone:
 *   - multiple deployment IDs (round-robin)
 *   - an SNI rotation pool
 *   - log level / verify_ssl / parallel_relay knobs
 * Anything else gets phone-appropriate defaults.
 */
/**
 * How the foreground service exposes the proxy to the rest of the device.
 *
 * - [VPN_TUN] — the default; `VpnService` claims a TUN interface and every
 *   app's traffic goes through `tun2proxy` → our SOCKS5 → Apps Script.
 *   Requires the user to accept the system "VPN connection request"
 *   dialog on first Start.
 *
 * - [PROXY_ONLY] — just runs the HTTP (`127.0.0.1:8080`) and SOCKS5
 *   (`127.0.0.1:1081`) listeners; no VpnService, no TUN. The user sets
 *   their Wi-Fi proxy (or a per-app proxy setting) to those addresses.
 *   Useful when the device already has another VPN up, or the user
 *   specifically wants per-app opt-in, or on rooted/specialized devices
 *   where VpnService is unwelcome.
 */
enum class ConnectionMode { VPN_TUN, PROXY_ONLY }
/**
 * App-splitting policy when in VPN_TUN mode.
 *
 * - [ALL]  — tunnel every app (default; the package list is ignored).
 * - [ONLY] — allow-list: tunnel ONLY the apps in `splitApps`. Everything
 *   else bypasses the VPN. Useful when you want mhrv-f for a specific
 *   browser / messenger and nothing else.
 * - [EXCEPT] — deny-list: tunnel everything EXCEPT the apps in
 *   `splitApps`. Useful for excluding a banking app that would break
 *   under MITM anyway, or a self-updater you don't want going through
 *   the quota-limited relay.
 *
 * Our own package (`packageName`) is always excluded regardless of mode
 * — that's the loop-avoidance rule from day one, not a user toggle.
 */
enum class SplitMode { ALL, ONLY, EXCEPT }
/**
 * UI language preference. AUTO respects the device locale; FA / EN
 * force the app into Persian / English with proper RTL / LTR layout
 * on next app launch (AppCompatDelegate.setApplicationLocales is
 * applied at Application.onCreate).
 */
enum class UiLang { AUTO, FA, EN }
/**
 * Operating mode. Mirrors the Rust-side `Mode` enum.
 *
 * - [APPS_SCRIPT] (default) — full DPI bypass through the user's deployed
 *   Apps Script relay. Requires a Deployment ID + Auth key.
 * - [SERVERLESS_JSON] — no-VPS JSON relay through Vercel or Netlify Edge.
 *   Serialized as Rust's compatibility mode name, `vercel_edge`.
 * - [DIRECT] - no Apps Script relay; SNI-rewrite only (Google plus config-file fronting_groups).
 *   No Deployment ID / Auth key needed. Legacy "google_only" configs load as DIRECT.
 * - [FULL] — full tunnel mode. ALL traffic is tunneled end-to-end through
 *   Apps Script + a remote tunnel node. No certificate installation needed.
 */
enum class Mode { APPS_SCRIPT, SERVERLESS_JSON, DIRECT, FULL }
data class MhrvConfig(
    val mode: Mode = Mode.APPS_SCRIPT,
    val listenHost: String = "127.0.0.1",
    val listenPort: Int = 8080,
    val socks5Port: Int? = 1081,
    /** One Apps Script ID or deployment URL per entry. */
    val appsScriptUrls: List<String> = emptyList(),
    val authKey: String = "",
    /**
     * Canonical `account_groups` JSON imported from Desktop/Rust configs.
     *
     * Android's normal setup edits the first enabled group for mobile
     * ergonomics, but this preserves additional Desktop-created groups and
     * advanced per-group fields during save/share round-trips.
     */
    val preservedAccountGroupsJson: String = "",
    /** Native serverless JSON relay fields; serialized under Rust's `vercel` key. */
    val serverlessBaseUrl: String = "",
    val serverlessAuthKey: String = "",
    val serverlessRelayPath: String = "/api/api",
    val frontDomain: String = "www.google.com",
    /** Rotation pool of SNI hostnames; empty means "let Rust auto-expand". */
    val sniHosts: List<String> = emptyList(),
    val googleIp: String = "142.251.36.68",
    val verifySsl: Boolean = true,
    val logLevel: String = "info",
    val parallelRelay: Int = 1,
    val coalesceStepMs: Int = 40,
    val coalesceMaxMs: Int = 1000,
    val upstreamSocks5: String = "",
    /**
     * User-configured hostnames that bypass Apps Script relay entirely
     * and use plain-TCP passthrough (via upstreamSocks5 if set). Entries
     * are exact hostnames, leading-dot suffixes, or "*.example.com" aliases;
     * Rust owns the semantics.
     */
    val passthroughHosts: List<String> = emptyList(),
    /** Drop SOCKS5 UDP/443 in full mode so QUIC clients fall back to TCP. */
    val blockQuic: Boolean = false,
    /**
     * Opt-out for the DoH bypass. Default false means known DoH endpoints
     * bypass Apps Script on TCP/443. Set true to keep DoH inside the tunnel.
     */
    val tunnelDoh: Boolean = true,
    /** Extra DoH hostnames added to the built-in bypass list. */
    val bypassDohHosts: List<String> = emptyList(),
    /** Raw config-only fronting_groups JSON; preserved even though Android has no editor yet. */
    val frontingGroupsJson: String = "",
    /** VPN_TUN (everything routed) vs PROXY_ONLY (user configures per-app). */
    val connectionMode: ConnectionMode = ConnectionMode.VPN_TUN,
    /** ALL / ONLY / EXCEPT — scope of app splitting inside VPN_TUN mode. */
    val splitMode: SplitMode = SplitMode.ALL,
    /** Package names used by ONLY and EXCEPT. Empty under ALL. */
    val splitApps: List<String> = emptyList(),
    /** UI language toggle. Non-Rust; honoured only by the Android wrapper. */
    val uiLang: UiLang = UiLang.AUTO,
) {
    /**
     * Serialize the phone-editable config subset plus preserved advanced JSON.
     * Apps Script credentials are always written as canonical `account_groups`;
     * legacy top-level `script_ids` / `auth_key` are import-only.
     */
    fun toJson(): String {
        val ids = appsScriptUrls
            .map { extractScriptId(it) }
            .filter { it.isNotEmpty() }
            .distinct()
        val obj = JSONObject().apply {
            // `mode` is required — without it serde errors with
            // "missing field `mode`" and startProxy silently returns 0.
            put("mode", when (mode) {
                Mode.APPS_SCRIPT -> "apps_script"
                Mode.SERVERLESS_JSON -> "vercel_edge"
                Mode.DIRECT -> "direct"
                Mode.FULL -> "full"
            })
            put("listen_host", listenHost)
            put("listen_port", listenPort)
            socks5Port?.let { put("socks5_port", it) }
            // Canonical Apps Script / full-mode credentials. Android edits a
            // simple primary group, but preserves imported Desktop groups.
            canonicalAccountGroups(ids, authKey, preservedAccountGroupsJson)?.let {
                put("account_groups", it)
            }
            put("vercel", JSONObject().apply {
                put("base_url", serverlessBaseUrl.trim())
                put("relay_path", serverlessRelayPath.trim().ifEmpty { DEFAULT_RELAY_PATH })
                put("auth_key", serverlessAuthKey.trim())
                put("verify_tls", true)
            })
            put("front_domain", frontDomain)
            if (sniHosts.isNotEmpty()) {
                put("sni_hosts", JSONArray().apply { sniHosts.forEach { put(it) } })
            }
            put("google_ip", googleIp)
            put("verify_ssl", verifySsl)
            put("log_level", logLevel)
            put("parallel_relay", parallelRelay)
            if (coalesceStepMs != 40) put("coalesce_step_ms", coalesceStepMs)
            if (coalesceMaxMs != 1000) put("coalesce_max_ms", coalesceMaxMs)
            if (upstreamSocks5.isNotBlank()) {
                put("upstream_socks5", upstreamSocks5.trim())
            }
            if (passthroughHosts.isNotEmpty()) {
                put("passthrough_hosts", JSONArray().apply { passthroughHosts.forEach { put(it) } })
            }
            if (blockQuic) put("block_quic", true)
            if (tunnelDoh) put("tunnel_doh", true)
            val cleanBypassDohHosts = bypassDohHosts
                .map { it.trim() }
                .filter { it.isNotEmpty() }
                .distinct()
            if (cleanBypassDohHosts.isNotEmpty()) {
                put("bypass_doh_hosts", JSONArray().apply { cleanBypassDohHosts.forEach { put(it) } })
            }
            if (frontingGroupsJson.isNotBlank()) put("fronting_groups", JSONArray(frontingGroupsJson))
            // Phone-scoped scan defaults. We don't expose these in the UI
            // because a phone isn't where you'd run a full /16 scan; users
            // who need it can do that on the desktop UI and paste the IP.
            put("fetch_ips_from_api", false)
            put("max_ips_to_scan", 20)
            // Android-only: surfaced in the UI dropdown. The Rust side
            // doesn't read this key (serde ignores unknown fields), which
            // is intentional — proxy-vs-TUN is a service-layer decision
            // that belongs to the Android wrapper, not the crate.
            put("connection_mode", when (connectionMode) {
                ConnectionMode.VPN_TUN -> "vpn_tun"
                ConnectionMode.PROXY_ONLY -> "proxy_only"
            })
            put("split_mode", when (splitMode) {
                SplitMode.ALL -> "all"
                SplitMode.ONLY -> "only"
                SplitMode.EXCEPT -> "except"
            })
            if (splitApps.isNotEmpty()) {
                put("split_apps", JSONArray().apply { splitApps.forEach { put(it) } })
            }
            put("ui_lang", when (uiLang) {
                UiLang.AUTO -> "auto"
                UiLang.FA -> "fa"
                UiLang.EN -> "en"
            })
        }
        return obj.toString(2)
    }
    /** Convenience: is there at least one usable deployment ID? */
    val hasDeploymentId: Boolean get() =
        appsScriptUrls.any { extractScriptId(it).isNotEmpty() }
    val hasServerlessConfig: Boolean get() =
        serverlessBaseUrl.isNotBlank() &&
            serverlessAuthKey.isNotBlank() &&
            serverlessRelayPath.trim().startsWith("/")
}
object ConfigStore {
    private const val FILE = "config.json"
    private const val HASH_PREFIX = "mhrvf://"
    private const val LEGACY_HASH_PREFIX = "mhrv-rs://"
    fun load(ctx: Context): MhrvConfig {
        val f = File(ctx.filesDir, FILE)
        if (!f.exists()) return MhrvConfig()
        return try {
            val obj = JSONObject(f.readText())
            val accountGroups = projectAccountGroups(obj)
            val sni = obj.optJSONArray("sni_hosts")?.let { arr ->
                buildList { for (i in 0 until arr.length()) add(arr.optString(i)) }
            }?.filter { it.isNotBlank() }.orEmpty()
            MhrvConfig(
                mode = when (obj.optString("mode", "apps_script")) {
                    "vercel_edge" -> Mode.SERVERLESS_JSON
                    "direct", "google_only" -> Mode.DIRECT
                    "full" -> Mode.FULL
                    else -> Mode.APPS_SCRIPT
                },
                listenHost = obj.optString("listen_host", "127.0.0.1"),
                listenPort = obj.optInt("listen_port", 8080),
                socks5Port = obj.optInt("socks5_port", 1081).takeIf { it > 0 },
                appsScriptUrls = accountGroups.urls,
                authKey = accountGroups.authKey,
                preservedAccountGroupsJson = accountGroups.preservedJson,
                serverlessBaseUrl = obj.optJSONObject("vercel")?.optString("base_url", "") ?: "",
                serverlessAuthKey = obj.optJSONObject("vercel")?.optString("auth_key", "") ?: "",
                serverlessRelayPath = obj.optJSONObject("vercel")?.optString("relay_path", DEFAULT_RELAY_PATH)
                    ?.takeIf { it.isNotBlank() } ?: DEFAULT_RELAY_PATH,
                frontDomain = obj.optString("front_domain", "www.google.com"),
                sniHosts = sni,
                googleIp = obj.optString("google_ip", DEFAULT_ANDROID_GOOGLE_IP),
                verifySsl = obj.optBoolean("verify_ssl", true),
                logLevel = obj.optString("log_level", "info"),
                parallelRelay = obj.optInt("parallel_relay", 1),
                coalesceStepMs = obj.optInt("coalesce_step_ms", 40),
                coalesceMaxMs = obj.optInt("coalesce_max_ms", 1000),
                upstreamSocks5 = obj.optString("upstream_socks5", ""),
                passthroughHosts = obj.optJSONArray("passthrough_hosts")?.let { arr ->
                    buildList { for (i in 0 until arr.length()) add(arr.optString(i)) }
                }?.filter { it.isNotBlank() }.orEmpty(),
                blockQuic = obj.optBoolean("block_quic", false),
                tunnelDoh = obj.optBoolean("tunnel_doh", true),
                bypassDohHosts = obj.optJSONArray("bypass_doh_hosts")?.let { arr ->
                    buildList { for (i in 0 until arr.length()) add(arr.optString(i)) }
                }?.filter { it.isNotBlank() }.orEmpty(),
                frontingGroupsJson = obj.optJSONArray("fronting_groups")?.toString() ?: "",
                connectionMode = when (obj.optString("connection_mode", "vpn_tun")) {
                    "proxy_only" -> ConnectionMode.PROXY_ONLY
                    else -> ConnectionMode.VPN_TUN  // default for unknown/missing
                },
                splitMode = when (obj.optString("split_mode", "all")) {
                    "only" -> SplitMode.ONLY
                    "except" -> SplitMode.EXCEPT
                    else -> SplitMode.ALL
                },
                splitApps = obj.optJSONArray("split_apps")?.let { arr ->
                    buildList { for (i in 0 until arr.length()) add(arr.optString(i)) }
                }?.filter { it.isNotBlank() }.orEmpty(),
                uiLang = when (obj.optString("ui_lang", "auto")) {
                    "fa" -> UiLang.FA
                    "en" -> UiLang.EN
                    else -> UiLang.AUTO
                },
            )
        } catch (_: Throwable) {
            MhrvConfig()
        }
    }
    fun save(ctx: Context, cfg: MhrvConfig) {
        val f = File(ctx.filesDir, FILE)
        f.writeText(cfg.toJson())
    }
    fun encode(cfg: MhrvConfig): String {
        val defaults = MhrvConfig()
        val obj = JSONObject()
        obj.put("mode", when (cfg.mode) {
            Mode.APPS_SCRIPT -> "apps_script"
            Mode.SERVERLESS_JSON -> "vercel_edge"
            Mode.DIRECT -> "direct"
            Mode.FULL -> "full"
        })
        val ids = cfg.appsScriptUrls
            .map { url -> extractScriptId(url) }
            .filter { it.isNotEmpty() }
            .distinct()
        canonicalAccountGroups(ids, cfg.authKey, cfg.preservedAccountGroupsJson)?.let {
            obj.put("account_groups", it)
        }
        if (cfg.serverlessBaseUrl.isNotBlank() || cfg.serverlessAuthKey.isNotBlank()) {
            obj.put("vercel", JSONObject().apply {
                put("base_url", cfg.serverlessBaseUrl.trim())
                put("relay_path", cfg.serverlessRelayPath.trim().ifEmpty { DEFAULT_RELAY_PATH })
                put("auth_key", cfg.serverlessAuthKey.trim())
                put("verify_tls", true)
            })
        }
        if (cfg.googleIp != defaults.googleIp) obj.put("google_ip", cfg.googleIp)
        if (cfg.frontDomain != defaults.frontDomain) obj.put("front_domain", cfg.frontDomain)
        if (cfg.sniHosts.isNotEmpty()) obj.put("sni_hosts", JSONArray().apply { cfg.sniHosts.forEach { put(it) } })
        if (cfg.verifySsl != defaults.verifySsl) obj.put("verify_ssl", cfg.verifySsl)
        if (cfg.logLevel != defaults.logLevel) obj.put("log_level", cfg.logLevel)
        if (cfg.parallelRelay != defaults.parallelRelay) obj.put("parallel_relay", cfg.parallelRelay)
        if (cfg.coalesceStepMs != defaults.coalesceStepMs) obj.put("coalesce_step_ms", cfg.coalesceStepMs)
        if (cfg.coalesceMaxMs != defaults.coalesceMaxMs) obj.put("coalesce_max_ms", cfg.coalesceMaxMs)
        if (cfg.upstreamSocks5.isNotBlank()) obj.put("upstream_socks5", cfg.upstreamSocks5)
        if (cfg.passthroughHosts.isNotEmpty()) obj.put("passthrough_hosts", JSONArray().apply { cfg.passthroughHosts.forEach { put(it) } })
        if (cfg.blockQuic) obj.put("block_quic", true)
        if (cfg.tunnelDoh) obj.put("tunnel_doh", true)
        if (cfg.bypassDohHosts.isNotEmpty()) obj.put("bypass_doh_hosts", JSONArray().apply { cfg.bypassDohHosts.forEach { put(it) } })
        if (cfg.frontingGroupsJson.isNotBlank()) obj.put("fronting_groups", JSONArray(cfg.frontingGroupsJson))
        val jsonBytes = obj.toString().toByteArray(Charsets.UTF_8)
        val compressed = java.io.ByteArrayOutputStream().also { bos ->
            java.util.zip.DeflaterOutputStream(bos).use { it.write(jsonBytes) }
        }.toByteArray()
        val b64 = android.util.Base64.encodeToString(
            compressed,
            android.util.Base64.NO_WRAP or android.util.Base64.URL_SAFE,
        )
        return "$HASH_PREFIX$b64"
    }
    fun decode(encoded: String): MhrvConfig? {
        val trimmed = encoded.trim()
        if (trimmed.startsWith("{")) {
            return try { loadFromJson(JSONObject(trimmed)) } catch (_: Throwable) { null }
        }
        val payload = when {
            trimmed.startsWith(HASH_PREFIX) -> trimmed.removePrefix(HASH_PREFIX)
            trimmed.startsWith(LEGACY_HASH_PREFIX) -> trimmed.removePrefix(LEGACY_HASH_PREFIX)
            else -> trimmed
        }
        return try {
            val raw = android.util.Base64.decode(
                payload,
                android.util.Base64.NO_WRAP or android.util.Base64.URL_SAFE,
            )
            val text = try {
                java.util.zip.InflaterInputStream(raw.inputStream()).bufferedReader().readText()
            } catch (_: Throwable) {
                String(raw, Charsets.UTF_8)
            }
            loadFromJson(JSONObject(text))
        } catch (_: Throwable) {
            null
        }
    }
    internal fun loadFromJson(obj: JSONObject): MhrvConfig {
        val accountGroups = projectAccountGroups(obj)
        val sni = obj.optJSONArray("sni_hosts")?.let { arr ->
            buildList { for (i in 0 until arr.length()) add(arr.optString(i)) }
        }?.filter { it.isNotBlank() }.orEmpty()
        return MhrvConfig(
            mode = when (obj.optString("mode", "apps_script")) {
                "vercel_edge" -> Mode.SERVERLESS_JSON
                "direct", "google_only" -> Mode.DIRECT
                "full" -> Mode.FULL
                else -> Mode.APPS_SCRIPT
            },
            listenHost = obj.optString("listen_host", "127.0.0.1"),
            listenPort = obj.optInt("listen_port", 8080),
            socks5Port = obj.optInt("socks5_port", 1081).takeIf { it > 0 },
            appsScriptUrls = accountGroups.urls,
            authKey = accountGroups.authKey,
            preservedAccountGroupsJson = accountGroups.preservedJson,
            serverlessBaseUrl = obj.optJSONObject("vercel")?.optString("base_url", "") ?: "",
            serverlessAuthKey = obj.optJSONObject("vercel")?.optString("auth_key", "") ?: "",
            serverlessRelayPath = obj.optJSONObject("vercel")?.optString("relay_path", DEFAULT_RELAY_PATH)
                ?.takeIf { it.isNotBlank() } ?: DEFAULT_RELAY_PATH,
            frontDomain = obj.optString("front_domain", "www.google.com"),
            sniHosts = sni,
            googleIp = obj.optString("google_ip", DEFAULT_ANDROID_GOOGLE_IP),
            verifySsl = obj.optBoolean("verify_ssl", true),
            logLevel = obj.optString("log_level", "info"),
            parallelRelay = obj.optInt("parallel_relay", 1),
            coalesceStepMs = obj.optInt("coalesce_step_ms", 40),
            coalesceMaxMs = obj.optInt("coalesce_max_ms", 1000),
            upstreamSocks5 = obj.optString("upstream_socks5", ""),
            passthroughHosts = obj.optJSONArray("passthrough_hosts")?.let { arr ->
                buildList { for (i in 0 until arr.length()) add(arr.optString(i)) }
            }?.filter { it.isNotBlank() }.orEmpty(),
            blockQuic = obj.optBoolean("block_quic", false),
            tunnelDoh = obj.optBoolean("tunnel_doh", true),
            bypassDohHosts = obj.optJSONArray("bypass_doh_hosts")?.let { arr ->
                buildList { for (i in 0 until arr.length()) add(arr.optString(i)) }
            }?.filter { it.isNotBlank() }.orEmpty(),
            frontingGroupsJson = obj.optJSONArray("fronting_groups")?.toString() ?: "",
            connectionMode = when (obj.optString("connection_mode", "vpn_tun")) {
                "proxy_only" -> ConnectionMode.PROXY_ONLY
                else -> ConnectionMode.VPN_TUN
            },
            splitMode = when (obj.optString("split_mode", "all")) {
                "only" -> SplitMode.ONLY
                "except" -> SplitMode.EXCEPT
                else -> SplitMode.ALL
            },
            splitApps = obj.optJSONArray("split_apps")?.let { arr ->
                buildList { for (i in 0 until arr.length()) add(arr.optString(i)) }
            }?.filter { it.isNotBlank() }.orEmpty(),
            uiLang = when (obj.optString("ui_lang", "auto")) {
                "fa" -> UiLang.FA
                "en" -> UiLang.EN
                else -> UiLang.AUTO
            },
        )
    }
}
/**
 * Default SNI rotation pool. Mirrors `DEFAULT_GOOGLE_SNI_POOL` from the
 * Rust `domain_fronter` module — keep the lists in sync, or leave the
 * user's sniHosts empty and let Rust auto-expand.
 */
val DEFAULT_SNI_POOL: List<String> = listOf(
    "www.google.com",
    "mail.google.com",
    "drive.google.com",
    "docs.google.com",
    "calendar.google.com",
    // accounts.google.com (not accounts.googl.com — the typo domain is
    // not in Google's GFE cert SAN, so TLS validation fails with verify_ssl=true).
    "accounts.google.com",
    // Same DPI-passing profile on some Iranian mobile networks.
    "scholar.google.com",
    // More rotation for DPI-fingerprint spread; a couple of
    // SNIs (maps/play) that pass DPI where shorter *.google.com names don't.
    "maps.google.com",
    "chat.google.com",
    "translate.google.com",
    "play.google.com",
    "lens.google.com",
    "chromewebstore.google.com",
)
