import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { getState, newGame, sendInput, tick } from './api'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

beforeEach(() => vi.mocked(invoke).mockReset())

describe('api', () => {
  it('maps each wrapper to its Tauri command', async () => {
    await newGame(7)
    expect(invoke).toHaveBeenLastCalledWith('new_game', { seed: 7 })
    await getState()
    expect(invoke).toHaveBeenLastCalledWith('get_state')
    await sendInput('rotate')
    expect(invoke).toHaveBeenLastCalledWith('send_input', { input: 'rotate' })
    await tick(16)
    expect(invoke).toHaveBeenLastCalledWith('tick', { dtMs: 16 })
  })
})
