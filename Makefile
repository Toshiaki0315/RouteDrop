# RouteDrop（Tauri v2）の開発コマンド入口。OboeGaki と同じ流儀。

.DEFAULT_GOAL := help
.PHONY: help setup dev app dmg icon test test-rust coverage check

help:             ## コマンド一覧
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | sed -E 's/:[^#]*## /\t/' | awk -F'\t' '{printf "  make %-10s %s\n", $$1, $$2}'

setup:            ## 初回セットアップ（npm の依存を入れる）
	npm install

# dev サーバのポート。既定 1430（1420 は Tauri テンプレートの既定値で
# 他アプリと衝突する）。`ROUTEDROP_DEV_PORT=1500 make dev` で変えられる。
# vite.config.ts はこの環境変数を読む。tauri.conf.json の devUrl は静的なので、
# 同じ値を --config で被せて揃える。
ROUTEDROP_DEV_PORT ?= 1430

dev:              ## 開発起動（Tauri dev。初回は Rust ビルドで数分）
	ROUTEDROP_DEV_PORT=$(ROUTEDROP_DEV_PORT) npm run tauri dev -- \
	  --config '{"build":{"devUrl":"http://localhost:$(ROUTEDROP_DEV_PORT)"}}'

# DMG まで要らないとき用。tauri build は既定で targets "all" を組むため
# DMG まで作って伸びる。`--bundles app` で .app だけにする。
APP := src-tauri/target/release/bundle/macos/RouteDrop.app

app:              ## アプリ（.app）を組む。DMG は作らない
	npm run tauri build -- --bundles app
	@echo "アプリ: $(APP)"
	@echo "開く:   open $(APP)"
	@echo "※ 署名・公証はまだ。初回は右クリック →「開く」で Gatekeeper を通す"

dmg:              ## インストール用 DMG を新規ビルドから作る
	npm run tauri build
	@echo "DMG: src-tauri/target/release/bundle/dmg/"
	@echo "※ 署名・公証はまだ（Apple Developer アカウント取得後）"

icon:             ## アプリのアイコンを描き直して全サイズを作る（scripts/make_icon.swift）
	swift scripts/make_icon.swift src-tauri/icons/icon-source.png
	npx tauri icon src-tauri/icons/icon-source.png -o src-tauri/icons

test:             ## フロントエンド (Vitest) のテスト
	npx vitest run

test-rust:        ## Rust 側のテスト
	cd src-tauri && cargo test

coverage:         ## カバレッジ計測（TS と Rust の core。core は 100% 必須）
	npm run test:coverage
	npm run test:rust

check:            ## コミット前チェック（型 + 全テスト + core カバレッジ 100%）
	npx tsc --noEmit
	npx vitest run
	cd src-tauri && cargo fmt --check
	cd src-tauri && cargo test
	npm run test:rust
