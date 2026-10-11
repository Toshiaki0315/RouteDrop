import type { Snapshot } from './types'

// テスト用の小さな盤面 (3x2)。駅は (1,0)、始発駅は (0,1)。
export function makeSnapshot(overrides: Partial<Snapshot> = {}): Snapshot {
  return {
    grid: {
      width: 3,
      height: 2,
      cells: [
        'wall',
        'station',
        { panel: { kind: 'straight', facing: 'north' } },
        'origin',
        { panel: { kind: 'turn_right', facing: 'east' } },
        { panel: { kind: 'turn_left', facing: 'south' } },
      ],
    },
    piece: { pos: { x: 0, y: 1 }, panel: { kind: 'straight', facing: 'north' } },
    next: { kind: 'straight', facing: 'east' },
    next_column: 1,
    train: { pos: { x: 0, y: 1 }, heading: 'east' },
    rows_to_station: 15,
    stations_reached: 0,
    status: 'playing',
    score: 0,
    ...overrides,
  }
}
