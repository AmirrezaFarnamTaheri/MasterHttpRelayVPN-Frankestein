package com.farnam.mhrvf.ui

import android.widget.Toast
import androidx.annotation.StringRes
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.ErrorOutline
import androidx.compose.material.icons.filled.ExpandLess
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.HourglassBottom
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.farnam.mhrvf.CaInstall
import com.farnam.mhrvf.ConfigStore
import com.farnam.mhrvf.DEFAULT_SNI_POOL
import com.farnam.mhrvf.MhrvConfig
import com.farnam.mhrvf.Mode
import com.farnam.mhrvf.Native
import com.farnam.mhrvf.ConnectionMode
import com.farnam.mhrvf.NetworkDetect
import com.farnam.mhrvf.R
import com.farnam.mhrvf.ReadinessIds
import com.farnam.mhrvf.ReadinessRepairAnchors
import com.farnam.mhrvf.ReadinessRepairTargets
import com.farnam.mhrvf.SplitMode
import com.farnam.mhrvf.UiLang
import com.farnam.mhrvf.VpnState
import androidx.compose.ui.res.stringResource
import com.farnam.mhrvf.ui.theme.ErrRed
import com.farnam.mhrvf.ui.theme.OkGreen
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import org.json.JSONObject

@Composable
private fun WelcomeIntroCard() {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.55f),
        tonalElevation = 1.dp,
        shadowElevation = 0.dp,
    ) {
        Text(
            text = stringResource(R.string.screen_intro_welcome),
            modifier = Modifier.padding(horizontal = 18.dp, vertical = 14.dp),
            style = MaterialTheme.typography.bodyMedium.copy(lineHeight = 22.sp),
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
private fun ModeOverviewCard(mode: Mode, connectionMode: ConnectionMode) {
    val (title, body, accent) = when (mode) {
        Mode.APPS_SCRIPT -> Triple(
            "Apps Script relay",
            "Use deployment IDs plus auth_key, install the user CA, then route through VPN or proxy-only.",
            MaterialTheme.colorScheme.primary,
        )
        Mode.SERVERLESS_JSON -> Triple(
            "Serverless JSON relay",
            "Use the Vercel/Netlify Base URL, AUTH_KEY, and /api/api route. This is no-VPS but still uses the user CA.",
            MaterialTheme.colorScheme.tertiary,
        )
        Mode.DIRECT -> Triple(
            "Direct fronting",
            "No relay credentials. Uses Google-edge SNI rewrite plus any config-file fronting_groups for Vercel/Fastly/Netlify targets.",
            MaterialTheme.colorScheme.primaryContainer,
        )
        Mode.FULL -> Triple(
            "Full tunnel",
            "Use Apps Script plus tunnel-node on your VPS. No local user CA is required in this mode.",
            MaterialTheme.colorScheme.secondary,
        )
    }
    val route = when (connectionMode) {
        ConnectionMode.VPN_TUN -> "Device VPN routing is enabled; use App splitting for per-app control."
        ConnectionMode.PROXY_ONLY -> "Proxy-only is enabled; apps must opt in to HTTP/SOCKS manually."
    }
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.42f),
        border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant),
    ) {
        Column(
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 14.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Text(title, style = MaterialTheme.typography.titleSmall, color = accent, fontWeight = FontWeight.SemiBold)
            Text(body, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurface)
            Text(route, style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
private fun SectionHint(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 2.dp, vertical = 0.dp)
            .padding(bottom = 6.dp),
    )
}

private data class AndroidReadinessItem(
    val id: String,
    val label: String,
    val ok: Boolean,
    val detail: String,
    val blocksConnect: Boolean = true,
)

private data class AndroidReadinessRepair(
    @StringRes val labelRes: Int,
    val target: String,
    val anchor: String?,
    @StringRes val bodyRes: Int,
)

private fun isLanBoundHost(host: String): Boolean =
    host.trim() == "0.0.0.0" || host.trim() == "::"

private fun parseLanAllowlist(text: String): List<String> =
    text.split('\n', ',', ';', ' ', '\t')
        .map { it.trim() }
        .filter { it.isNotEmpty() }
        .distinct()

private fun androidRepairForId(id: String): AndroidReadinessRepair? {
    val target = ReadinessRepairTargets.targetForId(id) ?: return null
    val anchor = ReadinessRepairAnchors.anchorForTarget(target)?.android
    return when (id) {
        ReadinessIds.ACCOUNT_GROUPS_ENABLED,
        ReadinessIds.ACCOUNT_GROUPS_SCRIPT_IDS -> AndroidReadinessRepair(
            labelRes = R.string.repair_account_groups_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_account_groups_body,
        )
        ReadinessIds.ACCOUNT_GROUPS_AUTH_KEY -> AndroidReadinessRepair(
            labelRes = R.string.repair_account_auth_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_account_auth_body,
        )
        ReadinessIds.VERCEL_BASE_URL -> AndroidReadinessRepair(
            labelRes = R.string.repair_serverless_base_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_serverless_base_body,
        )
        ReadinessIds.VERCEL_RELAY_PATH -> AndroidReadinessRepair(
            labelRes = R.string.repair_serverless_path_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_serverless_path_body,
        )
        ReadinessIds.VERCEL_AUTH_KEY -> AndroidReadinessRepair(
            labelRes = R.string.repair_serverless_auth_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_serverless_auth_body,
        )
        ReadinessIds.DIRECT_GOOGLE_IP -> AndroidReadinessRepair(
            labelRes = R.string.repair_direct_ip_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_direct_ip_body,
        )
        ReadinessIds.DIRECT_FRONT_DOMAIN -> AndroidReadinessRepair(
            labelRes = R.string.repair_direct_sni_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_direct_sni_body,
        )
        ReadinessIds.LOCAL_LISTENER,
        ReadinessIds.LOCAL_PORTS -> AndroidReadinessRepair(
            labelRes = R.string.repair_local_listener_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_local_listener_body,
        )
        ReadinessIds.CA_TRUST -> AndroidReadinessRepair(
            labelRes = R.string.repair_ca_trust_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_ca_trust_body,
        )
        ReadinessIds.ANDROID_APP_CA_TRUST -> AndroidReadinessRepair(
            labelRes = R.string.repair_android_app_ca_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_android_app_ca_body,
        )
        ReadinessIds.LAN_EXPOSURE -> AndroidReadinessRepair(
            labelRes = R.string.repair_lan_exposure_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_lan_exposure_body,
        )
        ReadinessIds.LAN_TOKEN -> AndroidReadinessRepair(
            labelRes = R.string.repair_lan_token_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_lan_token_body,
        )
        ReadinessIds.LAN_ALLOWLIST -> AndroidReadinessRepair(
            labelRes = R.string.repair_lan_allowlist_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_lan_allowlist_body,
        )
        ReadinessIds.FULL_CODEFULL_DEPLOYMENT -> AndroidReadinessRepair(
            labelRes = R.string.repair_full_codefull_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_full_codefull_body,
        )
        ReadinessIds.FULL_TUNNEL_NODE_URL -> AndroidReadinessRepair(
            labelRes = R.string.repair_full_node_url_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_full_node_url_body,
        )
        ReadinessIds.FULL_TUNNEL_AUTH -> AndroidReadinessRepair(
            labelRes = R.string.repair_full_auth_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_full_auth_body,
        )
        ReadinessIds.FULL_UDP_SUPPORT -> AndroidReadinessRepair(
            labelRes = R.string.repair_full_udp_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_full_udp_body,
        )
        ReadinessIds.FULL_TUNNEL_HEALTH -> AndroidReadinessRepair(
            labelRes = R.string.repair_full_health_label,
            target = target,
            anchor = anchor,
            bodyRes = R.string.repair_full_health_body,
        )
        else -> null
    }
}

private fun androidReadinessItems(cfg: MhrvConfig, caInstalled: Boolean): List<AndroidReadinessItem> {
    val items = mutableListOf<AndroidReadinessItem>()
    when (cfg.mode) {
        Mode.APPS_SCRIPT, Mode.FULL -> {
            items += AndroidReadinessItem(
                id = ReadinessIds.ACCOUNT_GROUPS_SCRIPT_IDS,
                label = "Deployment IDs",
                ok = cfg.hasDeploymentId,
                detail = if (cfg.hasDeploymentId) {
                    "${cfg.appsScriptUrls.size} deployment ID(s)"
                } else {
                    "Add at least one Apps Script deployment URL or ID."
                },
            )
            items += AndroidReadinessItem(
                id = ReadinessIds.ACCOUNT_GROUPS_AUTH_KEY,
                label = "AUTH_KEY",
                ok = cfg.authKey.isNotBlank(),
                detail = if (cfg.authKey.isNotBlank()) {
                    "Configured for the primary Android-editable group."
                } else {
                    "Must match AUTH_KEY inside Code.gs or CodeFull.gs."
                },
            )
        }
        Mode.SERVERLESS_JSON -> {
            items += AndroidReadinessItem(
                id = ReadinessIds.VERCEL_BASE_URL,
                label = "Serverless origin",
                ok = cfg.serverlessBaseUrl.trim().startsWith("http://") ||
                    cfg.serverlessBaseUrl.trim().startsWith("https://"),
                detail = cfg.serverlessBaseUrl.ifBlank {
                    "Paste the Vercel or Netlify site origin."
                },
            )
            items += AndroidReadinessItem(
                id = ReadinessIds.VERCEL_RELAY_PATH,
                label = "Relay path",
                ok = cfg.serverlessRelayPath.trim().startsWith("/"),
                detail = cfg.serverlessRelayPath.ifBlank { "/api/api" },
            )
            items += AndroidReadinessItem(
                id = ReadinessIds.VERCEL_AUTH_KEY,
                label = "AUTH_KEY",
                ok = cfg.serverlessAuthKey.isNotBlank(),
                detail = if (cfg.serverlessAuthKey.isNotBlank()) {
                    "Configured for the JSON relay."
                } else {
                    "Must match the serverless AUTH_KEY environment variable."
                },
            )
        }
        Mode.DIRECT -> {
            items += AndroidReadinessItem(
                id = ReadinessIds.DIRECT_GOOGLE_IP,
                label = "Google edge IP",
                ok = cfg.googleIp.isBlank() || cfg.googleIp.parseAsIpOrNull() != null,
                detail = cfg.googleIp.ifBlank { "Auto-detected on connect when possible." },
            )
            items += AndroidReadinessItem(
                id = ReadinessIds.DIRECT_FRONT_DOMAIN,
                label = "Front SNI",
                ok = cfg.frontDomain.isBlank() || cfg.frontDomain.parseAsIpOrNull() == null,
                detail = cfg.frontDomain.ifBlank { "Defaults to www.google.com on connect." },
            )
        }
    }
    items += AndroidReadinessItem(
        id = ReadinessIds.ANDROID_CONNECTION_MODE,
        label = "Routing mode",
        ok = true,
        detail = when (cfg.connectionMode) {
            ConnectionMode.VPN_TUN -> "VPN/TUN captures eligible apps automatically."
            ConnectionMode.PROXY_ONLY -> "Proxy-only requires each app or Wi-Fi profile to opt in."
        },
        blocksConnect = false,
    )
    if (isLanBoundHost(cfg.listenHost)) {
        val allowlistCount = cfg.lanAllowlist.map { it.trim() }.filter { it.isNotEmpty() }.distinct().size
        val hasToken = cfg.lanToken.isNotBlank()
        val hasAllowlist = allowlistCount > 0
        items += AndroidReadinessItem(
            id = ReadinessIds.LAN_EXPOSURE,
            label = "LAN exposure",
            ok = false,
            detail = "Proxy is shared on ${cfg.listenHost}; local-network devices can reach it when Wi-Fi/firewall allows.",
            blocksConnect = false,
        )
        items += AndroidReadinessItem(
            id = ReadinessIds.LAN_TOKEN,
            label = "LAN access guard",
            ok = hasToken || hasAllowlist,
            detail = when {
                hasToken -> "HTTP/CONNECT token configured."
                hasAllowlist -> "$allowlistCount allowlist entries configured."
                else -> "Set a LAN token or allowed IPs before sharing HTTP/CONNECT on LAN."
            },
            blocksConnect = false,
        )
        if (cfg.socks5Port != null) {
            items += AndroidReadinessItem(
                id = ReadinessIds.LAN_ALLOWLIST,
                label = "SOCKS5 LAN allowlist",
                ok = hasAllowlist,
                detail = if (hasAllowlist) {
                    "$allowlistCount allowlist entries configured."
                } else {
                    "SOCKS5 cannot carry token headers; add allowed IPs before exposing it on LAN."
                },
                blocksConnect = false,
            )
        }
    }
    if (cfg.mode != Mode.FULL) {
        items += AndroidReadinessItem(
            id = ReadinessIds.CA_TRUST,
            label = "Local CA trust",
            ok = caInstalled,
            detail = if (caInstalled) {
                "Generated CA is present in the Android user credential store."
            } else {
                "Install and trust the generated CA before routing HTTPS clients."
            },
            blocksConnect = false,
        )
        items += AndroidReadinessItem(
            id = ReadinessIds.ANDROID_APP_CA_TRUST,
            label = "Android app CA trust",
            ok = false,
            detail = "Android 7+ apps may ignore user CAs unless they opt in; browsers and apps vary.",
            blocksConnect = false,
        )
    } else {
        items += AndroidReadinessItem(
            id = ReadinessIds.FULL_CODEFULL_DEPLOYMENT,
            label = "CodeFull deployment",
            ok = false,
            detail = "Verify each full-mode Apps Script deployment uses CodeFull.gs.",
            blocksConnect = false,
        )
        items += AndroidReadinessItem(
            id = ReadinessIds.FULL_TUNNEL_NODE_URL,
            label = "Tunnel-node URL",
            ok = false,
            detail = "CodeFull.gs must point at the public tunnel-node origin.",
            blocksConnect = false,
        )
        items += AndroidReadinessItem(
            id = ReadinessIds.FULL_TUNNEL_AUTH,
            label = "Tunnel auth",
            ok = false,
            detail = "TUNNEL_AUTH_KEY must match between CodeFull.gs and tunnel-node.",
            blocksConnect = false,
        )
        items += AndroidReadinessItem(
            id = ReadinessIds.FULL_UDP_SUPPORT,
            label = "UDP/SOCKS5 path",
            ok = cfg.socks5Port != null,
            detail = if (cfg.socks5Port != null) {
                "SOCKS5 listener configured for UDP-capable clients."
            } else {
                "Set a SOCKS5 port if apps need UDP ASSOCIATE in full mode."
            },
            blocksConnect = false,
        )
        items += AndroidReadinessItem(
            id = ReadinessIds.FULL_TUNNEL_HEALTH,
            label = "Tunnel health",
            ok = false,
            detail = "Check /healthz, tunnel-node logs, and public-IP verification.",
            blocksConnect = false,
        )
    }
    return items
}

private fun androidConnectBlockerId(cfg: MhrvConfig): String? =
    androidReadinessItems(cfg, caInstalled = true).firstOrNull { !it.ok && it.blocksConnect }?.id

@Composable
private fun ModeReadinessCard(cfg: MhrvConfig, caInstalled: Boolean) {
    val readiness = remember(cfg, caInstalled) { androidReadinessItems(cfg, caInstalled) }
    val allBlockersReady = readiness.none { !it.ok && it.blocksConnect }
    val hasWarnings = readiness.any { !it.ok && !it.blocksConnect }
    var selectedRepair by remember { mutableStateOf<AndroidReadinessRepair?>(null) }
    ElevatedCard(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.elevatedCardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.42f),
        ),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "Selected-mode readiness",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.SemiBold,
                )
                AssistChip(
                    onClick = {},
                    enabled = false,
                    label = { Text(if (!allBlockersReady) "blocked" else if (hasWarnings) "check" else "ready") },
                    leadingIcon = {
                        Icon(
                            imageVector = if (allBlockersReady && !hasWarnings) Icons.Filled.CheckCircle else Icons.Filled.ErrorOutline,
                            contentDescription = null,
                        )
                    },
                )
            }
            readiness.forEach { item ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalAlignment = Alignment.Top,
                ) {
                    Icon(
                        imageVector = if (item.ok) Icons.Filled.CheckCircle else Icons.Filled.ErrorOutline,
                        contentDescription = null,
                        tint = if (item.ok) OkGreen else if (item.blocksConnect) ErrRed else MaterialTheme.colorScheme.tertiary,
                        modifier = Modifier.size(20.dp),
                    )
                    Column(Modifier.weight(1f)) {
                        Text(
                            text = item.label,
                            style = MaterialTheme.typography.bodyMedium,
                            fontWeight = FontWeight.SemiBold,
                        )
                        Text(
                            text = "${item.id}: ${item.detail}",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        val repair = androidRepairForId(item.id)
                        if (!item.ok && repair != null) {
                            TextButton(
                                onClick = { selectedRepair = repair },
                                contentPadding = PaddingValues(horizontal = 0.dp, vertical = 0.dp),
                            ) {
                                Text(stringResource(R.string.btn_fix))
                            }
                        }
                    }
                }
            }
            if (cfg.preservedAccountGroupsJson.isNotBlank()) {
                HorizontalDivider()
                Text(
                    text = stringResource(R.string.warn_preserved_account_groups),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.tertiary,
                )
            }
        }
    }
    selectedRepair?.let { repair ->
        AlertDialog(
            onDismissRequest = { selectedRepair = null },
            title = { Text(stringResource(R.string.repair_dialog_title)) },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        text = stringResource(repair.labelRes),
                        style = MaterialTheme.typography.bodyMedium,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Text(
                        text = stringResource(repair.bodyRes),
                        style = MaterialTheme.typography.bodySmall,
                    )
                    Text(
                        text = stringResource(R.string.repair_target, repair.target),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    repair.anchor?.let { anchor ->
                        Text(
                            text = stringResource(R.string.repair_anchor, anchor),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.primary,
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(onClick = { selectedRepair = null }) {
                    Text(stringResource(R.string.btn_close))
                }
            },
        )
    }
}

/**
 * UI state returned by the Activity after the CA install flow finishes,
 * so the screen can show a matching snackbar. Kept as a sum type — a raw
 * string message would conflate "installed" vs. "failed to export".
 */
sealed class CaInstallOutcome {
    object Installed : CaInstallOutcome()
    /**
     * Cert not found in the AndroidCAStore after the Settings activity
     * returned. Carries an optional downloadPath so the snackbar can tell
     * the user where the file landed (Downloads or app-private external).
     */
    data class NotInstalled(val downloadPath: String?) : CaInstallOutcome()
    data class Failed(val message: String) : CaInstallOutcome()
}

/**
 * Top-level screen. Intentionally one scrollable page rather than tabs —
 * first-run users need to see everything (deployment IDs, cert button,
 * Start) on one surface. Anything that isn't first-run critical lives in
 * collapsible sections (SNI pool, Advanced, Logs) so the default view
 * stays short.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    onStart: () -> Unit,
    onStop: () -> Unit,
    onInstallCaConfirmed: () -> Unit,
    caOutcome: CaInstallOutcome?,
    onCaOutcomeConsumed: () -> Unit,
    onLangChange: (UiLang) -> Unit = {},
) {
    val ctx = LocalContext.current
    val scope = rememberCoroutineScope()
    val snackbar = remember { SnackbarHostState() }

    // Persisted form state. Any edit writes back to disk immediately —
    // cheap at this write rate, avoids "I tapped Start before saving" bugs.
    var cfg by remember { mutableStateOf(ConfigStore.load(ctx)) }
    fun persist(new: MhrvConfig) {
        cfg = new
        ConfigStore.save(ctx, new)
    }

    // CA install dialog visibility.
    var showInstallDialog by rememberSaveable { mutableStateOf(false) }
    val caInstalled = remember(caOutcome) { CaInstall.isInstalled(ctx) }

    // One-shot auto update check on first composition. Silent if we're
    // already on the latest (no point nagging about a network miss or an
    // up-to-date install); surfaces a snackbar only when a newer tag is
    // available. rememberSaveable so it doesn't re-fire on every config
    // change / rotation.
    var autoUpdateChecked by rememberSaveable { mutableStateOf(false) }
    LaunchedEffect(autoUpdateChecked) {
        if (autoUpdateChecked) return@LaunchedEffect
        autoUpdateChecked = true
        val json = withContext(Dispatchers.IO) {
            runCatching { Native.checkUpdate() }.getOrNull()
        }
        if (json != null) {
            val obj = runCatching { JSONObject(json) }.getOrNull()
            if (obj?.optString("kind") == "updateAvailable") {
                snackbar.showSnackbar(
                    ctx.getString(
                        R.string.snack_update_available,
                        obj.optString("current"),
                        obj.optString("latest"),
                        obj.optString("url"),
                    ),
                    withDismissAction = true,
                )
            }
        }
    }

    // Gate Start/Stop on the service's actual state transition instead of a
    // fixed timer. Teardown can outlive a short cooldown while native sockets
    // are still being released, so wait until VpnState settles, with a backstop.
    var awaitingRunning by remember { mutableStateOf<Boolean?>(null) }
    val transitioning = awaitingRunning != null
    LaunchedEffect(awaitingRunning) {
        val target = awaitingRunning ?: return@LaunchedEffect
        try {
            withTimeoutOrNull(12_000) {
                VpnState.isRunning.first { it == target }
            }
        } finally {
            awaitingRunning = null
        }
    }

    // Surface CA install result as a snackbar. We consume the outcome
    // after showing so a recomposition doesn't re-trigger it.
    LaunchedEffect(caOutcome) {
        val o = caOutcome ?: return@LaunchedEffect
        val msg = when (o) {
            is CaInstallOutcome.Installed ->
                ctx.getString(R.string.snack_cert_installed)
            is CaInstallOutcome.NotInstalled -> buildString {
                append(ctx.getString(R.string.snack_cert_not_installed))
                if (!o.downloadPath.isNullOrBlank()) {
                    append(" ")
                    append(ctx.getString(R.string.snack_cert_saved_to, o.downloadPath))
                    append(" ")
                    append(ctx.getString(R.string.snack_cert_settings_hint))
                } else {
                    append(" ")
                    append(ctx.getString(R.string.snack_cert_tap_install_again))
                }
            }
            is CaInstallOutcome.Failed -> o.message
        }
        snackbar.showSnackbar(msg, withDismissAction = true)
        onCaOutcomeConsumed()
    }

    @Composable
    fun ConnectActionButton() {
        SectionHint(stringResource(R.string.help_before_connect))
        val isVpnRunning by VpnState.isRunning.collectAsState()
        val connectBlockerId = remember(cfg) { androidConnectBlockerId(cfg) }
        val canConnect = connectBlockerId == null
        Button(
            onClick = {
                if (isVpnRunning) {
                    awaitingRunning = false
                    onStop()
                } else {
                    awaitingRunning = true
                    scope.launch {
                        var updated = cfg
                        if (updated.googleIp.isBlank()) {
                            val fresh = withContext(Dispatchers.IO) {
                                NetworkDetect.resolveGoogleIp()
                            }
                            if (!fresh.isNullOrBlank()) {
                                updated = updated.copy(googleIp = fresh)
                            }
                        }
                        if (updated.frontDomain.isBlank() ||
                            updated.frontDomain.parseAsIpOrNull() != null
                        ) {
                            updated = updated.copy(frontDomain = "www.google.com")
                        }
                        if (updated !== cfg) persist(updated)
                        onStart()
                    }
                }
            },
            enabled = (isVpnRunning || canConnect) && !transitioning,
            colors = ButtonDefaults.buttonColors(
                containerColor = if (isVpnRunning) ErrRed else OkGreen,
                contentColor = androidx.compose.ui.graphics.Color.White,
                disabledContainerColor = MaterialTheme.colorScheme.surfaceVariant,
            ),
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 52.dp),
        ) {
            Text(
                when {
                    transitioning -> "..."
                    isVpnRunning -> stringResource(R.string.btn_disconnect)
                    else -> stringResource(R.string.btn_save_and_connect)
                },
                style = MaterialTheme.typography.titleMedium,
            )
        }
        if (!isVpnRunning && connectBlockerId != null) {
            Text(
                text = stringResource(R.string.connect_blocked_by, connectBlockerId),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(top = 6.dp),
            )
        }
    }

    Scaffold(
        topBar = {
            CenterAlignedTopAppBar(
                title = {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(
                            text = stringResource(R.string.app_name),
                            style = MaterialTheme.typography.titleSmall,
                            fontWeight = FontWeight.SemiBold,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                            lineHeight = 18.sp,
                        )
                        Text(
                            text = stringResource(R.string.tb_tagline),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                },
                actions = {
                    // Language toggle — cycles AUTO → FA → EN → AUTO.
                    // Saving writes to config.json and triggers activity
                    // recreate, which re-applies the AppCompatDelegate
                    // locale (and flips LTR ↔ RTL accordingly). Kept as
                    // a small label button instead of an icon because
                    // "AUTO/FA/EN" communicates the current state at a
                    // glance; a flag icon alone would be ambiguous.
                    TextButton(
                        onClick = {
                            val next = when (cfg.uiLang) {
                                UiLang.AUTO -> UiLang.FA
                                UiLang.FA -> UiLang.EN
                                UiLang.EN -> UiLang.AUTO
                            }
                            persist(cfg.copy(uiLang = next))
                            onLangChange(next)
                        },
                    ) {
                        Text(
                            text = when (cfg.uiLang) {
                                UiLang.AUTO -> "AUTO"
                                UiLang.FA -> "FA"
                                UiLang.EN -> "EN"
                            },
                            style = MaterialTheme.typography.labelSmall,
                        )
                    }

                    // Tap the version label to check for updates.
                    var checking by remember { mutableStateOf(false) }
                    TextButton(
                        onClick = {
                            if (checking) return@TextButton
                            checking = true
                            scope.launch {
                                val json = withContext(Dispatchers.IO) {
                                    runCatching { Native.checkUpdate() }.getOrNull()
                                }
                                val msg = summarizeUpdateCheck(ctx, json)
                                snackbar.showSnackbar(msg, withDismissAction = true)
                                checking = false
                            }
                        },
                        modifier = Modifier.padding(end = 4.dp),
                    ) {
                        Text(
                            text = if (checking) stringResource(R.string.tb_check_update_checking)
                                   else stringResource(R.string.tb_version_prefix) +
                                        runCatching { Native.version() }.getOrDefault("?"),
                            style = MaterialTheme.typography.labelMedium,
                        )
                    }
                },
            )
        },
        snackbarHost = { SnackbarHost(snackbar) },
    ) { inner ->
        Column(
            modifier = Modifier
                .padding(inner)
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 18.dp, vertical = 16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            WelcomeIntroCard()

            SectionHeader(stringResource(R.string.field_mode))
            SectionHint(stringResource(R.string.help_section_mode))
            ModeDropdown(
                mode = cfg.mode,
                onChange = { persist(cfg.copy(mode = it)) },
            )
            ModeOverviewCard(mode = cfg.mode, connectionMode = cfg.connectionMode)
            ModeReadinessCard(cfg, caInstalled)
            ConnectActionButton()

            Spacer(Modifier.height(4.dp))
            val appsScriptEnabled = cfg.mode == Mode.APPS_SCRIPT || cfg.mode == Mode.FULL
            val appsScriptNeedsSetup = cfg.appsScriptUrls.isEmpty() || cfg.authKey.isBlank()
            CollapsibleSection(
                title = stringResource(R.string.sec_apps_script_relay),
                initiallyExpanded = appsScriptEnabled && appsScriptNeedsSetup,
            ) {
                SectionHint(stringResource(R.string.help_section_relay))
                DeploymentIdsField(
                    urls = cfg.appsScriptUrls,
                    onChange = { persist(cfg.copy(appsScriptUrls = it)) },
                    enabled = appsScriptEnabled,
                )

                OutlinedTextField(
                    value = cfg.authKey,
                    onValueChange = { persist(cfg.copy(authKey = it)) },
                    label = { Text(stringResource(R.string.field_auth_key)) },
                    singleLine = true,
                    enabled = appsScriptEnabled,
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                    modifier = Modifier.fillMaxWidth(),
                    supportingText = {
                        Text(stringResource(R.string.help_auth_key))
                    },
                )
            }

            val serverlessEnabled = cfg.mode == Mode.SERVERLESS_JSON
            val serverlessNeedsSetup = cfg.serverlessBaseUrl.isBlank() || cfg.serverlessAuthKey.isBlank()
            CollapsibleSection(
                title = stringResource(R.string.sec_serverless_json_relay),
                initiallyExpanded = serverlessEnabled && serverlessNeedsSetup,
            ) {
                SectionHint(stringResource(R.string.help_section_serverless_json))
                OutlinedTextField(
                    value = cfg.serverlessBaseUrl,
                    onValueChange = { persist(cfg.copy(serverlessBaseUrl = it)) },
                    label = { Text(stringResource(R.string.field_serverless_base_url)) },
                    singleLine = true,
                    enabled = serverlessEnabled,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri, imeAction = ImeAction.Next),
                    modifier = Modifier.fillMaxWidth(),
                    supportingText = { Text(stringResource(R.string.help_serverless_base_url)) },
                )
                OutlinedTextField(
                    value = cfg.serverlessAuthKey,
                    onValueChange = { persist(cfg.copy(serverlessAuthKey = it)) },
                    label = { Text(stringResource(R.string.field_serverless_auth_key)) },
                    singleLine = true,
                    enabled = serverlessEnabled,
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                    modifier = Modifier.fillMaxWidth(),
                    supportingText = { Text(stringResource(R.string.help_serverless_auth_key)) },
                )
                OutlinedTextField(
                    value = cfg.serverlessRelayPath,
                    onValueChange = { persist(cfg.copy(serverlessRelayPath = it.ifBlank { "/api/api" })) },
                    label = { Text(stringResource(R.string.field_serverless_relay_path)) },
                    singleLine = true,
                    enabled = serverlessEnabled,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri, imeAction = ImeAction.Next),
                    modifier = Modifier.fillMaxWidth(),
                    supportingText = { Text(stringResource(R.string.help_serverless_relay_path)) },
                )
            }

            Spacer(Modifier.height(4.dp))
            SectionHeader(stringResource(R.string.sec_network))
            SectionHint(stringResource(R.string.help_section_network))

            ConnectionModeDropdown(
                mode = cfg.connectionMode,
                onChange = { persist(cfg.copy(connectionMode = it)) },
                httpPort = cfg.listenPort,
                socks5Port = cfg.socks5Port ?: (cfg.listenPort + 1),
            )

            Column(
                modifier = Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                OutlinedTextField(
                    value = cfg.googleIp,
                    onValueChange = { persist(cfg.copy(googleIp = it)) },
                    label = { Text(stringResource(R.string.field_google_ip)) },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri),
                    modifier = Modifier.fillMaxWidth(),
                    supportingText = { Text(stringResource(R.string.help_google_ip)) },
                )
                OutlinedTextField(
                    value = cfg.frontDomain,
                    onValueChange = { persist(cfg.copy(frontDomain = it)) },
                    label = { Text(stringResource(R.string.field_front_domain)) },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri),
                    modifier = Modifier.fillMaxWidth(),
                    supportingText = { Text(stringResource(R.string.help_front_domain)) },
                )
            }
            // "Auto-detect" forces a fresh DNS resolution now. Start also
            // auto-resolves transparently, but exposing a button makes the
            // "I'm getting connect timeouts, is my google_ip stale?" case
            // a one-tap fix without needing to look up nslookup output.
            TextButton(
                onClick = {
                    scope.launch {
                        val fresh = withContext(Dispatchers.IO) {
                            NetworkDetect.resolveGoogleIp()
                        }
                        if (!fresh.isNullOrBlank()) {
                            var updated = cfg
                            if (fresh != updated.googleIp) {
                                updated = updated.copy(googleIp = fresh)
                            }
                            // Same repair logic as the Start button —
                            // if front_domain has been corrupted into an
                            // IP we can't use it for SNI, so put the
                            // default hostname back.
                            if (updated.frontDomain.isBlank() ||
                                updated.frontDomain.parseAsIpOrNull() != null
                            ) {
                                updated = updated.copy(frontDomain = "www.google.com")
                            }
                            // Captured up-front so the lambda has access
                            // to the format-string resources via context
                            // before running on the IO dispatcher.
                            if (updated !== cfg) {
                                persist(updated)
                                snackbar.showSnackbar(
                                    ctx.getString(R.string.snack_google_ip_updated, fresh),
                                )
                            } else {
                                snackbar.showSnackbar(
                                    ctx.getString(R.string.snack_google_ip_current, fresh),
                                )
                            }
                        } else {
                            snackbar.showSnackbar(ctx.getString(R.string.snack_dns_lookup_failed))
                        }
                    }
                },
                modifier = Modifier.align(Alignment.End),
            ) { Text(stringResource(R.string.btn_auto_detect_google_ip)) }

            // App splitting — only makes sense in VPN_TUN mode.
            // PROXY_ONLY has no system-level routing to partition.
            if (cfg.connectionMode == ConnectionMode.VPN_TUN) {
                CollapsibleSection(title = stringResource(R.string.sec_app_splitting)) {
                    AppSplittingEditor(cfg = cfg, onChange = ::persist)
                }
            }

            // SNI pool: collapsed by default. Users without a reason to
            // touch it should leave Rust's auto-expansion to handle it.
            CollapsibleSection(title = stringResource(R.string.sec_sni_pool_tester)) {
                SniPoolEditor(
                    cfg = cfg,
                    onChange = ::persist,
                )
            }

            // Advanced settings: collapsed by default.
            CollapsibleSection(title = stringResource(R.string.sec_advanced)) {
                AdvancedSettings(
                    cfg = cfg,
                    onChange = ::persist,
                )
            }

            Spacer(Modifier.height(4.dp))
            // Secondary accent button — FilledTonalButton reads as a lower-
            // priority action next to Start/Stop, matching the desktop UI's
            // visual hierarchy where Install CA is offered as a helper
            // button rather than the headline action.
            FilledTonalButton(
                onClick = { showInstallDialog = true },
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(stringResource(R.string.btn_install_mitm))
            }

            UsageTodayCard()

            CollapsibleSection(title = stringResource(R.string.sec_live_logs), initiallyExpanded = false) {
                LiveLogPane()
            }

            Spacer(Modifier.height(16.dp))
            // Wrapped in a collapsible so the big prose block doesn't
            // dominate the form after the user has learned the flow.
            // Starts expanded once for a fresh install so the first-run
            // instructions are immediately visible.
            CollapsibleSection(
                title = stringResource(R.string.sec_how_to_use),
                initiallyExpanded = cfg.appsScriptUrls.isEmpty() || cfg.authKey.isBlank(),
            ) {
                HowToUseBody(
                    httpPort = cfg.listenPort,
                    socks5Port = cfg.socks5Port ?: (cfg.listenPort + 1),
                )
            }
        }
    }

    // ---- CA install confirmation dialog ---------------------------------
    if (showInstallDialog) {
        // Export eagerly so we can show the fingerprint in the dialog body
        // — builds user confidence ("yes, that's the cert I'm trusting")
        // and gives us a usable failure path if the CA doesn't exist yet.
        val exported = remember { CaInstall.export(ctx) }
        val fp = remember(exported) { if (exported) CaInstall.fingerprint(ctx) else null }
        val cn = remember(exported) { if (exported) CaInstall.subjectCn(ctx) else null }

        AlertDialog(
            onDismissRequest = { showInstallDialog = false },
            title = { Text(stringResource(R.string.dialog_install_mitm_title)) },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(stringResource(R.string.dialog_install_mitm_intro))
                    Text(stringResource(R.string.dialog_install_mitm_steps))
                    if (fp != null) {
                        Text("Subject: ${cn ?: "(unknown)"}", style = MaterialTheme.typography.labelMedium)
                        Text(
                            text = "SHA-256: ${CaInstall.fingerprintHex(fp)}",
                            style = MaterialTheme.typography.labelSmall,
                            fontFamily = FontFamily.Monospace,
                        )
                    } else {
                        Text(
                            stringResource(R.string.dialog_install_mitm_no_cert),
                            color = MaterialTheme.colorScheme.error,
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        showInstallDialog = false
                        if (fp != null) onInstallCaConfirmed()
                    },
                    enabled = fp != null,
                ) { Text(stringResource(R.string.btn_install)) }
            },
            dismissButton = {
                TextButton(onClick = { showInstallDialog = false }) {
                    Text(stringResource(R.string.btn_cancel))
                }
            },
        )
    }
}

// =========================================================================
// App splitting — ALL / ONLY / EXCEPT, plus a picker for the package list.
// =========================================================================

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AppSplittingEditor(
    cfg: MhrvConfig,
    onChange: (MhrvConfig) -> Unit,
) {
    val ctx = LocalContext.current
    var pickerOpen by remember { mutableStateOf(false) }

    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(
            stringResource(R.string.help_app_splitting),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        // Radio-style mode selector. Using Column-of-Row-with-RadioButton
        // instead of a dropdown because all three options deserve to be
        // visible simultaneously — the labels explain the contract.
        SplitModeRow(
            label = stringResource(R.string.split_all),
            selected = cfg.splitMode == SplitMode.ALL,
            onClick = { onChange(cfg.copy(splitMode = SplitMode.ALL)) },
        )
        SplitModeRow(
            label = stringResource(R.string.split_only),
            selected = cfg.splitMode == SplitMode.ONLY,
            onClick = { onChange(cfg.copy(splitMode = SplitMode.ONLY)) },
        )
        SplitModeRow(
            label = stringResource(R.string.split_except),
            selected = cfg.splitMode == SplitMode.EXCEPT,
            onClick = { onChange(cfg.copy(splitMode = SplitMode.EXCEPT)) },
        )

        if (cfg.splitMode != SplitMode.ALL) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(
                    stringResource(R.string.sni_selected_count, cfg.splitApps.size),
                    style = MaterialTheme.typography.labelMedium,
                    modifier = Modifier.weight(1f),
                )
                TextButton(onClick = { pickerOpen = true }) {
                    Text(stringResource(R.string.split_pick_apps))
                }
            }
        }
    }

    if (pickerOpen) {
        AppPickerDialog(
            initial = cfg.splitApps.toSet(),
            ownPackage = ctx.packageName,
            onSave = { picked ->
                onChange(cfg.copy(splitApps = picked))
                pickerOpen = false
            },
            onDismiss = { pickerOpen = false },
        )
    }
}

@Composable
private fun SplitModeRow(label: String, selected: Boolean, onClick: () -> Unit) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.fillMaxWidth(),
    ) {
        RadioButton(selected = selected, onClick = onClick)
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier.weight(1f),
        )
    }
}

// =========================================================================
// Connection mode — VPN (TUN) vs Proxy-only.
// =========================================================================

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ConnectionModeDropdown(
    mode: ConnectionMode,
    onChange: (ConnectionMode) -> Unit,
    httpPort: Int,
    socks5Port: Int,
) {
    val labelVpn = stringResource(R.string.mode_vpn_tun)
    val labelProxy = stringResource(R.string.mode_proxy_only)
    val currentLabel = when (mode) {
        ConnectionMode.VPN_TUN -> labelVpn
        ConnectionMode.PROXY_ONLY -> labelProxy
    }
    var expanded by remember { mutableStateOf(false) }

    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        ExposedDropdownMenuBox(
            expanded = expanded,
            onExpandedChange = { expanded = !expanded },
        ) {
            OutlinedTextField(
                value = currentLabel,
                onValueChange = {},
                readOnly = true,
                label = { Text(stringResource(R.string.field_connection_mode)) },
                trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = expanded) },
                modifier = Modifier.fillMaxWidth().menuAnchor(),
            )
            ExposedDropdownMenu(
                expanded = expanded,
                onDismissRequest = { expanded = false },
            ) {
                DropdownMenuItem(
                    text = { Text(labelVpn) },
                    onClick = {
                        onChange(ConnectionMode.VPN_TUN)
                        expanded = false
                    },
                )
                DropdownMenuItem(
                    text = { Text(labelProxy) },
                    onClick = {
                        onChange(ConnectionMode.PROXY_ONLY)
                        expanded = false
                    },
                )
            }
        }

        // Helper text under the dropdown explains what the user is
        // signing up for in each mode — especially important for
        // PROXY_ONLY, where "tap Connect" alone doesn't route anything
        // until they set the Wi-Fi proxy themselves.
        val help = when (mode) {
            ConnectionMode.VPN_TUN ->
                stringResource(R.string.help_mode_vpn_tun)
            ConnectionMode.PROXY_ONLY ->
                stringResource(R.string.help_mode_proxy_only, httpPort, socks5Port)
        }
        Text(
            help,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(
            stringResource(R.string.help_mode_overview),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.92f),
            modifier = Modifier.padding(top = 4.dp),
        )
    }
}

// =========================================================================
// Deployment IDs editor — one row per ID, with add/remove buttons.
// =========================================================================

/** Split a bulk-pasted blob into individual entries. */
private val ID_SEPARATORS = Regex("[\\s,;]+")

@Composable
private fun DeploymentIdsField(
    urls: List<String>,
    onChange: (List<String>) -> Unit,
    enabled: Boolean = true,
) {
    var newEntry by remember { mutableStateOf("") }

    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(
            stringResource(R.string.field_deployment_urls),
            style = MaterialTheme.typography.labelLarge,
        )

        // Existing entries — each with its own row and a remove button.
        urls.forEachIndexed { index, url ->
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth(),
            ) {
                OutlinedTextField(
                    value = url,
                    onValueChange = { edited ->
                        val parts = edited.split(ID_SEPARATORS).filter { it.isNotBlank() }
                        val updated = urls.toMutableList()
                        if (parts.size > 1) {
                            // Bulk paste into this row: expand in place.
                            updated.removeAt(index)
                            updated.addAll(index, parts)
                        } else {
                            // Normal typing — keep raw input stable for caret behavior.
                            updated[index] = edited
                        }
                        onChange(updated)
                    },
                    enabled = enabled,
                    modifier = Modifier.weight(1f),
                    singleLine = true,
                    textStyle = MaterialTheme.typography.bodySmall,
                    label = { Text("#${index + 1}") },
                )
                IconButton(
                    onClick = {
                        onChange(urls.filterIndexed { i, _ -> i != index })
                    },
                    enabled = enabled,
                ) {
                    Text("✕", color = MaterialTheme.colorScheme.error)
                }
            }
        }

        // "Add" row: multi-line text field + button (supports bulk paste).
        Row(
            verticalAlignment = Alignment.Top,
            modifier = Modifier.fillMaxWidth(),
        ) {
            OutlinedTextField(
                value = newEntry,
                onValueChange = { newEntry = it },
                enabled = enabled,
                modifier = Modifier.weight(1f),
                singleLine = false,
                minLines = 1,
                maxLines = 6,
                placeholder = { Text(stringResource(R.string.placeholder_paste_ids)) },
            )
            Spacer(Modifier.width(8.dp))
            Button(
                onClick = {
                    val parts = newEntry.split(ID_SEPARATORS).filter { it.isNotBlank() }
                    if (parts.isNotEmpty()) {
                        onChange(urls + parts)
                        newEntry = ""
                    }
                },
                enabled = enabled && newEntry.isNotBlank(),
                contentPadding = PaddingValues(horizontal = 12.dp),
            ) {
                Text("+ Add")
            }
        }

        Text(
            stringResource(R.string.help_deployment_urls),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

// =========================================================================
// Mode dropdown: selectable Rust backend modes.
// =========================================================================

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ModeDropdown(
    mode: Mode,
    onChange: (Mode) -> Unit,
) {
    val labelApps = "Apps Script (MITM)"
    val labelServerless = "Serverless JSON (no VPS)"
    val labelDirect = "Direct fronting (no relay)"
    val labelFull = "Full tunnel (no cert)"
    val currentLabel = when (mode) {
        Mode.APPS_SCRIPT -> labelApps
        Mode.SERVERLESS_JSON -> labelServerless
        Mode.DIRECT -> labelDirect
        Mode.FULL -> labelFull
    }
    var expanded by remember { mutableStateOf(false) }

    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        ExposedDropdownMenuBox(
            expanded = expanded,
            onExpandedChange = { expanded = !expanded },
        ) {
            OutlinedTextField(
                value = currentLabel,
                onValueChange = {},
                readOnly = true,
                label = { Text(stringResource(R.string.field_mode)) },
                trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = expanded) },
                modifier = Modifier.fillMaxWidth().menuAnchor(),
            )
            ExposedDropdownMenu(
                expanded = expanded,
                onDismissRequest = { expanded = false },
            ) {
                DropdownMenuItem(
                    text = { Text(labelApps) },
                    onClick = { onChange(Mode.APPS_SCRIPT); expanded = false },
                )
                DropdownMenuItem(
                    text = { Text(labelServerless) },
                    onClick = { onChange(Mode.SERVERLESS_JSON); expanded = false },
                )
                DropdownMenuItem(
                    text = { Text(labelDirect) },
                    onClick = { onChange(Mode.DIRECT); expanded = false },
                )
                DropdownMenuItem(
                    text = { Text(labelFull) },
                    onClick = { onChange(Mode.FULL); expanded = false },
                )
            }
        }

        val help = when (mode) {
            Mode.APPS_SCRIPT ->
                "Full DPI bypass through your deployed Apps Script relay."
            Mode.SERVERLESS_JSON ->
                "No-VPS JSON fetch relay hosted on Vercel or Netlify. Fill Base URL, AUTH_KEY, and keep /api/api unless you changed the tool route."
            Mode.DIRECT ->
                "SNI-rewrite only, no relay. Reach Google setup pages, plus configured fronting_groups such as Vercel, Fastly, and Netlify/CloudFront."
            Mode.FULL ->
                "All traffic tunneled end-to-end through Apps Script + remote tunnel node. No certificate needed."
        }
        Text(
            help,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

// =========================================================================
// SNI pool editor + per-SNI probe.
// =========================================================================

private sealed class ProbeState {
    object Idle : ProbeState()
    object InFlight : ProbeState()
    data class Ok(val latencyMs: Int) : ProbeState()
    data class Err(val message: String) : ProbeState()
}

@Composable
private fun SniPoolEditor(
    cfg: MhrvConfig,
    onChange: (MhrvConfig) -> Unit,
) {
    val scope = rememberCoroutineScope()

    // Build the displayed list: union of the default pool + the config's
    // sniHosts + the current front_domain. Order: front_domain first,
    // defaults, then user customs. Deduped.
    val displayed: List<String> = remember(cfg) {
        val seen = linkedSetOf<String>()
        if (cfg.frontDomain.isNotBlank()) seen.add(cfg.frontDomain.trim())
        DEFAULT_SNI_POOL.forEach { seen.add(it) }
        cfg.sniHosts.forEach { if (it.isNotBlank()) seen.add(it.trim()) }
        seen.toList()
    }

    // A host is enabled if it appears in cfg.sniHosts. Empty sniHosts
    // means "let Rust auto-expand" — we reflect that as "default pool
    // enabled, customs not".
    val enabledSet: Set<String> = remember(cfg.sniHosts) {
        if (cfg.sniHosts.isNotEmpty()) cfg.sniHosts.toSet()
        else DEFAULT_SNI_POOL.toSet() + setOfNotNull(cfg.frontDomain.takeIf { it.isNotBlank() })
    }

    val probeState = remember { mutableStateMapOf<String, ProbeState>() }

    fun probe(sni: String) {
        probeState[sni] = ProbeState.InFlight
        scope.launch {
            val json = withContext(Dispatchers.IO) {
                runCatching { Native.testSni(cfg.googleIp, sni) }.getOrNull()
            }
            probeState[sni] = parseProbeResult(json)
        }
    }

    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(
            stringResource(R.string.help_sni_pool),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        displayed.forEach { sni ->
            val enabled = sni in enabledSet
            SniRow(
                sni = sni,
                enabled = enabled,
                state = probeState[sni] ?: ProbeState.Idle,
                onToggle = { nowEnabled ->
                    val next = if (nowEnabled) {
                        (cfg.sniHosts.takeIf { it.isNotEmpty() } ?: emptyList()) + sni
                    } else {
                        val current = if (cfg.sniHosts.isNotEmpty()) cfg.sniHosts else enabledSet.toList()
                        current.filter { it != sni }
                    }
                    onChange(cfg.copy(sniHosts = next.distinct()))
                },
                onTest = { probe(sni) },
            )
        }

        // Custom-add row.
        var custom by remember { mutableStateOf("") }
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(6.dp),
            modifier = Modifier.fillMaxWidth(),
        ) {
            OutlinedTextField(
                value = custom,
                onValueChange = { custom = it },
                label = { Text(stringResource(R.string.field_add_custom_sni)) },
                // Accept a pasted list so users can dump a
                // whole list of subdomains in one go. We split on newlines,
                // commas, semicolons, and whitespace so formats like
                //   www.google.com\nmail.google.com\ndrive.google.com
                //   www.google.com, mail.google.com
                //   www.google.com mail.google.com
                // all do the right thing on Add.
                singleLine = false,
                maxLines = 6,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri),
                modifier = Modifier.weight(1f),
            )
            TextButton(
                onClick = {
                    // Tokenise on any whitespace, comma, or semicolon so one
                    // Add click absorbs a pasted list. Deduplicate within
                    // the paste before merging into the existing list.
                    val tokens = custom.split(Regex("[\\s,;]+"))
                        .map { it.trim() }
                        .filter { it.isNotEmpty() }
                    if (tokens.isNotEmpty()) {
                        val base = cfg.sniHosts.takeIf { it.isNotEmpty() } ?: enabledSet.toList()
                        val next = (base + tokens).distinct()
                        onChange(cfg.copy(sniHosts = next))
                        custom = ""
                    }
                },
                enabled = custom.isNotBlank(),
            ) { Text(stringResource(R.string.btn_add)) }
        }

        TextButton(
            onClick = { displayed.forEach { probe(it) } },
            modifier = Modifier.align(Alignment.End),
        ) { Text(stringResource(R.string.btn_test_all)) }
    }
}

