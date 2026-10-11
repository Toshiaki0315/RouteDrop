import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import App, { RESTART_KEY_GUARD_MS } from './App'
import * as api from './api'
import { makeSnapshot } from './fixtures'

vi.mock('./api')

let frames: FrameRequestCallback[] = []

beforeEach(() => {
  vi.clearAllMocks()
  frames = []
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => frames.push(cb))
  vi.stubGlobal('cancelAnimationFrame', vi.fn())
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(null)
  vi.mocked(api.newGame).mockResolvedValue(makeSnapshot({ score: 0 }))
  vi.mocked(api.sendInput).mockResolvedValue(makeSnapshot({ score: 5 }))
  vi.mocked(api.tick).mockResolvedValue(makeSnapshot({ score: 10 }))
})

afterEach(() => {
  vi.unstubAllGlobals()
  vi.restoreAllMocks()
})

const runFrame = async (now: number) => {
  await act(async () => frames[frames.length - 1](now))
}

describe('App', () => {
  it('starts a new game on mount and renders the board', async () => {
    render(<App />)
    expect(await screen.findByLabelText('game board')).toBeInTheDocument()
    expect(api.newGame).toHaveBeenCalled()
    expect(screen.getByText(/Score: 000000/)).toBeInTheDocument()
  })

  it('sends mapped keys to the backend and shows the new state', async () => {
    render(<App />)
    await screen.findByLabelText('game board')
    fireEvent.keyDown(window, { key: 'ArrowLeft' })
    await waitFor(() => expect(api.sendInput).toHaveBeenCalledWith('move_left'))
    expect(await screen.findByText(/Score: 000005/)).toBeInTheDocument()
  })

  it('ignores unmapped keys', async () => {
    render(<App />)
    await screen.findByLabelText('game board')
    fireEvent.keyDown(window, { key: 'x' })
    expect(api.sendInput).not.toHaveBeenCalled()
  })

  it('advances the game with elapsed time on each animation frame', async () => {
    render(<App />)
    await screen.findByLabelText('game board')
    await runFrame(performance.now() + 16)
    expect(api.tick).toHaveBeenCalledTimes(1)
    expect(await screen.findByText(/Score: 000010/)).toBeInTheDocument()
  })

  it('does not overlap tick calls while one is in flight', async () => {
    let resolve!: (s: ReturnType<typeof makeSnapshot>) => void
    vi.mocked(api.tick).mockReturnValue(new Promise((r) => (resolve = r)))
    render(<App />)
    await screen.findByLabelText('game board')
    const t = performance.now()
    await runFrame(t + 16)
    await runFrame(t + 32)
    expect(api.tick).toHaveBeenCalledTimes(1)
    await act(async () => resolve(makeSnapshot()))
  })

  it('stops ticking once the game has ended', async () => {
    vi.mocked(api.newGame).mockResolvedValue(makeSnapshot({ status: 'game_over' }))
    render(<App />)
    await screen.findByLabelText('game board')
    await runFrame(performance.now() + 16)
    expect(api.tick).not.toHaveBeenCalled()
  })

  it('skips ticking when no time has elapsed', async () => {
    render(<App />)
    await screen.findByLabelText('game board')
    await runFrame(0)
    expect(api.tick).not.toHaveBeenCalled()
  })

  it('shows no Restart button or game over message while playing', async () => {
    render(<App />)
    await screen.findByLabelText('game board')
    expect(screen.queryByRole('button', { name: 'Restart' })).toBeNull()
    expect(screen.queryByText('GAME OVER')).toBeNull()
  })

  it('shows GAME OVER over the field and restarts from its button', async () => {
    vi.mocked(api.newGame).mockResolvedValueOnce(makeSnapshot({ status: 'game_over' }))
    render(<App />)
    expect(await screen.findByText('GAME OVER')).toBeInTheDocument()
    vi.mocked(api.newGame).mockClear()
    fireEvent.click(screen.getByRole('button', { name: 'Restart' }))
    await waitFor(() => expect(api.newGame).toHaveBeenCalledTimes(1))
  })

  describe('restarting from the keyboard on GAME OVER', () => {
    const gameOver = async () => {
      vi.mocked(api.newGame).mockResolvedValueOnce(makeSnapshot({ status: 'game_over' }))
      render(<App />)
      await screen.findByText('GAME OVER')
      vi.mocked(api.newGame).mockClear()
    }
    const afterGuard = () => {
      const now = performance.now()
      vi.spyOn(performance, 'now').mockReturnValue(now + RESTART_KEY_GUARD_MS + 1)
    }

    it.each(['Enter', ' '])('restarts when %j is pressed', async (key) => {
      await gameOver()
      afterGuard()
      fireEvent.keyDown(window, { key })
      await waitFor(() => expect(api.newGame).toHaveBeenCalledTimes(1))
    })

    it('ignores the keys right after the game ends, so mashed keys do not restart by accident', async () => {
      await gameOver()
      fireEvent.keyDown(window, { key: ' ' })
      fireEvent.keyDown(window, { key: 'Enter' })
      expect(api.newGame).not.toHaveBeenCalled()
    })

    it('ignores held-down key repeats', async () => {
      await gameOver()
      afterGuard()
      fireEvent.keyDown(window, { key: 'Enter', repeat: true })
      expect(api.newGame).not.toHaveBeenCalled()
    })

    it('does not send game inputs while the game is over', async () => {
      await gameOver()
      afterGuard()
      fireEvent.keyDown(window, { key: 'ArrowLeft' })
      expect(api.sendInput).not.toHaveBeenCalled()
      expect(api.newGame).not.toHaveBeenCalled()
    })

    it('keeps Space as rotate while playing', async () => {
      render(<App />)
      await screen.findByLabelText('game board')
      fireEvent.keyDown(window, { key: ' ' })
      await waitFor(() => expect(api.sendInput).toHaveBeenCalledWith('rotate'))
      expect(api.newGame).toHaveBeenCalledTimes(1) // 起動時の 1 回だけ
    })
  })
})
