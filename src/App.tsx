import { useCallback, useEffect, useRef, useState } from 'react'
import { newGame, sendInput, tick } from './api'
import { GameCanvas } from './components/GameCanvas'
import { GameOver } from './components/GameOver'
import { Hud } from './components/Hud'
import type { Snapshot } from './types'
import { keyToInput } from './utils/keymap'
import './App.css'

/** ゲームオーバーになってから、Enter / Space でリスタートできるようになるまでの保護時間 (ms)。
 *  プレイ中に連打していたキーで、意図せず即リスタートしてしまうのを防ぐ。 */
export const RESTART_KEY_GUARD_MS = 500

const RESTART_KEYS = ['Enter', ' ']

export default function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null)
  const statusRef = useRef<Snapshot['status']>('playing')
  const gameOverAtRef = useRef(0)

  const update = useCallback((next: Snapshot) => {
    if (next.status === 'game_over' && statusRef.current !== 'game_over') {
      gameOverAtRef.current = performance.now()
    }
    statusRef.current = next.status
    setSnapshot(next)
  }, [])

  const restart = useCallback(() => {
    newGame(Math.floor(Math.random() * 2 ** 32)).then(update)
  }, [update])

  useEffect(restart, [restart])

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (statusRef.current === 'game_over') {
        // GAME OVER 中は Enter / Space が Restart ボタンの代わりになる。
        if (RESTART_KEYS.includes(e.key)) {
          e.preventDefault()
          if (!e.repeat && performance.now() - gameOverAtRef.current >= RESTART_KEY_GUARD_MS) restart()
        }
        return
      }
      const input = keyToInput(e.key)
      if (!input) return
      e.preventDefault()
      sendInput(input).then(update)
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [update, restart])

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
      <Hud
        score={snapshot.score}
        stationsReached={snapshot.stations_reached}
        rowsToStation={snapshot.rows_to_station}
      />
      <div className="board">
        <GameCanvas snapshot={snapshot} />
        {snapshot.status === 'game_over' && <GameOver onRestart={restart} />}
      </div>
    </main>
  )
}
