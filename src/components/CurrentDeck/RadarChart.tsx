import {
  Radar,
  RadarChart as ReRadarChart,
  PolarGrid,
  PolarAngleAxis,
  PolarRadiusAxis,
  ResponsiveContainer,
  Tooltip,
} from 'recharts'
import type { PlayerStats } from '../../types/stats'

// Dark chart chrome + single-series accent, per the validated palette
// (dataviz skill references/palette.md: dark categorical slot 1 = blue).
const ACCENT = '#3987e5'
const GRIDLINE = '#2c2c2a'
const AXIS_TEXT = '#c3c2b7'

const AXIS_LABELS: Record<keyof Pick<PlayerStats, 'vol' | 'spd' | 'nct' | 'slf' | 'emo'>, string> = {
  vol: 'VOL',
  spd: 'SPD',
  nct: 'NCT',
  slf: 'SLF',
  emo: 'EMO',
}

export function StatsRadarChart({ stats, compact = false }: { stats: PlayerStats; compact?: boolean }) {
  const data = (Object.keys(AXIS_LABELS) as Array<keyof typeof AXIS_LABELS>).map((key) => ({
    axis: AXIS_LABELS[key],
    value: stats[key],
  }))

  return (
    <ResponsiveContainer width="100%" height={compact ? 190 : 280}>
      <ReRadarChart data={data} outerRadius="75%">
        <PolarGrid stroke={GRIDLINE} />
        <PolarAngleAxis dataKey="axis" tick={{ fill: AXIS_TEXT, fontSize: compact ? 11 : 13, fontWeight: 600 }} />
        <PolarRadiusAxis domain={[0, 99]} tick={false} axisLine={false} />
        <Radar
          name="Stats"
          dataKey="value"
          stroke={ACCENT}
          strokeWidth={2}
          fill={ACCENT}
          fillOpacity={0.28}
          dot={{ r: 4, fill: ACCENT, strokeWidth: 0 }}
          isAnimationActive={false}
        />
        <Tooltip
          contentStyle={{
            background: '#1a1a19',
            border: '1px solid rgba(255,255,255,0.1)',
            borderRadius: 8,
            color: '#ffffff',
          }}
          labelStyle={{ color: '#c3c2b7' }}
          formatter={(value) => [String(value), 'score']}
        />
      </ReRadarChart>
    </ResponsiveContainer>
  )
}
