# Contributing to DevCards

DevCards is meant to be forked and extended — new data sources, new
archetypes. This doc explains the architecture and walks through the exact
files to touch for the most common changes.

## Architecture

```
src-tauri/  Rust, Tauri v2 backend
  src/scanner/        one module per data source, each implements HistorySource
    claude_code.rs
    hermes.rs
    ollama.rs
    mod.rs            the HistorySource trait + ScanError
  src/models.rs        Interaction, Role, Source, ScanResult — the shared
                       shape every scanner must produce
  src/scoring.rs       Vec<Interaction> -> PlayerStats (5 stats, archetype,
                       punchline, per-stat explanations)
  src/snapshot.rs      daily JSONL history of PlayerStats
  src/commands.rs      #[tauri::command] functions — the only place that
                       calls run_scan()/scoring::compute() and wires
                       persistence in

src/        React + TypeScript frontend
  components/CurrentDeck/
    FUTCard.tsx           the card itself
    RadarChart.tsx        single-series radar (used on the card)
    ArchetypeRules.tsx    the "Class Rules" panel — human-readable mirror
                          of scoring.rs::archetype_for
    StatsRecap.tsx        punchline + per-stat explanation list
    archetypeArt.ts       archetype -> illustration
    archetypeLore.ts      archetype -> flavor text
    cardTheme.ts          archetype -> {gradient, accent, shineOpacity}
    statMeta.ts           the 5 stats' {key, label, icon} — single source
                          of truth, reused everywhere
  components/EvolutionProgress/
    EvolutionProgress.tsx     picks the comparison snapshot, renders deltas
    EvolutionRadarChart.tsx   two-series (Then/Now) radar
  lib/tauri-api.ts       the *only* file that imports invoke() from
                         @tauri-apps/api/core — every command gets one
                         typed wrapper here
  hooks/useImageDataUrl.ts   fetches + center-crops an image to a square
                             data: URI (used for the GitHub avatar)
```

## Adding a new history source

Say you want to add support for Cursor, Aider, Windsurf, or anything else
that keeps a local chat log.

1. Create `src-tauri/src/scanner/my_tool.rs`:

   ```rust
   use super::{HistorySource, ScanError};
   use crate::models::{Interaction, Role, ScanResult, Source};
   use std::path::PathBuf;

   pub struct MyToolSource {
       pub path: PathBuf,
   }

   impl MyToolSource {
       pub fn default_path() -> Option<PathBuf> {
           dirs::home_dir().map(|h| h.join(".my-tool").join("history"))
       }

       pub fn new(path: PathBuf) -> Self {
           Self { path }
       }
   }

   impl HistorySource for MyToolSource {
       fn scan(&self) -> Result<ScanResult, ScanError> {
           // Parse your format, build Interaction::new(...) for each
           // user/assistant turn. Skip anything that isn't a real message
           // (system/tool/meta events) rather than counting it.
           todo!()
       }
   }
   ```

2. Add `Source::MyTool` to the enum in `models.rs`.
3. Register `pub mod my_tool;` in `scanner/mod.rs`.
4. Wire it into `run_scan()` in `commands.rs`, following the exact pattern
   already used for Claude Code / Hermes / Ollama: on failure push a
   `warnings` entry and move on, never `?`-propagate a hard error out of
   `run_scan()` — one broken source must not blank the whole card.
5. Add a unit test with a couple of fixture `Interaction`s (see the bottom
   of `hermes.rs` or `snapshot.rs` for the pattern: a scratch temp file,
   assert on the parsed result, clean up after).

**Constraints your scanner must respect**, because the scoring engine
depends on them:

