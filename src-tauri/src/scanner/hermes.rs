use super::{HistorySource, ScanError};
use crate::models::{Interaction, Role, ScanResult, Source};
use chrono::{TimeZone, Utc};
use rusqlite::Connection;
use std::path::PathBuf;

/// Reads `~/.hermes/state.db` (the Hermes agent's own SQLite message log —
/// a separate personal agent, not Claude Code or Ollama). Opened read-only
/// so we never contend with Hermes' own writer for its WAL file.
pub struct HermesSource {
    pub db_path: PathBuf,
}

impl HermesSource {
    pub fn default_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".hermes").join("state.db"))
    }

    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

impl HistorySource for HermesSource {
    fn scan(&self) -> Result<ScanResult, ScanError> {
        if !self.db_path.exists() {
            return Ok(ScanResult {
                interactions: vec![],
                warnings: vec![format!("no Hermes database found at {}", self.db_path.display())],
            });
        }

        let conn = Connection::open_with_flags(
            &self.db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| ScanError::Io {
            path: self.db_path.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
        })?;

        let mut stmt = conn
            .prepare(
                "SELECT session_id, role, content, timestamp FROM messages \
                 WHERE role IN ('user', 'assistant') AND active = 1 AND content IS NOT NULL \
                 ORDER BY timestamp",
            )
            .map_err(|e| ScanError::Io {
                path: self.db_path.display().to_string(),
                source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
            })?;

        let mut interactions = Vec::new();
        let mut warnings = Vec::new();

        let rows = stmt
            .query_map([], |row| {
                let session_id: String = row.get(0)?;
                let role: String = row.get(1)?;
                let content: String = row.get(2)?;
                let ts: f64 = row.get(3)?;
                Ok((session_id, role, content, ts))
            })
            .map_err(|e| ScanError::Io {
                path: self.db_path.display().to_string(),
                source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
            })?;

        for row in rows {
            let (session_id, role_str, content, ts) = match row {
                Ok(r) => r,
                Err(e) => {
                    warnings.push(format!("skipped a row: {e}"));
                    continue;
                }
            };

            // Cron/skill-trigger system messages are stored as role=user but
            // aren't human-authored text; they'd wildly inflate VOL/EMO.
            if content.trim_start().starts_with("[IMPORTANT:") {
                continue;
            }

            let role = match role_str.as_str() {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                _ => continue,
            };

            let Some(timestamp) = Utc.timestamp_opt(ts as i64, 0).single() else {
                warnings.push(format!("invalid timestamp {ts} in session {session_id}"));
                continue;
            };

            interactions.push(Interaction::new(
                timestamp,
                role,
                content,
                Source::Hermes,
                Some("hermes-agent".to_string()),
                Some(session_id),
            ));
        }

        Ok(ScanResult {
            interactions,
            warnings,
        })
    }
}
