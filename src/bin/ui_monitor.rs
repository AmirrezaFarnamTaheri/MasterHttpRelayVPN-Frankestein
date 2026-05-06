use std::collections::VecDeque;
use std::time::{Duration, Instant};

use mhrv_jni::domain_fronter::StatsSnapshot;

use crate::ui_format::{fmt_bytes, fmt_duration};

pub(crate) fn traffic_stat_rows(stats: &StatsSnapshot) -> Vec<(&'static str, String)> {
    vec![
        ("relay calls", stats.relay_calls.to_string()),
        ("failures", stats.relay_failures.to_string()),
        ("coalesced", stats.coalesced.to_string()),
        ("today calls", stats.today_calls.to_string()),
        (
            "cache hits",
            format!(
                "{} / {}  ({:.0}%)",
                stats.cache_hits,
                stats.cache_hits + stats.cache_misses,
                stats.hit_rate()
            ),
        ),
        ("cache size", format!("{} KB", stats.cache_bytes / 1024)),
        ("bytes relayed", fmt_bytes(stats.bytes_relayed)),
        ("today bytes", fmt_bytes(stats.today_bytes)),
        (
            "reset in",
            fmt_duration(Duration::from_secs(stats.today_reset_secs)),
        ),
        (
            "degrade",
            format!(
                "L{} ({})",
                stats.degrade_level,
                String::from_utf8_lossy(&stats.degrade_reason)
                    .trim_matches(char::from(0))
                    .trim()
            ),
        ),
        (
            "active scripts",
            format!(
                "{} / {}",
                stats.total_scripts - stats.blacklisted_scripts,
                stats.total_scripts
            ),
        ),
    ]
}

pub(crate) fn quota_calls_per_hour(today_calls: u64, today_reset_secs: u64) -> f64 {
    let secs_since_reset = 86_400u64.saturating_sub(today_reset_secs.min(86_400));
    if secs_since_reset == 0 {
        0.0
    } else {
        (today_calls as f64) / (secs_since_reset as f64) * 3600.0
    }
}

pub(crate) fn degradation_changes(
    history: &VecDeque<(Instant, u8, String)>,
) -> Vec<(Duration, u8, String)> {
    let mut changes: Vec<(Duration, u8, String)> = Vec::new();
    let mut last: Option<(u8, &str)> = None;
    for (t, lvl, reason) in history.iter() {
        let r = reason.as_str();
        if last.map(|(pl, pr)| pl == *lvl && pr == r).unwrap_or(false) {
            continue;
        }
        last = Some((*lvl, r));
        changes.push((t.elapsed(), *lvl, reason.clone()));
    }
    changes.reverse();
    changes.truncate(10);
    changes
}

pub(crate) fn notable_failure_lines<I>(lines: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut notable: Vec<String> = lines
        .into_iter()
        .filter(|line| {
            line.contains("degrade:")
                || line.contains("range-parallel:")
                || line.contains("timeout")
                || line.contains("unreachable")
                || line.contains("overloaded")
                || line.contains("quota")
                || line.contains("429")
        })
        .collect();
    notable.reverse();
    notable.truncate(12);
    notable
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats_fixture() -> StatsSnapshot {
        StatsSnapshot {
            relay_calls: 7,
            relay_failures: 2,
            coalesced: 3,
            bytes_relayed: 2048,
            cache_hits: 4,
            cache_misses: 1,
            cache_bytes: 4096,
            total_scripts: 5,
            blacklisted_scripts: 2,
            today_calls: 24,
            today_bytes: 8192,
            today_reset_secs: 3600,
            degrade_level: 1,
            degrade_reason: {
                let mut reason = [0u8; 32];
                reason[..5].copy_from_slice(b"quota");
                reason
            },
        }
    }

    #[test]
    fn traffic_rows_keep_expected_metric_labels() {
        let rows = traffic_stat_rows(&stats_fixture());
        let labels: Vec<_> = rows.iter().map(|(label, _)| *label).collect();
        assert_eq!(
            labels,
            vec![
                "relay calls",
                "failures",
                "coalesced",
                "today calls",
                "cache hits",
                "cache size",
                "bytes relayed",
                "today bytes",
                "reset in",
                "degrade",
                "active scripts",
            ]
        );
        assert!(rows
            .iter()
            .any(|(label, value)| { *label == "active scripts" && value == "3 / 5" }));
    }

    #[test]
    fn quota_rate_handles_midnight_reset_boundary() {
        assert_eq!(quota_calls_per_hour(10, 86_400), 0.0);
        assert!((quota_calls_per_hour(10, 82_800) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn degradation_changes_collapse_adjacent_duplicates_and_cap() {
        let now = Instant::now();
        let mut history = VecDeque::new();
        history.push_back((now, 0, "none".into()));
        history.push_back((now, 0, "none".into()));
        history.push_back((now, 1, "quota".into()));
        history.push_back((now, 1, "quota".into()));
        history.push_back((now, 2, "timeout".into()));

        let changes = degradation_changes(&history);
        let levels: Vec<_> = changes.iter().map(|(_, level, _)| *level).collect();
        assert_eq!(levels, vec![2, 1, 0]);
    }

    #[test]
    fn notable_failure_lines_filters_reverses_and_caps() {
        let lines = vec![
            "ok".to_string(),
            "timeout on relay".to_string(),
            "quota 429".to_string(),
            "boring".to_string(),
        ];
        assert_eq!(
            notable_failure_lines(lines),
            vec!["quota 429".to_string(), "timeout on relay".to_string()]
        );
    }
}
