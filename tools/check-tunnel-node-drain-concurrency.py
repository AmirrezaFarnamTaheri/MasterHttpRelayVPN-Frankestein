#!/usr/bin/env python3
"""Static guard for tunnel-node batch drain/concurrency invariants.

This gate protects the v1.9.9 absorption fixes that are easy to regress during
future tunnel-node edits:

- watcher tasks must be abort-on-drop so `select!` cancellation cannot detach
  stale notify waiters;
- batch `data` / drain paths must carry cloned session `Arc`s instead of
  holding the global sessions map across per-session awaits;
- mixed TCP+UDP waits must wake on either side, not pay the slower side's
  long-poll deadline;
- EOF cleanup must follow `drain_now` / `drain_udp_now` return values, not raw
  EOF atomics that may be true while over-cap tail bytes are still buffered.

It is intentionally static and conservative. The executable regression tests in
`tunnel-node` remain the deeper contract; this script makes local/CI repo-sanity
fail quickly if the high-risk code shape is accidentally undone.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "tunnel-node" / "src" / "main.rs"


def die(msg: str) -> None:
    print(f"tunnel-node drain/concurrency check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def find_fn(text: str, name: str) -> str:
    match = re.search(rf"(?:pub\s+)?async\s+fn\s+{re.escape(name)}\s*\(", text)
    if not match:
        die(f"missing async fn {name}")
    start = match.start()
    brace = text.find("{", match.end())
    if brace == -1:
        die(f"missing body for async fn {name}")
    depth = 0
    for idx in range(brace, len(text)):
        ch = text[idx]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return text[start : idx + 1]
    die(f"unterminated body for async fn {name}")


def check_abort_on_drop(text: str) -> None:
    require(text, "struct AbortOnDrop(tokio::task::JoinHandle<()>);", "AbortOnDrop wrapper")
    require(text, "impl Drop for AbortOnDrop", "AbortOnDrop Drop impl")
    require(text, "self.0.abort();", "AbortOnDrop abort call")

    for fn_name in ("wait_for_any_drainable", "wait_for_any_udp_drainable"):
        body = find_fn(text, fn_name)
        require(body, "let mut _watchers = Vec::with_capacity", f"{fn_name} watcher owner")
        require(body, "AbortOnDrop(tokio::spawn(async move", f"{fn_name} wrapped spawn")
        if re.search(r"let\s+mut\s+watchers\s*[:=]", body):
            die(f"{fn_name} reintroduced bare watcher handles")
        if re.search(r"\bJoinHandle\b", body):
            die(f"{fn_name} should not store bare JoinHandle values")
        require(body, "tokio::select!", f"{fn_name} notify/deadline select")


def check_handle_batch(text: str) -> None:
    body = find_fn(text, "handle_batch")

    require(
        body,
        "let mut tcp_drains: Vec<(usize, String, Arc<SessionInner>)> = Vec::new();",
        "TCP drains carrying Arc<SessionInner>",
    )
    require(
        body,
        "let mut udp_drains: Vec<(usize, String, Arc<UdpSessionInner>)> = Vec::new();",
        "UDP drains carrying Arc<UdpSessionInner>",
    )
    require(
        body,
        "sessions.get(&sid).map(|s| s.inner.clone())",
        "batch data path clones inner under sessions map lock",
    )
    require(body, "let mut w = inner.writer.lock().await;", "batch data path writes through cloned inner")
    require(body, "tokio::select!", "mixed TCP/UDP select")
    require(
        body,
        "_ = wait_for_any_drainable(&tcp_inners, deadline)",
        "TCP wait select arm",
    )
    require(
        body,
        "_ = wait_for_any_udp_drainable(&udp_inners, deadline)",
        "UDP wait select arm",
    )
    require(body, "match (tcp_inners.is_empty(), udp_inners.is_empty())", "empty-aware wait dispatch")

    require(body, "let mut tcp_eof_sids: Vec<String> = Vec::new();", "TCP EOF cleanup list")
    require(body, "let (data, eof) = drain_now(inner).await;", "TCP drain return drives EOF")
    require(body, "tcp_eof_sids.push(sid.clone());", "TCP EOF push from drain return")
    require(body, "for sid in &tcp_eof_sids", "TCP cleanup iterates returned EOF sids")
    require(body, "sessions.remove(sid)", "TCP cleanup removes returned EOF sids")

    require(body, "let mut udp_eof_sids: Vec<String> = Vec::new();", "UDP EOF cleanup list")
    require(body, "let (packets, eof) = drain_udp_now(inner).await;", "UDP drain return drives EOF")
    require(body, "udp_eof_sids.push(sid.clone());", "UDP EOF push from drain return")
    require(body, "for sid in &udp_eof_sids", "UDP cleanup iterates returned EOF sids")

    cleanup_start = body.find("let mut tcp_eof_sids")
    cleanup_end = body.find("results.sort_by_key", cleanup_start)
    cleanup = body[cleanup_start:cleanup_end]
    if ".eof.load(Ordering::Acquire)" in cleanup:
        die("batch EOF cleanup must not read raw eof atomics")
    if "for (_, sid) in &tcp_drains" in cleanup or "for (_, sid) in &udp_drains" in cleanup:
        die("batch cleanup must not iterate all drains and re-check raw EOF")

    if "tokio::join!(\n            wait_for_any_drainable" in body:
        die("mixed TCP/UDP wait reintroduced conjunctive tokio::join!")


def check_single_data_path(text: str) -> None:
    body = find_fn(text, "handle_data_single")
    require(
        body,
        "sessions.get(&sid).map(|s| s.inner.clone())",
        "single data path clones inner under sessions map lock",
    )
    require(body, "*inner.last_active.lock().await", "single data path last_active update")
    require(body, "let mut w = inner.writer.lock().await;", "single data path writer lock")
    require(body, "wait_and_drain(&inner", "single data path drains through cloned inner")
    old_shape = "let sessions = state.sessions.lock().await;\n    let session = match sessions.get(&sid)"
    if old_shape in body:
        die("single data path reintroduced sessions map lock held across awaits")


def check_tests_and_tuning(text: str) -> None:
    require(
        text,
        "const STRAGGLER_SETTLE_STEP: Duration = Duration::from_millis(10);",
        "10ms tunnel-node settle step",
    )
    require(
        text,
        "const STRAGGLER_SETTLE_MAX: Duration = Duration::from_millis(1000);",
        "1000ms tunnel-node settle max",
    )
    require(
        text,
        "async fn batch_keeps_over_cap_session_until_tail_is_drained()",
        "over-cap tail preservation regression test",
    )
    require(
        text,
        "async fn batch_tcp_ready_does_not_pay_udp_longpoll_deadline()",
        "mixed TCP/UDP long-poll regression test",
    )


def main() -> int:
    if not SRC.is_file():
        die(f"missing source file: {SRC}")
    text = SRC.read_text(encoding="utf-8")
    check_abort_on_drop(text)
    check_handle_batch(text)
    check_single_data_path(text)
    check_tests_and_tuning(text)
    print("tunnel-node drain/concurrency check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
