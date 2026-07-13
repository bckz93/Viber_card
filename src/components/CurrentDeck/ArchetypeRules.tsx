// Mirrors the tiered, mutually-exclusive zones in
// src-tauri/src/scoring.rs::archetype_for. Keep in sync manually if
// thresholds change there. Each condition already excludes everything
// claimed by the rows above it — that's what makes every class reachable
// (see the doc comment on archetype_for for why that matters).
const RULES: Array<{ archetype: string; blurb: string; condition: string }> = [
  {
    archetype: 'Nocturnal Panic Coder',
    blurb: 'A stressed-out night owl — most sessions land after dark and it shows.',
    condition: 'NCT > 60 and EMO > 50',
  },
  {
    archetype: 'Nocturnal Warrior',
    blurb: 'A calm night owl — codes late but never loses their cool.',
    condition: 'NCT > 60 and EMO ≤ 50',
  },
  {
    archetype: 'Emo-Driven Coder',
    blurb: "Frustration runs the session — you're mostly a daytime coder, but panic signals show up a lot.",
    condition: 'NCT ≤ 60 and EMO > 60',
  },
  {
    archetype: 'Token Exterminator',
    blurb:
      'Verbose and autonomous, and the token count proves it — long, detailed prompts asking for real engineering work, burning serious volume.',
    condition: 'SLF > 80, VOL > 80, and total tokens > 200,000 (NCT ≤ 60, EMO ≤ 60)',
  },
  {
    archetype: 'Self-Reliant Sage',
    blurb: "Autonomous without the bloat — asks for real engineering work but doesn't need an essay to get there.",
    condition:
      'SLF > 80 and VOL ≤ 80 — or SLF > 80, VOL > 80, SLF ≥ VOL, and tokens ≤ 200,000 (NCT ≤ 60, EMO ≤ 60)',
  },
  {
    archetype: 'The Novelist',
    blurb: 'Writes prompts longer than some technical specs — context is a love language.',
    condition:
      'VOL > 80 and SLF ≤ 80 — or VOL > 80, SLF > 80, VOL > SLF, and tokens ≤ 200,000 (NCT ≤ 60, EMO ≤ 60)',
  },
  {
    archetype: 'Spam Cannon',
    blurb: 'Fires off many short messages back to back instead of one detailed one.',
    condition: 'SPD > 80 and VOL < 40 (SLF ≤ 80, VOL ≤ 80, NCT ≤ 60, EMO ≤ 60)',
  },
  {
    archetype: 'Rapid-Fire Debugger',
    blurb: 'Quick back-and-forth — fast pace, but prompts still have enough meat on them.',
    condition: 'SPD > 80 and VOL ≥ 40 (SLF ≤ 80, VOL ≤ 80, NCT ≤ 60, EMO ≤ 60)',
  },
  {
    archetype: 'Balanced Vibe Coder',
    blurb: "Nothing stands out — none of your 5 stats crosses into 'extreme' territory this week.",
    condition: 'Default — no other rule matched',
  },
]

function RuleRow({
  title,
  blurb,
  condition,
  index,
  active,
}: {
  title: string
  blurb: string
  condition: string
  index: number
  active: boolean
}) {
  return (
    <div
      className="flex items-baseline gap-3 rounded-lg px-3 py-2"
      style={{
        background: active ? 'rgba(57,135,229,0.15)' : 'transparent',
        border: active ? '1px solid rgba(57,135,229,0.5)' : '1px solid transparent',
      }}
    >
      <div className="text-[10px] text-white/30 w-4 shrink-0">{index}</div>
      <div>
        <div className={`text-sm font-semibold ${active ? 'text-white' : 'text-white/80'}`}>
          {title}
          {active && <span className="ml-2 text-[10px] text-[#3987e5] font-bold">YOU</span>}
        </div>
        <div className="text-xs text-white/60 leading-snug mt-0.5 mb-1">{blurb}</div>
        <div className="text-xs font-mono text-white/40">{condition}</div>
      </div>
    </div>
  )
}

export function ArchetypeRules({ currentArchetype }: { currentArchetype?: string }) {
  return (
    <div
      className="w-full h-full rounded-2xl p-5 flex flex-col"
      style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.08)' }}
    >
      <div className="text-xs font-bold uppercase tracking-[0.2em] text-white/30 mb-1">
        Class Rules
      </div>
      <p className="text-xs text-white/40 mb-3">
        Evaluated top to bottom, but each row's condition already excludes everything above it —
        these are mutually exclusive zones, not a priority list, so no class can block another.
      </p>
      <div className="flex flex-col gap-2 flex-1 min-h-0 overflow-y-auto">
        {RULES.map((rule, i) => (
          <RuleRow
            key={rule.archetype}
            title={rule.archetype}
            blurb={rule.blurb}
            condition={rule.condition}
            index={i + 1}
            active={rule.archetype === currentArchetype}
          />
        ))}
      </div>
    </div>
  )
}
