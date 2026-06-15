# 🕵️ First Light Mod Manager (FLMM)

> **007: First Light** 向けの軽量オープンソース Mod マネージャー

---

## Changes from upstream ([Welsker-dev/first-light-mod-manager](https://github.com/Welsker-dev/first-light-mod-manager))

This fork contains the following fixes and enhancements on top of the original:

### `src-tauri/src/commands.rs`

- **Fixed XTEA keys and header** — Corrected the XTEA decryption keys and the 16-byte magic header constant to match the actual 007: First Light `packagedefinition.txt` format.
- **Fixed `packagedefinition.txt` encoding** — Replaced strict `String::from_utf8` with Latin-1 (ISO 8859-1) fixed decoding. UTF-8 BOM (`EF BB BF`) is stripped before decoding.
- **MOD patch number auto-assignment** — MOD `.rpkg` files are now assigned patch numbers starting at **100** (`MOD_PATCH_START = 100`) to avoid colliding with official game patches. Previously, chunk0 started at patch2 and other chunks at patch1.
- **Backup-aware slot reservation** — `used_patch_slots` now also scans `Runtime_backup_original` to treat official patch slots as reserved, preventing overwrite on re-install.
- **Rewrote backend from scratch** — The upstream `commands.rs` was a stub. This fork contains a full implementation: settings persistence, game auto-detection (Steam/Epic), mod install/uninstall/toggle/delete, ZIP package support, mod metadata, package definition patching, and Nexus Mods update check.
- **68 unit tests added** — Rust バックエンドの全主要関数をカバー (XTEA暗号・CRC32・パーサー・設定保存・ファイルシステム操作等)

### `tests/unit/utils.spec.js` (New)

- **20 JS ユニットテスト追加** — `formatBytes`, `escapeHtml`, `normalizeSettings` の純粋関数ロジックを境界値・エラー系も含めて検証

### `scripts/tauri.ps1` / `tests/e2e/app.spec.js`

- PowerShell build helper and Playwright E2E test suite (47 tests) added.

### `README.md`

- Rewritten in Japanese only.

---


![License](https://img.shields.io/github/license/Welsker-dev/first-light-mod-manager)
![Platform](https://img.shields.io/badge/platform-Windows-blue)
![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri-24C8D8)
![Status](https://img.shields.io/badge/status-in%20development-yellow)

---

## 概要

**First Light Mod Manager (FLMM)** は **007: First Light** (IO Interactive, 2026 / Glacier 2 エンジン) 専用の無料オープンソース Mod マネージャーです。

Hitman: World of Assassination コミュニティのモッディングツールに触発され、`.rpkg` / `.zip` 形式の Mod を手動でファイルを触ることなく簡単にインストール・管理・削除できます。

---

## 機能

| 機能 | 説明 |
|------|------|
| 🔍 自動検出 | Steam / Epic Games Store のインストール先を自動検出 |
| 📦 ワンクリックインストール | `.rpkg` または `.zip` をドラッグ＆ドロップするだけ |
| 👁️ インストール前プレビュー | ターゲットファイル名とパッチ番号を事前確認 |
| ✍️ パッチレベル自動調整 | `packagedefinition.txt` のパーティション patchlevel を自動更新 |
| 🔢 パッチ番号自動採番 | 公式パッチと衝突しないよう番号 (100〜) を自動割り当て |
| 🛡️ 自動バックアップ | インストール前に `Runtime` フォルダを自動バックアップ |
| 🗑️ ワンクリックアンインストール | バックアップからゲームを即座に完全復元 |
| ⚡ 軽量 | 追加ランタイム依存なし |

---

## 使い方

1. [Releases](../../releases) からインストーラーまたはポータブル `.exe` をダウンロード
2. FLMM を起動 — 007: First Light のインストール先を自動検出します
3. 検出されない場合は **「手動で選択」** からゲームフォルダを指定
4. **「Mod をインストール」** タブへ移動
5. `.rpkg` または `.zip` ファイルをドラッグ＆ドロップ（またはクリックして参照）
6. インストール先ファイルのプレビューを確認し **「Mod をインストール」** をクリック
7. 全 Mod を削除するにはホーム画面の **「アンインストール」** をクリック

---

## 動作環境

- Windows 10 / 11 (64-bit)
- Steam または Epic Games Store 版 007: First Light

---

## 開発

```powershell
npm install
npm test
npm run tauri build
```

`npm test` は Rust ユニットテストと Playwright E2E テストを実行します。Tauri スクリプトは Windows 上でデフォルトの Rust ツールチェーンパスを自動追加します。

また、以下の方法でテストカバレッジを測定できます：

- **Rust バックエンドカバレッジ**: `cargo llvm-cov` を用いて計測
- **フロントエンドカバレッジ**: Playwright テスト実行時に `monocart-reporter` を介して自動計測 (結果は `test-results/coverage/` に出力)

VS Code をご利用の場合は、`Ctrl+Shift+B` もしくは「タスクの実行」から `Coverage (Rust)` / `Coverage (JS/Frontend)` / `Coverage (All)` タスクを使用して、ワンクリックでカバレッジ測定からレポート表示まで行えます。

---

## 使用技術

- [Tauri](https://tauri.app/) — 軽量デスクトップアプリフレームワーク (Rust + HTML/CSS/JS)
- [Rust](https://www.rust-lang.org/) — バックエンドロジック (ファイル操作・ゲーム検出・RPKG 処理)
- Vanilla HTML/CSS/JS — フロントエンド UI

---

## 関連プロジェクト・クレジット

- [Glacier Modding](https://glaciermodding.org/) — Glacier 2 エンジン向けコミュニティツール
- [RPKG Tool](https://github.com/glacier-modding/RPKG-Tool) — RPKG 処理の参考元
- [Simple Mod Framework](https://www.nexusmods.com/hitman3/mods/200) — Hitman: WoA 向け Mod マネージャー (本プロジェクトの着想元)

---

## コントリビューション

プルリクエスト歓迎。バグ報告や機能提案は [Issue](../../issues) からどうぞ。

---

## ライセンス

[MIT License](LICENSE)

---

> Made with ❤️ by [DublaX Team](https://discord.gg/aMfk6wgA) — *「Dublamos jogos!」*
>
> 007 FIRST LIGHT © 2026 IOI / Metro-Goldwyn-Mayer Studios Inc. 本ツールは IO Interactive・Danjaq・EON Productions とは無関係です。
