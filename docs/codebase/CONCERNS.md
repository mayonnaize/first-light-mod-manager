# Codebase Concerns

## Core Sections (Required)

### 1) Top Risks (Prioritized)

| Severity | Concern | Evidence | Impact | Suggested action |
|----------|---------|----------|--------|------------------|
| High | **packagedefinition.txt corruption** | [crypto.rs:L143-L157](file:///e:/MOD/first-light-mod-manager/src-tauri/src/crypto.rs#L143-L157) | If the app crashes midway during encryption/decryption write, it will break game boot. | Restructure to write to a temp file and swap atomically. |

### 2) Technical Debt

| Debt item | Why it exists | Where | Risk if ignored | Suggested fix |
|-----------|---------------|-------|-----------------|---------------|
| Co-located tests | Initial speed of setup | [mod_manager.rs:L700-L974](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs#L700-L974) | Increased compilation times, harder to browse core commands | Move helper integration tests to `src-tauri/tests/` |
| Duplicate Settings Models | Dynamic synchronization and i18n support | [main.js:L225-L230](file:///e:/MOD/first-light-mod-manager/src/main.js#L225-L230) vs [settings.rs:L7-L10](file:///e:/MOD/first-light-mod-manager/src-tauri/src/settings.rs#L7-L10) | Risk of drift between Rust structs and JS state structures | Generate TS/JS definitions from Rust models using a schema compiler. |

### 3) Security Concerns

| Risk | OWASP category (if applicable) | Evidence | Current mitigation | Gap |
|------|--------------------------------|----------|--------------------|-----|
| Arbitrary Mod File Extraction | A03:2021-Injection | [mod_manager.rs:L303-L356](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs#L303-L356) | Validates `.zip` contents by matching path formats and checks extensions. | Path traversal vulnerability (Zip Slip) checks are not explicitly defensive. |

### 4) Performance and Scaling Concerns

| Concern | Evidence | Current symptom | Scaling risk | Suggested improvement |
|---------|----------|-----------------|-------------|-----------------------|
| Mod list latency | [mod_manager.rs:L610-L659](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs#L610-L659) | Loops and scans the entire game Runtime directory every time list is refreshed | Scanning delays when managing hundreds of mod files | Cache mod directories and parse changes on file watcher events. |

### 5) Fragile/High-Churn Areas

| Area | Why fragile | Churn signal | Safe change strategy |
|------|-------------|-------------|----------------------|
| [src-tauri/src/mod_manager.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs) | Contains core mod management operations, zip extraction, metadata parsing, and local settings interactions. | Git log churn count of 11 commits in the last 90 days. | Write standalone unit tests before touching filesystem APIs. |
| [src/main.js](file:///e:/MOD/first-light-mod-manager/src/main.js) | Handles UI routing, translations, custom drag and drop event bindings, and toast notifications. | Git log churn count of 11 commits in the last 90 days. | Ensure all UI changes are validated in headless Playwright tests. |

### 6) `[ASK USER]` Questions

1. [ASK USER] None. All previous design questions have been resolved (Nexus Mods API check removed and `commands.rs` modularized).

### 7) Evidence

- [docs/codebase/.codebase-scan.txt](file:///e:/MOD/first-light-mod-manager/docs/codebase/.codebase-scan.txt) (churn statistics)
- [src-tauri/src/mod_manager.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/mod_manager.rs)
