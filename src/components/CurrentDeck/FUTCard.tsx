import type { PlayerStats } from '../../types/stats'
import { StatsRadarChart } from './RadarChart'
import { themeFor } from './cardTheme'
import { getArchetypeArt } from './archetypeArt'
import { ARCHETYPE_LORE } from './archetypeLore'
import { STAT_META } from './statMeta'
import { useImageDataUrl } from '../../hooks/useImageDataUrl'

function formatCompact(n: number): string {
  return new Intl.NumberFormat('en-US', { notation: 'compact', maximumFractionDigits: 1 }).format(n)
}

export function FUTCard({ githubUsername, stats }: { githubUsername: string; stats: PlayerStats }) {
  const theme = themeFor(stats.archetype)
  // avatars.githubusercontent.com/<user> responds directly (no redirect) and
  // sends Access-Control-Allow-Origin — github.com/<user>.png 302-redirects
  // there but the redirect response itself lacks CORS headers, which makes
  // fetch() (used to inline the image as a data: URI, see useImageDataUrl)
  // fail outright before it ever reaches the CORS-friendly destination.
  const avatarUrl = githubUsername ? `https://avatars.githubusercontent.com/${githubUsername}` : undefined
  const avatarDataUrl = useImageDataUrl(avatarUrl)
  const archetypeArt = getArchetypeArt(stats.archetype)
  const lore = ARCHETYPE_LORE[stats.archetype]

  return (
    <div
      className="w-full max-w-sm mx-auto rounded-[26px] p-[3px]"
      style={{
        background: `linear-gradient(135deg, rgba(255,255,255,0.9), ${theme.accent}, rgba(0,0,0,0.6), ${theme.accent})`,
        boxShadow: `0 20px 60px -20px ${theme.accent}88, 0 0 40px -10px ${theme.accent}55`,
      }}
    >
    <div
      className="relative rounded-[23px] overflow-hidden"
      style={{
        background: theme.gradient,
        boxShadow: `0 0 0 1px rgba(255,255,255,0.06) inset`,
      }}
    >
      {/* holo-shine diagonal sweep */}
      <div
        className="pointer-events-none absolute inset-0"
        style={{
          background: `linear-gradient(115deg, transparent 30%, rgba(255,255,255,${theme.shineOpacity}) 48%, rgba(255,255,255,${theme.shineOpacity * 1.6}) 52%, transparent 70%)`,
        }}
      />

      <div className="relative flex flex-col gap-3 pb-4">
        {/* header: archetype title, level with the github avatar — the row
            and both its children stretch to fill the full card width/height
            available here, instead of floating in leftover space. */}
        <div className="w-full px-3 pt-3 flex items-stretch justify-between gap-3">
          <div className="flex-1 flex items-center justify-center text-center text-lg font-extrabold uppercase tracking-wide text-white leading-tight">
            {stats.archetype}
          </div>

          <div className="flex flex-col items-center justify-center shrink-0">
            <div
              className="rounded-full overflow-hidden shrink-0"
              style={{
                width: 52,
                height: 52,
                border: `2px solid ${theme.accent}`,
                boxShadow: `0 0 16px -2px ${theme.accent}aa`,
              }}
            >
              {avatarDataUrl ? (
                <img src={avatarDataUrl} alt={githubUsername} className="w-full h-full object-cover" />
              ) : (
                <div className="w-full h-full flex items-center justify-center bg-white/5 text-lg text-white/30">
                  ?
                </div>
              )}
            </div>
            <div className="mt-1 text-[10px] font-semibold text-white/60 max-w-[80px] truncate text-center">
              {githubUsername || 'unknown-dev'}
            </div>
          </div>
        </div>

        {/* hero illustration — full card width, no side gutter */}
        {archetypeArt && <img src={archetypeArt} alt={stats.archetype} className="w-full h-auto block" />}

        {/* class summary, right under the art */}
        {lore && (
          <div className="px-4">
            <p
              className="text-xs italic text-white/60 leading-snug rounded-lg px-3 py-2"
              style={{ background: 'rgba(0,0,0,0.2)', border: '1px solid rgba(255,255,255,0.06)' }}
            >
              “{lore}”
            </p>
          </div>
        )}

        <div className="px-4 flex flex-col gap-3">
          {/* radar with holo-foil patch behind it */}
          <div className="relative">
            <div
              className="pointer-events-none absolute inset-0 blur-2xl opacity-40"
              style={{
                background:
                  'conic-gradient(from 90deg, #3987e5, #9085e9, #e66767, #c98500, #199e70, #3987e5)',
              }}
            />
            <div className="relative" style={{ filter: `drop-shadow(0 0 10px ${theme.accent}44)` }}>
              <StatsRadarChart stats={stats} compact />
            </div>
          </div>

          {/* bottom stat strip, FUT-style, with per-axis icon */}
          <div className="flex justify-between items-start">
            {STAT_META.map((row, i) => (
              <div key={row.key} className="flex items-start gap-1.5">
                <div className="text-center w-14">
                  <div className="text-xs mb-0.5">{row.icon}</div>
                  <div className="text-xl font-bold text-white leading-none">{stats[row.key]}</div>
                  <div className="text-[9px] font-semibold text-white/40 tracking-wide mt-1 leading-tight">
                    {row.label}
                  </div>
                </div>
                {i < STAT_META.length - 1 && <div className="w-px h-8 bg-white/10" />}
              </div>
            ))}
          </div>

          {/* secondary stats footer */}
          <div
            className="flex justify-around rounded-lg py-1.5"
            style={{ background: 'rgba(0,0,0,0.25)', border: '1px solid rgba(255,255,255,0.06)' }}
          >
            <div className="text-center">
              <div className="text-sm font-bold text-white">{stats.sample_size}</div>
              <div className="text-[10px] text-white/40 uppercase tracking-wide">Prompts</div>
            </div>
            <div className="w-px bg-white/10" />
            <div className="text-center">
              <div className="text-sm font-bold text-white">{formatCompact(stats.total_tokens)}</div>
              <div className="text-[10px] text-white/40 uppercase tracking-wide">Tokens</div>
            </div>
          </div>
        </div>
      </div>
    </div>
    </div>
  )
}
