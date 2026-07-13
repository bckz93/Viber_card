import type { PlayerStats } from '../../types/stats'
import { FUTCard } from '../CurrentDeck/FUTCard'
import { ARCHETYPE_RULES } from '../CurrentDeck/archetypeRules'

// A frozen instant shared by every card in the gallery — these aren't real
// scans, so range_start/range_end are cosmetic here, not "as measured from
// X to Y" like on the real Current Deck card.
const now = new Date().toISOString()

export function AllCards({ githubUsername }: { githubUsername: string }) {
  return (
    <div>
      <p className="text-xs text-white/40 mb-6 max-w-2xl">
        Every class DevCards can hand out, with example stats that land squarely inside its zone
        (see <span className="font-mono">archetypeRules.ts</span> — mirrors
        <span className="font-mono"> scoring.rs::archetype_for</span>). Not your data — just what
        it takes to get each one.
      </p>
      <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-8">
        {ARCHETYPE_RULES.map((rule) => {
          const stats: PlayerStats = {
            ...rule.exampleStats,
            archetype: rule.archetype,
            punchline: '',
            insights: [],
            sample_size: 150,
            range_start: now,
            range_end: now,
          }

          return (
            <div key={rule.archetype} className="flex flex-col gap-3">
              <FUTCard githubUsername={githubUsername} stats={stats} />
              <div
                className="rounded-xl px-3 py-2.5"
                style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.08)' }}
              >
                <div className="text-xs text-white/60 leading-snug mb-1">{rule.blurb}</div>
                <div className="text-[11px] font-mono text-white/40">{rule.condition}</div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
