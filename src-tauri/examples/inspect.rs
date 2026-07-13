//! Quick manual test: scans every known local AI-history source on this
//! machine (Claude Code + Hermes + Ollama) and prints a combined summary,
//! without needing the full Tauri app running.
//! Run with: cargo run --example inspect

use vibercard_lib::models::{Role, ScanResult, Source};
use vibercard_lib::scanner::claude_code::ClaudeCodeSource;
use vibercard_lib::scanner::hermes::HermesSource;
use vibercard_lib::scanner::ollama::OllamaSource;
use vibercard_lib::scanner::HistorySource;
use vibercard_lib::scoring;
use std::collections::HashSet;

fn main() {
    let mut result = ScanResult::default();

    if let Ok(root) = ClaudeCodeSource::default_root() {
        println!("Scanning Claude Code: {}", root.display());
        match ClaudeCodeSource::new(root).scan() {
            Ok(r) => result = result.merge(r),
            Err(e) => println!("  ! {e}"),
        }
    }

    if let Some(path) = HermesSource::default_path() {
        println!("Scanning Hermes: {}", path.display());
        match HermesSource::new(path).scan() {
            Ok(r) => result = result.merge(r),
            Err(e) => println!("  ! {e}"),
        }
    }

    if let Some(path) = OllamaSource::default_path() {
        println!("Scanning Ollama: {}", path.display());
        match OllamaSource::new(path).scan() {
            Ok(r) => result = result.merge(r),
            Err(e) => println!("  ! {e}"),
        }
    }

    let user_msgs = result
        .interactions
        .iter()
        .filter(|i| matches!(i.role, Role::User))
        .count();
    let assistant_msgs = result.interactions.len() - user_msgs;

    let projects: HashSet<_> = result
        .interactions
        .iter()
        .filter_map(|i| i.project.as_deref())
        .collect();

    let night_msgs = result
        .interactions
        .iter()
        .filter(|i| matches!(i.role, Role::User))
        .filter(|i| {
            use chrono::Timelike;
            let h = i.timestamp.hour();
            !(6..22).contains(&h)
        })
        .count();

    println!("interactions: {}", result.interactions.len());
    println!("  user: {user_msgs}, assistant: {assistant_msgs}");
    for source in [Source::ClaudeCode, Source::Hermes, Source::Ollama] {
        let n = result.interactions.iter().filter(|i| i.source == source).count();
        println!("  from {source:?}: {n}");
    }
    println!("projects seen: {}", projects.len());
    for p in &projects {
        println!("  - {p}");
    }
    println!(
        "nocturnal user messages (22h-6h UTC): {night_msgs} ({:.1}% of user msgs)",
        if user_msgs > 0 {
            100.0 * night_msgs as f32 / user_msgs as f32
        } else {
            0.0
        }
    );

    if !result.warnings.is_empty() {
        println!("\n{} warnings (first 10 shown):", result.warnings.len());
        for w in result.warnings.iter().take(10) {
            println!("  ! {w}");
        }
    }

    // The Current Deck card is a rolling 7-day window, not a lifetime
    // average — same cutoff as commands::CARD_WINDOW_DAYS.
    let range_end = chrono::Utc::now();
    let range_start = range_end - chrono::Duration::days(7);
    let recent = result.clone().keep_recent(7);
    println!(
        "\n(scoring the last 7 days: {} of {} total interactions)",
        recent.interactions.len(),
        result.interactions.len()
    );

    let stats = scoring::compute(&recent.interactions, range_start, range_end);
    println!(
        "\nPLAYER CARD [{}] ({} to {})\n  VOL {}  SPD {}  NCT {}  SLF {}  EMO {}",
        stats.archetype,
        stats.range_start.format("%Y-%m-%d"),
        stats.range_end.format("%Y-%m-%d"),
        stats.vol,
        stats.spd,
        stats.nct,
        stats.slf,
        stats.emo
    );

    if let Some(path) = vibercard_lib::snapshot::snapshots_path() {
        let saved = vibercard_lib::snapshot::snapshot_if_new_day(&path, &stats).unwrap_or(false);
        let history = vibercard_lib::snapshot::load_snapshots(&path).unwrap_or_default();
        println!(
            "\nsnapshot: {} (file: {}, {} total)",
            if saved { "saved (new day)" } else { "already have one for today" },
            path.display(),
            history.len()
        );
    }

    if let Some(first) = result.interactions.iter().min_by_key(|i| i.timestamp) {
        println!("\nsample earliest interaction:");
        println!("  [{}] {:?}: {}", first.timestamp, first.role, truncate(&first.content, 120));
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}
