#!/usr/bin/env python3
"""Shared Android config drift allowlists.

Single source of truth for keys that are legitimate on Android but not (or not
only) as Rust `Config` registry roots. Imported by:

  - tools/check-android-config-keys.py (also uses NESTED_KEYS for nested JSON ops)
  - tools/check-android-owned-keys-list.py

When adding an Android-only root or legacy import key, update this module once.
"""

from __future__ import annotations

# Android wrapper-only roots (not part of Rust Config; serde ignores them).
ANDROID_ONLY_KEYS: frozenset[str] = frozenset(
    {
        "connection_mode",
        "split_mode",
        "split_apps",
        "ui_lang",
    }
)

# Legacy import-only roots (Rust migrates into canonical account_groups).
LEGACY_KEYS: frozenset[str] = frozenset(
    {
        "script_ids",
        "auth_key",
    }
)

# Nested-object keys Android touches under `vercel`, `account_groups[]`, etc.
# Not registry roots; validated via nested_fields / registry elsewhere.
NESTED_KEYS: frozenset[str] = frozenset(
    {
        "base_url",
        "relay_path",
        "verify_tls",
        "label",
        "weight",
        "enabled",
        "script_ids",
    }
)
