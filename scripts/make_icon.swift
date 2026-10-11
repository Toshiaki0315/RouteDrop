// アプリのアイコンの元絵（1024px PNG）を描く。
//
//     make icon   （= swift scripts/make_icon.swift && npx tauri icon …）
//
// 外部の画像編集ソフトも Python の依存も要らないよう、macOS の CoreGraphics だけで描く。
//
// デザインは「RouteDrop」— 草地の上の線路を、顔つきの蒸気機関車（SL）が
// こちらへ走ってくる。ゲーム中の SL（青いボディ・赤い運転室・金の煙突）と同じ配色。

import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

let size: CGFloat = 1024
let output = CommandLine.arguments.dropFirst().first ?? "src-tauri/icons/icon-source.png"

func color(_ hex: String, _ alpha: CGFloat = 1) -> CGColor {
    var value: UInt64 = 0
    Scanner(string: String(hex.dropFirst())).scanHexInt64(&value)
    let r = CGFloat((value >> 16) & 0xff) / 255
    let g = CGFloat((value >> 8) & 0xff) / 255
    let b = CGFloat(value & 0xff) / 255
    return CGColor(colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!, components: [r, g, b, alpha])!
}

let grassTop = color("#A8E39B")
let grassBottom = color("#78C474")
let edge = color("#4E9B55")
let bed = color("#F3E3C3")
let sleeper = color("#C28B55")
let railShadow = color("#8D97A8")
let railShine = color("#F4F7FB")
let blue = color("#5AA9E6")
let blueLight = color("#8EC9F5")
let blueDark = color("#3D86C0")
let red = color("#EE6B6B")
let redLight = color("#FF9A9A")
let gold = color("#FFD166")
let goldDark = color("#E0A93A")
let dark = color("#3A3347")
let white = color("#FFFFFF")
let cheek = color("#FFB3B3")

