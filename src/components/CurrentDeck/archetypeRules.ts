import type { PlayerStats } from '../../types/stats'

// Mirrors the tiered, mutually-exclusive zones in
// src-tauri/src/scoring.rs::archetype_for. Keep in sync manually if
// thresholds change there. Each condition already excludes everything
// claimed by the rows above it — that's what makes every class reachable
// (see the doc comment on archetype_for for why that matters).
//
// `exampleStats` sits comfortably inside each archetype's own region (not
// on a threshold boundary) — used by AllCards to render a representative
// card for every class without needing a real scan.
export interface ArchetypeRule {
  archetype: string
  blurb: string
  condition: string
  exampleStats: Pick<PlayerStats, 'vol' | 'spd' | 'nct' | 'slf' | 'emo' | 'total_tokens'>
}

export const ARCHETYPE_RULES: ArchetypeRule[] = [
  {
    archetype: 'Nocturnal Panic Coder',
    blurb: 'A stressed-out night owl — most sessions land after dark and it shows.',
    condition: 'NCT > 60 and EMO > 50',
    exampleStats: { vol: 55, spd: 55, nct: 85, slf: 50, emo: 75, total_tokens: 60_000 },
  },
  {
    archetype: 'Nocturnal Warrior',
    blurb: 'A calm night owl — codes late but never loses their cool.',
    condition: 'NCT > 60 and EMO ≤ 50',
    exampleStats: { vol: 55, spd: 45, nct: 85, slf: 55, emo: 15, total_tokens: 60_000 },
  },
  {
    archetype: 'Emo-Driven Coder',
    blurb: "Frustration runs the session — you're mostly a daytime coder, but panic signals show up a lot.",
    condition: 'NCT ≤ 60 and EMO > 60',
    exampleStats: { vol: 50, spd: 50, nct: 20, slf: 45, emo: 80, total_tokens: 50_000 },
  },
  {
    archetype: 'Token Exterminator',
    blurb:
      'Verbose and autonomous, and the token count proves it — long, detailed prompts asking for real engineering work, burning serious volume.',
    condition: 'SLF > 80, VOL > 80, and total tokens > 200,000 (NCT ≤ 60, EMO ≤ 60)',
    exampleStats: { vol: 90, spd: 55, nct: 20, slf: 90, emo: 20, total_tokens: 280_000 },
  },
  {
    archetype: 'Self-Reliant Sage',
    blurb: "Autonomous without the bloat — asks for real engineering work but doesn't need an essay to get there.",
    condition:
      'SLF > 80 and VOL ≤ 80 — or SLF > 80, VOL > 80, SLF ≥ VOL, and tokens ≤ 200,000 (NCT ≤ 60, EMO ≤ 60)',
    exampleStats: { vol: 55, spd: 45, nct: 15, slf: 90, emo: 15, total_tokens: 60_000 },
  },
  {
    archetype: 'The Novelist',
    blurb: 'Writes prompts longer than some technical specs — context is a love language.',
    condition:
      'VOL > 80 and SLF ≤ 80 — or VOL > 80, SLF > 80, VOL > SLF, and tokens ≤ 200,000 (NCT ≤ 60, EMO ≤ 60)',
    exampleStats: { vol: 90, spd: 45, nct: 15, slf: 55, emo: 15, total_tokens: 70_000 },
  },
  {
    archetype: 'Spam Cannon',
    blurb: 'Fires off many short messages back to back instead of one detailed one.',
    condition: 'SPD > 80 and VOL < 40 (SLF ≤ 80, VOL ≤ 80, NCT ≤ 60, EMO ≤ 60)',
    exampleStats: { vol: 25, spd: 90, nct: 15, slf: 35, emo: 15, total_tokens: 40_000 },
  },
  {
    archetype: 'Rapid-Fire Debugger',
    blurb: 'Quick back-and-forth — fast pace, but prompts still have enough meat on them.',
    condition: 'SPD > 80 and VOL ≥ 40 (SLF ≤ 80, VOL ≤ 80, NCT ≤ 60, EMO ≤ 60)',
    exampleStats: { vol: 60, spd: 90, nct: 15, slf: 45, emo: 15, total_tokens: 70_000 },
  },
  {
    archetype: 'Balanced Vibe Coder',
    blurb: "Nothing stands out — none of your 5 stats crosses into 'extreme' territory this week.",
    condition: 'Default — no other rule matched',
    exampleStats: { vol: 50, spd: 45, nct: 25, slf: 45, emo: 20, total_tokens: 45_000 },
  },
]
