import { useEffect, useRef, useState } from 'react'
import { getPlayerCard } from './lib/tauri-api'
import type { PlayerStats } from './types/stats'
import { FUTCard } from './components/CurrentDeck/FUTCard'
import { StatsRecap } from './components/CurrentDeck/StatsRecap'
import { ArchetypeRules } from './components/CurrentDeck/ArchetypeRules'
import { ExportCardButton } from './components/CurrentDeck/ExportCardButton'
import { EvolutionProgress } from './components/EvolutionProgress/EvolutionProgress'
import { AllCards } from './components/AllCards/AllCards'

function App() {
  const [githubUsername, setGithubUsername] = useState(
    () => localStorage.getItem('vibercard.githubUsername') ?? '',
  )
  const [stats, setStats] = useState<PlayerStats | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [scanning, setScanning] = useState(false)
  const [view, setView] = useState<'deck' | 'allcards'>('deck')
  const cardRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    localStorage.setItem('vibercard.githubUsername', githubUsername)
  }, [githubUsername])

  function runScan() {
    setError(null)
    setScanning(true)
    getPlayerCard()
      .then(setStats)
      .catch((e) => setError(String(e)))
      .finally(() => setScanning(false))
  }

  useEffect(() => {
    runScan()
  }, [])

  return (
    <div
      className="min-h-screen px-8 py-6"
      style={{
        background:
          'radial-gradient(ellipse 80% 60% at 15% -10%, rgba(57,135,229,0.12), transparent), radial-gradient(ellipse 60% 50% at 100% 0%, rgba(179,155,255,0.08), transparent), #0b0d12',
      }}
    >
      <header className="flex items-center justify-between mb-10">
        <div className="flex items-center gap-2">
          <img src="/favicon.svg" alt="" className="w-7 h-7 rounded-md" />
          <h1 className="text-2xl font-black tracking-tight text-white">ViberCard</h1>
        </div>
        <input
          value={githubUsername}
          onChange={(e) => setGithubUsername(e.target.value)}
          placeholder="pseudo GitHub"
          className="bg-white/5 border border-white/10 rounded-lg px-3 py-1.5 text-sm text-white placeholder:text-white/30 outline-none focus:border-white/30"
        />
      </header>

      <main className="max-w-[1600px] mx-auto">
        <div className="mb-6 flex gap-1 border-b" style={{ borderColor: 'rgba(255,255,255,0.08)' }}>
          {(
            [
              ['deck', 'Current Deck'],
              ['allcards', 'All Cards'],
            ] as const
          ).map(([key, label]) => {
            const tabActive = view === key
            return (
              <button
                key={key}
                onClick={() => setView(key)}
                className="text-sm font-semibold px-4 py-2 transition-colors"
                style={{
                  color: tabActive ? '#ffffff' : 'rgba(255,255,255,0.5)',
                  borderBottom: tabActive ? '2px solid #3987e5' : '2px solid transparent',
                }}
              >
                {label}
              </button>
            )
          })}
        </div>

        {view === 'allcards' ? (
          <AllCards githubUsername={githubUsername} />
        ) : (
          <>
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-baseline gap-2">
                <h2 className="text-sm uppercase tracking-wide text-white/40">Current Deck</h2>
                {stats && (
                  <span className="text-xs text-white/30">
                    · {new Date(stats.range_start).toLocaleDateString()} – {new Date(stats.range_end).toLocaleDateString()}{' '}
                    (rolling 7 days)
                  </span>
                )}
              </div>
              <button
                onClick={runScan}
                disabled={scanning}
                className="text-xs font-semibold px-3 py-1.5 rounded-lg transition-colors disabled:opacity-50"
                style={{
                  background: 'rgba(255,255,255,0.05)',
                  border: '1px solid rgba(255,255,255,0.1)',
                  color: 'rgba(255,255,255,0.8)',
                }}
              >
                {scanning ? 'Scanning…' : 'Rescan'}
              </button>
            </div>
            {error && (
              <div className="rounded-lg border border-red-500/30 bg-red-500/10 text-red-300 text-sm p-4">
                {error}
              </div>
            )}
            {!error && !stats && <div className="text-white/40 text-sm">Scan en cours…</div>}
            {stats && (
              <>
                {/* Hero: the card next to its class rules, front and center. items-stretch so
                    Class Rules matches the card's height exactly (its own list scrolls
                    internally instead of the panel growing/shrinking independently). */}
                <div className="grid grid-cols-1 lg:grid-cols-[24rem_1fr] gap-6 items-stretch mb-6">
                  <div className="flex flex-col gap-3">
                    <div ref={cardRef}>
                      <FUTCard githubUsername={githubUsername} stats={stats} />
                    </div>
                    {/* mt-auto: when Class Rules is taller than the card, items-stretch
                        stretches this column to match, so push the button down to that
                        stretched bottom edge instead of leaving it stuck under the card. */}
                    <div className="mt-auto">
                      <ExportCardButton targetRef={cardRef} filenameHint={stats.archetype} />
                    </div>
                  </div>
                  <ArchetypeRules currentArchetype={stats.archetype} />
                </div>

                <div className="mb-6">
                  <StatsRecap stats={stats} />
                </div>

                <div>
                  <h2 className="text-sm uppercase tracking-wide text-white/40 mb-3">
                    Evolution Progress
                  </h2>
                  <EvolutionProgress currentStats={stats} />
                </div>
              </>
            )}
          </>
        )}
      </main>
    </div>
  )
}

export default App
