import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { Hud } from './Hud'

describe('Hud', () => {
  it('shows the score and no status while playing', () => {
    render(<Hud score={120} status="playing" onRestart={() => {}} />)
    expect(screen.getByText('Score: 120')).toBeInTheDocument()
    expect(screen.queryByRole('status')).toBeNull()
  })

  it('shows CLEAR! when cleared', () => {
    render(<Hud score={0} status="cleared" onRestart={() => {}} />)
    expect(screen.getByRole('status')).toHaveTextContent('CLEAR!')
  })

  it('shows GAME OVER when the game is over', () => {
    render(<Hud score={0} status="game_over" onRestart={() => {}} />)
    expect(screen.getByRole('status')).toHaveTextContent('GAME OVER')
  })

  it('calls onRestart when the button is clicked', () => {
    const onRestart = vi.fn()
    render(<Hud score={0} status="playing" onRestart={onRestart} />)
    fireEvent.click(screen.getByRole('button', { name: 'Restart' }))
    expect(onRestart).toHaveBeenCalledTimes(1)
  })
})
