use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

use crate::assessment::adaptive::{check_adaptive_triggers, AdaptivePrompt, AdaptiveTriggerData};
use crate::db::connection::Database;
use crate::error::AppError;
use crate::models::assessment::SkillProfile;
use crate::models::game::GameRecord;
use crate::models::puzzle::PuzzleSessionStats;
use crate::CurrentPlayerId;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub skill_profile: SkillProfile,
    pub recent_games: Vec<GameRecord>,
    pub puzzle_stats: PuzzleSessionStats,
    pub daily_recommendation: DailyRecommendation,
    pub streak: SessionStreak,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyRecommendation {
    pub text: String,
    pub target_activity: String,
    pub target_category: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStreak {
    pub current_days: u32,
    pub longest_days: u32,
    pub games_today: u32,
    pub puzzles_today: u32,
}

/// Pure function: compute daily recommendation from skill profile and activity data.
fn compute_recommendation(
    profile: &SkillProfile,
    has_recent_game: bool,
    has_recent_puzzle: bool,
    days_since_last_game: Option<u32>,
) -> DailyRecommendation {
    // No skill ratings → new player
    if profile.ratings.is_empty() {
        return DailyRecommendation {
            text: "Start with some tactical puzzles to establish your baseline".to_string(),
            target_activity: "problems".to_string(),
            target_category: Some("tactical".to_string()),
        };
    }

    let weakest = profile.weakest_category.as_deref();

    // Weakest category with < 5 games → focus there
    if let Some(weak_cat) = weakest {
        let weak_rating = profile.ratings.iter().find(|r| r.category == weak_cat);
        if let Some(r) = weak_rating {
            if r.games_count < 5 {
                return DailyRecommendation {
                    text: format!(
                        "Focus on {} puzzles to strengthen your weak area",
                        weak_cat
                    ),
                    target_activity: "problems".to_string(),
                    target_category: Some(weak_cat.to_string()),
                };
            }
        }
    }

    // No game in 3+ days → play a game
    if let Some(days) = days_since_last_game {
        if days >= 3 {
            return DailyRecommendation {
                text: "Play a game to stay sharp — it's been a few days".to_string(),
                target_activity: "play".to_string(),
                target_category: None,
            };
        }
    }

    // Only puzzles recently (has puzzles but no games today) → openings
    if has_recent_puzzle && !has_recent_game {
        return DailyRecommendation {
            text: "Mix things up with an opening drill".to_string(),
            target_activity: "openings".to_string(),
            target_category: None,
        };
    }

    // Default: puzzles in weakest category
    DailyRecommendation {
        text: format!(
            "Work on {} puzzles to keep improving",
            weakest.unwrap_or("tactical")
        ),
        target_activity: "problems".to_string(),
        target_category: weakest.map(|s| s.to_string()),
    }
}

/// Compute session streak from sorted activity dates (most recent first).
fn compute_streak(activity_dates: &[String], today: &str) -> (u32, u32) {
    if activity_dates.is_empty() {
        return (0, 0);
    }

    // Parse dates as "YYYY-MM-DD" and count consecutive days ending today
    let mut current_streak: u32 = 0;

    // Check if today counts
    if activity_dates.first().map(|d| d.as_str()) == Some(today) {
        current_streak = 1;
        let mut expected_date = subtract_day(today);
        for date in activity_dates.iter().skip(1) {
            if date == &expected_date {
                current_streak += 1;
                expected_date = subtract_day(&expected_date);
            } else if date != today {
                break;
            }
        }
    }

    // Longest streak: scan all dates
    let mut longest: u32 = 0;
    let mut running: u32 = 1;
    for i in 1..activity_dates.len() {
        let prev = &activity_dates[i - 1];
        let curr = &activity_dates[i];
        if curr == prev {
            continue; // duplicate date
        }
        let expected = subtract_day(prev);
        if *curr == expected {
            running += 1;
        } else {
            longest = longest.max(running);
            running = 1;
        }
    }
    longest = longest.max(running).max(current_streak);

    (current_streak, longest)
}

/// Naive date subtraction: "YYYY-MM-DD" → previous day.
/// Uses simple calendar math (adequate for streak counting).
fn subtract_day(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return String::new();
    }
    let y: i32 = parts[0].parse().unwrap_or(2024);
    let m: u32 = parts[1].parse().unwrap_or(1);
    let d: u32 = parts[2].parse().unwrap_or(1);

    if d > 1 {
        return format!("{y:04}-{m:02}-{:02}", d - 1);
    }

    // Day is 1 → go to previous month
    let (prev_y, prev_m) = if m == 1 { (y - 1, 12) } else { (y, m - 1) };
    let days_in_prev_month = match prev_m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (prev_y % 4 == 0 && prev_y % 100 != 0) || prev_y % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    };

    format!("{prev_y:04}-{prev_m:02}-{days_in_prev_month:02}")
}

