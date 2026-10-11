import type { Status } from '../types'

const MESSAGES: Record<Status, string | null> = {
  playing: null,
  cleared: 'CLEAR!',
  game_over: 'GAME OVER',
}

interface Props {
  score: number
  status: Status
  onRestart: () => void
}

export function Hud({ score, status, onRestart }: Props) {
  const message = MESSAGES[status]
  return (
    <div className="hud">
      <div>Score: {score}</div>
      {message && <div role="status">{message}</div>}
      <button onClick={onRestart}>Restart</button>
      <p className="help">← → 移動 / Space 回転 / ↓ ソフトドロップ / ↑ ハードドロップ</p>
    </div>
  )
}
