# Phase 3: Export & Delete Commands

## Context
- [Brainstorm](../../plans/reports/brainstorm-260407-2058-license-change-v1.md)
- [license_gen commands](../../src-tauri/src/commands/license_gen.rs)
- [license_audit_repo](../../src-tauri/src/db/license_audit_repo.rs)
- [lib.rs command registration](../../src-tauri/src/lib.rs) — line 151-153

## Overview
- **Priority:** P1
- **Status:** Complete
- Add `export_license` and `delete_license` Tauri commands
- Register in `lib.rs`

## Related Code Files

### Modify
- `src-tauri/src/commands/license_gen.rs` — add 2 commands
- `src-tauri/src/lib.rs` — register commands

## Implementation Steps

### 1. Add `export_license` command

```rust
#[tauri::command]
pub async fn export_license(
    audit_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 1. Fetch audit row by id
    let row = license_audit_repo::get_audit_by_id(&state.db, &audit_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Audit record not found")?;

    // 2. Check blob exists
    let blob = row.license_blob.ok_or("No license data stored for this record")?;

    // 3. Resolve output path: SF\LICENSE\{safe_name}
    <!-- Updated: Validation Session 1 - Use shared sanitize_user_name() helper -->
    let safe_name = sanitize_user_name(&row.user_name);
    let sub_path = format!("SF\\LICENSE\\{}", safe_name);
    let output_dir = crate::output_dir::resolve_output_dir(&state.db, None, &sub_path).await?;

    // 4. Write file
    let file_name = format!("{}-license.dat", safe_name);
    let path = std::path::Path::new(&output_dir).join(&file_name);
    tokio::fs::write(&path, &blob).await
        .map_err(|e| format!("Failed to write: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}
```

### 2. Add `delete_license` command

```rust
#[tauri::command]
pub async fn delete_license(
    audit_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 1. Fetch audit row to get output_path
    let row = license_audit_repo::get_audit_by_id(&state.db, &audit_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Audit record not found")?;

    // 2. Delete file from disk (ignore if already gone)
    let path = std::path::Path::new(&row.output_path);
    if path.exists() {
        tokio::fs::remove_file(path).await
            .map_err(|e| format!("Failed to delete file: {}", e))?;
    }

    // 3. Delete DB record
    license_audit_repo::delete_audit(&state.db, &audit_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

### 3. Register commands in `lib.rs`

Add to the `invoke_handler` at line ~151:
```rust
commands::license_gen::export_license,
commands::license_gen::delete_license,
```

## Todo
- [x] Add `export_license` command
- [x] Add `delete_license` command
- [x] Register both in `lib.rs` invoke_handler
- [x] Run `cargo check`

## Success Criteria
- `export_license` writes file from DB blob to correct path, returns path
- `delete_license` removes DB row + disk file
- Both commands registered and compile
