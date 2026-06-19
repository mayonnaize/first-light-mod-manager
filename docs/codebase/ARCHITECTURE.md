# Architecture

## Core Sections (Required)

### 1) Architectural Style

- Primary style: Tauri Multi-process (Core & WebView) / IPC-based Client-Server architecture.
- Why this classification: The UI runs in a secure frontend webview (isolated from direct OS calls) and communicates strictly with a native Rust core process via Tauri's JSON-RPC-based `invoke()` mechanism.
- Primary constraints:
  1. Frontend has no direct filesystem or OS registry access.
  2. Long-running or OS-level operations (XTEA crypto, Zip parsing, registry access) must reside on the Rust side as non-blocking `async` command handlers.
  3. Single-threaded JS runtime vs multi-threaded Rust backend demands careful async coordination.

### 2) System Flow

```text
[Frontend UI Interaction] -> [Tauri IPC (invoke)] -> [Rust Backend commands.rs] -> [OS File System / Registry] -> [JSON Result Response] -> [Frontend State & UI Update]
```

1. **User Action:** The user drag-and-drops a mod file (e.g. `.rpkg` / `.zip`) onto the frontend drop zone or clicks a checkbox to toggle a mod.
2. **IPC Dispatch:** JavaScript calls `invoke('install_mod', ...)` or `invoke('toggle_mod', ...)`.
3. **Rust Validation & Backup:** The Rust core receives the call, locates the game path via stored settings, and performs a minimal backup of `packagedefinition.txt` to `.flmm_backup/` if it's the first install.
4. **Mod File Parsing & Target Assignment:** Rust unpacks zip archives and matches RPKG mod patch levels automatically, incrementing starting from 100 to avoid conflicts.
5. **Patching packagedefinition.txt:** The backend reads, decrypts (via XTEA), patches `patchlevel` entries, encrypts, and writes the updated `packagedefinition.txt` using Latin-1 encoding.
6. **UI Refresh:** The backend command returns success, and JavaScript displays a success toast and updates the mod list view.

### 3) Layer/Module Responsibilities

| Layer or module | Owns | Must not own | Evidence |
|-----------------|------|--------------|----------|
| Frontend Webview (`src/`) | UI Rendering, dynamic language translation (i18n), localStorage cache | Filesystem manipulation, game path discovery logic | [src/main.js](file:///e:/MOD/first-light-mod-manager/src/main.js) |
| Tauri Command Layer (`src-tauri/src/lib.rs`) | IPC Router registration for commands | Core domain logic, XTEA key declarations | [src-tauri/src/lib.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/lib.rs) |
| Backend Core Logic (`src-tauri/src/mod_manager.rs`, `src-tauri/src/settings.rs`, etc.) | Game path resolution, XTEA crypto encryption/decryption, metadata loading, zip extracting | HTML DOM manipulation, styling rules | [src-tauri/src/mod_manager.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs) |

### 4) Reused Patterns

| Pattern | Where found | Why it exists |
|---------|-------------|---------------|
| Client-Server Command Routing | [lib.rs:L6-L19](file:///e:/MOD/first-light-mod-manager/src-tauri/src/lib.rs#L6-L19) | Routes asynchronous IPC queries from frontend to corresponding Rust command functions |
| Checksum Validation (CRC32) | [crypto.rs:L85-L89](file:///e:/MOD/first-light-mod-manager/src-tauri/src/crypto.rs#L85-L89) | Used to verify integrity of `packagedefinition.txt` after decrypting and before saving |
| Path Normalization | [settings.rs:L86-L112](file:///e:/MOD/first-light-mod-manager/src-tauri/src/settings.rs#L86-L112) | Ensures user-input paths correctly map to game folder layout regardless of whether they selected `Runtime` or the root folder |

### 5) Known Architectural Risks

- **Destructive File Patching:** Since the mod manager edits `packagedefinition.txt`, a crash during file write could corrupt the game installation.
  - *Mitigation:* The application implements automatic backup to `.flmm_backup/packagedefinition.txt` before any mod installation and restores it completely on uninstall.
- **Race Conditions in State Updates:** Fast successive calls to `toggle_mod` might try to write to `packagedefinition.txt` concurrently.
  - *Mitigation:* Tauri commands are spawned in async tasks, but filesystem access is sequentialized through file handles.

### 6) Evidence

- [src-tauri/src/lib.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/lib.rs)
- [src-tauri/src/mod_manager.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs)
- [src-tauri/src/settings.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/settings.rs)
- [src/main.js](file:///e:/MOD/first-light-mod-manager/src/main.js)
