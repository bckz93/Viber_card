use crate::models::ScanResult;
use crate::scanner::claude_code::ClaudeCodeSource;
use crate::scanner::hermes::HermesSource;
use crate::scanner::ollama::OllamaSource;
use crate::scanner::HistorySource;
use crate::scoring::{self, PlayerStats};
use crate::snapshot;
use chrono::{DateTime, Datelike, Duration, Utc};
use serde::Serialize;

/// The Current Deck card is a rolling window, not a lifetime average.
pub const CARD_WINDOW_DAYS: i64 = 7;

/// Scans every known local AI-history source on this machine and merges
/// them into one combined result. A source that's missing or unreadable
/// only adds a warning — it never blocks the others.
fn run_scan() -> ScanResult {
    let mut result = ScanResult::default();

    match ClaudeCodeSource::default_root() {
        Ok(root) => match ClaudeCodeSource::new(root).scan() {
            Ok(r) => result = result.merge(r),
            Err(e) => result.warnings.push(format!("Claude Code: {e}")),
        },
        Err(e) => result.warnings.push(format!("Claude Code: {e}")),
    }

    if let Some(path) = HermesSource::default_path() {
        match HermesSource::new(path).scan() {
            Ok(r) => result = result.merge(r),
            Err(e) => result.warnings.push(format!("Hermes: {e}")),
        }
    }

    if let Some(path) = OllamaSource::default_path() {
        match OllamaSource::new(path).scan() {
            Ok(r) => result = result.merge(r),
            Err(e) => result.warnings.push(format!("Ollama: {e}")),
        }
    }

    result
}

#[tauri::command]
pub async fn get_player_card() -> Result<PlayerStats, String> {
    let range_end = Utc::now();
    let range_start = range_end - Duration::days(CARD_WINDOW_DAYS);
    let scan = run_scan().keep_recent(CARD_WINDOW_DAYS);
    let stats = scoring::compute(&scan.interactions, range_start, range_end);

    // Best-effort: history accrual shouldn't block showing the card if the
    // data dir isn't writable for some reason.
    if let Some(path) = snapshot::snapshots_path() {
        let _ = snapshot::snapshot_if_new_day(&path, &stats);
    }

    Ok(stats)
}

/// Monday 00:00 UTC of the week containing `dt`.
fn start_of_week(dt: DateTime<Utc>) -> DateTime<Utc> {
    let days_from_monday = dt.weekday().num_days_from_monday() as i64;
    (dt - Duration::days(days_from_monday))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
}

#[derive(Debug, Clone, Serialize)]
pub struct WeekRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Every completed calendar week (Monday-Sunday) that has at least one
/// interaction, most recent first. The current, still-in-progress week is
/// never included — Evolution Progress's default comparison already covers
/// "now" via its own rolling window.
#[tauri::command]
pub async fn list_available_weeks() -> Result<Vec<WeekRange>, String> {
    let scan = run_scan();
    let Some(min_ts) = scan.interactions.iter().map(|i| i.timestamp).min() else {
        return Ok(Vec::new());
    };

    let current_week_start = start_of_week(Utc::now());
    let mut weeks = Vec::new();
    let mut week_start = start_of_week(min_ts);

    while week_start < current_week_start {
        let week_end = week_start + Duration::days(7);
        let has_data = scan
            .interactions
            .iter()
            .any(|i| i.timestamp >= week_start && i.timestamp < week_end);
        if has_data {
            weeks.push(WeekRange { start: week_start, end: week_end });
        }
        week_start = week_end;
    }

    weeks.reverse();
    Ok(weeks)
}

/// Sums/aggregates stats fresh from raw interactions in `[start, end)` —
/// e.g. an exact calendar week — rather than deriving them from an
/// already-computed rolling-window snapshot.
#[tauri::command]
pub async fn get_stats_for_range(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<PlayerStats, String> {
    let scan = run_scan().keep_between(start, end);
    Ok(scoring::compute(&scan.interactions, start, end))
}

/// Writes a base64-encoded PNG (from html-to-image on the frontend) to an
/// absolute path chosen by the user via the save dialog. Plain `std::fs`
/// rather than the fs plugin, so it isn't bound by the fs plugin's scope
/// ACL — the path already came from a native save dialog the user approved.
#[tauri::command]
pub async fn save_png_file(path: String, base64_data: String) -> Result<(), String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("invalid base64 image data: {e}"))?;
    std::fs::write(&path, bytes).map_err(|e| format!("could not write {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_of_week_finds_monday_midnight() {
        // 2026-07-09 is a Thursday.
        let thursday = "2026-07-09T18:30:00Z".parse::<DateTime<Utc>>().unwrap();
        let monday = start_of_week(thursday);
        assert_eq!(monday.to_rfc3339(), "2026-07-06T00:00:00+00:00");
    }

    #[test]
    fn start_of_week_is_idempotent_on_monday_midnight() {
        let monday = "2026-07-06T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(start_of_week(monday), monday);
    }

    #[test]
    fn start_of_week_handles_sunday() {
        // 2026-07-12 is a Sunday — still part of the week starting 2026-07-06.
        let sunday = "2026-07-12T23:59:00Z".parse::<DateTime<Utc>>().unwrap();
        let monday = start_of_week(sunday);
        assert_eq!(monday.to_rfc3339(), "2026-07-06T00:00:00+00:00");
    }
}
