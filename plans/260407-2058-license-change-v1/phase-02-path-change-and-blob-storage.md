# Phase 2: Path Change + Blob Storage

## Context
- [Brainstorm](../../plans/reports/brainstorm-260407-2058-license-change-v1.md)
- [generate_license command](../../src-tauri/src/commands/license_gen.rs) — lines 79-218
- [output_dir.rs](../../src-tauri/src/output_dir.rs)
- [signer.rs](../../src-tauri/src/license_gen/signer.rs)

## Overview
- **Priority:** P1
- **Status:** Complete
- Change output path from `{base}\LICENSE` to `{base}\SF\LICENSE\{User_name}`
- Save license blob to DB after generation

## Related Code Files

### Modify
- `src-tauri/src/commands/license_gen.rs` — `generate_license` command

## Implementation Steps

### 1. Extract `sanitize_user_name()` helper
<!-- Updated: Validation Session 1 - Extract shared sanitize_user_name() helper -->

Add a shared helper function in `license_gen.rs` (or a utility module):
```rust
/// Sanitize user name for safe filesystem path usage
pub fn sanitize_user_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
```

### 2. Change output path in `generate_license`

Current (line 117-121):
```rust
let output_dir = crate::output_dir::resolve_output_dir(&state.db, None, "LICENSE").await?;
```

Change to:
```rust
let safe_name = sanitize_user_name(&credential.user_name);
let sub_path = format!("SF\\LICENSE\\{}", safe_name);
let output_dir = crate::output_dir::resolve_output_dir(&state.db, None, &sub_path).await?;
```

This reuses `resolve_output_dir` with the new sub_path. The `SF\LICENSE\{User_name}` folder is auto-created by `create_dir_all`.

### 3. Save license blob to DB after generation

After the `spawn_blocking` block returns `(output_path, server_serial, payload_data)`, also capture `license_content` from the blocking thread.

**Modify spawn_blocking return type** to include the blob string:
```rust
let (output_path, server_serial, payload_data, license_content) = ...
```

The `license_content` is the Base64 string from `assemble_license_dat()` — already available inside the blocking closure.

**After audit insert**, save blob:
```rust
if let Err(e) = license_audit_repo::update_license_blob(&state.db, &audit_id, &license_content).await {
    eprintln!("WARNING: blob save failed: {}", e);
}
```

**Note:** `insert_audit` needs to return the `id` so we can call `update_license_blob`. Change `insert_audit` to return `Result<String, sqlx::Error>` (return the UUID).

### 4. Update `insert_audit` return type

In `license_audit_repo.rs`, change `insert_audit` to return the generated `id`:
```rust
pub async fn insert_audit(...) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    // ... insert ...
    Ok(id)
}
```

Then in `generate_license` command, use the returned `id` to save blob.

## Todo
- [x] Extract `sanitize_user_name()` helper function
- [x] Change `resolve_output_dir` sub_path to `SF\LICENSE\{safe_name}` using helper
- [x] Add `license_content` to spawn_blocking return tuple
- [x] Change `insert_audit` to return `Result<String>`
- [x] Call `update_license_blob` after audit insert with returned id
- [x] Run `cargo check`

## Success Criteria
- License files written to `{output_data_dir}\SF\LICENSE\{User_name}\{userName}-license.dat`
- `license_blob` column populated for new generations
- Existing audit rows unaffected (NULL blob)
