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

        assertTrue(snapshot.contains("schema: android-support-snapshot/v1"))
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
}