/// Get today's date as "YYYY-MM-DD" in UTC-like form from a unix timestamp.
fn today_date_str() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Simple conversion to date (good enough, same pattern as the rest of the codebase)
    let days = secs / 86400;
    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
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

    format!("{y:04}-{m:02}-{d:02}")
}

#[tauri::command]
pub fn get_dashboard_data(
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
) -> Result<DashboardData, AppError> {
    let player_id = player_state.get()?;
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    let skill_profile = db.get_skill_profile(&player_id)?;
    let recent_games = db.get_game_history(5, 0)?;
    let puzzle_stats = db.get_puzzle_stats(&player_id)?;

    let today = today_date_str();
    let (games_today, puzzles_today) = db.get_today_counts(&player_id, &today)?;
    let activity_dates = db.get_activity_dates(&player_id)?;

    let days_since_last_game = if recent_games.is_empty() {
        None
    } else {
        // started_at is a unix-style timestamp string; extract date
        let last_date = &activity_dates
            .iter()
            .find(|_| true)
            .cloned()
            .unwrap_or_default();
        if last_date.is_empty() {
            None
        } else {
            // Count days between last_date and today
            Some(count_days_between(last_date, &today))
        }
    };

    let (current_days, longest_days) = compute_streak(&activity_dates, &today);

    let daily_recommendation = compute_recommendation(
        &skill_profile,
        games_today > 0,
        puzzles_today > 0,
        days_since_last_game,
    );

    Ok(DashboardData {
        skill_profile,
        recent_games,
        puzzle_stats,
        daily_recommendation,
        streak: SessionStreak {
            current_days,
            longest_days,
            games_today,
            puzzles_today,
        },
    })
}

#[tauri::command]
pub fn check_adaptive_difficulty(
    db: State<'_, Mutex<Database>>,
    player_state: State<'_, CurrentPlayerId>,
) -> Result<AdaptivePrompt, AppError> {
    let player_id = player_state.get()?;
    let db = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    let recent_results = db.get_recent_game_results(&player_id, 5)?;
    let (solve_rate, puzzle_count) = db.get_recent_puzzle_solve_rate(&player_id, 10)?;
    let hint_usage = db.get_puzzle_hint_usage_rate(&player_id, 10)?;
    let profile = db.get_skill_profile(&player_id)?;
    let total_sessions = profile.ratings.iter().map(|r| r.games_count).sum::<u32>();

    // Compute rating stability (std dev of ratings)
    let rating_std_dev = if profile.ratings.len() >= 2 {
        let mean = profile.overall_rating;
        let variance = profile
            .ratings
            .iter()
            .map(|r| (r.rating - mean).powi(2))
            .sum::<f64>()
            / profile.ratings.len() as f64;
        variance.sqrt()
    } else {
        999.0 // not enough data
    };

    let data = AdaptiveTriggerData {
        recent_game_results: recent_results,
        recent_solve_rate: solve_rate,
        recent_puzzle_count: puzzle_count,
        hint_usage_rate: hint_usage,
        total_sessions,
        rating_std_dev,
        weakest_category: profile.weakest_category.clone(),
    };

    Ok(check_adaptive_triggers(&data))
}

