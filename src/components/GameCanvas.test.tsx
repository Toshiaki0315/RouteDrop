import { render } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { makeSnapshot } from '../fixtures'
import { GameCanvas } from './GameCanvas'

afterEach(() => vi.restoreAllMocks())

describe('GameCanvas', () => {
  it('sizes the canvas from the grid', () => {
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(null)
    const { getByLabelText } = render(<GameCanvas snapshot={makeSnapshot()} />)
    const canvas = getByLabelText('game board') as HTMLCanvasElement
    expect(canvas.width).toBe(120)
    expect(canvas.height).toBe(160)
  })

  it('draws on the 2d context when available', () => {
    const fillRect = vi.fn()
    const ctx = new Proxy({ fillRect }, { get: (t, k) => (k in t ? t[k as keyof typeof t] : vi.fn()) })
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(ctx as never)
    render(<GameCanvas snapshot={makeSnapshot()} />)
    expect(fillRect).toHaveBeenCalled()
  })

  it('does nothing when the 2d context is unavailable', () => {
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(null)
    expect(() => render(<GameCanvas snapshot={makeSnapshot()} />)).not.toThrow()
  })
})
