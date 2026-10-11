/** フィールドの真ん中に重ねて表示する GAME OVER と Restart ボタン。 */
export function GameOver({ onRestart }: { onRestart: () => void }) {
  return (
    <div className="overlay">
      <div role="status" className="overlay-title">
        GAME OVER
      </div>
      <button className="restart" onClick={onRestart}>
        Restart
      </button>
    </div>
  )
}
