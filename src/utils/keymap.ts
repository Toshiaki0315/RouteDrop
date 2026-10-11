import type { Input } from '../types'

const KEY_TO_INPUT: Record<string, Input> = {
  ArrowLeft: 'move_left',
  a: 'move_left',
  ArrowRight: 'move_right',
  d: 'move_right',
  ArrowDown: 'soft_drop',
  s: 'soft_drop',
  ArrowUp: 'hard_drop',
  w: 'hard_drop',
  ' ': 'rotate',
}

export function keyToInput(key: string): Input | null {
  return KEY_TO_INPUT[key] ?? null
}
