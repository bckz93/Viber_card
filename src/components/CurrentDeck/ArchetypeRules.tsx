import { ARCHETYPE_RULES } from './archetypeRules'

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
        {ARCHETYPE_RULES.map((rule, i) => (
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
