package com.farnam.mhrvf

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Test
import java.io.File

/**
 * Mirrors Rust `minimal_direct_json_matches_platform_defaults_contract` for the Android surface:
 * defaults must stay aligned with `docs/platform-defaults.json`.
 * Static CI: `tools/check-android-platform-defaults-test-static.py` — JVM: GitHub `android-unit-tests`.
 */
class PlatformDefaultsContractTest {

    private fun platformDefaultsJson(): JSONObject {
        var dir = File(System.getProperty("user.dir") ?: ".").canonicalFile
        repeat(8) {
            val candidate = File(dir, "docs/platform-defaults.json")
            if (candidate.isFile) {
                return JSONObject(candidate.readText(Charsets.UTF_8))
            }
            dir = dir.parentFile ?: break
        }
        throw AssertionError(
            "docs/platform-defaults.json not found (started from ${File(".").canonicalPath})",
        )
    }

    @Test
    fun mhrvConfigDefaultsMatchPlatformContractJson() {
        val contract = platformDefaultsJson()
        val shared = contract.getJSONObject("shared")
        val parity = contract.getJSONObject("parity_shared_defaults")
        val android = contract.getJSONObject("android")
        val d = MhrvConfig()

        assertEquals(android.getString("google_ip_default"), d.googleIp)
        assertEquals(android.getInt("listen_port_default"), d.listenPort)
        assertEquals(android.getInt("socks5_port_default"), d.socks5Port)
        assertEquals(android.getString("log_level_default"), d.logLevel)
        assertEquals(android.getInt("parallel_relay_default"), d.parallelRelay)
        assertEquals(android.getInt("coalesce_step_ms_default"), d.coalesceStepMs)
        assertEquals(android.getInt("coalesce_max_ms_default"), d.coalesceMaxMs)

        assertEquals(shared.getString("front_domain"), d.frontDomain)
        assertEquals(shared.getString("listen_host_loopback"), d.listenHost)

        assertEquals(parity.getBoolean("verify_ssl"), d.verifySsl)
        assertEquals(parity.getBoolean("youtube_via_relay"), d.youtubeViaRelay)
        assertEquals(parity.getBoolean("block_quic"), d.blockQuic)
        assertEquals(parity.getBoolean("tunnel_doh"), d.tunnelDoh)
        assertEquals(parity.getString("serverless_relay_path"), d.serverlessRelayPath)
    }

    @Test
    fun loadFromJsonMinimalDirectMatchesContractDefaults() {
        val contract = platformDefaultsJson()
        val shared = contract.getJSONObject("shared")
        val parity = contract.getJSONObject("parity_shared_defaults")
        val android = contract.getJSONObject("android")
        val cfg = ConfigStore.loadFromJson(JSONObject("""{"mode":"direct"}"""))

        assertEquals(android.getString("google_ip_default"), cfg.googleIp)
        assertEquals(android.getInt("listen_port_default"), cfg.listenPort)
        assertEquals(android.getInt("socks5_port_default"), cfg.socks5Port)
        assertEquals(android.getString("log_level_default"), cfg.logLevel)
        assertEquals(android.getInt("parallel_relay_default"), cfg.parallelRelay)
        assertEquals(android.getInt("coalesce_step_ms_default"), cfg.coalesceStepMs)
        assertEquals(android.getInt("coalesce_max_ms_default"), cfg.coalesceMaxMs)

        assertEquals(shared.getString("front_domain"), cfg.frontDomain)
        assertEquals(shared.getString("listen_host_loopback"), cfg.listenHost)

        assertEquals(parity.getBoolean("verify_ssl"), cfg.verifySsl)
        assertEquals(parity.getBoolean("youtube_via_relay"), cfg.youtubeViaRelay)
        assertEquals(parity.getBoolean("block_quic"), cfg.blockQuic)
        assertEquals(parity.getBoolean("tunnel_doh"), cfg.tunnelDoh)
        assertEquals(parity.getString("serverless_relay_path"), cfg.serverlessRelayPath)
    }
}
