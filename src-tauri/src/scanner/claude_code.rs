use super::{HistorySource, ScanError};
use crate::models::{Interaction, Role, ScanResult, Source};
use serde_json::Value;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Reads `~/.claude/projects/<encoded-project-path>/<session-uuid>.jsonl`.
/// Each line is one JSON event; we only care about `type: "user"` and
/// `type: "assistant"` entries, everything else (summaries, tool results,
/// meta) is skipped rather than treated as an error.
pub struct ClaudeCodeSource {
    pub root: PathBuf,
}

impl ClaudeCodeSource {
    /// Resolves to `~/.claude/projects`.
    pub fn default_root() -> Result<PathBuf, ScanError> {
        dirs::home_dir()
            .map(|h| h.join(".claude").join("projects"))
            .ok_or(ScanError::NoHomeDir)
    }

    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Best-effort decode of Claude Code's directory-name encoding, where
    /// the project's absolute path has its `/` separators replaced by `-`
    /// (e.g. `/mnt/foo/bar` -> `-mnt-foo-bar`). This is lossy if the real
    /// path contains literal dashes, so it's only used as a human-readable
    /// label, never as a filesystem path.
    fn decode_project_name(dir_name: &str) -> String {
        if let Some(stripped) = dir_name.strip_prefix('-') {
            format!("/{}", stripped.replace('-', "/"))
        } else {
            dir_name.replace('-', "/")
        }
    }

    fn parse_line(line: &str, project: &str) -> Result<Option<Interaction>, String> {
        if line.trim().is_empty() {
            return Ok(None);
        }
        let value: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;

        let entry_type = value.get("type").and_then(Value::as_str).unwrap_or("");
        let role = match entry_type {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            _ => return Ok(None), // summaries, tool-results, meta events, etc.
        };

        let timestamp = value
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let Some(timestamp) = timestamp else {
            return Err("missing/invalid timestamp".to_string());
        };

        let content = Self::extract_text(value.get("message").and_then(|m| m.get("content")));
        if content.trim().is_empty() {
            return Ok(None); // pure tool-use turns with no human-readable text
        }

        let session_id = value
            .get("sessionId")
            .and_then(Value::as_str)
            .map(str::to_string);

        Ok(Some(Interaction::new(
            timestamp,
            role,
            content,
            Source::ClaudeCode,
            Some(project.to_string()),
            session_id,
        )))
    }

    /// `message.content` is either a plain string or an array of content
    /// blocks (`{"type": "text", "text": "..."}`, tool_use, tool_result...).
    /// We only keep the text blocks for the VOL/EMO/SLF scoring.
    fn extract_text(content: Option<&Value>) -> String {
        match content {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Array(blocks)) => blocks
                .iter()
                .filter_map(|b| b.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n"),
            _ => String::new(),
        }
    }
}

impl HistorySource for ClaudeCodeSource {
    fn scan(&self) -> Result<ScanResult, ScanError> {
        if !self.root.exists() {
            return Ok(ScanResult {
                interactions: vec![],
                warnings: vec![format!(
                    "no Claude Code history found at {}",
                    self.root.display()
                )],
            });
        }

        let mut interactions = Vec::new();
        let mut warnings = Vec::new();

        for project_dir in WalkDir::new(&self.root)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir())
        {
            let project = Self::decode_project_name(
                &project_dir.file_name().to_string_lossy(),
            );

            for entry in WalkDir::new(project_dir.path())
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("jsonl"))
            {
                let path = entry.path();
                let raw = match std::fs::read_to_string(path) {
                    Ok(r) => r,
                    Err(e) => {
                        warnings.push(format!("skipped {}: {e}", path.display()));
                        continue;
                    }
                };

                for (line_no, line) in raw.lines().enumerate() {
                    match Self::parse_line(line, &project) {
                        Ok(Some(interaction)) => interactions.push(interaction),
                        Ok(None) => {}
                        Err(e) => warnings.push(format!(
                            "{}:{} - {e}",
                            path.display(),
                            line_no + 1
                        )),
                    }
                }
            }
        }

        Ok(ScanResult {
            interactions,
            warnings,
        })
    }
}