- `Interaction::new` needs a real `DateTime<Utc>` timestamp. If your source
  has no reliable per-message timestamp (like Ollama's plain-text history),
  document your approximation clearly — it directly skews NCT (nocturnal %)
  and SPD (pace), which are both time-based. Ollama currently uses the
  file's mtime for every line as a documented, admitted approximation.
- Map roles to `Role::User` / `Role::Assistant` only; drop everything else
  (tool calls, system messages). If your source injects boilerplate/system
  text under the `user` role (Hermes does this for cron/skill triggers),
  filter it out — see the `[IMPORTANT:` check in `hermes.rs` for the
  pattern. Otherwise it inflates VOL/EMO with text the human never typed.

## How the 5 stats are computed

Everything lives in `src-tauri/src/scoring.rs`, computed over `Role::User`
messages only, from a **rolling 7-day window** (`CARD_WINDOW_DAYS` in
`commands.rs`) — not your lifetime history. This matters: the whole point of
Evolution Progress is comparing two non-overlapping weeks, which a lifetime
average would wash out.

| Stat | What it measures | How |
|---|---|---|
| **VOL** | Volume / context size | Average words per message, scaled so ~150 words/msg averages out to ~99. |
| **SPD** | Pace of requests | Median seconds between consecutive messages in the same session; ≤15s ≈ 99, ≥10min ≈ 0. |
| **NCT** | Nocturnal activity | % of messages sent outside 6am–10pm (UTC — not yet timezone-aware, see Known limitations). |
| **SLF** | Self-reliance / autonomy | Blend of prompt length and hit-rate against a fixed list of "complex engineering" keywords (`refactor`, `architecture`, `test`, `async`, …). |
| **EMO** | Frustration/panic | % of messages matching a frustration keyword list, containing `"!!"`, or mostly uppercase ("shouting"). |

All thresholds are **deliberate judgment calls for a humorous stat card**,
not a rigorous productivity metric — say so in your PR description if you're
tuning one, and explain the *why*, not just the new number.

There is intentionally **no single combined "overall" score**. An earlier
version had one; it turned out incoherent (should nocturnal activity count
as "good"? should frustration count as "good" when high, just because it's
a big number?). Removing it was a deliberate decision — don't reintroduce a
combined score without solving that polarity problem for every stat you fold
into it.

## How Evolution Progress sources its comparison

Deliberately **not** derived from `snapshots.jsonl`. Comparing two
already-computed rolling-window snapshots would double-smear the numbers
(each snapshot is itself an average over its own trailing 7 days, so two
snapshots a week apart still share 6 of those 7 days). Instead,
`get_stats_for_range(start, end)` (`commands.rs`) re-scans and re-computes
stats fresh, summed only over the raw interactions that actually fall in
`[start, end)` — the same `scoring::compute()` used everywhere else, just
given a different exact window:

- **Default ("now" vs "then")**: `[now-7d, now)` vs `[now-14d, now-7d)` —
  two adjacent, non-overlapping rolling weeks.
- **A specific calendar week** (`list_available_weeks()` + user picks one
  from the dropdown): Monday 00:00 UTC to the following Monday, for any
  completed week that has at least one interaction. The current, still
  in-progress week is never listed.

This means old raw logs (e.g. Claude Code JSONL files) need to still exist
on disk for a week to be comparable — if the underlying tool has since
rotated/deleted that history, that week silently won't appear in
`list_available_weeks()`. `snapshots.jsonl` remains a durable fallback
record for that scenario, just not the one Evolution Progress reads today.

## How the archetype ("class") is decided

`archetype_for()` in `scoring.rs` is a priority-ordered list of
`(condition, name)` rules — first match wins, with a `"Balanced Vibe Coder"`
fallback. **Every rule requires at least two stats to agree.** This is a
hard convention, not a style preference: a single extreme stat is never
enough to justify a class (e.g. high NCT alone can't tell a calm night owl
from someone panicking at 3am — EMO is what disambiguates the two, hence
`Nocturnal Warrior` vs `Nocturnal Panic Coder`).

## Adding a new archetype

Adding, say, `"Weekend Warrior"` touches these files, all keyed by the exact
same string:

1. **`src-tauri/src/scoring.rs`**, `archetype_for()` — add a
   `(condition, "Weekend Warrior")` entry in priority order (two-stat
   minimum, see above).
2. **Same file**, `punchline_for()` — add a one-line joke for the match arm.
3. **Same file**, the `#[cfg(test)] mod tests` block — add a case asserting
   the new combo actually resolves to your new archetype.
4. **`src/components/CurrentDeck/ArchetypeRules.tsx`**, `RULES` — the same
   condition, written as human-readable text (e.g. `"NCT > 60 and EMO ≤
   50"`). There's no shared source of truth between Rust and this list; a
   comment at the top of the file exists specifically to remind you to keep
   them in sync.
5. **`src/components/CurrentDeck/archetypeLore.ts`**, `ARCHETYPE_LORE` — one
   short, Pokédex-style flavor sentence.
6. **`src/components/CurrentDeck/cardTheme.ts`**, `ARCHETYPE_THEME` — a
   `{ gradient, accent, shineOpacity }` entry matching the archetype's vibe
   (existing entries: panic = red, calm night = navy, gold rush = amber,
   emo = pink/violet, sage = teal, novelist = sepia, detective = steel blue,
   balanced = soft iridescent — pick something distinct from all of these).
7. **Card art** — commission or generate an illustration (existing ones are
   ~370×230, no debug text/labels baked in, transparent-safe corners — see
   `src/assets/archetypes/*.png` for reference), save it there, then import
   and register it in `src/components/CurrentDeck/archetypeArt.ts`.

None of these steps will crash if skipped — a missing entry just quietly
falls back to nothing (no art) or a default (theme). That's exactly why a PR
adding a new archetype should touch all seven in one go; a half-added class
looks broken, not absent.

## Persistence

Everything is plain JSON under the OS data dir (`dirs::data_dir()` —
`~/.local/share/devcards/` on Linux, the platform equivalent elsewhere):

- **`snapshots.jsonl`** — one `{ taken_at, stats }` line appended per day (at
  most once/day, deduplicated). A durable local record independent of the
  underlying tools' own log retention — **not** what Evolution Progress
  reads from (see below).

No SQLite, no external DB for DevCards' own data — keep it that way unless
there's a strong reason. The point is a user can `cat`, inspect, or back up
their own data with zero tooling.

## Frontend/backend boundary

`src/lib/tauri-api.ts` is the *only* file that should import `invoke` from
`@tauri-apps/api/core`. Every Tauri command gets exactly one typed wrapper
function there. Don't call `invoke()` directly from a component — it makes
the command surface (and what the Rust side needs to support) grep-able in
one place.

## Testing

- **Backend**: `cd src-tauri && cargo test` — unit tests per module
  (scanner parsing, scoring thresholds, snapshot persistence roundtrips).
  `cargo run --example inspect` runs a full scan against your
  real local data and prints a summary — handy for eyeballing a scoring
  change against real history instead of only fixtures.
- **Frontend**: `npm run build` (tsc typecheck + Vite build) is the main
  correctness gate today. There's no component test suite yet — a
  contribution adding one (Vitest + Testing Library) is welcome.
- **Manual**: `npm run tauri dev` and actually look at the card. Anything
  touching layout, an archetype, or the radar chart should be visually
  checked, not just typechecked.

## Known limitations / good first contributions

- **NCT is UTC-only**, not timezone-aware — a night owl in a UTC+9
  timezone may get scored as if they work business hours. Converting to
  local time in `score_nct` (and in the frontend's explanation text) would
  be a solid, scoped PR.
- **Frustration/complexity keyword lists are mixed French/English** and
  fairly small (`FRUSTRATION_KEYWORDS`, `COMPLEXITY_KEYWORDS` in
  `scoring.rs`). An English-only or non-French/English speaker's real
  frustration may go undetected. Expanding or internationalizing these
  lists is welcome — keep them as plain `&[&str]` constants, no need for a
  config file for this.
- **Ollama has no per-message timestamps** — its contribution to NCT/SPD is
  a documented approximation (file mtime). If Ollama ever ships a richer
  history format, `scanner/ollama.rs` is the only file that needs to change.

## Pull requests

1. Fork, branch off `main`.
2. Keep PRs scoped — a new archetype is one PR, a new data source is
   another, a scoring threshold tweak is a third. Don't bundle unrelated
   changes.
3. Run `cargo test` and `npm run build` before opening the PR.
4. Explain *why*, not just *what* — especially for scoring changes, since
   every threshold here is a judgment call and the reasoning is what future
   contributors will need to revisit it.
