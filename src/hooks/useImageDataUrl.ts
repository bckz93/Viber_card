import { useEffect, useState } from 'react'

/** Crops `blob` to a centered square and scales it to `size`×`size`, baked
 * into the pixels via canvas. html-to-image doesn't reliably honor CSS
 * `object-fit: cover` when rasterizing an <img> during export, so the crop
 * has to already be correct in the image data itself, not left to CSS. */
async function toSquareDataUrl(blob: Blob, size: number): Promise<string> {
  const bitmap = await createImageBitmap(blob)
  const side = Math.min(bitmap.width, bitmap.height)
  const sx = (bitmap.width - side) / 2
  const sy = (bitmap.height - side) / 2

  const canvas = document.createElement('canvas')
  canvas.width = size
  canvas.height = size
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('canvas 2d context unavailable')
  ctx.drawImage(bitmap, sx, sy, side, side, 0, 0, size, size)

  return canvas.toDataURL('image/png')
}

/** Fetches `url` and resolves to a square `data:` URI at `size` pixels,
 * pre-cropped/scaled so it displays and exports identically. Also sidesteps
 * cross-origin canvas tainting during PNG export — data URIs are always
 * canvas-safe, unlike a remote <img src> even with crossOrigin set. */
export function useImageDataUrl(url: string | undefined, size = 200): string | null {
  const [dataUrl, setDataUrl] = useState<string | null>(null)

  useEffect(() => {
    if (!url) {
      setDataUrl(null)
      return
    }

    let cancelled = false

    fetch(url)
      .then((res) => {
        if (!res.ok) throw new Error(`fetch failed: ${res.status}`)
        return res.blob()
      })
      .then((blob) => toSquareDataUrl(blob, size))
      .then((result) => {
        if (!cancelled) setDataUrl(result)
      })
      .catch(() => {
        if (!cancelled) setDataUrl(null)
      })

    return () => {
      cancelled = true
    }
  }, [url, size])

  return dataUrl
}