/// Count days between two "YYYY-MM-DD" date strings.
fn count_days_between(from: &str, to: &str) -> u32 {
    // Simple approach: parse both to day-of-epoch and subtract
    fn parse_to_days(d: &str) -> Option<i64> {
        let parts: Vec<&str> = d.split('-').collect();
        if parts.len() != 3 {
            return None;
        }
        let y: i64 = parts[0].parse().ok()?;
        let m: i64 = parts[1].parse().ok()?;
        let d: i64 = parts[2].parse().ok()?;
        // Rough day count (good enough for difference)
        Some(y * 365 + y / 4 - y / 100 + y / 400 + m * 30 + d)
    }

    let from_days = parse_to_days(from).unwrap_or(0);
    let to_days = parse_to_days(to).unwrap_or(0);
    (to_days - from_days).unsigned_abs() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::assessment::{SkillProfile, SkillRating};

    fn empty_profile() -> SkillProfile {
        SkillProfile {
            ratings: Vec::new(),
            overall_rating: 1200.0,
            strongest_category: None,
            weakest_category: None,
        }
    }

    fn profile_with_weak(weak: &str, games: u32) -> SkillProfile {
        SkillProfile {
            ratings: vec![
                SkillRating {
                    id: "r1".to_string(),
                    player_id: "p1".to_string(),
                    category: "tactical".to_string(),
                    rating: 1400.0,
                    rd: 100.0,
                    volatility: 0.06,
                    games_count: 10,
                    last_updated: String::new(),
                },
                SkillRating {
                    id: "r2".to_string(),
                    player_id: "p1".to_string(),
                    category: weak.to_string(),
                    rating: 1000.0,
                    rd: 150.0,
                    volatility: 0.06,
                    games_count: games,
                    last_updated: String::new(),
                },
            ],
            overall_rating: 1200.0,
            strongest_category: Some("tactical".to_string()),
            weakest_category: Some(weak.to_string()),
        }
    }

    #[test]
    fn recommendation_new_player() {
        let r = compute_recommendation(&empty_profile(), false, false, None);
        assert_eq!(r.target_activity, "problems");
        assert!(r.text.contains("tactical"));
    }

    #[test]
    fn recommendation_weak_category() {
        let r = compute_recommendation(&profile_with_weak("endgame", 3), false, false, Some(1));
        assert_eq!(r.target_activity, "problems");
        assert_eq!(r.target_category, Some("endgame".to_string()));
    }

    #[test]
    fn recommendation_no_recent_game() {
        let r = compute_recommendation(&profile_with_weak("endgame", 10), false, true, Some(5));
        assert_eq!(r.target_activity, "play");
    }

    #[test]
    fn recommendation_mix_it_up() {
        let r = compute_recommendation(&profile_with_weak("endgame", 10), false, true, Some(1));
        assert_eq!(r.target_activity, "openings");
    }

    #[test]
    fn streak_empty() {
        let (current, longest) = compute_streak(&[], "2026-03-02");
        assert_eq!(current, 0);
        assert_eq!(longest, 0);
    }

    #[test]
    fn streak_today_only() {
        let dates = vec!["2026-03-02".to_string()];
        let (current, longest) = compute_streak(&dates, "2026-03-02");
        assert_eq!(current, 1);
        assert_eq!(longest, 1);
    }

    #[test]
    fn streak_three_days() {
        let dates = vec![
            "2026-03-02".to_string(),
            "2026-03-01".to_string(),
            "2026-02-28".to_string(),
        ];
        let (current, longest) = compute_streak(&dates, "2026-03-02");
        assert_eq!(current, 3);
        assert_eq!(longest, 3);
    }

    #[test]
    fn streak_gap() {
        let dates = vec![
            "2026-03-02".to_string(),
            "2026-02-28".to_string(),
            "2026-02-27".to_string(),
        ];
        let (current, longest) = compute_streak(&dates, "2026-03-02");
        assert_eq!(current, 1);
        assert_eq!(longest, 2);
    }

    #[test]
    fn subtract_day_normal() {
        assert_eq!(subtract_day("2026-03-15"), "2026-03-14");
    }

    #[test]
    fn subtract_day_month_boundary() {
        assert_eq!(subtract_day("2026-03-01"), "2026-02-28");
    }

    #[test]
    fn subtract_day_year_boundary() {
        assert_eq!(subtract_day("2026-01-01"), "2025-12-31");
    }

    #[test]
    fn subtract_day_leap_year() {
        assert_eq!(subtract_day("2024-03-01"), "2024-02-29");
    }
}
