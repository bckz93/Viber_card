use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Where a given interaction was scanned from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    ClaudeCode,
    Ollama,
    Hermes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

/// A single normalized message, regardless of which tool produced it.
/// Every scanner (Claude Code, Ollama, Hermes) must be able to produce
/// a `Vec<Interaction>` — this is the common contract the scoring
/// engine (VOL/SPD/NCT/SLF/EMO) is built on top of.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub timestamp: DateTime<Utc>,
    pub role: Role,
    pub content: String,
    pub source: Source,
    pub project: Option<String>,
    pub session_id: Option<String>,
    /// Cheap approximation (chars / 4), refined later by the scoring engine.
    pub token_estimate: u32,
}

impl Interaction {
    pub fn new(
        timestamp: DateTime<Utc>,
        role: Role,
        content: String,
        source: Source,
        project: Option<String>,
        session_id: Option<String>,
    ) -> Self {
        let token_estimate = (content.len() as f32 / 4.0).ceil() as u32;
        Self {
            timestamp,
            role,
            content,
            source,
            project,
            session_id,
            token_estimate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanResult {
    pub interactions: Vec<Interaction>,
    /// Files that were seen but skipped/failed to parse, kept for the UI
    /// ("scanned 812 lines, 3 skipped") instead of silently failing the scan.
    pub warnings: Vec<String>,
}

impl ScanResult {
    /// Combines results from multiple sources (Claude Code + Hermes + Ollama)
    /// into one. A source that fails entirely contributes only a warning,
    /// not a hard error — the other sources still count.
    pub fn merge(mut self, other: ScanResult) -> Self {
        self.interactions.extend(other.interactions);
        self.warnings.extend(other.warnings);
        self
    }

    /// Keeps only interactions from the last `days` days. The Current Deck
    /// card is a rolling window, not an all-time average — otherwise a
    /// single extreme week gets permanently diluted into the lifetime mean
    /// and Evolution Progress deltas would barely move.
    pub fn keep_recent(mut self, days: i64) -> Self {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        self.interactions.retain(|i| i.timestamp >= cutoff);
        self
    }

    /// Keeps only interactions in `[start, end)`. Used to score an exact
    /// calendar week (or any other fixed range) fresh from raw interactions
    /// — summed over whatever actually happened in that window, not derived
    /// from an already-computed rolling-window snapshot.
    pub fn keep_between(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.interactions.retain(|i| i.timestamp >= start && i.timestamp < end);
        self
    }
}
