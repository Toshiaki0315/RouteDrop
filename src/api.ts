import { invoke } from '@tauri-apps/api/core'
import type { Input, Snapshot } from './types'

// Rust の Tauri Command への薄いラッパー。ゲームロジックは持たない。
export const newGame = (seed: number) => invoke<Snapshot>('new_game', { seed })
export const getState = () => invoke<Snapshot>('get_state')
export const sendInput = (input: Input) => invoke<Snapshot>('send_input', { input })
export const tick = (dtMs: number) => invoke<Snapshot>('tick', { dtMs })
