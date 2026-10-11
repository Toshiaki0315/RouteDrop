import { describe, expect, it } from 'vitest'
import { keyToInput } from './keymap'

describe('keyToInput', () => {
  it.each([
    ['ArrowLeft', 'move_left'],
    ['a', 'move_left'],
    ['ArrowRight', 'move_right'],
    ['d', 'move_right'],
    ['ArrowDown', 'soft_drop'],
    ['s', 'soft_drop'],
    ['ArrowUp', 'hard_drop'],
    ['w', 'hard_drop'],
    [' ', 'rotate'],
  ])('%s -> %s', (key, expected) => {
    expect(keyToInput(key)).toBe(expected)
  })

  it('returns null for unmapped keys', () => {
    expect(keyToInput('x')).toBeNull()
  })
})
