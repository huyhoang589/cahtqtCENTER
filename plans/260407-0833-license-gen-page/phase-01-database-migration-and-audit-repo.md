# Phase 1 — Database Migration + Audit Repo

## Context Links
- Spec: `feature/3. licenseGen/2F_Hardware_Bound_License_CAHTQT_Spec.docx` §3.4
- Existing pattern: `src-tauri/src/db/logs_repo.rs`, `src-tauri/src/db/mod.rs`
- Migration numbering: next is `005`

## Overview
- **Priority:** P1 (foundation for all other phases)
- **Status:** Complete
- **Description:** Create SQLite migration for `license_audit` table and repository module for CRUD operations.

## Key Insights
- Existing migration runner uses `PRAGMA user_version` with incremental version numbers
- Repo pattern: standalone async functions taking `&Pool<Sqlite>`, returning `Result<T, sqlx::Error>`
- UUID v4 for primary keys (consistent with logs_repo)
- `chrono::Utc::now().timestamp()` for timestamps via `db::now_secs()`

## Requirements

### Functional
- `license_audit` table stores all license issuance records
- Support insert (on generation) and list (for history table on page)
- Fields from spec §3.4: timestamp, server_serial, user_name, unit_name, token_serial, machine_fp, expires_at, product

### Non-functional
- Idempotent migration (IF NOT EXISTS)
- Index on `created_at DESC` for efficient history queries

## Architecture

```
license_audit table schema:
  id            TEXT PRIMARY KEY  (UUID v4)
  server_serial TEXT NOT NULL     (server token serial used for signing)
  user_name     TEXT NOT NULL     (CN from client cert)
  unit_name     TEXT NOT NULL     (organizational unit)
  token_serial  TEXT NOT NULL     (client Bit4ID token serial)
  machine_fp    TEXT NOT NULL     (16-char hex fingerprint)
  cpu_id        TEXT NOT NULL     (raw CPU processor ID)
  board_serial  TEXT NOT NULL     (raw motherboard serial)
  product       TEXT NOT NULL     (e.g. "CAHTQT_CLIENT")
  expires_at    INTEGER           (Unix timestamp, NULL = perpetual)
  output_path   TEXT NOT NULL     (path where license.dat was written)
  created_at    INTEGER NOT NULL  (Unix timestamp)
```

## Related Code Files

### Create
- `src-tauri/migrations/005_license_audit.sql`
- `src-tauri/src/db/license_audit_repo.rs`

### Modify
- `src-tauri/src/db/mod.rs` — add `pub mod license_audit_repo;` + migration block

## Implementation Steps

1. Create `src-tauri/migrations/005_license_audit.sql`:
   ```sql
   CREATE TABLE IF NOT EXISTS license_audit (
       id            TEXT PRIMARY KEY,
       server_serial TEXT NOT NULL,
       user_name     TEXT NOT NULL,
       unit_name     TEXT NOT NULL,
       token_serial  TEXT NOT NULL,
       machine_fp    TEXT NOT NULL,
       cpu_id        TEXT NOT NULL,
       board_serial  TEXT NOT NULL,
       product       TEXT NOT NULL,
       expires_at    INTEGER,
       output_path   TEXT NOT NULL,
       created_at    INTEGER NOT NULL
   );
   CREATE INDEX IF NOT EXISTS idx_license_audit_created_at ON license_audit(created_at DESC);
   ```

2. Add migration block in `src-tauri/src/db/mod.rs` — add `if version < 5` block following existing pattern:
   ```rust
   if version < 5 {
       let sql = include_str!("../../migrations/005_license_audit.sql");
       let stmts: Vec<&str> = sql.split(';').collect();
       run_migration(pool, &stmts, 5).await.map_err(|e| e)?;
   }
   ```

3. Add `pub mod license_audit_repo;` to `src-tauri/src/db/mod.rs`

4. Create `src-tauri/src/db/license_audit_repo.rs` with:
   - `LicenseAuditRow` struct (derive `Serialize`, `sqlx::FromRow`)
   - `insert_audit(pool, row_data) -> Result<(), sqlx::Error>`
   - `list_audit(pool, limit, offset) -> Result<Vec<LicenseAuditRow>, sqlx::Error>`

## Todo List
- [x] Create migration SQL file `005_license_audit.sql`
- [x] Add migration runner block in `db/mod.rs`
- [x] Add `pub mod license_audit_repo;` in `db/mod.rs`
- [x] Implement `LicenseAuditRow` struct
- [x] Implement `insert_audit` function
- [x] Implement `list_audit` function
- [x] Run `cargo build` to verify compilation

## Success Criteria
- `cargo build` succeeds with new migration + repo
- Migration creates table on fresh DB (user_version bumps to 5)
- Existing DBs (user_version=4) gain new table without data loss

## Risk Assessment
- **Low risk**: Standard migration pattern, well-established in codebase
- If migration fails on existing DB: `IF NOT EXISTS` makes it idempotent

## Security Considerations
- No sensitive data in audit table (token serials are public identifiers, not secrets)
- machine_fp is a hash, not raw hardware data (though cpu_id/board_serial are raw — acceptable for server-side audit)
