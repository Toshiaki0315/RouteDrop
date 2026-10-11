import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { GameOver } from './GameOver'

describe('GameOver', () => {
  it('announces GAME OVER', () => {
    render(<GameOver onRestart={() => {}} />)
    expect(screen.getByRole('status')).toHaveTextContent('GAME OVER')
  })

  it('restarts when the Restart button is clicked', () => {
    const onRestart = vi.fn()
    render(<GameOver onRestart={onRestart} />)
    fireEvent.click(screen.getByRole('button', { name: 'Restart' }))
    expect(onRestart).toHaveBeenCalledTimes(1)
  })
})
