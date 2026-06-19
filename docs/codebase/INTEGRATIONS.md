# External Integrations

## Core Sections (Required)

### 1) Integration Inventory

No external network integrations or third-party APIs are utilized by the application.

### 2) Data Stores

| Store | Role | Access layer | Key risk | Evidence |
|-------|------|--------------|----------|----------|
| `settings.json` | Persistent user configurations (game path, language) | Backend (reads and writes JSON) | Corruption or deletion of user preferences | [settings.rs:L62-L83](file:///e:/MOD/first-light-mod-manager/src-tauri/src/settings.rs#L62-L83) |
| `localStorage` | UI caching for fast settings bootstrap | Frontend (syncs settings from JS inputs) | Desynchronization with Rust settings | [main.js:L225-L230](file:///e:/MOD/first-light-mod-manager/src/main.js#L225-L230) |

### 3) Secrets and Credentials Handling

- Credential sources: None. No API keys or credentials are required or saved.
- Hardcoding checks: No secrets, credentials, or development keys are hardcoded in the codebase.
- Lifecycle: N/A.

### 4) Reliability and Failure Behavior

- Retry/backoff behavior: N/A.
- Timeout policy: N/A.
- Fallback behavior: N/A.

### 5) Observability for Integrations

- Logging: N/A.
- Metrics/tracing coverage: None.
- Missing visibility gaps: None.

### 6) Evidence

- [src-tauri/src/settings.rs](file:///e:/MOD/first-light-mod-manager/src-tauri/src/settings.rs)
- [src/main.js](file:///e:/MOD/first-light-mod-manager/src/main.js)
