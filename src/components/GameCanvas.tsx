import { useEffect, useRef } from 'react'
import type { Snapshot } from '../types'
import { canvasSize, drawSnapshot } from '../utils/render'

export function GameCanvas({ snapshot }: { snapshot: Snapshot }) {
  const ref = useRef<HTMLCanvasElement>(null)
  const { width, height } = canvasSize(snapshot)

  useEffect(() => {
    const ctx = ref.current?.getContext('2d')
    if (ctx) drawSnapshot(ctx, snapshot)
  }, [snapshot])

  return <canvas ref={ref} width={width} height={height} aria-label="game board" />
}
