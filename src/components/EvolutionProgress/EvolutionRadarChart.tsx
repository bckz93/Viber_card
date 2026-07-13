import {
  Radar,
  RadarChart as ReRadarChart,
  PolarGrid,
  PolarAngleAxis,
  PolarRadiusAxis,
  ResponsiveContainer,
  Tooltip,
  Legend,
} from 'recharts'
import type { PlayerStats } from '../../types/stats'

// "Then" = muted neutral (past), "Now" = the app's validated accent (present).
const THEN_COLOR = '#8a8d93'
const NOW_COLOR = '#3987e5'
const GRIDLINE = '#2c2c2a'
const AXIS_TEXT = '#c3c2b7'

const AXIS_LABELS: Record<keyof Pick<PlayerStats, 'vol' | 'spd' | 'nct' | 'slf' | 'emo'>, string> = {
  vol: 'VOL',
  spd: 'SPD',
  nct: 'NCT',
  slf: 'SLF',
  emo: 'EMO',
}

export function EvolutionRadarChart({ then, now }: { then: PlayerStats; now: PlayerStats }) {
  const data = (Object.keys(AXIS_LABELS) as Array<keyof typeof AXIS_LABELS>).map((key) => ({
    axis: AXIS_LABELS[key],
    Then: then[key],
    Now: now[key],
  }))

  return (
    <ResponsiveContainer width="100%" height={230}>
      <ReRadarChart data={data} outerRadius="70%">
        <PolarGrid stroke={GRIDLINE} />
        <PolarAngleAxis dataKey="axis" tick={{ fill: AXIS_TEXT, fontSize: 13, fontWeight: 600 }} />
        <PolarRadiusAxis domain={[0, 99]} tick={false} axisLine={false} />
        <Radar
          name="Then"
          dataKey="Then"
          stroke={THEN_COLOR}
          strokeWidth={2}
          fill={THEN_COLOR}
          fillOpacity={0.12}
          dot={{ r: 3, fill: THEN_COLOR, strokeWidth: 0 }}
          isAnimationActive={false}
        />
        <Radar
          name="Now"
          dataKey="Now"
          stroke={NOW_COLOR}
          strokeWidth={2}
          fill={NOW_COLOR}
          fillOpacity={0.25}
          dot={{ r: 4, fill: NOW_COLOR, strokeWidth: 0 }}
          isAnimationActive={false}
        />
        <Legend
          wrapperStyle={{ fontSize: 12, color: AXIS_TEXT }}
          formatter={(value) => <span style={{ color: '#e8eaf0' }}>{value}</span>}
        />
        <Tooltip
          contentStyle={{
            background: '#1a1a19',
            border: '1px solid rgba(255,255,255,0.1)',
            borderRadius: 8,
            color: '#ffffff',
          }}
          labelStyle={{ color: '#c3c2b7' }}
        />
      </ReRadarChart>
    </ResponsiveContainer>
  )
}
