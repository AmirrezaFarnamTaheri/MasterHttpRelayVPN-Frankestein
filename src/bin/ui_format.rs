use std::time::Duration;

pub(crate) fn fmt_duration(d: Duration) -> String {
    let s = d.as_secs();
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}

pub(crate) fn fmt_bytes(b: u64) -> String {
    const K: u64 = 1024;
    const M: u64 = K * K;
    const G: u64 = M * K;
    if b >= G {
        format!("{:.2} GB", b as f64 / G as f64)
    } else if b >= M {
        format!("{:.2} MB", b as f64 / M as f64)
    } else if b >= K {
        format!("{:.1} KB", b as f64 / K as f64)
    } else {
        format!("{} B", b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_is_hh_mm_ss() {
        assert_eq!(fmt_duration(Duration::from_secs(0)), "00:00:00");
        assert_eq!(fmt_duration(Duration::from_secs(65)), "00:01:05");
        assert_eq!(fmt_duration(Duration::from_secs(3661)), "01:01:01");
    }

    #[test]
    fn bytes_use_existing_units_and_precision() {
        assert_eq!(fmt_bytes(0), "0 B");
        assert_eq!(fmt_bytes(42), "42 B");
        assert_eq!(fmt_bytes(1024), "1.0 KB");
        assert_eq!(fmt_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(fmt_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}
