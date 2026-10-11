import type { Cell, Direction, Panel, Robot, Snapshot } from '../types'

export const CELL_SIZE = 40

const COLORS = {
  margin: '#262a35',
  grass: '#4f9a4a',
  grassAlt: '#58a553',
  gridLine: 'rgba(0, 0, 0, 0.15)',
  wall: '#6b6f80',
  platform: '#cfc8b4',
  highlight: 'rgba(255, 232, 120, 0.5)',
  sleeper: '#8a5a2b',
  rail: '#d4d7dd',
  sign: '#d9453a',
  signText: '#ffffff',
  labelText: '#e8eaf0',
  wheel: '#1b1b1f',
  boiler: '#2e2f38',
  cab: '#c0392b',
  gold: '#e6b422',
  lamp: '#fff3a0',
}

const HEADING_ANGLE: Record<Direction, number> = {
  north: 0,
  east: Math.PI / 2,
  south: Math.PI,
  west: -Math.PI / 2,
}

/** 盤面の上下に 1 マスずつ駅 (ゴール / スタート) の余白を持つ。 */
export const canvasSize = (snapshot: Snapshot, cell = CELL_SIZE) => ({
  width: snapshot.grid.width * cell,
  height: (snapshot.grid.height + 2) * cell,
})

/** 盤面座標 (駅は盤面の外) のマス中心を画面座標に変換する。 */
const center = (x: number, y: number, cell: number) => ({
  cx: x * cell + cell / 2,
  cy: (y + 1) * cell + cell / 2,
})

function straightRail(ctx: CanvasRenderingContext2D, c: number) {
  ctx.strokeStyle = COLORS.sleeper
  ctx.lineWidth = c * 0.07
  for (let i = 0; i < 5; i++) {
    const y = -c * 0.4 + i * c * 0.2
    ctx.beginPath()
    ctx.moveTo(-c * 0.28, y)
    ctx.lineTo(c * 0.28, y)
    ctx.stroke()
  }
  ctx.strokeStyle = COLORS.rail
  ctx.lineWidth = c * 0.05
  for (const x of [-c * 0.16, c * 0.16]) {
    ctx.beginPath()
    ctx.moveTo(x, -c / 2)
    ctx.lineTo(x, c / 2)
    ctx.stroke()
  }
}

