import { describe, expect, it, vi } from 'vitest'
import { makeSnapshot } from '../fixtures'
import { CELL_SIZE, canvasSize, drawSnapshot, topMargin } from './render'

function mockCtx() {
  const texts: string[] = []
  const ctx = {
    fillRect: vi.fn(),
    strokeRect: vi.fn(),
    fillText: vi.fn((text: string, _x?: number, _y?: number) => texts.push(text)),
    save: vi.fn(),
    restore: vi.fn(),
    translate: vi.fn(),
    rotate: vi.fn(),
    scale: vi.fn(),
    beginPath: vi.fn(),
    moveTo: vi.fn(),
    lineTo: vi.fn(),
    closePath: vi.fn(),
    arc: vi.fn(),
    roundRect: vi.fn(),
    stroke: vi.fn(),
    fill: vi.fn(),
  }
  return { ctx: ctx as unknown as CanvasRenderingContext2D, mocks: ctx, texts }
}

const base = makeSnapshot()
const mapCells = (f: (c: (typeof base.grid.cells)[number]) => (typeof base.grid.cells)[number]) =>
  ({ ...base.grid, cells: base.grid.cells.map(f) })

const noStation = makeSnapshot({ grid: mapCells((c) => (c === 'station' ? 'empty' : c)) })
const stationAt = (index: number) => {
  const cells = Array(6).fill('empty')
  cells[index] = 'station'
  return makeSnapshot({ grid: { ...base.grid, cells }, piece: null })
}
// 曲線パネルだけを空きマスに置き換えた盤面 (駅・始発駅・直進レールは残す)
const straightOnly = makeSnapshot({
  grid: mapCells((c) => (typeof c === 'object' && c.panel.kind !== 'straight' ? 'empty' : c)),
  piece: null,
})

const translated = (mocks: ReturnType<typeof mockCtx>['mocks']) =>
  mocks.translate.mock.calls.map(([x, y]) => `${x},${y}`)

describe('canvasSize', () => {
  it('leaves a margin above the field for the NEXT box, with a gap to the field', () => {
    expect(topMargin(48)).toBe(82) // アイコン 48 + ラベルと出現位置の目印 34
    expect(canvasSize(base)).toEqual({ width: 3 * CELL_SIZE, height: 2 * CELL_SIZE + topMargin() })
    expect(canvasSize(base, 12)).toEqual({ width: 36, height: 24 + 20 })
  })
})