let space = CGColorSpace(name: CGColorSpace.sRGB)!
let ctx = CGContext(
    data: nil, width: Int(size), height: Int(size), bitsPerComponent: 8, bytesPerRow: 0,
    space: space, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
ctx.setAllowsAntialiasing(true)
ctx.setShouldAntialias(true)
// CoreGraphics は左下が原点。上下を返して「下が +y」で描く
ctx.translateBy(x: 0, y: size)
ctx.scaleBy(x: 1, y: -1)

func s(_ v: CGFloat) -> CGFloat { v * size }

func roundedRect(_ rect: CGRect, _ radius: CGFloat) -> CGPath {
    CGPath(roundedRect: rect, cornerWidth: radius, cornerHeight: radius, transform: nil)
}

func fill(_ path: CGPath, _ c: CGColor) {
    ctx.setFillColor(c)
    ctx.addPath(path)
    ctx.fillPath()
}

func circle(_ cx: CGFloat, _ cy: CGFloat, _ r: CGFloat, _ c: CGColor) {
    fill(CGPath(ellipseIn: CGRect(x: cx - r, y: cy - r, width: r * 2, height: r * 2), transform: nil), c)
}

func line(_ x1: CGFloat, _ y1: CGFloat, _ x2: CGFloat, _ y2: CGFloat, _ width: CGFloat, _ c: CGColor) {
    ctx.setStrokeColor(c)
    ctx.setLineWidth(width)
    ctx.setLineCap(.round)
    ctx.move(to: CGPoint(x: x1, y: y1))
    ctx.addLine(to: CGPoint(x: x2, y: y2))
    ctx.strokePath()
}

// ---- 背景: 角丸の草地 -----------------------------------------------------
let inset = s(0.06)
let body = roundedRect(CGRect(x: inset, y: inset, width: size - inset * 2, height: size - inset * 2), s(0.22))
ctx.saveGState()
ctx.addPath(body)
ctx.clip()
let gradient = CGGradient(colorsSpace: space, colors: [grassTop, grassBottom] as CFArray, locations: [0, 1])!
ctx.drawLinearGradient(gradient, start: CGPoint(x: 0, y: 0), end: CGPoint(x: 0, y: size), options: [])
// 草地の市松もよう (ゲームの盤面と同じ)
let tile = s(0.125)
for row in 0..<8 {
    for col in 0..<8 where (row + col) % 2 == 0 {
        fill(CGPath(rect: CGRect(x: CGFloat(col) * tile, y: CGFloat(row) * tile, width: tile, height: tile), transform: nil),
             color("#FFFFFF", 0.07))
    }
}

// ---- 線路: 手前へ広がる遠近 -------------------------------------------------
let cx = size / 2
let horizonY = s(0.56)
let nearY = size
let topHalf = s(0.20)       // 遠い側 (列車の下) の道床の半幅
let bottomHalf = s(0.40)    // 手前の道床の半幅
let bedPath = CGMutablePath()
bedPath.move(to: CGPoint(x: cx - topHalf, y: horizonY))
bedPath.addLine(to: CGPoint(x: cx + topHalf, y: horizonY))
bedPath.addLine(to: CGPoint(x: cx + bottomHalf, y: nearY))
bedPath.addLine(to: CGPoint(x: cx - bottomHalf, y: nearY))
bedPath.closeSubpath()
fill(bedPath, bed)

func lerp(_ a: CGFloat, _ b: CGFloat, _ t: CGFloat) -> CGFloat { a + (b - a) * t }
// 枕木: 手前ほど太く・間隔が広い
for i in 0..<7 {
    let t = pow(CGFloat(i) / 6, 1.6)
    let y = lerp(horizonY + s(0.05), nearY - s(0.03), t)
    let k = (y - horizonY) / (nearY - horizonY)
    let half = lerp(topHalf, bottomHalf, k) * 0.9
    line(cx - half, y, cx + half, y, lerp(s(0.018), s(0.05), k), sleeper)
}
// レール 2 本 (影 + つや)
for sign in [-1.0, 1.0] as [CGFloat] {
    for (c, w) in [(railShadow, s(0.026)), (railShine, s(0.012))] {
        line(cx + sign * topHalf * 0.55, horizonY,
             cx + sign * bottomHalf * 0.55, nearY, w, c)
    }
}

// ---- 列車 (正面) ------------------------------------------------------------
// 運転室 (うしろ・赤)
fill(roundedRect(CGRect(x: cx - s(0.27), y: s(0.15), width: s(0.54), height: s(0.26)), s(0.08)), red)
fill(roundedRect(CGRect(x: cx - s(0.22), y: s(0.17), width: s(0.44), height: s(0.10)), s(0.05)), redLight)
// 煙突 (金)
fill(roundedRect(CGRect(x: cx - s(0.075), y: s(0.20), width: s(0.15), height: s(0.16)), s(0.025)), gold)
fill(roundedRect(CGRect(x: cx - s(0.10), y: s(0.17), width: s(0.20), height: s(0.05)), s(0.025)), goldDark)
// 煙
circle(cx + s(0.17), s(0.125), s(0.036), color("#FFFFFF", 0.95))
circle(cx + s(0.245), s(0.105), s(0.028), color("#FFFFFF", 0.8))
circle(cx + s(0.30), s(0.09), s(0.02), color("#FFFFFF", 0.6))
// 車輪
for sign in [-1.0, 1.0] as [CGFloat] {
    circle(cx + sign * s(0.25), s(0.66), s(0.075), dark)
    circle(cx + sign * s(0.25), s(0.66), s(0.03), color("#8D97A8"))
}
// ボイラー (顔)
let face = CGRect(x: cx - s(0.26), y: s(0.30), width: s(0.52), height: s(0.42))
fill(roundedRect(face, s(0.20)), blue)
fill(roundedRect(CGRect(x: face.minX + s(0.06), y: face.minY + s(0.07), width: s(0.08), height: s(0.22)), s(0.04)), blueLight)
// ヘッドライト
circle(cx, s(0.345), s(0.038), gold)
circle(cx, s(0.345), s(0.02), color("#FFF3A0"))
// 目
for sign in [-1.0, 1.0] as [CGFloat] {
    circle(cx + sign * s(0.095), s(0.46), s(0.068), white)
    circle(cx + sign * s(0.095), s(0.47), s(0.036), dark)
    circle(cx + sign * s(0.085), s(0.455), s(0.012), white)
    circle(cx + sign * s(0.17), s(0.545), s(0.032), cheek)
}
// 笑顔
ctx.setStrokeColor(dark)
ctx.setLineWidth(s(0.014))
ctx.setLineCap(.round)
ctx.addArc(center: CGPoint(x: cx, y: s(0.52)), radius: s(0.055), startAngle: .pi * 0.15, endAngle: .pi * 0.85, clockwise: false)
ctx.strokePath()
// カウキャッチャー
let catcher = CGMutablePath()
catcher.move(to: CGPoint(x: cx - s(0.20), y: s(0.70)))
catcher.addLine(to: CGPoint(x: cx + s(0.20), y: s(0.70)))
catcher.addLine(to: CGPoint(x: cx + s(0.14), y: s(0.77)))
catcher.addLine(to: CGPoint(x: cx - s(0.14), y: s(0.77)))
catcher.closeSubpath()
fill(catcher, red)
fill(CGPath(rect: CGRect(x: cx - s(0.20), y: s(0.69), width: s(0.40), height: s(0.02)), transform: nil), blueDark)
ctx.restoreGState()

// 縁の線: 白い地や Finder の背景で、草地の縁が溶けないように
ctx.setStrokeColor(edge)
ctx.setLineWidth(s(0.008))
ctx.addPath(body)
ctx.strokePath()

// ---- 書き出し ---------------------------------------------------------------
let image = ctx.makeImage()!
let url = URL(fileURLWithPath: output) as CFURL
let dest = CGImageDestinationCreateWithURL(url, UTType.png.identifier as CFString, 1, nil)!
CGImageDestinationAddImage(dest, image, nil)
guard CGImageDestinationFinalize(dest) else {
    FileHandle.standardError.write(Data("PNG を書き出せませんでした: \(output)\n".utf8))
    exit(1)
}
print("アイコンの元絵: \(output)")
