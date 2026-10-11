import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Hud } from './Hud'

describe('Hud', () => {
  it('shows score, stations reached and rows to the next station in one zero-padded line', () => {
    render(<Hud score={10} stationsReached={1} rowsToStation={15} />)
    expect(screen.getByText('Score: 000010 / 到着駅: 001 / 次の駅まで: 015')).toBeInTheDocument()
  })

  it('pads small values and never shows a negative distance', () => {
    render(<Hud score={0} stationsReached={0} rowsToStation={-3} />)
    expect(screen.getByText('Score: 000000 / 到着駅: 000 / 次の駅まで: 000')).toBeInTheDocument()
  })

  it('keeps big values as they are', () => {
    render(<Hud score={1234567} stationsReached={12} rowsToStation={120} />)
    expect(screen.getByText('Score: 1234567 / 到着駅: 012 / 次の駅まで: 120')).toBeInTheDocument()
  })
})
