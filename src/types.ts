// Rust 側 (src-tauri/src/core) が serde で出力する状態の型定義。

export type Direction = 'north' | 'east' | 'south' | 'west'
export type PanelKind = 'straight' | 'turn_right' | 'turn_left'
export interface Panel {
  kind: PanelKind
  facing: Direction
}
export type Cell = 'empty' | 'wall' | { panel: Panel }
export type Status = 'playing' | 'cleared' | 'game_over'
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

export interface Robot {
  pos: Pos
  heading: Direction
}

export interface Snapshot {
  grid: Grid
  piece: Piece | null
  /** 次に降ってくるパネル */
  next: Panel
  robot: Robot
  /** 盤面の外にあるスタート駅とゴール駅 */
  start: Pos
  goal: Pos
  status: Status
  score: number
}
