//! Local JSON history of `PlayerStats` over time, appended to once per day.
//! Evolution Progress does *not* read from this — it re-derives stats fresh
//! from raw interactions instead (see `commands::get_stats_for_range`) — so
//! this is purely a durable local record for the user's own inspection
//! (`cargo run --example inspect` prints from it). Stored as JSON Lines (one
//! snapshot per line) so appending never requires reading or rewriting the
//! whole file.

use crate::scoring::PlayerStats;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub taken_at: DateTime<Utc>,
    pub stats: PlayerStats,
}

/// `~/.local/share/vibercard/snapshots.jsonl` (or the OS equivalent).
pub fn snapshots_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("vibercard").join("snapshots.jsonl"))
}

pub fn append_snapshot(path: &Path, stats: &PlayerStats) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let snapshot = Snapshot {
        taken_at: Utc::now(),
        stats: stats.clone(),
    };
    let line = serde_json::to_string(&snapshot)?;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{line}")
}

/// Malformed lines are skipped rather than failing the whole read — a
/// half-written line from a crash shouldn't wipe out prior history.
pub fn load_snapshots(path: &Path) -> std::io::Result<Vec<Snapshot>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let snapshots = reader
        .lines()
        .map_while(Result::ok)
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<Snapshot>(&l).ok())
        .collect();

    Ok(snapshots)
}

/// Appends a snapshot only if the most recent one isn't from today —
/// called on every app launch so history accrues roughly daily without
/// piling up duplicate entries per session.
pub fn snapshot_if_new_day(path: &Path, stats: &PlayerStats) -> std::io::Result<bool> {
    let existing = load_snapshots(path)?;
    let today = Utc::now().date_naive();
    let already_taken_today = existing
        .last()
        .is_some_and(|s| s.taken_at.date_naive() == today);

    if already_taken_today {
        return Ok(false);
    }

    append_snapshot(path, stats)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::StatInsight;

    fn sample_stats() -> PlayerStats {
        PlayerStats {
            vol: 10,
            spd: 20,
            nct: 30,
            slf: 40,
            emo: 50,
            archetype: "Balanced Vibe Coder".to_string(),
            punchline: "test".to_string(),
            insights: vec![StatInsight {
                key: "vol".to_string(),
                label: "Volume".to_string(),
                value: 10,
                explanation: "test".to_string(),
            }],
            sample_size: 5,
            total_tokens: 100,
            range_start: Utc::now(),
            range_end: Utc::now(),
        }
    }

    fn scratch_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("vibercard-test-{name}-{}.jsonl", std::process::id()))
    }

    #[test]
    fn append_then_load_roundtrips() {
        let path = scratch_path("roundtrip");
        let _ = std::fs::remove_file(&path);

        append_snapshot(&path, &sample_stats()).unwrap();
        append_snapshot(&path, &sample_stats()).unwrap();

        let loaded = load_snapshots(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].stats.vol, 10);

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn missing_file_loads_as_empty_not_an_error() {
        let path = scratch_path("missing");
        let _ = std::fs::remove_file(&path);
        assert_eq!(load_snapshots(&path).unwrap().len(), 0);
    }

    #[test]
    fn snapshot_if_new_day_does_not_duplicate_same_day() {
        let path = scratch_path("dedupe");
        let _ = std::fs::remove_file(&path);

        assert!(snapshot_if_new_day(&path, &sample_stats()).unwrap());
        assert!(!snapshot_if_new_day(&path, &sample_stats()).unwrap());
        assert_eq!(load_snapshots(&path).unwrap().len(), 1);

        std::fs::remove_file(&path).unwrap();
    }
}
