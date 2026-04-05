# Phase 4 — Cleanup + Compile Verify

**Context:** `src-tauri/Cargo.toml` | all modified files
**Depends on:** Phases 1–3 complete

## Overview
- **Priority:** P2
- **Status:** Complete
- **Effort:** 0.5h
- Remove `rand` crate from Cargo.toml, verify full `cargo build` passes, confirm no dead code warnings.

## Related Code Files
- **Modify:** `src-tauri/Cargo.toml`
- **Verify:** all 4 modified files compile cleanly

## Implementation Steps

1. Before removing `rand`, verify it's not used transitively by any other crate: <!-- Updated: Validation Session 1 - must verify with cargo tree before removal -->
   ```bash
   cd src-tauri && cargo tree | grep rand
   ```
   If only `htqt_crypto` / `cahtqt-center` appears in the output (no other crate depending on rand), proceed with removal. Otherwise keep it.

2. Remove `rand = "0.8"` line from `[dependencies]` in `Cargo.toml` (if step 1 confirms safe)
   - Verify no other file imports `rand` after callbacks rewrite: `grep -r "use rand" src-tauri/src/`

3. Run `cargo build` from `src-tauri/`:
   ```bash
   cd src-tauri && cargo build 2>&1
   ```
   Expected: 0 errors. Warnings about unused imports are acceptable but should be fixed.

4. Fix any leftover unused import warnings:
   - `rsa::sha2::Sha256` — should be removed in Phase 3
   - `rand::thread_rng` — removed with `rand` dep
   - Any other dead code from removed batch types

5. Verify `BatchSfDecryptParams` + `FnDecHTQTSf` have zero references:
   ```bash
   grep -r "BatchSfDecryptParams\|FnDecHTQTSf\|dec_sf\b" src-tauri/src/
   ```
   Expected: no matches.

6. Verify `decrypt_one_sfv1` is referenced correctly:
   ```bash
   grep -r "decrypt_one_sfv1" src-tauri/src/
   ```
   Expected: in `types.rs`, `lib_loader.rs`, `commands/decrypt.rs`.

## Todo List
- [x] `cargo tree | grep rand` — confirm rand safe to remove
- [x] Remove `rand = "0.8"` from Cargo.toml (if confirmed unused)
- [x] Grep confirm no remaining `use rand` in source
- [x] `cargo build` — 0 errors
- [x] Fix any unused import warnings from Phase 3 cleanup
- [x] Grep confirm zero references to `BatchSfDecryptParams`, `FnDecHTQTSf`, `dec_sf`
- [x] Grep confirm `decrypt_one_sfv1` referenced in types/lib_loader/decrypt

## Success Criteria
- `cargo build` exits 0
- No dead code or unused import warnings related to changes
- `rand` crate removed from dependency tree

## Risk Assessment
- Low risk — compile errors surface any missed references immediately
- If `rand` still referenced elsewhere: keep it, only remove if truly unused
