/// SM-2 spaced repetition algorithm for puzzle scheduling.

#[derive(Debug, Clone)]
pub struct SrsUpdate {
    pub interval: f64,
    pub ease_factor: f64,
    /// ISO 8601 date string for the next review
    pub next_review: String,
}

/// Map puzzle outcome to SM-2 quality (0–5).
///
/// - 5: Solved, 0 hints, < 30s
/// - 4: Solved, 0 hints, >= 30s
/// - 3: Solved, 1 hint
/// - 2: Solved, 2+ hints
/// - 1: Failed
pub fn quality_from_attempt(solved: bool, hints_used: u32, time_ms: u64) -> u8 {
    if !solved {
        return 1;
    }
    match hints_used {
        0 if time_ms < 30_000 => 5,
        0 => 4,
        1 => 3,
        _ => 2,
    }
}

/// Compute the next SRS interval and ease factor using SM-2 rules.
///
/// - `prev_interval`: current interval in days (1.0 for first review)
/// - `prev_ease`: current ease factor (2.5 default)
/// - `quality`: 0–5 performance rating
/// - `attempt_count`: how many times this puzzle has been attempted (1-based)
pub fn compute_srs_update(
    prev_interval: f64,
    prev_ease: f64,
    quality: u8,
    attempt_count: u32,
) -> SrsUpdate {
    let q = quality as f64;

    // Update ease factor: EF' = EF + (0.1 - (5-q) * (0.08 + (5-q) * 0.02))
    let new_ease = (prev_ease + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02))).max(1.3);

    let new_interval = if quality < 3 {
        // Failure: reset to 1 day
        1.0
    } else if attempt_count <= 1 {
        1.0
    } else if attempt_count == 2 {
        6.0
    } else {
        prev_interval * new_ease
    };

    // Compute next review date (days from now)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let next_secs = now + (new_interval * 86400.0) as u64;
    let next_review = format_unix_as_datetime(next_secs);

    SrsUpdate {
        interval: new_interval,
        ease_factor: new_ease,
        next_review,
    }
}

fn format_unix_as_datetime(secs: u64) -> String {
    // Convert unix timestamp to ISO 8601 datetime (UTC)
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Simple date calculation from days since 1970-01-01
    let (year, month, day) = days_to_ymd(days_since_epoch);
    format!("{year:04}-{month:02}-{day:02} {hours:02}:{minutes:02}:{seconds:02}")
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_perfect_solve() {
        assert_eq!(quality_from_attempt(true, 0, 15_000), 5);
    }

    #[test]
    fn quality_slow_solve() {
        assert_eq!(quality_from_attempt(true, 0, 45_000), 4);
    }

    #[test]
    fn quality_one_hint() {
        assert_eq!(quality_from_attempt(true, 1, 10_000), 3);
    }

    #[test]
    fn quality_multiple_hints() {
        assert_eq!(quality_from_attempt(true, 2, 10_000), 2);
        assert_eq!(quality_from_attempt(true, 3, 60_000), 2);
    }

    #[test]
    fn quality_failed() {
        assert_eq!(quality_from_attempt(false, 0, 10_000), 1);
        assert_eq!(quality_from_attempt(false, 3, 90_000), 1);
    }

    #[test]
    fn srs_first_attempt_success() {
        let update = compute_srs_update(1.0, 2.5, 5, 1);
        assert_eq!(update.interval, 1.0);
        assert!(update.ease_factor > 2.5); // EF increases for quality 5
    }

    #[test]
    fn srs_second_attempt_success() {
        let update = compute_srs_update(1.0, 2.5, 4, 2);
        assert_eq!(update.interval, 6.0);
    }

    #[test]
    fn srs_third_attempt_uses_multiplier() {
        let update = compute_srs_update(6.0, 2.5, 4, 3);
        // interval = 6.0 * new_ef; new_ef = 2.5 + (0.1 - 1*(0.08+1*0.02)) = 2.5
        assert!((update.interval - 15.0).abs() < 0.1);
        assert!((update.ease_factor - 2.5).abs() < 0.01);
    }

    #[test]
    fn srs_failure_resets_interval() {
        let update = compute_srs_update(30.0, 2.5, 1, 5);
        assert_eq!(update.interval, 1.0);
    }

    #[test]
    fn srs_ease_factor_never_below_1_3() {
        // Quality 1 drops EF significantly
        let update = compute_srs_update(1.0, 1.3, 1, 1);
        assert!(update.ease_factor >= 1.3);
    }

    #[test]
    fn srs_quality_2_borderline() {
        // Quality 2 is "solved with 2+ hints" — below threshold for pass
        let update = compute_srs_update(6.0, 2.5, 2, 3);
        // quality < 3 → interval resets to 1
        assert_eq!(update.interval, 1.0);
    }

    #[test]
    fn srs_next_review_is_valid_datetime() {
        let update = compute_srs_update(1.0, 2.5, 5, 1);
        // Should look like "YYYY-MM-DD HH:MM:SS"
        assert!(update.next_review.len() >= 19);
        assert!(update.next_review.contains('-'));
        assert!(update.next_review.contains(':'));
    }
}
