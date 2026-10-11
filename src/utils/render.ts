import type { Cell, Direction, Panel, Snapshot } from '../types'

export const CELL_SIZE = 48

// やさしいパステル調。草地・砂利の道床・木の枕木・銀のレール。
const COLORS = {
  margin: '#262a35',
  grass: '#8ed081',
  grassAlt: '#99d98c',
  gridLine: 'rgba(60, 110, 60, 0.18)',
  wall: '#9aa1b5',
  bed: '#f3e3c3',
  sleeper: '#c28b55',
  railShadow: '#8d97a8',
  railShine: '#f4f7fb',
  platform: '#fff4dc',
  stationRoof: '#ee6b6b',
  originRoof: '#5aa9e6',
  stationWall: '#fffaf0',
  sign: '#ff8a8a',
  originSign: '#4a98d8',
  window: '#bfe3ff',
  door: '#c28b55',
  signText: '#ffffff',
  highlight: 'rgba(255, 238, 140, 0.6)',
  spawnColumn: 'rgba(255, 255, 255, 0.28)',
  spawnMarker: '#ffb703',
  labelText: '#fff8e7',
  wheel: '#4a4458',
  boiler: '#5aa9e6',
  boilerLight: '#8ec9f5',
  cab: '#ee6b6b',
  cabRoof: '#ff9a9a',
  gold: '#ffd166',
  eyeWhite: '#ffffff',
  eyeDark: '#3a3347',
  cheek: '#ffb3b3',
}

const HEADING_ANGLE: Record<Direction, number> = {
  north: 0,
  east: Math.PI / 2,
  south: Math.PI,
  west: -Math.PI / 2,
}

/** 始発駅に停まっている列車の縮尺と、マス中心からの下げ幅 (マスに対する比)。 */
const ORIGIN_PARK_SCALE = 0.62
const ORIGIN_PARK_OFFSET = 0.24

/** 盤面の上の余白: 次のレールのアイコン 1 マス + その下のラベル + 出現位置の目印の行。 */
export const topMargin = (cell = CELL_SIZE) => cell + Math.round(cell * 0.7)

export const canvasSize = (snapshot: Snapshot, cell = CELL_SIZE) => ({
  width: snapshot.grid.width * cell,
  height: snapshot.grid.height * cell + topMargin(cell),
})

/** 盤面座標のマス中心を画面座標に変換する (上の余白ぶん下がる)。 */
const center = (x: number, y: number, cell: number) => ({
  cx: x * cell + cell / 2,
  cy: topMargin(cell) + y * cell + cell / 2,
})

function roundedRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number) {
  ctx.beginPath()
  ctx.roundRect(x, y, w, h, r)
}

// ---- レール -------------------------------------------------------------

function straightRail(ctx: CanvasRenderingContext2D, c: number) {
  ctx.fillStyle = COLORS.bed
  ctx.fillRect(-c * 0.3, -c / 2, c * 0.6, c)
  ctx.lineCap = 'round'
  ctx.strokeStyle = COLORS.sleeper
  ctx.lineWidth = c * 0.09
  for (let i = 0; i < 4; i++) {
    const y = -c * 0.375 + i * c * 0.25
    ctx.beginPath()
    ctx.moveTo(-c * 0.26, y)
    ctx.lineTo(c * 0.26, y)
    ctx.stroke()
  }
  for (const [color, width] of [
    [COLORS.railShadow, c * 0.07],
    [COLORS.railShine, c * 0.035],
  ] as const) {
    ctx.strokeStyle = color
    ctx.lineWidth = width
    for (const x of [-c * 0.16, c * 0.16]) {
      ctx.beginPath()
      ctx.moveTo(x, -c / 2)
      ctx.lineTo(x, c / 2)
      ctx.stroke()
    }
  }
}

