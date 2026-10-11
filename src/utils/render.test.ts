import { describe, expect, it, vi } from 'vitest'
import { makeSnapshot } from '../fixtures'
import { CELL_SIZE, canvasSize, drawSnapshot } from './render'

function mockCtx() {
  const texts: string[] = []
  const ctx = {
    fillRect: vi.fn(),
    strokeRect: vi.fn(),
    fillText: vi.fn((text: string) => texts.push(text)),
    save: vi.fn(),
    restore: vi.fn(),
    translate: vi.fn(),
    rotate: vi.fn(),
    beginPath: vi.fn(),
    moveTo: vi.fn(),
    lineTo: vi.fn(),
    arc: vi.fn(),
    stroke: vi.fn(),
    fill: vi.fn(),
  }
  return { ctx: ctx as unknown as CanvasRenderingContext2D, mocks: ctx, texts }
}

const straightOnly = makeSnapshot({
  grid: { width: 3, height: 2, cells: Array(6).fill('empty') },
  piece: null,
})

describe('canvasSize', () => {
  it('adds one station row above and below the field', () => {
    expect(canvasSize(makeSnapshot())).toEqual({ width: 3 * CELL_SIZE, height: 4 * CELL_SIZE })
    expect(canvasSize(makeSnapshot(), 10)).toEqual({ width: 30, height: 40 })
  })
})

describe('drawSnapshot', () => {
  it('fills the whole canvas and outlines every field cell', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot())
    expect(mocks.fillRect).toHaveBeenCalledWith(0, 0, 3 * CELL_SIZE, 4 * CELL_SIZE)
    expect(mocks.strokeRect).toHaveBeenCalledTimes(6)
  })

  it('labels the stations and the next-piece box', () => {
    const { ctx, texts } = mockCtx()
    drawSnapshot(ctx, makeSnapshot())
    expect(texts).toEqual(['スタート', 'ゴール', 'NEXT'])
  })

  it('places the stations outside the field, above and below it', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot())
    const translated = mocks.translate.mock.calls.map(([x, y]) => `${x},${y}`)
    // スタート駅 (盤面の下 y=2) とゴール駅 (盤面の上 y=-1) の中心
    expect(translated).toContain(`${1.5 * CELL_SIZE},${3.5 * CELL_SIZE}`)
    expect(translated).toContain(`${1.5 * CELL_SIZE},${0.5 * CELL_SIZE}`)
  })

  it('shows the next piece in the top-left margin cell', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot())
    const translated = mocks.translate.mock.calls.map(([x, y]) => `${x},${y}`)
    expect(translated).toContain(`${0.5 * CELL_SIZE},${0.5 * CELL_SIZE}`)
  })

  it('rotates rails to their facing, then the train to its heading', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot())
    const angles = mocks.rotate.mock.calls.map(([a]) => a)
    // 盤面: 北 / 東 / 南、駅 2 つ、落下パネル(北)、ネクスト(東)、列車(東)
    expect(angles).toEqual([0, Math.PI / 2, Math.PI, 0, 0, 0, Math.PI / 2, Math.PI / 2])
  })

  it('skips the falling piece when there is none', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot({ piece: null }))
    expect(mocks.rotate).toHaveBeenCalledTimes(7)
  })

  it.each([
    ['north', 0],
    ['east', Math.PI / 2],
    ['south', Math.PI],
    ['west', -Math.PI / 2],
  ] as const)('turns the locomotive to face %s', (heading, angle) => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot({ robot: { pos: { x: 1, y: 1 }, heading } }))
    const last = mocks.rotate.mock.calls[mocks.rotate.mock.calls.length - 1]
    expect(last).toEqual([angle])
  })

  it('draws the locomotive at the robot position, even on the station', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot({ robot: { pos: { x: 2, y: 0 }, heading: 'north' } }))
    expect(mocks.translate.mock.calls[mocks.translate.mock.calls.length - 1]).toEqual([2.5 * CELL_SIZE, 1.5 * CELL_SIZE])
  })

  it('draws curves with arcs and straight rails without them', () => {
    const withCurves = mockCtx()
    drawSnapshot(withCurves.ctx, makeSnapshot({ piece: null }))
    const without = mockCtx()
    drawSnapshot(without.ctx, straightOnly)
    // 盤面の右折・左折カーブが 2 本ずつ弧を描く (列車の弧は共通)
    expect(withCurves.mocks.arc.mock.calls.length - without.mocks.arc.mock.calls.length).toBe(4)
  })

  it('draws a left curve and a right curve on opposite sides', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(
      ctx,
      makeSnapshot({
        grid: {
          width: 3,
          height: 2,
          cells: [
            { panel: { kind: 'turn_right', facing: 'north' } },
            { panel: { kind: 'turn_left', facing: 'north' } },
            'empty',
            'empty',
            'empty',
            'empty',
          ],
        },
        piece: null,
      }),
    )
    const centersX = mocks.arc.mock.calls.slice(0, 4).map(([x]) => x)
    expect(centersX[0]).toBeGreaterThan(0)
    expect(centersX[2]).toBeLessThan(0)
  })
})