@Composable
private fun SniRow(
    sni: String,
    enabled: Boolean,
    state: ProbeState,
    onToggle: (Boolean) -> Unit,
    onTest: () -> Unit,
) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Checkbox(checked = enabled, onCheckedChange = onToggle)
            Text(
                sni,
                modifier = Modifier.weight(1f),
                style = MaterialTheme.typography.bodyMedium,
            )
            ProbeBadge(state)
            Spacer(Modifier.width(4.dp))
            TextButton(onClick = onTest, enabled = state !is ProbeState.InFlight) {
                Text(stringResource(R.string.btn_test))
            }
        }
        // Show the error reason on its own line when the probe failed —
        // a red dot with no explanation was confusing ("SNI test also
        // fails despite having internet"). Common reasons: "dns: ..." or
        // "connect: ...".
        if (state is ProbeState.Err) {
            Text(
                text = state.message,
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.labelSmall,
                modifier = Modifier.padding(start = 48.dp, bottom = 4.dp),
            )
        }
    }
}

@Composable
private fun ProbeBadge(state: ProbeState) {
    when (state) {
        is ProbeState.Idle -> {}
        is ProbeState.InFlight -> {
            CircularProgressIndicator(
                modifier = Modifier.size(14.dp),
                strokeWidth = 2.dp,
            )
        }
        is ProbeState.Ok -> {
            Row(verticalAlignment = Alignment.CenterVertically) {
                // Same green the desktop UI uses for OK status (OK_GREEN
                // in src/bin/ui.rs line 510) — kept in sync via Theme.kt.
                Icon(
                    Icons.Default.CheckCircle, null,
                    tint = OkGreen,
                    modifier = Modifier.size(16.dp),
                )
                Spacer(Modifier.width(2.dp))
                Text("${state.latencyMs} ms", style = MaterialTheme.typography.labelSmall)
            }
        }
        is ProbeState.Err -> {
            Icon(
                Icons.Default.ErrorOutline, state.message,
                tint = MaterialTheme.colorScheme.error,
                modifier = Modifier.size(16.dp),
            )
        }
    }
}

