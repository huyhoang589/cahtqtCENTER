# Phase 1: DB Migration + Repo Changes

## Context
- [Brainstorm](../../plans/reports/brainstorm-260407-2058-license-change-v1.md)
- [Current audit repo](../../src-tauri/src/db/license_audit_repo.rs)
- [Migration 005](../../src-tauri/migrations/005_license_audit.sql)

## Overview
- **Priority:** P1
- **Status:** Complete
- Add `license_blob TEXT` column to `license_audit` table
- Add repo functions for get-by-id, delete, and update-blob

## Related Code Files

### Modify
- `src-tauri/migrations/006_license_audit_blob.sql` (create)
- `src-tauri/src/db/license_audit_repo.rs`

## Implementation Steps

### 1. Create migration `006_license_audit_blob.sql`
```sql
ALTER TABLE license_audit ADD COLUMN license_blob TEXT;
```

### 2. Add repo functions to `license_audit_repo.rs`

**a) `get_audit_by_id(pool, id) -> Option<LicenseAuditRow>`**
- SELECT all columns + `license_blob` WHERE id = ?
- Update `LicenseAuditRow` struct to include `license_blob: Option<String>`

**b) `update_license_blob(pool, id, blob) -> Result<()>`**
- `UPDATE license_audit SET license_blob = ? WHERE id = ?`

**c) `delete_audit(pool, id) -> Result<()>`**
- `DELETE FROM license_audit WHERE id = ?`

### 3. Update `LicenseAuditRow` struct
Add field: `pub license_blob: Option<String>`

### 4. Update `list_audit` query
Add `license_blob` to SELECT columns in `list_audit()` function.

## Todo
- [x] Create `006_license_audit_blob.sql` migration
- [x] Add `license_blob` field to `LicenseAuditRow`
- [x] Add `get_audit_by_id()` function
- [x] Add `update_license_blob()` function
- [x] Add `delete_audit()` function
- [x] Update `list_audit()` SELECT to include `license_blob`
- [x] Run `cargo check`

## Success Criteria
- Migration applies cleanly
- Existing rows have `license_blob = NULL`
- All new repo functions compile
