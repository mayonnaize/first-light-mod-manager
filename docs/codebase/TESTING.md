# Testing Patterns

## Core Sections (Required)

### 1) Test Stack and Commands

- Primary test framework: Cargo Test (Backend, Rust 1.x), Playwright (Frontend E2E, v1.60.0).
- Assertion/mocking tools: `std::assert` and `TempDir` helper in Rust, mock-Tauri backend scripts embedded inside Playwright browser context.
- Commands:
  ```bash
  # Run all tests (Rust tests + E2E tests)
  npm test

  # Run Rust backend unit tests only
  npm run test:rust

  # Run Playwright E2E and browser unit tests
  npm run test:e2e
  ```

### 2) Test Layout

- Test file placement:
  - **Backend:** Embedded inside modular files (e.g. `src-tauri/src/mod_manager.rs` or `src-tauri/src/crypto.rs` under `#[cfg(test)] mod tests`).
  - **Frontend E2E:** Located in `tests/e2e/` (e.g. `app.spec.js`).
  - **Frontend JS Unit:** Located in `tests/unit/` (e.g. `utils.spec.js`), which execute inside the Playwright browser page structure to test real frontend utilities.

### 3) Test Scope Matrix

| Scope | Covered? | Typical target | Notes |
|-------|----------|----------------|-------|
| Unit (Backend) | Yes | XTEA encryption, CRC32 calculations, Latin-1 decoding, patch level parser | Tested with modular data inputs and files |
| Integration (Backend) | Yes | packagedefinition.txt read/write, mod zip extract, path normalization | Utilizes custom `TempDir` struct to create and tear down simulated game folders |
| E2E (Frontend) | Yes | Settings save, mod toggle logic, drag and drop visual checks | Uses Playwright to mock Tauri's `invoke` IPC interface and validates DOM updates |

### 4) Mocking and Isolation Strategy

- Main mocking approach: Tauri IPC mocks via custom mock window injection (stubbing out backend handlers and dialog/opener helper structures).
- Isolation guarantees: The Rust integration tests instantiate a random timestamp-suffixed directory for each test run and delete it on `Drop`. E2E tests reset browser states and mock structures between test runs.

### 5) Coverage and Quality Signals

- Coverage tool: `cargo llvm-cov` (Rust), Playwright V8 Coverage with `monocart-reporter` (JavaScript).
- Current reported coverage: Monocart reporter coverage runs are saved under [test-results/report.html](file:///e:/MOD/first-light-mod-manager/test-results/report.html).

### 6) Evidence

- [package.json:L296-L298](file:///e:/MOD/first-light-mod-manager/package.json#L296-L298)
- [src-tauri/src/mod_manager.rs:L700](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs#L700)
- [playwright.config.js](file:///e:/MOD/first-light-mod-manager/playwright.config.js)
