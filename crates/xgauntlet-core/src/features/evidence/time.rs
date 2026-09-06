//! Deterministic ISO 8601 UTC timestamp generation without external dependencies.

/// Generates an exact ISO 8601 UTC timestamp string without external dependencies.
///
/// Output format adheres strictly to ISO 8601: `YYYY-MM-DDTHH:MM:SSZ`.
///
/// # Examples
/// ```
/// use xgauntlet_core::features::evidence::current_iso_utc;
/// let ts = current_iso_utc();
/// assert!(ts.ends_with('Z'));
/// assert_eq!(&ts[10..11], "T");
/// ```
pub fn current_iso_utc() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let sec = secs % 60;
    let min = (secs / 60) % 60;
    let hour = (secs / 3600) % 24;
    let mut days = secs / 86400;

    let mut year = 1970;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if days >= days_in_year {
            days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &d in &month_days {
        if days >= d {
            days -= d;
            month += 1;
        } else {
            break;
        }
    }
    let day = days + 1;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_iso_utc_format() {
        let ts = current_iso_utc();
        assert_eq!(ts.len(), 20);
        assert!(ts.ends_with('Z'));
        assert_eq!(&ts[10..11], "T");
        let year: u32 = ts[0..4].parse().expect("valid year");
        assert!(year >= 2026);
    }
}