/**
 * Turn the JSON blob from `Native.checkUpdate()` into a one-line
 * snackbar message. Parsing is lenient — if the shape is anything other
 * than what we expect we fall back to "check failed" rather than
 * spewing the raw JSON at the user.
 */
private fun summarizeUpdateCheck(ctx: android.content.Context, json: String?): String {
    if (json.isNullOrBlank()) return ctx.getString(R.string.update_check_failed_no_response)
    return try {
        val obj = JSONObject(json)
        when (obj.optString("kind")) {
            "upToDate" -> ctx.getString(R.string.update_check_up_to_date, obj.optString("current"))
            "updateAvailable" -> {
                val cur = obj.optString("current")
                val latest = obj.optString("latest")
                val url = obj.optString("url")
                ctx.getString(R.string.snack_update_available, cur, latest, url)
            }
            "offline" -> ctx.getString(R.string.update_check_offline, obj.optString("reason", "no details"))
            "error" -> ctx.getString(R.string.update_check_error, obj.optString("reason", "no details"))
            else -> ctx.getString(R.string.update_check_failed_unknown_response)
        }
    } catch (_: Throwable) {
        ctx.getString(R.string.update_check_failed_bad_json)
    }
}

/**
 * Try to parse a string as an IPv4 or IPv6 literal. Returns null if it
 * looks like a hostname (or bogus) — which is what we want for
 * front_domain, where a hostname is required (goes into the TLS SNI on
 * the outbound leg).
 *
 * Intentionally strict: must be a valid literal AND must not contain a
 * letter anywhere. Plain `InetAddress.getByName(...)` would succeed for
 * hostnames too (it'd do a DNS lookup and return an IP), which would
 * false-positive every normal value like "www.google.com".
 */
