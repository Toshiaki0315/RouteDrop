import type { Snapshot } from './types'

// テスト用の小さな盤面 (3x2)。スタート駅は盤面の下、ゴール駅は盤面の上。
export function makeSnapshot(overrides: Partial<Snapshot> = {}): Snapshot {
  return {
    grid: {
      width: 3,
      height: 2,
      cells: [
        'wall',
        'empty',
        { panel: { kind: 'straight', facing: 'north' } },
        'empty',
        { panel: { kind: 'turn_right', facing: 'east' } },
        { panel: { kind: 'turn_left', facing: 'south' } },
      ],
    },
    piece: { pos: { x: 0, y: 1 }, panel: { kind: 'straight', facing: 'north' } },
    next: { kind: 'straight', facing: 'east' },
    robot: { pos: { x: 1, y: 2 }, heading: 'east' },
    start: { x: 1, y: 2 },
    goal: { x: 1, y: -1 },
    status: 'playing',
    score: 0,
    ...overrides,
  }
}
