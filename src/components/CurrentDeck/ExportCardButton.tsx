import { useState } from 'react'
import type { RefObject } from 'react'
import { toPng } from 'html-to-image'
import { save } from '@tauri-apps/plugin-dialog'
import { savePngFile } from '../../lib/tauri-api'

type Status = 'idle' | 'exporting' | 'done' | 'error'

export function ExportCardButton({
  targetRef,
  filenameHint,
}: {
  targetRef: RefObject<HTMLDivElement | null>
  filenameHint: string
}) {
  const [status, setStatus] = useState<Status>('idle')
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  async function handleExport() {
    if (!targetRef.current) return
    setStatus('exporting')
    setErrorMessage(null)
    try {
      const dataUrl = await toPng(targetRef.current, { pixelRatio: 3, cacheBust: true })
      const base64Data = dataUrl.split(',')[1]

      const slug = filenameHint.toLowerCase().replace(/[^a-z0-9]+/g, '-')
      const path = await save({
        defaultPath: `vibercard-${slug}.png`,
        filters: [{ name: 'PNG Image', extensions: ['png'] }],
      })
      if (!path) {
        setStatus('idle')
        return
      }

      await savePngFile(path, base64Data)
      setStatus('done')
      setTimeout(() => setStatus('idle'), 2000)
    } catch (e) {
      console.error(e)
      setErrorMessage(e instanceof Error ? e.message : String(e))
      setStatus('error')
    }
  }

  const label =
    status === 'exporting' ? 'Exporting…' : status === 'done' ? 'Saved ✓' : status === 'error' ? 'Failed' : 'Export PNG'

  return (
    <div className="flex flex-col gap-1">
      <button
        onClick={handleExport}
        disabled={status === 'exporting'}
        className="w-full text-xs font-semibold px-3 py-2 rounded-lg transition-colors disabled:opacity-50"
        style={{
          background: 'rgba(255,255,255,0.05)',
          border: '1px solid rgba(255,255,255,0.1)',
          color: status === 'error' ? '#e66767' : status === 'done' ? '#3987e5' : 'rgba(255,255,255,0.8)',
        }}
      >
        {label}
      </button>
      {errorMessage && <p className="text-[10px] text-red-400/80 break-words px-1">{errorMessage}</p>}
    </div>
  )
}
