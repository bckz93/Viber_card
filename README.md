# ViberCard

A local-first desktop app that scans your **Claude Code**, **Codex**, **Ollama**,
and **Hermes** history and turns your prompting habits into a humorous,
FUT/Pokémon-style stat card — with a class ("archetype"), a radar chart, and
week-over-week evolution tracking.

Everything runs on your machine. Nothing is uploaded anywhere.

## Features

- **Current Deck** — a shareable stat card: 5 stats (VOL/SPD/NCT/SLF/EMO,
  0-99), a humorous archetype/class with its own art, lore, and color theme,
  a radar chart, and a one-click HD PNG export.
- **Evolution Progress** — compares your current 7-day window against the
  snapshot closest to a week ago, so you can actually see behavior change
  instead of a lifetime average that barely moves.
- **Multi-source scan** — combines Claude Code, Codex, Ollama, and your
  Hermes agent (if present) into one profile. A source that's missing or
  unreadable just produces a warning; it never blocks the others.

## Stack

- **Backend**: [Tauri v2](https://v2.tauri.app/) (Rust)
- **Frontend**: React + TypeScript + Vite + Tailwind CSS v4
- **Charts**: Recharts

## Where your data comes from

| Source | Read from | Notes |
|---|---|---|
| Claude Code | `~/.claude/projects/**/*.jsonl` | Full timestamps, real conversation turns. |
| Codex | `~/.codex/sessions/**/rollout-*.jsonl` | Full timestamps; injected boilerplate (`<environment_context>`, `AGENTS.md`) is filtered out. |
| Hermes | `~/.hermes/state.db` (SQLite) | Only if the Hermes agent is installed. |
| Ollama | `~/.ollama/history` | CLI readline history only — no timestamps or assistant replies, so it barely affects time-based stats (NCT/SPD). |

Nothing is scanned outside these four local paths, and no history content
ever leaves the machine. The only network request the app makes is fetching
your own public GitHub avatar (`avatars.githubusercontent.com`) to display on
the card.

## Getting started

**Prerequisites**: Rust (stable), Node.js 18+, and the [Tauri v2 system
dependencies](https://v2.tauri.app/start/prerequisites/) for your OS (on
Linux: `webkit2gtk-4.1`, `libayatana-appindicator3`, etc.).

```bash
npm install
npm run tauri dev
```

Build a release binary:

```bash
npm run tauri build
```

## Project layout

```
src-tauri/   Rust backend — scanning, scoring, local persistence
  src/scanner/     one module per data source (all implement HistorySource)
  src/scoring.rs   Interaction[] -> PlayerStats (the 5 stats + archetype)
  src/snapshot.rs  daily JSONL history (~/.local/share/vibercard/snapshots.jsonl)
  src/commands.rs  the Tauri commands the frontend calls

src/         React frontend
  components/CurrentDeck/       the card, radar, rules, recap
  components/EvolutionProgress/ the week-over-week comparison panel
  lib/tauri-api.ts              the only file that calls invoke()
```

## Contributing

Adding a new history source or a new archetype/class are both intentionally
scoped, well-defined changes — see [CONTRIBUTING.md](./CONTRIBUTING.md) for
exactly which files to touch and why.

## License

MIT — see [LICENSE](./LICENSE).
