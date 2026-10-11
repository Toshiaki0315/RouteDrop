import { useCallback, useEffect, useRef, useState } from 'react'
import { newGame, sendInput, tick } from './api'
import { GameCanvas } from './components/GameCanvas'
import { Hud } from './components/Hud'
import type { Snapshot } from './types'
import { keyToInput } from './utils/keymap'
import './App.css'

export default function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null)
  const statusRef = useRef<Snapshot['status']>('playing')

  const update = useCallback((next: Snapshot) => {
    statusRef.current = next.status
    setSnapshot(next)
  }, [])

  const restart = useCallback(() => {
    newGame(Math.floor(Math.random() * 2 ** 32)).then(update)
  }, [update])

  useEffect(restart, [restart])

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const input = keyToInput(e.key)
      if (!input) return
      e.preventDefault()
      sendInput(input).then(update)
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [update])

  useEffect(() => {
    let frame = 0
    let last = performance.now()
    let inFlight = false
    const loop = (now: number) => {
      const dt = Math.round(now - last)
      last = now
      if (!inFlight && statusRef.current === 'playing' && dt > 0) {
        inFlight = true
        tick(dt)
          .then(update)
          .finally(() => {
            inFlight = false
          })
      }
      frame = requestAnimationFrame(loop)
    }
    frame = requestAnimationFrame(loop)
    return () => cancelAnimationFrame(frame)
  }, [update])

  if (!snapshot) return null

  return (
    <main className="app">
      <h1>RouteDrop</h1>
      <GameCanvas snapshot={snapshot} />
      <Hud score={snapshot.score} status={snapshot.status} onRestart={restart} />
    </main>
  )
}
