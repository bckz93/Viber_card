# Contributing to DevCards

*[Lire en français](./CONTRIBUTING.fr.md)*

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
    archetypeRules.ts     the actual data (blurb, condition, example stats)
                          behind ArchetypeRules.tsx and AllCards — single
                          source of truth for both
    StatsRecap.tsx        punchline + per-stat explanation list
    archetypeArt.ts       archetype -> illustration
    archetypeLore.ts      archetype -> flavor text
    cardTheme.ts          archetype -> {gradient, accent, shineOpacity}
    statMeta.ts           the 5 stats' {key, label, icon} — single source
                          of truth, reused everywhere
  components/AllCards/
    AllCards.tsx           gallery of every archetype, rendered from
                          archetypeRules.ts's example stats — not real scans
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
| **VOL** | Volume / context size | Average *prose* words per message (see below), scaled so ~150 words/msg averages out to ~99. |
| **SPD** | Pace of requests | Median seconds between consecutive messages in the same session; ≤15s ≈ 99, ≥10min ≈ 0. |
| **NCT** | Nocturnal activity | % of messages sent outside 6am–10pm (UTC — not yet timezone-aware, see Known limitations). |
| **SLF** | Self-reliance / autonomy | Blend of prose length and hit-rate against a fixed list of "complex engineering" keywords (`refactor`, `architecture`, `test`, `async`, …). |
| **EMO** | Frustration/panic | % of messages matching a frustration keyword list, containing `"!!"`, or mostly uppercase ("shouting"). |

VOL and SLF both count *prose* words, not raw words — `prose_word_count()`
strips ``` fenced content and any individual token that's long (>24 chars)
or majority-non-letters (`is_prose_token()`), so pasted code/logs/stack
traces don't inflate either stat just because they're wordy. This matters
most with a small sample size: a couple of pasted stack traces across ~80
prompts used to be able to single-handedly max out VOL and SLF even though
the user barely wrote any prose themselves.

`total_tokens` (sum of `content.len() / 4` over every interaction, both
roles, in the window) isn't one of the 5 core stats, but it does gate one
archetype — see below.

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

`archetype_for()` in `scoring.rs` is **tiered and mutually exclusive, not a
priority list.** Each tier's condition already excludes everything claimed
by the tiers above it, so exactly one archetype can ever match a given stat
tuple — which archetype "wins" is never an accident of ordering. This
replaced an earlier flat list of independent `(condition, name)` rules that
*looked* like a priority list but actually had silently overlapping
conditions: `Token Exterminator` (`SLF>80 && VOL>80`) and `Self-Reliant
Sage` (`SLF>80 && EMO<=30`) both matched whenever VOL and EMO were in range
at once, and because Exterminator came first in the list, Sage was
unreachable for anyone verbose — a real bug, not a hypothetical one. If
you're adding a rule, preserve mutual exclusivity; don't go back to a flat
list.

The tiers, in order:

1. **Time-of-day** (NCT) — `NCT > 60` → `Nocturnal Panic Coder` (EMO > 50)
   or `Nocturnal Warrior` (EMO ≤ 50). A single extreme stat (NCT) is never
   enough on its own; EMO disambiguates a calm night owl from someone
   panicking at 3am.
2. **Frustration**, for non-nocturnal profiles — `EMO > 60` →
   `Emo-Driven Coder`.
3. **Autonomy-vs-verbosity quadrant** (SLF × VOL), for calm/non-nocturnal
   profiles:
   - `SLF > 80 && VOL > 80` → `Token Exterminator`, but *only* if
     `total_tokens > TOKEN_EXTERMINATOR_THRESHOLD` (200,000) — the
     archetype is named after tokens, so being verbose and autonomous isn't
     enough on its own. Below that bar, it resolves to whichever of SLF/VOL
     is more pronounced (`Self-Reliant Sage` or `The Novelist`) instead of
     falling through to a later tier — two maxed-out stats must never end
     up "Balanced".
   - `SLF > 80 && VOL ≤ 80` → `Self-Reliant Sage`.
   - `VOL > 80 && SLF ≤ 80` → `The Novelist`.
4. **Pace**, only for the quadrant's remaining cell (`SLF ≤ 80 && VOL ≤
   80`) — `SPD > 80` → `Spam Cannon` (VOL < 40) or `Rapid-Fire Debugger`
   (VOL ≥ 40).
5. **Fallback** — `Balanced Vibe Coder`, reached only once VOL, SLF, SPD,
   NCT and EMO are *all* below their "extreme" thresholds. If you add a
   condition that can be true here while some stat is still maxed out,
   you've reintroduced the "everything falls through to Balanced" bug.

## Adding a new archetype

There's no single flat list to append to anymore — you have to decide
which tier your archetype belongs in, and make sure its condition doesn't
overlap with an existing one in that tier (or, if it must, that the overlap
is resolved explicitly rather than by accidental ordering — see above).
Once you know where it goes, these files are all keyed by the exact same
string:

1. **`src-tauri/src/scoring.rs`**, `archetype_for()` — add the branch in the
   right tier.
2. **Same file**, `punchline_for()` — add a one-line joke for the match arm.
3. **Same file**, the `#[cfg(test)] mod tests` block — add a case asserting
   the new combo resolves to your new archetype, *and* a case asserting a
   neighboring combo still resolves to whatever it resolved to before (the
   regression shape that caught the Sage/Exterminator bug).
4. **`src/components/CurrentDeck/archetypeRules.ts`**, `ARCHETYPE_RULES` —
   the same condition, written as human-readable text (e.g. `"NCT > 60 and
   EMO ≤ 50"`), including whatever it excludes from the tiers above it, plus
   `exampleStats` sitting comfortably inside the new archetype's region (not
   on a threshold boundary). This one list feeds both the "Class Rules"
   panel (`ArchetypeRules.tsx`) and the "All Cards" gallery (`AllCards.tsx`)
   — there's no shared source of truth between Rust and this file, though; a
   comment at its top exists specifically to remind you to keep them in
   sync.
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
- **`is_prose_token()` is a cheap heuristic, not real language detection** —
  it catches pasted content by token length/letter-ratio (dotted Java-style
  stack traces, file paths with a line number, hex hashes), but a short,
  letter-heavy paste (a Python traceback's `File "..."` line, for example)
  can still slip through and count toward VOL/SLF. A smarter classifier
  (e.g. line-level heuristics instead of per-token) is a welcome PR, as long
  as it stays a fast, dependency-free heuristic — no NLP libraries for a
  humorous stat card.

## Pull requests

1. Fork, branch off `main`.
2. Keep PRs scoped — a new archetype is one PR, a new data source is
   another, a scoring threshold tweak is a third. Don't bundle unrelated
   changes.
3. Run `cargo test` and `npm run build` before opening the PR.
4. Explain *why*, not just *what* — especially for scoring changes, since
   every threshold here is a judgment call and the reasoning is what future
   contributors will need to revisit it.
