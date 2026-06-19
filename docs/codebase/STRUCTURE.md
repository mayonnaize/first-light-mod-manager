# Codebase Structure

## Core Sections (Required)

### 1) Top-Level Map

| Path | Purpose | Evidence |
|------|---------|----------|
| `src/` | Frontend Vanilla JS, HTML, CSS source files | [src/main.js](file:///e:/MOD/first-light-mod-manager/src/main.js) |
| `src-tauri/` | Tauri native Rust backend config and sources | [src-tauri/src/commands.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/commands.rs) |
| `src-tauri/src/settings.rs` | App configurations and game path normalization | [settings.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/settings.rs) |
| `src-tauri/src/crypto.rs` | XTEA crypt and CRC32 implementations for packagedefinition | [crypto.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/crypto.rs) |
| `src-tauri/src/backup.rs` | Game configuration backups and restoration routines | [backup.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/backup.rs) |
| `src-tauri/src/game_detect.rs` | Steam/Epic path discovery logic | [game_detect.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/game_detect.rs) |
| `src-tauri/src/mod_manager.rs` | Mod extraction, path matching, toggle/delete management | [mod_manager.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs) |
| `scripts/` | Tooling and environment scripts (Tauri runner, version syncer) | [scripts/tauri.ps1](file:///e:/MOD/first-light-mod-manager/scripts/tauri.ps1) |
| `tests/` | Playwright E2E and browser JS unit tests | [tests/e2e/app.spec.js](file:///e:/MOD/first-light-mod-manager/tests/e2e/app.spec.js) |

### 2) Entry Points

- Main runtime entry (Frontend): [src/index.html](file:///e:/MOD/first-light-mod-manager/src/index.html) loaded by Tauri window.
- Main runtime entry (Backend): [src-tauri/src/main.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/main.rs) which bootstraps the Tauri application.
- Secondary entry points: None.
- How entry is selected: Configure via [tauri.conf.json](file:///e:/MOD/first-light-mod-manager/src-tauri/tauri.conf.json) (`frontendDist` pointing to `/src`).

### 3) Module Boundaries

| Boundary | What belongs here | What must not be here |
|----------|-------------------|------------------------|
| Frontend (`src/`) | UI Rendering, event handling, user preference memory (`localStorage`), calling Rust commands | File I/O, registry access, cryptographic routines, zip extraction |
| Backend Commands Router (`src-tauri/src/commands.rs`) | Declaring asynchronous Tauri command entry points | Direct crypto calculations, low-level FS parsing |
| Backend Submodules (`src-tauri/src/*`) | Domain models (settings, crypto, backups, game detection, mod lists) | UI state rendering, raw HTML templates |

### 4) Naming and Organization Rules

- File naming pattern: Lowercase with underscores for Rust files (e.g. `commands.rs`), lowercase/camelCase for JavaScript (e.g. `main.js`).
- Directory organization pattern: Layered structure separating frontend (`src/`), backend (`src-tauri/`), and tests (`tests/`).
- Import conventions: Standard ES modules (`import`/`export`) or standard Rust module trees (`mod`, `use`).

### 5) Evidence

- [src/main.js](file:///e:/MOD/first-light-mod-manager/src/main.js)
- [src-tauri/src/lib.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/lib.rs)
- [tauri.conf.json](file:///e:/MOD/first-light-mod-manager/src-tauri/tauri.conf.json)
