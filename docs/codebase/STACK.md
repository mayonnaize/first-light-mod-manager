# Technology Stack

## Core Sections (Required)

### 1) Runtime Summary

| Area | Value | Evidence |
|------|-------|----------|
| Primary language | Rust (Backend), JavaScript (Frontend) | [Cargo.toml](file:///e:/MOD/first-light-mod-manager/src-tauri/Cargo.toml), [package.json](file:///e:/MOD/first-light-mod-manager/package.json) |
| Runtime + version | Rust (Edition 2021), Node.js (>=18) | [Cargo.toml](file:///e:/MOD/first-light-mod-manager/src-tauri/Cargo.toml), [package.json](file:///e:/MOD/first-light-mod-manager/package.json) |
| Package manager | cargo (Rust), npm (Frontend) | [Cargo.lock](file:///e:/MOD/first-light-mod-manager/src-tauri/Cargo.lock), [package-lock.json](file:///e:/MOD/first-light-mod-manager/package-lock.json) |
| Module/build system | Cargo, Tauri CLI v2 | [Cargo.toml](file:///e:/MOD/first-light-mod-manager/src-tauri/Cargo.toml), [package.json](file:///e:/MOD/first-light-mod-manager/package.json) |

### 2) Production Frameworks and Dependencies

List only high-impact production dependencies (frameworks, data, transport, auth).

| Dependency | Version | Role in system | Evidence |
|------------|---------|----------------|----------|
| `tauri` | `2.1.1` (Cargo) | Desktop app framework | [Cargo.toml](file:///e:/MOD/first-light-mod-manager/src-tauri/Cargo.toml) |
| `serde` | `1.0` (Cargo) | Serialization / Deserialization | [Cargo.toml](file:///e:/MOD/first-light-mod-manager/src-tauri/Cargo.toml) |
| `serde_json` | `1.0` (Cargo) | JSON metadata and settings handling | [Cargo.toml](file:///e:/MOD/first-light-mod-manager/src-tauri/Cargo.toml) |
| `zip` | `2.1` (Cargo) | Extracting and reading .zip mods | [Cargo.toml](file:///e:/MOD/first-light-mod-manager/src-tauri/Cargo.toml) |
| `winreg` | `0.52` (Cargo) | Windows Registry auto-detection (Steam/Epic) | [Cargo.toml](file:///e:/MOD/first-light-mod-manager/src-tauri/Cargo.toml) |

### 3) Development Toolchain

| Tool | Purpose | Evidence |
|------|---------|----------|
| `@tauri-apps/cli` | Tauri app desktop runner and bundler | [package.json](file:///e:/MOD/first-light-mod-manager/package.json) |
| `@playwright/test` | End-to-End browser/IPC tests | [package.json](file:///e:/MOD/first-light-mod-manager/package.json) |
| `monocart-reporter` | Test coverage and report generation | [package.json](file:///e:/MOD/first-light-mod-manager/package.json) |

### 4) Key Commands

```bash
# Install NPM dependencies
npm install

# Run backend Rust unit tests
npm run test:rust

# Run Playwright E2E tests
npm run test:e2e

# Run both Rust and E2E tests
npm test

# Build production executable
npm run tauri build
```

### 5) Environment and Config

- Config sources: `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`
- Required env vars: None
- Deployment/runtime constraints: Windows-centric (winreg registry discovery runs only on Windows)

### 6) Evidence

- [package.json](file:///e:/MOD/first-light-mod-manager/package.json)
- [Cargo.toml](file:///e:/MOD/first-light-mod-manager/src-tauri/Cargo.toml)
- [tauri.conf.json](file:///e:/MOD/first-light-mod-manager/src-tauri/tauri.conf.json)