describe('drawSnapshot', () => {
  it('fills the whole canvas and outlines every field cell', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, base)
    expect(mocks.fillRect).toHaveBeenCalledWith(0, 0, 3 * CELL_SIZE, 2 * CELL_SIZE + topMargin())
    expect(mocks.strokeRect).toHaveBeenCalledTimes(6)
  })

  it('labels the stations and the next-rail box', () => {
    const { ctx, texts } = mockCtx()
    drawSnapshot(ctx, base)
    expect(texts).toEqual(['駅', '始発駅', '次のレール'])
  })

  it('shows no station sign when the field has no station', () => {
    const { ctx, texts, mocks } = mockCtx()
    drawSnapshot(ctx, noStation)
    expect(texts).toEqual(['始発駅', '次のレール'])
    expect(mocks.rotate).toHaveBeenCalledTimes(6) // 駅のレール 1 本ぶん少ない
  })

  it('draws a station inside the field wherever it is', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, stationAt(5))
    // 盤面 (3 列) の index 5 = x:2, y:1 → 上の余白ぶん下
    expect(translated(mocks)).toContain(`${2.5 * CELL_SIZE},${topMargin() + 1.5 * CELL_SIZE}`)
  })

  it('draws the origin station as a station building at its own cell, without a rail', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, base)
    // 看板の文字は始発駅マス (x:0, y:1 → 中心 x = 0.5 マス) の中央に出る
    const sign = mocks.fillText.mock.calls.find(([text]) => text === '始発駅')
    expect(sign?.[1]).toBe(0.5 * CELL_SIZE)
    // 駅舎の三角屋根
    expect(mocks.moveTo).toHaveBeenCalled()
  })

  it('writes 次のレール under the next-rail icon, not inside it', () => {
    const { ctx, mocks, texts } = mockCtx()
    drawSnapshot(ctx, base)
    expect(texts).not.toContain('NEXT')
    const label = mocks.fillText.mock.calls.find(([text]) => text === '次のレール')
    // アイコン (高さ 1 マス) より下で、フィールド上端より上
    expect(label?.[2]).toBeGreaterThan(CELL_SIZE)
    expect(label?.[2]).toBeLessThan(topMargin())
  })

  it.each([0, 1, 2])('marks column %i as the place the next rail will appear', (column) => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot({ next_column: column }))
    const cx = column * CELL_SIZE + CELL_SIZE / 2
    // ▼ の頂点がその列の中心、フィールド上端のすぐ上にある
    expect(mocks.lineTo).toHaveBeenCalledWith(cx, topMargin() - CELL_SIZE * 0.06)
  })

  it('shows the next piece at the top-left, above and apart from the field', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, base)
    expect(translated(mocks)).toContain(`${0.5 * CELL_SIZE},${0.5 * CELL_SIZE}`)
    // NEXT の箱の下端と、フィールドの上端のあいだにすき間がある
    expect(topMargin()).toBeGreaterThan(CELL_SIZE)
  })

  it('rotates rails to their facing, then the train to its heading', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, base)
    const angles = mocks.rotate.mock.calls.map(([a]) => a)
    // 盤面: 北 / 東 / 南、駅 (北)、落下パネル(北)、ネクスト(東)、列車(東)。始発駅は駅舎でレールなし
    expect(angles).toEqual([0, Math.PI / 2, Math.PI, 0, 0, Math.PI / 2, Math.PI / 2])
  })

  it('skips the falling piece when there is none', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot({ piece: null }))
    expect(mocks.rotate).toHaveBeenCalledTimes(6)
  })

  it.each([
    ['north', 0],
    ['east', Math.PI / 2],
    ['south', Math.PI],
    ['west', -Math.PI / 2],
  ] as const)('turns the locomotive to face %s', (heading, angle) => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot({ train: { pos: { x: 1, y: 1 }, heading } }))
    const last = mocks.rotate.mock.calls[mocks.rotate.mock.calls.length - 1]
    expect(last).toEqual([angle])
  })

  it('parks a small locomotive in front of the origin station building', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, base) // 列車は始発駅 (x:0, y:1) にいる
    const calls = mocks.translate.mock.calls
    expect(calls[calls.length - 1]).toEqual([0.5 * CELL_SIZE, topMargin() + 1.5 * CELL_SIZE + 0.24 * CELL_SIZE])
    expect(mocks.scale).toHaveBeenCalledWith(0.62, 0.62)
  })

  it('draws the locomotive at full size away from the origin station', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot({ train: { pos: { x: 2, y: 0 }, heading: 'north' } }))
    expect(mocks.scale).not.toHaveBeenCalled()
  })

  it('draws the locomotive at the train position', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(ctx, makeSnapshot({ train: { pos: { x: 2, y: 0 }, heading: 'north' } }))
    const calls = mocks.translate.mock.calls
    expect(calls[calls.length - 1]).toEqual([2.5 * CELL_SIZE, topMargin() + 0.5 * CELL_SIZE])
  })

  it('draws curves with arcs and straight rails without them', () => {
    const withCurves = mockCtx()
    drawSnapshot(withCurves.ctx, makeSnapshot({ piece: null }))
    const without = mockCtx()
    drawSnapshot(without.ctx, straightOnly)
    // 右折・左折カーブが 5 本ずつ弧を描く (道床 1 + レール 2 本 × 影とつやの 2 重ね。列車の弧は共通)
    expect(withCurves.mocks.arc.mock.calls.length - without.mocks.arc.mock.calls.length).toBe(10)
  })

  it('draws a left curve and a right curve on opposite sides', () => {
    const { ctx, mocks } = mockCtx()
    drawSnapshot(
      ctx,
      makeSnapshot({
        grid: {
          ...base.grid,
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
    const centersX = mocks.arc.mock.calls.map(([x]) => x)
    expect(centersX.some((x) => x > 0)).toBe(true)
    expect(centersX.some((x) => x < 0)).toBe(true)
  })
})
