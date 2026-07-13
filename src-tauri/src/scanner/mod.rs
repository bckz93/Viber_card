pub mod claude_code;
pub mod hermes;
pub mod ollama;

use crate::models::ScanResult;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("could not resolve home/data directory")]
    NoHomeDir,
    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Contract every history provider must satisfy. Adding a new tool
/// (e.g. a future "Cursor" or "Aider" source) means implementing this
/// trait, not touching the scoring engine or the Tauri command.
pub trait HistorySource {
    fn scan(&self) -> Result<ScanResult, ScanError>;
}
