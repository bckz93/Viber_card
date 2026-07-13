use super::{HistorySource, ScanError};
use crate::models::{Interaction, Role, ScanResult, Source};
use std::path::PathBuf;

/// Reads `~/.ollama/history` — the CLI's readline history file. It is a
/// flat list of prompts with **no per-line timestamp and no assistant
/// replies**: it only ever captures what the user typed into `ollama run`.
/// We approximate every line's timestamp with the file's mtime, which is
/// good enough for VOL/SLF/EMO (content-based) but means this source
/// contributes little signal to NCT/SPD (time-based).
pub struct OllamaSource {
    pub history_path: PathBuf,
}

impl OllamaSource {
    pub fn default_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".ollama").join("history"))
    }

    pub fn new(history_path: PathBuf) -> Self {
        Self { history_path }
    }
}

impl HistorySource for OllamaSource {
    fn scan(&self) -> Result<ScanResult, ScanError> {
        if !self.history_path.exists() {
            return Ok(ScanResult {
                interactions: vec![],
                warnings: vec![format!(
                    "no Ollama history found at {}",
                    self.history_path.display()
                )],
            });
        }

        let mtime = std::fs::metadata(&self.history_path)
            .and_then(|m| m.modified())
            .map(chrono::DateTime::<chrono::Utc>::from)
            .unwrap_or_else(|_| chrono::Utc::now());

        let raw = std::fs::read_to_string(&self.history_path).map_err(|e| ScanError::Io {
            path: self.history_path.display().to_string(),
            source: e,
        })?;

        let interactions = raw
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| {
                Interaction::new(
                    mtime,
                    Role::User,
                    line.to_string(),
                    Source::Ollama,
                    Some("ollama-cli".to_string()),
                    None,
                )
            })
            .collect();

        Ok(ScanResult {
            interactions,
            warnings: vec![
                "Ollama history has no per-message timestamps; all entries use the file's mtime, so they barely affect nocturnal/speed scoring.".to_string(),
            ],
        })
    }
}