// 北向きのとき、下辺中央から入って右 (side=1) / 左 (side=-1) の辺中央へ出る 1/4 円のカーブ。
function curveRail(ctx: CanvasRenderingContext2D, c: number, side: 1 | -1) {
  const cx = (side * c) / 2
  const cy = c / 2
  const r = c / 2
  const [from, to] = side === 1 ? [Math.PI, Math.PI * 1.5] : [-Math.PI / 2, 0]
  ctx.lineCap = 'butt'
  ctx.strokeStyle = COLORS.bed
  ctx.lineWidth = c * 0.6
  ctx.beginPath()
  ctx.arc(cx, cy, r, from, to)
  ctx.stroke()
  ctx.lineCap = 'round'
  ctx.strokeStyle = COLORS.sleeper
  ctx.lineWidth = c * 0.09
  for (let i = 0; i < 3; i++) {
    const a = from + ((to - from) * (i + 0.5)) / 3
    ctx.beginPath()
    ctx.moveTo(cx + Math.cos(a) * (r - c * 0.26), cy + Math.sin(a) * (r - c * 0.26))
    ctx.lineTo(cx + Math.cos(a) * (r + c * 0.26), cy + Math.sin(a) * (r + c * 0.26))
    ctx.stroke()
  }
  for (const [color, width] of [
    [COLORS.railShadow, c * 0.07],
    [COLORS.railShine, c * 0.035],
  ] as const) {
    ctx.strokeStyle = color
    ctx.lineWidth = width
    for (const rr of [r - c * 0.16, r + c * 0.16]) {
      ctx.beginPath()
      ctx.arc(cx, cy, rr, from, to)
      ctx.stroke()
    }
  }
}

/** レールを描く。facing 北のとき下から入って、直進は上へ・右折は右へ・左折は左へ出る。 */
function drawRail(ctx: CanvasRenderingContext2D, panel: Panel, cx: number, cy: number, cell: number) {
  ctx.save()
  ctx.translate(cx, cy)
  ctx.rotate(HEADING_ANGLE[panel.facing])
  if (panel.kind === 'straight') straightRail(ctx, cell)
  else curveRail(ctx, cell, panel.kind === 'turn_right' ? 1 : -1)
  ctx.restore()
}

// ---- 盤面 ---------------------------------------------------------------

function drawField(ctx: CanvasRenderingContext2D, snapshot: Snapshot, cell: number) {
  const { width, cells } = snapshot.grid
  cells.forEach((value: Cell, i) => {
    const x = i % width
    const y = Math.floor(i / width)
    ctx.fillStyle = value === 'wall' ? COLORS.wall : (x + y) % 2 ? COLORS.grassAlt : COLORS.grass
    ctx.fillRect(x * cell, topMargin(cell) + y * cell, cell, cell)
    ctx.strokeStyle = COLORS.gridLine
    ctx.lineWidth = 1
    ctx.strokeRect(x * cell, topMargin(cell) + y * cell, cell, cell)
  })
  cells.forEach((value: Cell, i) => {
    if (typeof value === 'object') {
      const { cx, cy } = center(i % width, Math.floor(i / width), cell)
      drawRail(ctx, value.panel, cx, cy, cell)
    }
  })
}

// ---- 駅 -----------------------------------------------------------------

/** 目的地の駅: ホーム・直進レール・小さな駅舎・看板。 */
function drawStation(ctx: CanvasRenderingContext2D, x: number, y: number, cell: number) {
  const c = cell
  const { cx, cy } = center(x, y, cell)
  ctx.fillStyle = COLORS.platform
  roundedRect(ctx, cx - c * 0.48, cy - c * 0.48, c * 0.96, c * 0.96, c * 0.14)
  ctx.fill()
  drawRail(ctx, { kind: 'straight', facing: 'north' }, cx, cy, cell)
  ctx.fillStyle = COLORS.stationWall
  roundedRect(ctx, cx + c * 0.2, cy - c * 0.2, c * 0.26, c * 0.34, c * 0.04)
  ctx.fill()
  ctx.fillStyle = COLORS.stationRoof
  roundedRect(ctx, cx + c * 0.17, cy - c * 0.3, c * 0.32, c * 0.18, c * 0.08)
  ctx.fill()
  ctx.fillStyle = COLORS.sign
  roundedRect(ctx, cx - c * 0.45, cy - c * 0.47, c * 0.62, c * 0.22, c * 0.11)
  ctx.fill()
  ctx.fillStyle = COLORS.signText
  ctx.font = `bold ${c * 0.18}px sans-serif`
  ctx.fillText('駅', cx - c * 0.14, cy - c * 0.36)
}

