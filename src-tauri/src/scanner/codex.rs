use super::{HistorySource, ScanError};
use crate::models::{Interaction, Role, ScanResult, Source};
use serde_json::Value;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Injected boilerplate that OpenAI Codex CLI stores under `role: "user"`
/// even though the human never typed it: a `<environment_context>` block
/// (cwd/shell) at the start of most sessions, and one synthetic message per
/// discovered `AGENTS.md` file. Neither is human-authored text, so both
/// would wildly inflate VOL/EMO if counted — same category of problem as
/// Hermes's `[IMPORTANT:` cron/skill messages.
const INJECTED_PREFIXES: &[&str] = &["<environment_context>", "# AGENTS.md instructions for"];

/// Reads `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl` — OpenAI Codex CLI's
/// session transcripts. Each line is a `{ "timestamp", "type", "payload" }`
/// envelope; we only care about `type: "response_item"` entries whose
/// payload is itself `{ "type": "message", "role", "content" }`. Everything
/// else (`session_meta`, `event_msg`, `function_call`,
/// `function_call_output`, reasoning) is skipped rather than treated as an
/// error — it's not a message a human read or wrote.
///
/// Sessions Codex has compressed into `rollout-*.jsonl.zst` (its own
/// inactive-session archival) are not decompressed — a documented gap, not
/// a silent one; recent/active sessions are always plain `.jsonl`.
pub struct CodexSource {
    pub root: PathBuf,
}

impl CodexSource {
    /// Resolves to `~/.codex/sessions`.
    pub fn default_root() -> Result<PathBuf, ScanError> {
        dirs::home_dir()
            .map(|h| h.join(".codex").join("sessions"))
            .ok_or(ScanError::NoHomeDir)
    }

    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// `payload.content` is an array of typed blocks — `{"type":
    /// "input_text", "text": "..."}` for user/developer turns, `{"type":
    /// "output_text", "text": "..."}` for the assistant's. We only keep
    /// those two text-bearing types; anything else (e.g. image refs) is
    /// dropped for scoring purposes.
    fn extract_text(content: Option<&Value>) -> String {
        match content {
            Some(Value::Array(blocks)) => blocks
                .iter()
                .filter(|b| matches!(b.get("type").and_then(Value::as_str), Some("input_text") | Some("output_text")))
                .filter_map(|b| b.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n"),
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        }
    }

    fn parse_line(line: &str, session_id: &str) -> Result<Option<Interaction>, String> {
        if line.trim().is_empty() {
            return Ok(None);
        }
        let value: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;

        if value.get("type").and_then(Value::as_str) != Some("response_item") {
            return Ok(None); // session_meta, event_msg, turn_context, etc.
        }
        let payload = value.get("payload");
        if payload.and_then(|p| p.get("type")).and_then(Value::as_str) != Some("message") {
            return Ok(None); // function_call, function_call_output, reasoning, etc.
        }

        let role = match payload.and_then(|p| p.get("role")).and_then(Value::as_str) {
            Some("user") => Role::User,
            Some("assistant") => Role::Assistant,
            _ => return Ok(None), // "developer" (system-level instructions) and anything else
        };

        let timestamp = value
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let Some(timestamp) = timestamp else {
            return Err("missing/invalid timestamp".to_string());
        };

        let content = Self::extract_text(payload.and_then(|p| p.get("content")));
        if content.trim().is_empty() {
            return Ok(None);
        }
        if role == Role::User && INJECTED_PREFIXES.iter().any(|p| content.trim_start().starts_with(p)) {
            return Ok(None);
        }

        Ok(Some(Interaction::new(
            timestamp,
            role,
            content,
            Source::Codex,
            None, // rollout files aren't organized per-project like Claude Code's are
            Some(session_id.to_string()),
        )))
    }
}

impl HistorySource for CodexSource {
    fn scan(&self) -> Result<ScanResult, ScanError> {
        if !self.root.exists() {
            return Ok(ScanResult {
                interactions: vec![],
                warnings: vec![format!("no Codex history found at {}", self.root.display())],
            });
        }

        let mut interactions = Vec::new();
        let mut warnings = Vec::new();

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file()
                    && e.file_name().to_string_lossy().starts_with("rollout-")
                    && e.path().extension().and_then(|x| x.to_str()) == Some("jsonl")
            })
        {
            let path = entry.path();
            let session_id = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());

            let raw = match std::fs::read_to_string(path) {
                Ok(r) => r,
                Err(e) => {
                    warnings.push(format!("skipped {}: {e}", path.display()));
                    continue;
                }
            };

            for (line_no, line) in raw.lines().enumerate() {
                match Self::parse_line(line, &session_id) {
                    Ok(Some(interaction)) => interactions.push(interaction),
                    Ok(None) => {}
                    Err(e) => warnings.push(format!("{}:{} - {e}", path.display(), line_no + 1)),
                }
            }
        }

        Ok(ScanResult { interactions, warnings })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_and_assistant_messages() {
        let user_line = r#"{"timestamp":"2026-07-01T10:00:00Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"fix the login bug"}]}}"#;
        let assistant_line = r#"{"timestamp":"2026-07-01T10:00:05Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Sure, looking now."}]}}"#;

        let user = CodexSource::parse_line(user_line, "s1").unwrap().unwrap();
        assert_eq!(user.role, Role::User);
        assert_eq!(user.content, "fix the login bug");

        let assistant = CodexSource::parse_line(assistant_line, "s1").unwrap().unwrap();
        assert_eq!(assistant.role, Role::Assistant);
        assert_eq!(assistant.content, "Sure, looking now.");
    }

    #[test]
    fn skips_non_message_response_items() {
        let function_call = r#"{"timestamp":"2026-07-01T10:00:00Z","type":"response_item","payload":{"type":"function_call","name":"exec_command","arguments":"{}","call_id":"call_1"}}"#;
        let session_meta = r#"{"timestamp":"2026-07-01T10:00:00Z","type":"session_meta","payload":{"cli_version":"0.1.0"}}"#;

        assert!(CodexSource::parse_line(function_call, "s1").unwrap().is_none());
        assert!(CodexSource::parse_line(session_meta, "s1").unwrap().is_none());
    }

    #[test]
    fn skips_developer_role() {
        let line = r#"{"timestamp":"2026-07-01T10:00:00Z","type":"response_item","payload":{"type":"message","role":"developer","content":[{"type":"input_text","text":"you are a helpful coding agent"}]}}"#;
        assert!(CodexSource::parse_line(line, "s1").unwrap().is_none());
    }

    // Regression: environment_context and AGENTS.md are injected under
    // role=user, not typed by a human — must not count toward VOL/EMO.
    #[test]
    fn filters_injected_boilerplate_under_user_role() {
        let env_context = r#"{"timestamp":"2026-07-01T10:00:00Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"<environment_context>\n<cwd>/home/dev/project</cwd>\n<shell>bash</shell>\n</environment_context>"}]}}"#;
        let agents_md = r##"{"timestamp":"2026-07-01T10:00:00Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"# AGENTS.md instructions for repo root\n\nUse tabs, not spaces."}]}}"##;

        assert!(CodexSource::parse_line(env_context, "s1").unwrap().is_none());
        assert!(CodexSource::parse_line(agents_md, "s1").unwrap().is_none());
    }

    #[test]
    fn missing_timestamp_is_an_error_not_a_silent_skip() {
        let line = r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"hi"}]}}"#;
        assert!(CodexSource::parse_line(line, "s1").is_err());
    }
}
