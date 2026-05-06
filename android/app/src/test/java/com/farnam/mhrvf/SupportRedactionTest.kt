package com.farnam.mhrvf

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SupportRedactionTest {
    @Test
    fun maskDeploymentIdNormalizesUrlsAndKeepsOnlyPrefixAndSuffix() {
        assertEquals(
            "AKfycb...cdef",
            maskDeploymentId("https://script.google.com/macros/s/AKfycb1234567890abcdef/exec"),
        )
        assertEquals("AKf...", maskDeploymentId("AKfy_short"))
        assertEquals("(blank)", maskDeploymentId("   "))
    }

    @Test
    fun androidSupportSnapshotOmitsSecretsAndMasksDeploymentIds() {
        val snapshot = androidSupportSnapshot(
            MhrvConfig(
                mode = Mode.APPS_SCRIPT,
                appsScriptUrls = listOf("https://script.google.com/macros/s/AKfycb1234567890abcdef/exec"),
                authKey = "android-secret",
                serverlessAuthKey = "serverless-secret",
                lanToken = "lan-secret-token",
                upstreamSocks5 = "user:pass@example.com:1080",
                preservedUnknownRootJson = """{"raw":"secret"}""",
            ),
            caInstalled = true,
        )

        assertTrue(snapshot.contains("schema: android-support-snapshot/v2"))
        assertTrue(snapshot.contains("apps_script_deployments_masked: AKfycb...cdef"))
        assertTrue(snapshot.contains("apps_script_auth_key_configured: yes"))
        assertTrue(snapshot.contains("serverless_auth_key_configured: yes"))
        assertTrue(snapshot.contains("lan_token_configured: yes"))
        assertTrue(snapshot.contains("upstream_socks5_configured: yes"))
        assertFalse(snapshot.contains("AKfycb1234567890abcdef"))
        assertFalse(snapshot.contains("android-secret"))
        assertFalse(snapshot.contains("serverless-secret"))
        assertFalse(snapshot.contains("lan-secret-token"))
        assertFalse(snapshot.contains("user:pass@example.com"))
        assertFalse(snapshot.contains("""{"raw":"secret"}"""))
    }

    @Test
    fun androidSupportSnapshotIncludesDoctorSummaryWithoutDetails() {
        val doctorJson = """
            {
              "ok": false,
              "items": [
                {"id":"config_file","level":"ok","title":"fine","detail":"safe","fix":null},
                {"id":"apps_script_urls","level":"warn","title":"URL warning","detail":"https://secret.example/path","fix":"paste secret deployment"},
                {"id":"auth_key","level":"fail","title":"secret title","detail":"android-secret","fix":"serverless-secret"}
              ]
            }
        """.trimIndent()
        val snapshot = androidSupportSnapshot(
            MhrvConfig(mode = Mode.APPS_SCRIPT),
            caInstalled = false,
            doctorJson = doctorJson,
        )

        assertTrue(snapshot.contains("doctor_available: yes"))
        assertTrue(snapshot.contains("doctor_ok: no"))
        assertTrue(snapshot.contains("doctor_items_total: 3"))
        assertTrue(snapshot.contains("doctor_items_ok: 1"))
        assertTrue(snapshot.contains("doctor_items_warn: 1"))
        assertTrue(snapshot.contains("doctor_items_fail: 1"))
        assertTrue(snapshot.contains("doctor_problem_ids: apps_script_urls, auth_key"))
        assertFalse(snapshot.contains("https://secret.example/path"))
        assertFalse(snapshot.contains("paste secret deployment"))
        assertFalse(snapshot.contains("android-secret"))
        assertFalse(snapshot.contains("serverless-secret"))
    }
}