/** 始発駅: 駅舎のアイコン (看板・三角屋根・時計・窓・ドア)。 */
function drawOriginStation(ctx: CanvasRenderingContext2D, x: number, y: number, cell: number) {
  const c = cell
  const { cx, cy } = center(x, y, cell)
  ctx.fillStyle = COLORS.platform
  roundedRect(ctx, cx - c * 0.48, cy - c * 0.48, c * 0.96, c * 0.96, c * 0.14)
  ctx.fill()
  // 壁
  ctx.fillStyle = COLORS.stationWall
  roundedRect(ctx, cx - c * 0.4, cy - c * 0.02, c * 0.8, c * 0.44, c * 0.05)
  ctx.fill()
  // 窓
  ctx.fillStyle = COLORS.window
  for (const wx of [-0.33, 0.19]) {
    roundedRect(ctx, cx + wx * c, cy + c * 0.08, c * 0.14, c * 0.14, c * 0.03)
    ctx.fill()
  }
  // ドア
  ctx.fillStyle = COLORS.door
  roundedRect(ctx, cx - c * 0.09, cy + c * 0.14, c * 0.18, c * 0.28, c * 0.06)
  ctx.fill()
  // 三角屋根
  ctx.fillStyle = COLORS.originRoof
  ctx.beginPath()
  ctx.moveTo(cx - c * 0.47, cy + c * 0.0)
  ctx.lineTo(cx, cy - c * 0.3)
  ctx.lineTo(cx + c * 0.47, cy + c * 0.0)
  ctx.closePath()
  ctx.fill()
  // 時計
  ctx.fillStyle = COLORS.eyeWhite
  ctx.beginPath()
  ctx.arc(cx, cy - c * 0.1, c * 0.075, 0, Math.PI * 2)
  ctx.fill()
  ctx.strokeStyle = COLORS.eyeDark
  ctx.lineWidth = c * 0.02
  ctx.lineCap = 'round'
  ctx.beginPath()
  ctx.moveTo(cx, cy - c * 0.1)
  ctx.lineTo(cx, cy - c * 0.15)
  ctx.moveTo(cx, cy - c * 0.1)
  ctx.lineTo(cx + c * 0.04, cy - c * 0.08)
  ctx.stroke()
  // 看板
  ctx.fillStyle = COLORS.originSign
  roundedRect(ctx, cx - c * 0.3, cy - c * 0.5, c * 0.6, c * 0.19, c * 0.095)
  ctx.fill()
  ctx.fillStyle = COLORS.signText
  ctx.font = `bold ${c * 0.15}px sans-serif`
  ctx.fillText('始発駅', cx, cy - c * 0.405)
}

function drawStations(ctx: CanvasRenderingContext2D, snapshot: Snapshot, cell: number) {
  const { width, cells } = snapshot.grid
  cells.forEach((value: Cell, i) => {
    const x = i % width
    const y = Math.floor(i / width)
    if (value === 'station') drawStation(ctx, x, y, cell)
    if (value === 'origin') drawOriginStation(ctx, x, y, cell)
  })
}

// ---- 列車・ネクスト ------------------------------------------------------

/** 左上に、フィールドから離して、次に降ってくるレールを表示する。ラベルはアイコンの下。 */
function drawNext(ctx: CanvasRenderingContext2D, next: Panel, cell: number) {
  const cx = cell / 2
  const cy = cell / 2
  ctx.fillStyle = COLORS.grass
  ctx.fillRect(cx - cell / 2, cy - cell / 2, cell, cell)
  ctx.fillStyle = COLORS.highlight
  roundedRect(ctx, cx - cell / 2 + 2, cy - cell / 2 + 2, cell - 4, cell - 4, cell * 0.16)
  ctx.fill()
  drawRail(ctx, next, cx, cy, cell)
  ctx.fillStyle = COLORS.labelText
  ctx.font = `bold ${cell * 0.25}px sans-serif`
  ctx.textAlign = 'left'
  ctx.fillText('次のレール', 0, cell + cell * 0.2)
  ctx.textAlign = 'center'
}

/** 次のレールが出現する列の目印: フィールド上端の ▼ と、その列の最上段のうすい色。 */
function drawSpawnMarker(ctx: CanvasRenderingContext2D, column: number, cell: number) {
  const top = topMargin(cell)
  const cx = column * cell + cell / 2
  ctx.fillStyle = COLORS.spawnColumn
  ctx.fillRect(column * cell, top, cell, cell)
  ctx.fillStyle = COLORS.spawnMarker
  ctx.beginPath()
  ctx.moveTo(cx - cell * 0.17, top - cell * 0.3)
  ctx.lineTo(cx + cell * 0.17, top - cell * 0.3)
  ctx.lineTo(cx, top - cell * 0.06)
  ctx.closePath()
  ctx.fill()
}

