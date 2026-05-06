package com.farnam.mhrvf

import org.json.JSONArray
import org.json.JSONObject

private fun isLanBoundHost(host: String): Boolean =
    host.trim() == "0.0.0.0" || host.trim() == "::"

private fun yesNo(value: Boolean): String = if (value) "yes" else "no"

private data class DoctorSupportSummary(
    val ok: Boolean,
    val total: Int,
    val okCount: Int,
    val warnCount: Int,
    val failCount: Int,
    val problemIds: List<String>,
)

fun maskDeploymentId(value: String): String {
    val id = value
        .substringAfter("/macros/s/", value)
        .substringBefore("/")
        .substringBefore("?")
        .trim()
    if (id.isEmpty()) return "(blank)"
    return if (id.length <= 10) {
        "${id.take(3)}..."
    } else {
        "${id.take(6)}...${id.takeLast(4)}"
    }
}

private fun doctorSupportSummary(doctorJson: String?): DoctorSupportSummary? {
    if (doctorJson.isNullOrBlank()) return null
    return runCatching {
        val obj = JSONObject(doctorJson)
        val items: JSONArray = obj.optJSONArray("items") ?: JSONArray()
        var okCount = 0
        var warnCount = 0
        var failCount = 0
        val problemIds = mutableListOf<String>()
        for (i in 0 until items.length()) {
            val item = items.optJSONObject(i) ?: continue
            val level = item.optString("level").trim().lowercase()
            when (level) {
                "ok" -> okCount += 1
                "warn" -> {
                    warnCount += 1
                    item.optString("id").trim().takeIf { it.isNotEmpty() }?.let(problemIds::add)
                }
                "fail" -> {
                    failCount += 1
                    item.optString("id").trim().takeIf { it.isNotEmpty() }?.let(problemIds::add)
                }
            }
        }
        DoctorSupportSummary(
            ok = obj.optBoolean("ok", false),
            total = items.length(),
            okCount = okCount,
            warnCount = warnCount,
            failCount = failCount,
            problemIds = problemIds.distinct(),
        )
    }.getOrNull()
}

fun androidSupportSnapshot(
    cfg: MhrvConfig,
    caInstalled: Boolean,
    doctorJson: String? = null,
): String {
    val needsUserCa = cfg.mode != Mode.FULL
    val doctor = doctorSupportSummary(doctorJson)
    return buildString {
        appendLine("mhrv-f Android support snapshot (redacted)")
        appendLine("schema: android-support-snapshot/v2")
        appendLine("mode: ${cfg.mode}")
        appendLine("connection_mode: ${cfg.connectionMode}")
        appendLine("split_mode: ${cfg.splitMode}")
        appendLine("split_apps_count: ${cfg.splitApps.size}")
        appendLine("user_ca_required: ${yesNo(needsUserCa)}")
        appendLine("android_ca_installed: ${yesNo(caInstalled)}")
        appendLine("listen_host: ${cfg.listenHost}")
        appendLine("listen_port: ${cfg.listenPort}")
        appendLine("socks5_port: ${cfg.socks5Port ?: "disabled"}")
        appendLine("google_ip_set: ${yesNo(cfg.googleIp.isNotBlank())}")
        appendLine("front_domain_set: ${yesNo(cfg.frontDomain.isNotBlank())}")
        appendLine("apps_script_deployments_count: ${cfg.appsScriptUrls.size}")
        appendLine(
            "apps_script_deployments_masked: ${
                cfg.appsScriptUrls.joinToString(", ") { maskDeploymentId(it) }.ifBlank { "(none)" }
            }",
        )
        appendLine("apps_script_auth_key_configured: ${yesNo(cfg.authKey.isNotBlank())}")
        appendLine("account_groups_preserved: ${yesNo(cfg.preservedAccountGroupsJson.isNotBlank())}")
        appendLine("serverless_base_configured: ${yesNo(cfg.serverlessBaseUrl.isNotBlank())}")
        appendLine("serverless_auth_key_configured: ${yesNo(cfg.serverlessAuthKey.isNotBlank())}")
        appendLine("serverless_relay_path_set: ${yesNo(cfg.serverlessRelayPath.isNotBlank())}")
        appendLine("sni_hosts_count: ${cfg.sniHosts.size}")
        appendLine("verify_ssl: ${yesNo(cfg.verifySsl)}")
        appendLine("log_level: ${cfg.logLevel}")
        appendLine("parallel_relay: ${cfg.parallelRelay}")
        appendLine("youtube_via_relay: ${yesNo(cfg.youtubeViaRelay)}")
        appendLine("upstream_socks5_configured: ${yesNo(cfg.upstreamSocks5.isNotBlank())}")
        appendLine("lan_shared: ${yesNo(isLanBoundHost(cfg.listenHost))}")
        appendLine("lan_token_configured: ${yesNo(cfg.lanToken.isNotBlank())}")
        appendLine("lan_allowlist_count: ${cfg.lanAllowlist.size}")
        appendLine("passthrough_hosts_count: ${cfg.passthroughHosts.size}")
        appendLine("block_quic: ${yesNo(cfg.blockQuic)}")
        appendLine("tunnel_doh: ${yesNo(cfg.tunnelDoh)}")
        appendLine("bypass_doh_hosts_count: ${cfg.bypassDohHosts.size}")
        appendLine("fronting_groups_preserved: ${yesNo(cfg.frontingGroupsJson.isNotBlank())}")
        appendLine("unknown_root_fields_preserved: ${yesNo(cfg.preservedUnknownRootJson.isNotBlank())}")
        appendLine("doctor_available: ${yesNo(doctor != null)}")
        if (doctor != null) {
            appendLine("doctor_ok: ${yesNo(doctor.ok)}")
            appendLine("doctor_items_total: ${doctor.total}")
            appendLine("doctor_items_ok: ${doctor.okCount}")
            appendLine("doctor_items_warn: ${doctor.warnCount}")
            appendLine("doctor_items_fail: ${doctor.failCount}")
            appendLine(
                "doctor_problem_ids: ${
                    doctor.problemIds.joinToString(", ").ifBlank { "(none)" }
                }",
            )
        }
        appendLine("")
        appendLine("redaction_notes:")
        appendLine("- auth_key, serverless AUTH_KEY, LAN token, upstream SOCKS5, and raw unknown JSON are not included.")
        appendLine("- deployment IDs are masked.")
        appendLine("- Doctor details/fixes are summarized by item id only; review full Doctor output before sharing.")
        appendLine("- copied live logs are separate; review them before sharing.")
    }
}
