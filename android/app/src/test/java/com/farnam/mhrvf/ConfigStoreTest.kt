package com.farnam.mhrvf

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ConfigStoreTest {
    @Test
    fun toJsonWritesCanonicalAccountGroups() {
        val json = JSONObject(
            MhrvConfig(
                mode = Mode.APPS_SCRIPT,
                appsScriptUrls = listOf(
                    "https://script.google.com/macros/s/AKfycb_primary/exec",
                    "AKfycb_backup",
                ),
                authKey = "android-secret",
            ).toJson(),
        )

        assertFalse(json.has("script_ids"))
        assertFalse(json.has("auth_key"))

        val groups = json.getJSONArray("account_groups")
        assertEquals(1, groups.length())
        val primary = groups.getJSONObject(0)
        assertEquals("android-secret", primary.getString("auth_key"))
        assertEquals("AKfycb_primary", primary.getJSONArray("script_ids").getString(0))
        assertEquals("AKfycb_backup", primary.getJSONArray("script_ids").getString(1))
        assertTrue(primary.getBoolean("enabled"))
    }

    @Test
    fun loadFromJsonReadsCanonicalAccountGroups() {
        val cfg = ConfigStore.loadFromJson(
            JSONObject(
                """
                {
                  "mode": "full",
                  "account_groups": [
                    {
                      "label": "primary",
                      "auth_key": "desktop-secret",
                      "script_ids": ["AKfycb_one", "AKfycb_two"],
                      "weight": 2,
                      "enabled": true
                    }
                  ]
                }
                """.trimIndent(),
            ),
        )

        assertEquals(Mode.FULL, cfg.mode)
        assertEquals("desktop-secret", cfg.authKey)
        assertEquals(
            listOf(
                "https://script.google.com/macros/s/AKfycb_one/exec",
                "https://script.google.com/macros/s/AKfycb_two/exec",
            ),
            cfg.appsScriptUrls,
        )
        assertTrue(cfg.preservedAccountGroupsJson.contains("\"weight\":2"))
    }

    @Test
    fun fullModeToJsonWritesCanonicalAccountGroups() {
        val json = JSONObject(
            MhrvConfig(
                mode = Mode.FULL,
                appsScriptUrls = listOf("AKfycb_full"),
                authKey = "full-secret",
            ).toJson(),
        )

        assertEquals("full", json.getString("mode"))
        assertFalse(json.has("script_ids"))
        assertFalse(json.has("auth_key"))
        assertEquals(
            "full-secret",
            json.getJSONArray("account_groups").getJSONObject(0).getString("auth_key"),
        )
    }

    @Test
    fun loadFromJsonStillReadsLegacyTopLevelScriptIds() {
        val cfg = ConfigStore.loadFromJson(
            JSONObject(
                """
                {
                  "mode": "apps_script",
                  "script_ids": ["AKfycb_legacy"],
                  "auth_key": "legacy-secret"
                }
                """.trimIndent(),
            ),
        )

        assertEquals("legacy-secret", cfg.authKey)
        assertEquals(listOf("https://script.google.com/macros/s/AKfycb_legacy/exec"), cfg.appsScriptUrls)
        assertEquals("", cfg.preservedAccountGroupsJson)
    }

    @Test
    fun importedMultiGroupConfigPreservesUneditedGroups() {
        val imported = ConfigStore.loadFromJson(
            JSONObject(
                """
                {
                  "mode": "apps_script",
                  "account_groups": [
                    {
                      "label": "primary",
                      "auth_key": "old-primary-secret",
                      "script_ids": ["AKfycb_primary"],
                      "weight": 1,
                      "enabled": true
                    },
                    {
                      "label": "backup",
                      "auth_key": "backup-secret",
                      "script_ids": ["AKfycb_backup"],
                      "weight": 5,
                      "enabled": false
                    }
                  ]
                }
                """.trimIndent(),
            ),
        )

        val saved = JSONObject(
            imported.copy(
                authKey = "new-primary-secret",
                appsScriptUrls = listOf("AKfycb_primary_new"),
            ).toJson(),
        )
        val groups = saved.getJSONArray("account_groups")

        assertEquals(2, groups.length())
        assertEquals("new-primary-secret", groups.getJSONObject(0).getString("auth_key"))
        assertEquals("AKfycb_primary_new", groups.getJSONObject(0).getJSONArray("script_ids").getString(0))
        assertEquals("backup", groups.getJSONObject(1).getString("label"))
        assertEquals("backup-secret", groups.getJSONObject(1).getString("auth_key"))
        assertEquals(5, groups.getJSONObject(1).getInt("weight"))
        assertFalse(groups.getJSONObject(1).getBoolean("enabled"))
    }

    @Test
    fun minimalShareJsonUsesCanonicalAccountGroups() {
        val shared = JSONObject(
            MhrvConfig(
                appsScriptUrls = listOf("AKfycb_share"),
                authKey = "share-secret",
            ).toJson(),
        )

        assertFalse(shared.has("script_ids"))
        assertFalse(shared.has("auth_key"))
        assertEquals(
            "share-secret",
            shared.getJSONArray("account_groups").getJSONObject(0).getString("auth_key"),
        )
    }
}