// 北向きのとき、下辺中央から入って右 (side=1) / 左 (side=-1) の辺中央へ出る 1/4 円のカーブ。
function curveRail(ctx: CanvasRenderingContext2D, c: number, side: 1 | -1) {
  const cx = (side * c) / 2
  const cy = c / 2
  const r = c / 2
  const [from, to] = side === 1 ? [Math.PI, Math.PI * 1.5] : [-Math.PI / 2, 0]
  ctx.strokeStyle = COLORS.sleeper
  ctx.lineWidth = c * 0.07
  for (let i = 0; i < 4; i++) {
    const a = from + ((to - from) * (i + 0.5)) / 4
    ctx.beginPath()
    ctx.moveTo(cx + Math.cos(a) * (r - c * 0.14), cy + Math.sin(a) * (r - c * 0.14))
    ctx.lineTo(cx + Math.cos(a) * (r + c * 0.14), cy + Math.sin(a) * (r + c * 0.14))
    ctx.stroke()
  }
  ctx.strokeStyle = COLORS.rail
  ctx.lineWidth = c * 0.05
  for (const rr of [r - c * 0.16, r + c * 0.16]) {
    ctx.beginPath()
    ctx.arc(cx, cy, rr, from, to)
    ctx.stroke()
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

function drawField(ctx: CanvasRenderingContext2D, snapshot: Snapshot, cell: number) {
  const { width, cells } = snapshot.grid
  cells.forEach((value: Cell, i) => {
    const x = i % width
    const y = Math.floor(i / width)
    ctx.fillStyle = value === 'wall' ? COLORS.wall : (x + y) % 2 ? COLORS.grassAlt : COLORS.grass
    ctx.fillRect(x * cell, (y + 1) * cell, cell, cell)
    ctx.strokeStyle = COLORS.gridLine
    ctx.lineWidth = 1
    ctx.strokeRect(x * cell, (y + 1) * cell, cell, cell)
  })
  cells.forEach((value: Cell, i) => {
    if (typeof value === 'object') {
      const { cx, cy } = center(i % width, Math.floor(i / width), cell)
      drawRail(ctx, value.panel, cx, cy, cell)
    }
  })
}

function drawStation(ctx: CanvasRenderingContext2D, x: number, y: number, label: string, signDy: number, cell: number) {
  const { cx, cy } = center(x, y, cell)
  ctx.fillStyle = COLORS.platform
  ctx.fillRect(cx - cell / 2, cy - cell / 2, cell, cell)
  drawRail(ctx, { kind: 'straight', facing: 'north' }, cx, cy, cell)
  ctx.fillStyle = COLORS.sign
  ctx.fillRect(cx - cell * 0.45, cy + signDy - cell * 0.11, cell * 0.9, cell * 0.22)
  ctx.fillStyle = COLORS.signText
  ctx.font = `bold ${cell * 0.2}px sans-serif`
  ctx.fillText(label, cx, cy + signDy)
}

/** 左上の余白マスに、次に降ってくるレールを表示する。 */
function drawNext(ctx: CanvasRenderingContext2D, next: Panel, cell: number) {
  const { cx, cy } = center(0, -1, cell)
  ctx.fillStyle = COLORS.grass
  ctx.fillRect(cx - cell / 2, cy - cell / 2, cell, cell)
  ctx.fillStyle = COLORS.highlight
  ctx.fillRect(cx - cell / 2, cy - cell / 2, cell, cell)
  drawRail(ctx, next, cx, cy, cell)
  ctx.fillStyle = COLORS.labelText
  ctx.font = `bold ${cell * 0.22}px sans-serif`
  ctx.fillText('NEXT', cx, cy - cell * 0.36)
}

/** 蒸気機関車 (SL) を上から見た絵。進行方向 (heading) に向けて描く。 */
function drawTrain(ctx: CanvasRenderingContext2D, robot: Robot, cell: number) {
  const c = cell
  const { cx, cy } = center(robot.pos.x, robot.pos.y, cell)
  ctx.save()
  ctx.translate(cx, cy)
  ctx.rotate(HEADING_ANGLE[robot.heading])
  ctx.fillStyle = COLORS.wheel
  for (const x of [-0.24, 0.17]) {
    ctx.fillRect(x * c, -0.28 * c, 0.07 * c, 0.16 * c)
    ctx.fillRect(x * c, 0.04 * c, 0.07 * c, 0.16 * c)
  }
  ctx.fillStyle = COLORS.boiler
  ctx.fillRect(-0.15 * c, -0.34 * c, 0.3 * c, 0.34 * c)
  ctx.fillStyle = COLORS.cab
  ctx.fillRect(-0.19 * c, 0, 0.38 * c, 0.26 * c)
  const dot = (y: number, r: number, color: string) => {
    ctx.fillStyle = color
    ctx.beginPath()
    ctx.arc(0, y * c, r * c, 0, Math.PI * 2)
    ctx.fill()
  }
  dot(-0.24, 0.06, COLORS.wheel) // 煙突
  dot(-0.08, 0.045, COLORS.gold) // ドーム
  dot(-0.34, 0.04, COLORS.lamp) // ヘッドライト
  ctx.restore()
}

export function drawSnapshot(ctx: CanvasRenderingContext2D, snapshot: Snapshot, cell = CELL_SIZE) {
  const { width, height } = canvasSize(snapshot, cell)
  ctx.fillStyle = COLORS.margin
  ctx.fillRect(0, 0, width, height)
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'

  drawField(ctx, snapshot, cell)
  drawStation(ctx, snapshot.start.x, snapshot.start.y, 'スタート', cell * 0.38, cell)
  drawStation(ctx, snapshot.goal.x, snapshot.goal.y, 'ゴール', -cell * 0.38, cell)

  if (snapshot.piece) {
    const { pos, panel } = snapshot.piece
    const { cx, cy } = center(pos.x, pos.y, cell)
    ctx.fillStyle = COLORS.highlight
    ctx.fillRect(cx - cell / 2, cy - cell / 2, cell, cell)
    drawRail(ctx, panel, cx, cy, cell)
  }
  drawNext(ctx, snapshot.next, cell)
  drawTrain(ctx, snapshot.robot, cell)
}
