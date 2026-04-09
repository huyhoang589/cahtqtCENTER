# Phase 5 — Integration + Compile Verify

## Context Links
- All prior phase files in this plan
- Build commands: `cargo build`, `npm run lint`
- Code standards: `docs/code-standards.md`

## Overview
- **Priority:** P1
- **Status:** Complete
- **Description:** Verify full integration — both Rust backend and React frontend compile cleanly. Ensure all new modules are properly wired together.

## Key Insights
- Must verify: Cargo.toml deps resolve, all `mod` declarations correct, Tauri command registration complete, TS types match Rust serialization
- Common failure points: serde field name mismatches between Rust (snake_case) and TS (camelCase), missing command registrations

## Requirements

### Functional
- `cargo build` succeeds with zero errors
- `npm run lint` passes (or only pre-existing warnings)
- All 3 Tauri commands registered and callable from frontend

### Non-functional
- No new clippy warnings from license_gen code

## Implementation Steps

1. Run `cargo build` — fix any Rust compilation errors
2. Run `cargo clippy -- -D warnings` — address any new warnings
3. Run `npm run lint` — fix TypeScript errors
4. Verify command registration: check all 3 commands listed in `lib.rs` `invoke_handler`
5. Verify module declarations:
   - `lib.rs`: `pub mod license_gen;`
   - `commands/mod.rs`: `pub mod license_gen;`
   - `db/mod.rs`: `pub mod license_audit_repo;`
6. Verify Tauri serde compatibility:
   - Rust `CredentialPreview` fields serialize as camelCase (Tauri default)
   - TS `CredentialPreview` interface field names match
7. Verify DB migration: fresh DB creates `license_audit` table (user_version=5)
8. Create `DATA/LICENSE` output subdirectory in `initialize_data_directories` (lib.rs)

## Todo List
- [x] `cargo build` — zero errors
- [x] `cargo clippy` — zero new warnings
- [x] `npm run lint` — passes
- [x] Verify 3 commands in invoke_handler
- [x] Verify all module declarations
- [x] Verify serde field name compatibility
- [x] Add LICENSE output subdirectory to init
- [x] Update `src-tauri/src/lib.rs` `initialize_data_directories` to include `data.join("LICENSE")`

## Success Criteria
- Both `cargo build` and `npm run lint` succeed
- Application launches without errors
- License Gen page renders in sidebar and navigates correctly

## Risk Assessment
- **LOW**: Standard integration phase. Most issues caught by compiler.

## Next Steps
- After compile verify: manual end-to-end test with real token
- Consider adding unit tests for `compute_machine_fp` and `validate_credential`
- Update `docs/system-architecture.md` and `docs/project-changelog.md`
