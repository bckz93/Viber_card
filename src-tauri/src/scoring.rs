//! Turns raw `Interaction`s into the 5 FUT-style stats (0-99). Every
//! threshold below is a deliberate, tunable judgment call for a humorous
//! stat card, not a rigorous productivity metric.

use crate::models::{Interaction, Role};
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const FRUSTRATION_KEYWORDS: &[&str] = &[
    "marche pas",
    "marche toujours pas",
    "ça bug",
    "ca bug",
    "erreur",
    "error",
    "pourquoi ça",
    "pourquoi ca",
    "n'importe quoi",
    "toujours pareil",
];

const COMPLEXITY_KEYWORDS: &[&str] = &[
    "refactor",
    "architecture",
    "optimis",
    "sécuri",
    "securi",
    "test",
    "design",
    "scalab",
    "performance",
    "concurrence",
    "async",
    "migration",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatInsight {
    pub key: String,
    pub label: String,
    pub value: u8,
    /// Plain-language explanation of *why* this particular score, grounded
    /// in the value itself (not just the archetype).
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub vol: u8,
    pub spd: u8,
    pub nct: u8,
    pub slf: u8,
    pub emo: u8,
    /// Humorous archetype label derived from the stat profile (e.g. "Nocturnal
    /// Warrior").
    pub archetype: String,
    /// One-liner joke tied to the archetype.
    pub punchline: String,
    /// Per-stat plain-language explanations, in VOL/SPD/NCT/SLF/EMO order.
    pub insights: Vec<StatInsight>,
    pub sample_size: usize,
    pub total_tokens: u64,
    /// The exact window these stats were summed over — shown to the user
    /// so "your stats" always comes with "as measured from X to Y", never
    /// an unstated implicit period.
    pub range_start: DateTime<Utc>,
    pub range_end: DateTime<Utc>,
}

pub fn compute(interactions: &[Interaction], range_start: DateTime<Utc>, range_end: DateTime<Utc>) -> PlayerStats {
    let user_msgs: Vec<&Interaction> = interactions
        .iter()
        .filter(|i| matches!(i.role, Role::User))
        .collect();

    if user_msgs.is_empty() {
        return PlayerStats {
            vol: 0,
            spd: 0,
            nct: 0,
            slf: 0,
            emo: 0,
            archetype: "Unranked".to_string(),
            punchline: "No data yet — go write some prompts.".to_string(),
            insights: Vec::new(),
            sample_size: 0,
            total_tokens: 0,
            range_start,
            range_end,
        };
    }

    let vol = score_vol(&user_msgs);
    let spd = score_spd(&user_msgs);
    let nct = score_nct(&user_msgs);
    let slf = score_slf(&user_msgs);
    let emo = score_emo(&user_msgs);
    let total_tokens = interactions.iter().map(|i| i.token_estimate as u64).sum();
    let archetype = archetype_for(vol, spd, nct, slf, emo, total_tokens);

    PlayerStats {
        vol,
        spd,
        nct,
        slf,
        emo,
        punchline: punchline_for(&archetype),
        insights: vec![
            StatInsight {
                key: "vol".to_string(),
                label: "Volume".to_string(),
                value: vol,
                explanation: explain_vol(vol),
            },
            StatInsight {
                key: "spd".to_string(),
                label: "Speed".to_string(),
                value: spd,
                explanation: explain_spd(spd),
            },
            StatInsight {
                key: "nct".to_string(),
                label: "Nocturnal".to_string(),
                value: nct,
                explanation: explain_nct(nct),
            },
            StatInsight {
                key: "slf".to_string(),
                label: "Self-Reliance".to_string(),
                value: slf,
                explanation: explain_slf(slf),
            },
            StatInsight {
                key: "emo".to_string(),
                label: "Emotion".to_string(),
                value: emo,
                explanation: explain_emo(emo),
            },
        ],
        archetype,
        sample_size: user_msgs.len(),
        total_tokens,
        range_start,
        range_end,
    }
}

fn clamp99(x: f32) -> u8 {
    x.clamp(0.0, 99.0).round() as u8
}

/// A whitespace-split token counts as prose only if it's short (≤24 chars)
/// and majority-letters. Pasted stack traces, file paths, and dotted/camelCase
/// identifiers (`com.example.Service.process(Service.java:142)`) rarely
/// contain spaces, so splitting on whitespace alone turns each pasted line
/// into one or two very long tokens — this catches those even when the
/// paste isn't wrapped in a ``` fence. 24 chars is well above any ordinary
/// English/French word, so it costs essentially nothing on genuine prose.
fn is_prose_token(token: &str) -> bool {
    let total = token.chars().count();
    if total == 0 || total > 24 {
        return false;
    }
    let letters = token.chars().filter(|c| c.is_alphabetic()).count();
    letters * 2 >= total
}

/// Word count of the parts of `content` outside ``` fenced blocks, further
/// filtered to prose-looking tokens (see `is_prose_token`). Pasted
/// code/logs/stack traces aren't "how much you wrote" — they're context you
/// dropped in — so neither fenced content nor unfenced code-like tokens
/// count toward it. Without this, a single big paste in an otherwise short
/// message inflates VOL and SLF together (both are built on this average),
/// which is especially distorting with a small sample size: a handful of
/// pasted stack traces across ~80 prompts can single-handedly max out both
/// stats even though the user barely wrote any prose themselves.
fn prose_word_count(content: &str) -> usize {
    let mut in_fence = false;
    let mut words = 0;
    for part in content.split("```") {
        if !in_fence {
            words += part.split_whitespace().filter(|w| is_prose_token(w)).count();
        }
        in_fence = !in_fence;
    }
    words
}

fn avg_words(msgs: &[&Interaction]) -> f32 {
    msgs.iter()
        .map(|m| prose_word_count(&m.content))
        .sum::<usize>() as f32
        / msgs.len() as f32
}

/// Average prose length in words (pasted ``` content excluded — see
/// `prose_word_count`). ~150 words/msg average maxes the score out.
fn score_vol(msgs: &[&Interaction]) -> u8 {
    clamp99(avg_words(msgs) / 150.0 * 99.0)
}

/// Median delay between consecutive user messages within the same session:
/// sub-15s median reads as rapid-fire "spamtivity" (near-max), 10min+ as calm (near 0).
fn score_spd(msgs: &[&Interaction]) -> u8 {
    let mut by_session: HashMap<&str, Vec<chrono::DateTime<chrono::Utc>>> = HashMap::new();
    for m in msgs {
        let key = m.session_id.as_deref().unwrap_or("_");
        by_session.entry(key).or_default().push(m.timestamp);
    }

    let mut deltas: Vec<f32> = Vec::new();
    for times in by_session.values_mut() {
        times.sort();
        for w in times.windows(2) {
            let d = (w[1] - w[0]).num_seconds();
            if d > 0 {
                deltas.push(d as f32);
            }
        }
    }

    if deltas.is_empty() {
        return 50; // not enough data to judge rhythm - neutral score
    }

    deltas.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = deltas[deltas.len() / 2];

    const FAST_SECS: f32 = 15.0;
    const SLOW_SECS: f32 = 600.0;
    let t = ((SLOW_SECS - median) / (SLOW_SECS - FAST_SECS)).clamp(0.0, 1.0);
    clamp99(t * 99.0)
}

/// % of user messages sent 22h-06h. Timestamps are UTC; a future version
/// should convert to the user's local timezone before bucketing.
fn score_nct(msgs: &[&Interaction]) -> u8 {
    let night = msgs
        .iter()
        .filter(|m| !(6..22).contains(&m.timestamp.hour()))
        .count();
    clamp99(night as f32 / msgs.len() as f32 * 99.0)
}

/// Blend of prose length (pasted ``` content excluded, see `prose_word_count`)
/// and "complex engineering vocabulary" hit-rate, as a proxy for
/// design/autonomy work vs basic hand-holding requests.
fn score_slf(msgs: &[&Interaction]) -> u8 {
    let complex_ratio = msgs
        .iter()
        .filter(|m| {
            let lower = m.content.to_lowercase();
            COMPLEXITY_KEYWORDS.iter().any(|k| lower.contains(k))
        })
        .count() as f32
        / msgs.len() as f32;

    clamp99((avg_words(msgs) / 100.0 * 50.0) + (complex_ratio * 49.0))
}

/// % of user messages carrying a frustration signal: keyword hit, "!!", or
/// a mostly-uppercase message (shouting).
fn score_emo(msgs: &[&Interaction]) -> u8 {
    let hits = msgs
        .iter()
        .filter(|m| {
            let lower = m.content.to_lowercase();
            let has_keyword = FRUSTRATION_KEYWORDS.iter().any(|k| lower.contains(k));
            let has_bangs = m.content.contains("!!");
            let letters = m.content.chars().filter(|c| c.is_alphabetic()).count();
            let uppercase = m.content.chars().filter(|c| c.is_uppercase()).count();
            let shouting = letters > 6 && uppercase as f32 / letters as f32 > 0.7;
            has_keyword || has_bangs || shouting
        })
        .count();

    clamp99(hits as f32 / msgs.len() as f32 * 99.0)
}

/// Tokens estimated over the 7-day window at which someone reads as
/// "burning through context at an industrial rate" rather than merely
/// verbose. Like every other threshold in this file, a tunable judgment
/// call for a humorous stat card, not a rigorous cutoff.
const TOKEN_EXTERMINATOR_THRESHOLD: u64 = 200_000;

/// Tiered, mutually-exclusive zones — NOT a priority list. Each tier's
/// condition already excludes everything claimed by the tiers above it, so
/// exactly one archetype can ever match a given stat tuple. (An earlier
/// version used independent conditions that overlapped — e.g. Token
/// Exterminator = SLF>80 && VOL>80 and Self-Reliant Sage = SLF>80 &&
/// EMO<=30 both matched whenever VOL and EMO were in range at once — so
/// being first in the list silently made Sage unreachable for anyone
/// verbose. The tiers below fix that by construction.)
///
/// 1. time-of-day (NCT)
/// 2. frustration, for non-nocturnal (EMO)
/// 3. autonomy-vs-verbosity quadrant (SLF x VOL), for calm/non-nocturnal —
///    the SLF>80 && VOL>80 cell additionally requires an actual heavy
///    token count to earn "Token Exterminator" specifically (the archetype
///    is named after tokens, so VOL/SLF alone was never enough). Falling
///    short of that bar still means both stats are extreme, so it resolves
///    to whichever of the two defining traits is more pronounced — it must
///    NOT fall through to pace/balanced, or two maxed-out stats could be
///    labeled "Balanced".
/// 4. pace, for the one cell that's neither verbose nor autonomous (SPD)
/// 5. fallback — reached only once VOL, SLF, SPD, NCT and EMO are all
///    below their "extreme" thresholds, i.e. actually balanced.
fn archetype_for(vol: u8, spd: u8, nct: u8, slf: u8, emo: u8, total_tokens: u64) -> String {
    if nct > 60 {
        return if emo > 50 { "Nocturnal Panic Coder" } else { "Nocturnal Warrior" }.to_string();
    }

    if emo > 60 {
        return "Emo-Driven Coder".to_string();
    }

    match (slf > 80, vol > 80) {
        (true, true) => {
            return if total_tokens > TOKEN_EXTERMINATOR_THRESHOLD {
                "Token Exterminator"
            } else if slf >= vol {
                "Self-Reliant Sage"
            } else {
                "The Novelist"
            }
            .to_string();
        }
        (true, false) => return "Self-Reliant Sage".to_string(),
        (false, true) => return "The Novelist".to_string(),
        (false, false) => {}
    }

    if spd > 80 {
        return if vol < 40 { "Spam Cannon" } else { "Rapid-Fire Debugger" }.to_string();
    }

    "Balanced Vibe Coder".to_string()
}

fn punchline_for(archetype: &str) -> String {
    match archetype {
        "Nocturnal Panic Coder" => {
            "3am, 12 stack traces, 0 coffee left. A truly cursed combo."
        }
        "Nocturnal Warrior" => "While everyone sleeps, you ship. Calmly. In the dark.",
        "Token Exterminator" => "You don't write code, you exterminate tickets bare-handed.",
        "Spam Cannon" => "Why send one message when fifteen will do?",
        "Emo-Driven Coder" => "Your code compiles. Your emotions don't.",
        "Self-Reliant Sage" => "You need no one. Not even Claude, really.",
        "The Novelist" => "Tolstoy would've loved your prompts.",
        "Rapid-Fire Debugger" => "Sherlock Holmes of bugs, minus the pipe, plus more caffeine.",
        _ => "Not too hot, not too cold. Balance, incarnate.",
    }
    .to_string()
}

fn explain_vol(v: u8) -> String {
    if v >= 80 {
        "Your prompts read like short novels — you dump all the context in one go.".to_string()
    } else if v >= 40 {
        "Decent-length prompts, not too short, not too long.".to_string()
    } else {
        "Short and to the point — minimal context, straight to the ask.".to_string()
    }
}

fn explain_spd(v: u8) -> String {
    if v >= 80 {
        "Messages fire off seconds apart — full rapid-fire mode.".to_string()
    } else if v >= 40 {
        "A steady pace, no long pauses between prompts.".to_string()
    } else {
        "You take your time between requests, no rush.".to_string()
    }
}

fn explain_nct(v: u8) -> String {
    if v >= 60 {
        "More than half your messages land between 10pm and 6am.".to_string()
    } else if v >= 20 {
        "A handful of late-night sessions, nothing excessive.".to_string()
    } else {
        "Strictly business hours — a model of sleep hygiene.".to_string()
    }
}

fn explain_slf(v: u8) -> String {
    if v >= 80 {
        "You ask for architecture, refactors, tests — running your own product roadmap.".to_string()
    } else if v >= 40 {
        "A healthy mix of autonomous asks and basic questions.".to_string()
    } else {
        "You prefer being walked through things step by step.".to_string()
    }
}

fn explain_emo(v: u8) -> String {
    if v >= 60 {
        "Lots of \"!!!\" and \"it doesn't work\" in your messages — panic is close.".to_string()
    } else if v >= 20 {
        "A few signs of frustration here and there, nothing alarming.".to_string()
    } else {
        "Zen in every circumstance — not a single rage-quit detected.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Source;

    fn user_msg(text: &str, ts: &str) -> Interaction {
        Interaction::new(
            ts.parse().unwrap(),
            Role::User,
            text.to_string(),
            Source::ClaudeCode,
            None,
            Some("s1".to_string()),
        )
    }

    /// The range only matters for display, not for these tests' assertions.
    fn test_compute(interactions: &[Interaction]) -> PlayerStats {
        compute(interactions, Utc::now(), Utc::now())
    }

    #[test]
    fn empty_input_does_not_panic() {
        let stats = test_compute(&[]);
        assert_eq!(stats.vol, 0);
        assert_eq!(stats.archetype, "Unranked");
    }

    #[test]
    fn frustration_keywords_drive_emo_up() {
        let msgs = vec![
            user_msg("ça marche pas !!", "2026-07-01T10:00:00Z"),
            user_msg("pourquoi ça bug encore", "2026-07-01T10:01:00Z"),
        ];
        let stats = test_compute(&msgs);
        assert!(stats.emo > 80, "expected high EMO, got {}", stats.emo);
    }

    #[test]
    fn calm_long_messages_keep_emo_low() {
        let msgs = vec![user_msg(
            "Peux-tu ajouter un endpoint pour lister les utilisateurs actifs, avec pagination et un filtre par date de dernière connexion.",
            "2026-07-01T10:00:00Z",
        )];
        let stats = test_compute(&msgs);
        assert_eq!(stats.emo, 0);
    }

    // Regression: archetype_for used to be a flat priority list where Token
    // Exterminator's condition (SLF>80 && VOL>80) overlapped with Self-Reliant
    // Sage's (SLF>80 && EMO<=30) and always won by virtue of coming first,
    // making Sage unreachable for any verbose-but-calm profile. The two now
    // partition the SLF>80 quadrant by VOL alone, so both are reachable.
    #[test]
    fn self_reliant_sage_reachable_with_high_slf_and_moderate_vol() {
        assert_eq!(archetype_for(70, 10, 10, 90, 10, 5_000), "Self-Reliant Sage");
    }

    #[test]
    fn token_exterminator_owns_high_vol_half_of_high_slf_quadrant_when_token_heavy() {
        assert_eq!(archetype_for(90, 10, 10, 90, 10, 250_000), "Token Exterminator");
    }

    // Regression: the archetype is named after tokens, so verbose (VOL>80) +
    // autonomous (SLF>80) alone must not be enough to earn it — a low total
    // token count in that same cell must resolve to one of the two defining
    // traits instead (never Token Exterminator).
    #[test]
    fn verbose_and_autonomous_but_token_light_falls_back_to_dominant_trait() {
        assert_eq!(archetype_for(90, 10, 10, 90, 10, 1_000), "Self-Reliant Sage");
        assert_eq!(archetype_for(99, 10, 10, 85, 10, 1_000), "The Novelist");
    }

    // Regression: two maxed-out stats (VOL=99, SLF=99) used to be able to
    // fall all the way through to "Balanced Vibe Coder" whenever pace and
    // token count were unremarkable — Balanced must require every stat to
    // actually be unremarkable, not just "matched no other rule".
    #[test]
    fn maxed_vol_and_slf_are_never_balanced() {
        assert_ne!(archetype_for(99, 50, 10, 99, 10, 50_000), "Balanced Vibe Coder");
    }

    #[test]
    fn genuinely_unremarkable_stats_are_balanced() {
        assert_eq!(archetype_for(50, 50, 20, 50, 20, 20_000), "Balanced Vibe Coder");
    }

    // Regression: a big pasted code block used to count fully toward
    // avg_words, so a handful of short prompts with large pastes could max
    // out VOL/SLF even though the user barely wrote anything themselves.
    #[test]
    fn pasted_code_block_does_not_inflate_vol_or_slf() {
        let padding = "line_of_code();\n".repeat(120); // ~360 words, all pasted
        let msgs = vec![
            user_msg(&format!("fix this\n```\n{padding}\n```"), "2026-07-01T10:00:00Z"),
            user_msg("still broken", "2026-07-01T10:05:00Z"),
        ];
        let stats = test_compute(&msgs);
        assert!(stats.vol < 20, "expected low VOL from ~1 prose word/msg, got {}", stats.vol);
        assert!(stats.slf < 20, "expected low SLF from ~1 prose word/msg, got {}", stats.slf);
    }

    #[test]
    fn genuinely_long_prose_still_drives_vol_up() {
        let msgs = vec![user_msg(
            &"word ".repeat(160),
            "2026-07-01T10:00:00Z",
        )];
        let stats = test_compute(&msgs);
        assert!(stats.vol >= 80, "expected high VOL from genuine prose, got {}", stats.vol);
    }

    // Regression: a pasted stack trace with no ``` fence used to still
    // count fully toward avg_words, since split_whitespace() alone doesn't
    // know a dotted Java-style trace line isn't prose.
    #[test]
    fn unfenced_stack_trace_does_not_inflate_vol_or_slf() {
        let trace = (0..40)
            .map(|i| format!("at com.example.service.ProcessingServiceImpl.handle{i}(ProcessingServiceImpl.java:{i})"))
            .collect::<Vec<_>>()
            .join("\n");
        let msgs = vec![
            user_msg(&format!("fix this\n{trace}"), "2026-07-01T10:00:00Z"),
            user_msg("still broken", "2026-07-01T10:05:00Z"),
        ];
        let stats = test_compute(&msgs);
        assert!(stats.vol < 20, "expected low VOL from unfenced trace paste, got {}", stats.vol);
        assert!(stats.slf < 20, "expected low SLF from unfenced trace paste, got {}", stats.slf);
    }
}