/** かわいい SL を上から見た絵。目と笑顔の顔つき。進行方向 (heading) に向けて描く。 */
function drawTrain(ctx: CanvasRenderingContext2D, snapshot: Snapshot, cell: number) {
  const { train } = snapshot
  const c = cell
  const { cx, cy } = center(train.pos.x, train.pos.y, cell)
  const { width, cells } = snapshot.grid
  const atOrigin = cells[train.pos.y * width + train.pos.x] === 'origin'
  ctx.save()
  // 始発駅にいる間は、駅舎が隠れないよう小さくして手前に停める。
  ctx.translate(cx, atOrigin ? cy + c * ORIGIN_PARK_OFFSET : cy)
  if (atOrigin) ctx.scale(ORIGIN_PARK_SCALE, ORIGIN_PARK_SCALE)
  ctx.rotate(HEADING_ANGLE[train.heading])
  // 車輪
  ctx.fillStyle = COLORS.wheel
  for (const x of [-0.27, 0.19]) {
    roundedRect(ctx, x * c, -0.3 * c, 0.08 * c, 0.2 * c, 0.03 * c)
    ctx.fill()
    roundedRect(ctx, x * c, 0.06 * c, 0.08 * c, 0.2 * c, 0.03 * c)
    ctx.fill()
  }
  // 運転室 (うしろ)
  ctx.fillStyle = COLORS.cab
  roundedRect(ctx, -0.21 * c, 0.0, 0.42 * c, 0.34 * c, 0.1 * c)
  ctx.fill()
  ctx.fillStyle = COLORS.cabRoof
  roundedRect(ctx, -0.14 * c, 0.06 * c, 0.28 * c, 0.2 * c, 0.07 * c)
  ctx.fill()
  // ボイラー (まえ)
  ctx.fillStyle = COLORS.boiler
  roundedRect(ctx, -0.19 * c, -0.4 * c, 0.38 * c, 0.46 * c, 0.15 * c)
  ctx.fill()
  ctx.fillStyle = COLORS.boilerLight
  roundedRect(ctx, -0.13 * c, -0.36 * c, 0.1 * c, 0.3 * c, 0.05 * c)
  ctx.fill()
  // 煙突 (金色)
  ctx.fillStyle = COLORS.gold
  ctx.beginPath()
  ctx.arc(0, -0.04 * c, 0.06 * c, 0, Math.PI * 2)
  ctx.fill()
  // 顔: 目・ほっぺ・笑顔
  for (const x of [-0.08, 0.08]) {
    ctx.fillStyle = COLORS.eyeWhite
    ctx.beginPath()
    ctx.arc(x * c, -0.26 * c, 0.06 * c, 0, Math.PI * 2)
    ctx.fill()
    ctx.fillStyle = COLORS.eyeDark
    ctx.beginPath()
    ctx.arc(x * c, -0.28 * c, 0.03 * c, 0, Math.PI * 2)
    ctx.fill()
    ctx.fillStyle = COLORS.cheek
    ctx.beginPath()
    ctx.arc(x * 1.9 * c, -0.17 * c, 0.035 * c, 0, Math.PI * 2)
    ctx.fill()
  }
  ctx.strokeStyle = COLORS.eyeDark
  ctx.lineWidth = c * 0.025
  ctx.lineCap = 'round'
  ctx.beginPath()
  ctx.arc(0, -0.17 * c, 0.05 * c, 0.2 * Math.PI, 0.8 * Math.PI)
  ctx.stroke()
  ctx.restore()
}

export function drawSnapshot(ctx: CanvasRenderingContext2D, snapshot: Snapshot, cell = CELL_SIZE) {
  const { width, height } = canvasSize(snapshot, cell)
  ctx.fillStyle = COLORS.margin
  ctx.fillRect(0, 0, width, height)
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'

  drawField(ctx, snapshot, cell)
  drawStations(ctx, snapshot, cell)

  if (snapshot.piece) {
    const { pos, panel } = snapshot.piece
    const { cx, cy } = center(pos.x, pos.y, cell)
    ctx.fillStyle = COLORS.highlight
    roundedRect(ctx, cx - cell / 2 + 2, cy - cell / 2 + 2, cell - 4, cell - 4, cell * 0.16)
    ctx.fill()
    drawRail(ctx, panel, cx, cy, cell)
  }
  drawSpawnMarker(ctx, snapshot.next_column, cell)
  drawNext(ctx, snapshot.next, cell)
  drawTrain(ctx, snapshot, cell)
}
