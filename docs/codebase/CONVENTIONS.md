# Coding Conventions

## Core Sections (Required)

### 1) Naming Rules

| Item | Rule | Example | Evidence |
|------|------|---------|----------|
| Files | `lower_snake_case` for Rust, `lowercase` / `kebab-case` for JS/configs | `commands.rs`, `main.js` | [scan.txt](file:///e:/MOD/first-light-mod-manager/docs/codebase/.codebase-scan.txt) |
| Functions/methods | `snake_case` in Rust, `camelCase` in JS | `toggle_mod`, `setModFile` | [mod_manager.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs), [main.js](file:///e:/MOD/first-light-mod-manager/src/main.js) |
| Types/interfaces | `PascalCase` in Rust | `AppSettings`, `ModInfo` | [settings.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/settings.rs), [mod_manager.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs) |
| Constants | `UPPER_SNAKE_CASE` in Rust and JS | `XTEA_KEYS`, `DISCORD_URL` | [crypto.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/crypto.rs), [main.js](file:///e:/MOD/first-light-mod-manager/src/main.js) |

### 2) Formatting and Linting

- Formatter: `rustfmt` for Rust code. No custom formatter setup in package.json for JS.
- Linter: `cargo clippy` for Rust. No custom linter setup for JS.
- Run commands:
  ```bash
  cargo fmt --manifest-path src-tauri/Cargo.toml --check
  cargo clippy --manifest-path src-tauri/Cargo.toml
  ```

### 3) Import and Module Conventions

- JavaScript: Direct `const { ... } = window.__TAURI__...` for Tauri APIs. No external module bundler is used; imports are vanilla script elements.
- Rust: Explicit imports at the top of the file, grouping standard library imports first, followed by external crates.

### 4) Error and Logging Conventions

- Error strategy by layer:
  - **Backend:** Returns errors as `Result<T, String>` where the `Err` string is translated dynamically using the `localized` helper: `localized(is_pt, "Portuguese message", "English message")`.
  - **Frontend:** Catches rejected promises from Tauri commands and displays them directly to the user using the `toast(msg, 'error')` notification function.
- Sensitive-data redaction: None (external API integration is removed).

### 5) Testing Conventions

- Test file naming/location rule:
  - Rust tests are co-located in the target implementation file (e.g. `#[cfg(test)] mod tests` at the bottom of modular files like `src-tauri/src/mod_manager.rs` or `src-tauri/src/crypto.rs`).
  - Frontend E2E tests are located in `tests/e2e/` (e.g. `app.spec.js`).
  - Frontend pure JS unit tests are located in `tests/unit/` (e.g. `utils.spec.js`).
- Mocking strategy: E2E Playwright test suite mocks Tauri IPC backend commands in the browser page context to run tests without requiring a real game installation.
- Coverage expectation: Backend coverage is tracked via `cargo llvm-cov`. Frontend coverage is measured via Playwright v8 coverage with `monocart-reporter`.

### 6) Evidence

- [src-tauri/src/mod_manager.rs:L700](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs#L700)
- [src/main.js:L945](file:///e:/MOD/first-light-mod-manager/src/main.js#L945)
- [package.json](file:///e:/MOD/first-light-mod-manager/package.json)
