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

Paths above are shown Unix-style for brevity — under the hood they're
resolved per-OS (`dirs::home_dir()`), so on Windows this is e.g.
`C:\Users\<you>\.claude\projects\`, not a literal `~`.

Nothing is scanned outside these four local paths, and no history content
ever leaves the machine. The only network request the app makes is fetching
your own public GitHub avatar (`avatars.githubusercontent.com`) to display on
the card.

## Getting started

**Prerequisites**: Rust (stable), Node.js 18+, and the Tauri v2 system
dependencies for your OS:

| OS | Install |
|---|---|
| **Linux** | `webkit2gtk-4.1`, `libayatana-appindicator3`, and the rest of the [Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux) (exact package names vary by distro). |
| **Windows** | [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload, and the MSVC Rust toolchain (`rustup default stable-msvc`). WebView2 is preinstalled on Windows 10 (1803+) and 11 — see the [Windows prerequisites](https://v2.tauri.app/start/prerequisites/#windows) if you need the Evergreen Bootstrapper. |
| **macOS** | Xcode Command Line Tools (`xcode-select --install`). Requires macOS 10.15 (Catalina) or later. |

```bash
npm install
npm run tauri dev
```

Build a release binary:

```bash
npm run tauri build
```

### Platform support

Developed and tested on **Linux** (Pop!_OS). Windows and macOS *should* work
— the Rust side only ever resolves paths through `dirs::home_dir()` /
`dirs::data_dir()` and `PathBuf::join()` (no hardcoded `/` or `~/...`), and
Tauri v2 itself targets all three natively — but neither has actually been
built or run there yet. If you try it on Windows or macOS, a bug report (or
just "it worked!") is very welcome.

## Project layout

```
src-tauri/   Rust backend — scanning, scoring, local persistence
  src/scanner/     one module per data source (all implement HistorySource)
  src/scoring.rs   Interaction[] -> PlayerStats (the 5 stats + archetype)
  src/snapshot.rs  daily JSONL history (~/.local/share/vibercard/snapshots.jsonl on Linux, OS equivalent elsewhere)
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
