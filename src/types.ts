// Rust 側 (src-tauri/src/core) が serde で出力する状態の型定義。

export type Direction = 'north' | 'east' | 'south' | 'west'
export type PanelKind = 'straight' | 'turn_right' | 'turn_left'
export interface Panel {
  kind: PanelKind
  facing: Direction
}
export type Cell = 'empty' | 'wall' | 'station' | 'origin' | { panel: Panel }
export type Status = 'playing' | 'game_over'
export type Input = 'move_left' | 'move_right' | 'soft_drop' | 'hard_drop' | 'rotate'

export interface Pos {
  x: number
  y: number
}

export interface Grid {
  width: number
  height: number
  cells: Cell[]
}

export interface Piece {
  pos: Pos
  panel: Panel
}

export interface Train {
  pos: Pos
  heading: Direction
}

export interface Snapshot {
  grid: Grid
  piece: Piece | null
  /** 次に降ってくるパネル */
  next: Panel
  /** 次のパネルが出現する列 */
  next_column: number
  train: Train
  /** 次の駅までの縦方向の距離 (マス)。駅が盤面の上に見えていない間も数える */
  rows_to_station: number
  /** 到着した駅の数 */
  stations_reached: number
  status: Status
  score: number
}
