interface Props {
  score: number
  stationsReached: number
  rowsToStation: number
}

const pad = (value: number, width: number) => String(Math.max(0, value)).padStart(width, '0')

/** 画面上部の 1 行表示: スコア / 到着駅 / 次の駅まで。 */
export function Hud({ score, stationsReached, rowsToStation }: Props) {
  return (
    <div className="hud">
      Score: {pad(score, 6)} / 到着駅: {pad(stationsReached, 3)} / 次の駅まで: {pad(rowsToStation, 3)}
    </div>
  )
}
