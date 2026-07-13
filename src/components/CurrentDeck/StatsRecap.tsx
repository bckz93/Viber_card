import type { PlayerStats } from '../../types/stats'
import { themeFor } from './cardTheme'
import { STAT_META } from './statMeta'

export function StatsRecap({ stats, className = '' }: { stats: PlayerStats; className?: string }) {
  const theme = themeFor(stats.archetype)

  return (
    <div
      className={`w-full rounded-2xl p-6 ${className}`}
      style={{
        background: 'rgba(255,255,255,0.03)',
        border: `1px solid ${theme.accent}33`,
      }}
    >
      <div className="text-xs font-bold uppercase tracking-[0.2em] text-white/30 mb-2">Recap</div>

      <p className="italic text-white text-2xl leading-snug mb-6" style={{ color: theme.accent }}>
        “{stats.punchline}”
      </p>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-5">
        {stats.insights.map((insight) => {
          const meta = STAT_META.find((m) => m.key === insight.key)
          return (
            <div key={insight.key} className="flex gap-3">
              <div className="text-xl shrink-0 w-6 text-center">{meta?.icon}</div>
              <div>
                <div className="text-sm font-semibold text-white">
                  {insight.label} <span className="text-white/40">— {insight.value}</span>
                </div>
                <div className="text-xs text-white/50 leading-snug mt-0.5">{insight.explanation}</div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
