---
phase: 1
title: "Implement Validation Improvements"
status: complete
priority: P2
effort: 0.5h
completedDate: 2026-04-07
---

# Phase 1 — Implement Validation Improvements

## Context

- [Brainstorm Report](../reports/brainstorm-260407-1833-license-gen-validation.md)
- [License Gen Plan](../260407-0833-license-gen-page/plan.md) (complete)

## Overview

Three targeted validation additions to existing license gen code. No new files, no architectural changes.

## Related Code Files

**Modify:**
- `src-tauri/src/license_gen/mod.rs` — add `registered_at` date validation (chrono)
- `src-tauri/src/commands/license_gen.rs` — add `expires_at` future check
<!-- Updated: Validation Session 1 - Removed file size cap from scope -->

## Implementation Steps

### 1. Add `registered_at` date validation (`mod.rs`)
<!-- Updated: Validation Session 1 - Use chrono instead of manual parse -->

In `validate_credential()`, add after the existing loop:

```rust
// Validate registered_at is a valid YYYY-MM-DD date
let date_str = cred.registered_at.trim();
if date_str.is_empty() {
    return Err("Invalid registered_at: empty".to_string());
}
if chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err() {
    return Err(format!("Invalid registered_at: '{}' is not a valid YYYY-MM-DD date", date_str));
}
```

**Note:** `chrono` is already in `Cargo.toml` — no new dependency needed.

### 2. Add `expires_at` future check (`commands/license_gen.rs`)

In `generate_license()`, after `validate_credential()`:

```rust
// Validate expires_at is in the future (if provided)
if let Some(exp) = expires_at {
    let now = crate::db::now_secs();
    if exp <= now {
        return Err("Expiry date must be in the future".to_string());
    }
}
```

### 3. Compile check

```bash
cd src-tauri && cargo check
```

## Todo List

- [x] Add `registered_at` date format validation in `mod.rs` (using chrono)
- [x] Add `expires_at` future-date check in `generate_license` command (strict, no buffer)
- [x] Run `cargo check` — no compile errors
<!-- Updated: Validation Session 1 - Removed file size cap todo -->
<!-- Completed: 2026-04-07 - All validation checks implemented and tested -->

## Success Criteria

- Importing a credential with `registered_at: "not-a-date"` returns validation error
- Generating license with past `expires_at` returns error
- All existing functionality unchanged
- Project compiles cleanly

## Risk Assessment

- **Low risk:** Additive validation only, no breaking changes
- **Dependency:** Check if `chrono` exists; if not, use manual date parse to avoid new dep
