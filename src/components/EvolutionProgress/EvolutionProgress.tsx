import { useEffect, useState } from 'react'
import { getStatsForRange, listAvailableWeeks } from '../../lib/tauri-api'
import type { PlayerStats, WeekRange } from '../../types/stats'
import { EvolutionRadarChart } from './EvolutionRadarChart'
import { STAT_META } from '../CurrentDeck/statMeta'

const DAY_MS = 24 * 60 * 60 * 1000

const SHORT_DATE: Intl.DateTimeFormatOptions = { month: 'short', day: 'numeric' }

function formatRange(start: string, end: string, inclusiveEnd = false): string {
  const startDate = new Date(start)
  const endDate = new Date(new Date(end).getTime() - (inclusiveEnd ? DAY_MS : 0))
  return `${startDate.toLocaleDateString(undefined, SHORT_DATE)} – ${endDate.toLocaleDateString(undefined, SHORT_DATE)}`
}

function formatDelta(before: number, after: number): string {
  const d = after - before
  if (d === 0) return '±0'
  return d > 0 ? `+${d}` : `${d}`
}

export function EvolutionProgress({ currentStats }: { currentStats: PlayerStats }) {
  const [weeks, setWeeks] = useState<WeekRange[] | null>(null)
  const [selectedRange, setSelectedRange] = useState<'default' | number>('default')
  const [thenStats, setThenStats] = useState<PlayerStats | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    listAvailableWeeks()
      .then(setWeeks)
      .catch((e) => setError(String(e)))
  }, [])

  useEffect(() => {
    let start: string
    let end: string
    if (selectedRange === 'default') {
      const now = Date.now()
      start = new Date(now - 14 * DAY_MS).toISOString()
      end = new Date(now - 7 * DAY_MS).toISOString()
    } else {
      const week = weeks?.[selectedRange]
      if (!week) return
      start = week.start
      end = week.end
    }

    getStatsForRange(start, end)
      .then(setThenStats)
      .catch((e) => setError(String(e)))
  }, [selectedRange, weeks])

  if (error) {
    return (
      <div className="rounded-lg border border-red-500/30 bg-red-500/10 text-red-300 text-sm p-4">
        {error}
      </div>
    )
  }

  if (!weeks || !thenStats) {
    return <div className="text-white/40 text-sm">Loading history…</div>
  }

  if (thenStats.sample_size === 0) {
    return (
      <div
        className="rounded-2xl p-6 text-sm text-white/50"
        style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.08)' }}
      >
        Not enough history yet for that period — come back once you've got at least two weeks of
        activity to compare.
      </div>
    )
  }

  return (
    <div
      className="rounded-2xl p-6"
      style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.08)' }}
    >
      <div className="flex flex-wrap items-center gap-x-2 gap-y-1 mb-4 text-xs">
        <span className="text-white/40">Comparing</span>
        <span className="text-white font-medium">
          {formatRange(currentStats.range_start, currentStats.range_end)}
        </span>
        <span className="text-white/40">to</span>
        <select
          value={selectedRange === 'default' ? 'default' : String(selectedRange)}
          onChange={(e) => setSelectedRange(e.target.value === 'default' ? 'default' : Number(e.target.value))}
          className="bg-white/5 border border-white/10 rounded-lg px-2 py-1 text-white/80 outline-none"
        >
          <option value="default">Previous 7 days (rolling)</option>
          {weeks.map((week, i) => (
            <option key={week.start} value={i}>
              Week of {formatRange(week.start, week.end, true)}
            </option>
          ))}
        </select>
        <span className="text-white/50">
          ({formatRange(thenStats.range_start, thenStats.range_end)})
        </span>
      </div>

      {/* Stacked (not side-by-side): this panel now lives in a narrower column next to the
          card rather than spanning the full page width, so chart-then-list reads better
          than squeezing both into one row. */}
      <EvolutionRadarChart then={thenStats} now={currentStats} />

      <div className="flex flex-col gap-1.5 mt-2">
        {STAT_META.map((meta) => {
          const before = thenStats[meta.key]
          const after = currentStats[meta.key]
          const delta = after - before
          return (
            <div
              key={meta.key}
              className="flex items-center justify-between gap-3 text-sm rounded-lg px-3 py-1.5"
              style={{ background: 'rgba(255,255,255,0.02)' }}
            >
              <span className="text-white/60">
                {meta.icon} {meta.label}
              </span>
              <span className="font-mono text-xs">
                <span className="text-white/40">{before}</span>
                <span className="text-white/30"> → </span>
                <span className="text-white font-semibold">{after}</span>
                <span
                  className="ml-2 font-sans font-semibold"
                  style={{ color: delta > 0 ? '#3987e5' : delta < 0 ? '#e66767' : '#6b7280' }}
                >
                  ({formatDelta(before, after)})
                </span>
              </span>
            </div>
          )
        })}
      </div>
    </div>
  )
}
