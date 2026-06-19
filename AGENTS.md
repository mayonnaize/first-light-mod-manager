# First Light Mod Manager (FLMM) - Project Instructions & Context

This file contains architecture rules, conventions, and workflows for the `first-light-mod-manager` repository. These guidelines must be followed in all future interactions.

## 1. Project Overview
**First Light Mod Manager (FLMM)** is a lightweight desktop application designed to manage mods for the PC game **007: First Light** (Glacier 2 engine). It allows users to install, toggle, and uninstall `.rpkg` and `.zip` mods safely via drag-and-drop, handling game engine patching (`packagedefinition.txt`) automatically.

## 2. Architecture & Tech Stack
The application is built using **Tauri** (v2), which combines a web frontend with a native Rust backend.

### 2.1 Frontend
- **Stack:** Vanilla HTML (`src/index.html`), CSS (`src/styles.css`), and JavaScript (`src/main.js`).
- **Frameworks:** **NO** frontend frameworks (like React, Vue, Angular) or CSS frameworks (like TailwindCSS) are used. Stick strictly to Vanilla JS and Vanilla CSS.
- **Role:** Handles UI, user interaction, and presentation. Communicates entirely with the backend using Tauri's IPC `invoke()` mechanism.

### 2.2 Backend
- **Stack:** Rust (located in `src-tauri/src/`). Main logic resides in `commands.rs`.
- **Role:** Handles OS-level interactions:
  - Game path auto-detection (Steam/Epic via Windows Registry using `winreg`).
  - File management (extracting `.zip`, copying `.rpkg`).
  - Mod registration and patching `packagedefinition.txt` (including XTEA decryption/encryption and Latin-1 fixed decoding).
- **Patch Numbering:** Mod patch numbers are automatically assigned starting at **100** to avoid colliding with official patches.

## 3. Testing & Workflows
- **Unit Tests (Backend):** Rust unit tests are located in `src-tauri/src/` (within `commands.rs`). Run via `npm run test:rust`. **77 tests** covering XTEA crypto, CRC32, path normalization, patchlevel logic, metadata parsing, VDF parsing, settings persistence, backup/restore/migration, etc. Coverage is measured via `cargo llvm-cov`.
- **Unit Tests (Frontend):** Pure-function JS unit tests located in `tests/unit/`. Run via `npm run test:e2e` (with single worker). **20 tests** for `formatBytes`, `escapeHtml`, `normalizeSettings` executing in the browser page context to test real code.
- **E2E Tests (Frontend):** Playwright is used for End-to-End testing, located in `tests/e2e/`. Run via `npm run test:e2e`. **47 tests** covering navigation, mod install flow, toggle/delete, language switching, settings persistence, update checking, etc. V8 coverage is measured via Playwright E2E/unit tests using `monocart-reporter` (reports output to `test-results/coverage/`).
- **Combined Test Command:** `npm test` runs both Rust and E2E tests.
- **Building:** Run `npm run tauri build` (which utilizes the `scripts/tauri.ps1` helper for setting up the environment).

## 4. Coding Conventions & Best Practices
- **Frontend Changes:** Keep DOM manipulation direct and clean. Update translations and localized strings where applicable. Avoid adding external dependencies unless absolutely necessary.
- **Backend Changes:** Ensure strict error handling and path validations. The game uses a specific XTEA key and 16-byte magic header for its `packagedefinition.txt`, and file encoding requires specific UTF-8 BOM stripping before Latin-1 decoding. Backup strategy stores only `packagedefinition.txt` in `.flmm_backup/` (not the entire Runtime folder); legacy `Runtime_backup_original/` is auto-migrated on first use.
- **Safety:** Never stage or commit changes unless explicitly instructed by the user. Do not break type safety in Rust or suppress warnings without a clear architectural reason.
- **Repository Guidelines:** This is a forked repository. Never modify any section of `README.md` except the "Key Improvements from Upstream (For General Users)" section. 