private fun String.parseAsIpOrNull(): java.net.InetAddress? {
    val s = trim()
    if (s.isEmpty() || s.any { it.isLetter() }) return null
    return try {
        // Literal-only parse: rejects anything that would need DNS.
        java.net.InetAddress.getByName(s).takeIf {
            it.hostAddress?.let { addr -> addr == s || addr.contains(s) } == true
        }
    } catch (_: Throwable) {
        null
    }
}

private fun parseProbeResult(json: String?): ProbeState {
    if (json.isNullOrBlank()) return ProbeState.Err("no response")
    return try {
        val obj = JSONObject(json)
        if (obj.optBoolean("ok", false)) {
            ProbeState.Ok(obj.optInt("latencyMs", -1))
        } else {
            ProbeState.Err(obj.optString("error", "failed"))
        }
    } catch (_: Throwable) {
        ProbeState.Err("bad json")
    }
}

// =========================================================================
// Advanced settings.
// =========================================================================

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AdvancedSettings(
    cfg: MhrvConfig,
    onChange: (MhrvConfig) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        // verify_ssl
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(stringResource(R.string.adv_verify_tls), style = MaterialTheme.typography.bodyMedium)
                Text(
                    stringResource(R.string.adv_verify_tls_help),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Switch(
                checked = cfg.verifySsl,
                onCheckedChange = { onChange(cfg.copy(verifySsl = it)) },
            )
        }

        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(stringResource(R.string.adv_lan_sharing), style = MaterialTheme.typography.bodyMedium)
                Text(
                    stringResource(R.string.adv_lan_sharing_help),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Switch(
                checked = cfg.listenHost == "0.0.0.0",
                onCheckedChange = { enabled ->
                    onChange(cfg.copy(listenHost = if (enabled) "0.0.0.0" else "127.0.0.1"))
                },
            )
        }

        OutlinedTextField(
            value = cfg.lanToken,
            onValueChange = { onChange(cfg.copy(lanToken = it)) },
            label = { Text(stringResource(R.string.adv_lan_token)) },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            supportingText = {
                Text(stringResource(R.string.adv_lan_token_help))
            },
        )

        OutlinedTextField(
            value = cfg.lanAllowlist.joinToString("\n"),
            onValueChange = { onChange(cfg.copy(lanAllowlist = parseLanAllowlist(it))) },
            label = { Text(stringResource(R.string.adv_lan_allowlist)) },
            minLines = 2,
            maxLines = 4,
            modifier = Modifier.fillMaxWidth(),
            supportingText = {
                Text(stringResource(R.string.adv_lan_allowlist_help))
            },
        )

        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(stringResource(R.string.adv_youtube_via_relay), style = MaterialTheme.typography.bodyMedium)
                Text(
                    stringResource(R.string.adv_youtube_via_relay_help),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Switch(
                checked = cfg.youtubeViaRelay,
                onCheckedChange = { onChange(cfg.copy(youtubeViaRelay = it)) },
            )
        }

        // log_level dropdown
        var expanded by remember { mutableStateOf(false) }
        val levels = listOf("trace", "debug", "info", "warn", "error", "off")
        ExposedDropdownMenuBox(
            expanded = expanded,
            onExpandedChange = { expanded = !expanded },
        ) {
            OutlinedTextField(
                value = cfg.logLevel,
                onValueChange = {},
                readOnly = true,
                label = { Text(stringResource(R.string.adv_log_level)) },
                trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = expanded) },
                modifier = Modifier.fillMaxWidth().menuAnchor(),
            )
            ExposedDropdownMenu(
                expanded = expanded,
                onDismissRequest = { expanded = false },
            ) {
                levels.forEach { lvl ->
                    DropdownMenuItem(
                        text = { Text(lvl) },
                        onClick = {
                            onChange(cfg.copy(logLevel = lvl))
                            expanded = false
                        },
                    )
                }
            }
        }

        // parallel_relay slider
        Column {
            Text(
                stringResource(R.string.adv_parallel_relay, cfg.parallelRelay),
                style = MaterialTheme.typography.bodyMedium,
            )
            Slider(
                value = cfg.parallelRelay.toFloat(),
                onValueChange = { onChange(cfg.copy(parallelRelay = it.toInt().coerceIn(1, 5))) },
                valueRange = 1f..5f,
                steps = 3,  // yields 1,2,3,4,5 positions
            )
            Text(
                stringResource(R.string.adv_parallel_relay_help),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        Column {
            Text(
                "Coalesce step: ${cfg.coalesceStepMs}ms",
                style = MaterialTheme.typography.bodyMedium,
            )
            Slider(
                value = cfg.coalesceStepMs.toFloat(),
                onValueChange = {
                    onChange(cfg.copy(coalesceStepMs = it.toInt().coerceIn(10, 500)))
                },
                valueRange = 10f..500f,
            )
        }

        Column {
            Text(
                "Coalesce max: ${cfg.coalesceMaxMs}ms",
                style = MaterialTheme.typography.bodyMedium,
            )
            Slider(
                value = cfg.coalesceMaxMs.toFloat(),
                onValueChange = {
                    onChange(cfg.copy(coalesceMaxMs = it.toInt().coerceIn(100, 2000)))
                },
                valueRange = 100f..2000f,
            )
        }

        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text("Block QUIC", style = MaterialTheme.typography.bodyMedium)
                Text(
                    "Drop UDP/443 so clients fall back to TCP.",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Switch(
                checked = cfg.blockQuic,
                onCheckedChange = { onChange(cfg.copy(blockQuic = it)) },
            )
        }

        OutlinedTextField(
            value = cfg.upstreamSocks5,
            onValueChange = { onChange(cfg.copy(upstreamSocks5 = it)) },
            label = { Text(stringResource(R.string.adv_upstream_socks5)) },
            placeholder = { Text("host:port") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            supportingText = {
                Text(stringResource(R.string.adv_upstream_socks5_help))
            },
        )
    }
}

// =========================================================================
// Live log pane — polls Native.drainLogs() on a 500ms tick.
// =========================================================================

@Composable
private fun LiveLogPane() {
    val lines = remember { mutableStateListOf<String>() }
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()
    val clipboard = LocalClipboardManager.current
    val ctx = LocalContext.current

    // Pull from the ring buffer periodically. We pull even while the
    // section is collapsed (cheap), so re-expanding shows fresh tail.
    LaunchedEffect(Unit) {
        while (true) {
            val blob = withContext(Dispatchers.IO) {
                runCatching { Native.drainLogs() }.getOrNull()
            }
            if (!blob.isNullOrEmpty()) {
                blob.split("\n").forEach { if (it.isNotBlank()) lines.add(it) }
                // Cap the visible list so we don't grow unboundedly.
                while (lines.size > 500) lines.removeAt(0)
                // Follow tail.
                if (lines.isNotEmpty()) {
                    listState.scrollToItem(lines.size - 1)
                }
            }
            delay(500)
        }
    }

    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                stringResource(R.string.logs_lines_count, lines.size),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.weight(1f),
            )
            TextButton(onClick = { lines.clear() }) { Text(stringResource(R.string.btn_clear)) }
            TextButton(
                enabled = lines.isNotEmpty(),
                onClick = {
                    clipboard.setText(AnnotatedString(lines.joinToString("\n")))
                    Toast.makeText(
                        ctx,
                        ctx.getString(R.string.snack_logs_copied),
                        Toast.LENGTH_SHORT,
                    ).show()
                },
            ) { Text(stringResource(R.string.btn_copy)) }
        }
        Surface(
            color = MaterialTheme.colorScheme.surfaceVariant,
            shape = RoundedCornerShape(8.dp),
            modifier = Modifier.fillMaxWidth().heightIn(min = 160.dp, max = 320.dp),
        ) {
            // SelectionContainer makes log lines selectable for manual copy of
            // partial ranges; for "copy everything" the Copy button above is
            // the reliable path.
            SelectionContainer {
                LazyColumn(
                    state = listState,
                    modifier = Modifier.padding(8.dp),
                ) {
                    items(lines) { line ->
                        Text(
                            line,
                            style = MaterialTheme.typography.bodySmall,
                            fontFamily = FontFamily.Monospace,
                            fontSize = 11.sp,
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun UsageTodayCard() {
    val freeQuotaPerDay = 20_000L
    val handle by VpnState.proxyHandle.collectAsState()
    val isRunning by VpnState.isRunning.collectAsState()
    if (!isRunning || handle == 0L) return

    var statsJson by remember { mutableStateOf("") }
    LaunchedEffect(handle) {
        statsJson = ""
        while (true) {
            statsJson = withContext(Dispatchers.IO) {
                runCatching { Native.statsJson(handle) }.getOrDefault("")
            }
            delay(1000)
        }
    }

    val obj = remember(statsJson) {
        if (statsJson.isBlank()) null else runCatching { JSONObject(statsJson) }.getOrNull()
    } ?: return

    val todayCalls = obj.optLong("today_calls", 0L)
    val todayBytes = obj.optLong("today_bytes", 0L)
    val resetSecs = obj.optLong("today_reset_secs", 0L)
    val pct = if (freeQuotaPerDay > 0) {
        (todayCalls.toDouble() / freeQuotaPerDay.toDouble()) * 100.0
    } else 0.0
    val ctx = LocalContext.current

    Spacer(Modifier.height(8.dp))
    ElevatedCard(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Text(
                stringResource(R.string.sec_usage_today),
                style = MaterialTheme.typography.titleSmall,
            )
            UsageRow(
                label = stringResource(R.string.label_calls_today),
                value = stringResource(
                    R.string.usage_calls_of_quota,
                    todayCalls,
                    freeQuotaPerDay,
                    pct,
                ),
            )
            UsageRow(
                label = stringResource(R.string.label_bytes_today),
                value = fmtBytes(todayBytes),
            )
            UsageRow(
                label = stringResource(R.string.label_resets_in),
                value = stringResource(
                    R.string.usage_resets_hm,
                    (resetSecs / 3600).toInt(),
                    ((resetSecs / 60) % 60).toInt(),
                ),
            )
            TextButton(
                onClick = {
                    val intent = android.content.Intent(
                        android.content.Intent.ACTION_VIEW,
                        android.net.Uri.parse("https://script.google.com/home/usage"),
                    )
                    intent.addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)
                    runCatching { ctx.startActivity(intent) }
                },
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(stringResource(R.string.btn_view_quota_on_google))
            }
            Text(
                stringResource(R.string.usage_today_note),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun UsageRow(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(
            label,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(
            value,
            style = MaterialTheme.typography.bodyMedium,
            fontFamily = FontFamily.Monospace,
        )
    }
}

private fun fmtBytes(b: Long): String {
    val k = 1024L
    val m = k * k
    val g = m * k
    return when {
        b >= g -> String.format("%.2f GB", b.toDouble() / g)
        b >= m -> String.format("%.2f MB", b.toDouble() / m)
        b >= k -> String.format("%.1f KB", b.toDouble() / k)
        else -> "$b B"
    }
}

// =========================================================================
// Small shared pieces.
// =========================================================================

@Composable
private fun SectionHeader(text: String) {
    Column(Modifier.fillMaxWidth().padding(top = 14.dp, bottom = 4.dp)) {
        Text(
            text = text,
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.primary,
            fontWeight = FontWeight.SemiBold,
        )
        Spacer(
            Modifier
                .fillMaxWidth()
                .height(1.dp)
                .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.35f)),
        )
    }
}

/**
 * Minimal disclosure widget. Compose has no stock "expandable card" in
 * Material3 yet, so we build it from a clickable header + AnimatedVisibility
 * wrapping the content.
 */
@Composable
private fun CollapsibleSection(
    title: String,
    initiallyExpanded: Boolean = false,
    content: @Composable ColumnScope.() -> Unit,
) {
    var expanded by rememberSaveable(title) { mutableStateOf(initiallyExpanded) }
    ElevatedCard(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.elevatedCardElevation(defaultElevation = 1.dp),
        colors = CardDefaults.elevatedCardColors(
            containerColor = MaterialTheme.colorScheme.surface,
        ),
    ) {
        Column(modifier = Modifier.padding(horizontal = 18.dp, vertical = 15.dp)) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(
                    title,
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.SemiBold,
                    modifier = Modifier.weight(1f),
                )
                TextButton(onClick = { expanded = !expanded }) {
                    Icon(
                        if (expanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                        contentDescription = if (expanded) "Collapse" else "Expand",
                    )
                }
            }
            AnimatedVisibility(visible = expanded) {
                Column(
                    modifier = Modifier.padding(top = 4.dp, bottom = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp),
                    content = content,
                )
            }
        }
    }
}

@Composable
private fun SectionTitle(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.titleSmall,
        fontWeight = FontWeight.SemiBold,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.padding(top = 10.dp),
    )
}

@Composable
private fun HowToUseBody(httpPort: Int, socks5Port: Int) {
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        SectionTitle(stringResource(R.string.guide_title_overview))
        Text(stringResource(R.string.guide_body_overview), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_first_run))
        Text(stringResource(R.string.guide_body_first_run), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_relay))
        Text(stringResource(R.string.guide_body_relay), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_groups))
        Text(stringResource(R.string.guide_body_groups), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_advanced_tuning))
        Text(stringResource(R.string.guide_body_advanced_tuning), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_certificate))
        Text(stringResource(R.string.guide_body_certificate), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_sni))
        Text(stringResource(R.string.guide_body_sni), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_vpn_flow))
        Text(stringResource(R.string.guide_body_vpn_flow), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_app_split))
        Text(stringResource(R.string.guide_body_app_split), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_proxy_only))
        Text(
            stringResource(R.string.guide_body_proxy_only, httpPort, socks5Port),
            style = MaterialTheme.typography.bodyMedium,
        )

        SectionTitle(stringResource(R.string.guide_title_troubleshoot))
        Text(stringResource(R.string.guide_body_troubleshoot), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_turnstile))
        Text(stringResource(R.string.guide_body_turnstile), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_docs))
        Text(stringResource(R.string.guide_body_docs), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_limits))
        Text(stringResource(R.string.guide_body_limits), style = MaterialTheme.typography.bodyMedium)

        SectionTitle(stringResource(R.string.guide_title_updates))
        Text(stringResource(R.string.guide_body_updates), style = MaterialTheme.typography.bodyMedium)
    }
}
