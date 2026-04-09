---
title: "License Change v1 — UI Split, Path Change, DB Export/Delete"
description: "Three incremental changes to license gen: split preview cards, new output path, DB blob storage with export/delete"
status: complete
priority: P1
effort: 3h
branch: feature/licenseGen
tags: [feature, backend, frontend, database]
blockedBy: []
blocks: []
created: 2026-04-07
---

# License Change v1

## Overview

Three changes to the license gen feature per `feature/3. licenseGen/3. license_change_v1.txt`:

1. **UI**: Split credential preview into 2 cards (left=credential, right=license payload), black text, unmask token serial
2. **Path**: Output to `{output_data_dir}\SF\LICENSE\{User_name}\` instead of `{output_data_dir}\LICENSE\`
3. **DB**: Store license blob in `license_audit`, add export & delete commands + frontend buttons

**Brainstorm:** `plans/reports/brainstorm-260407-2058-license-change-v1.md`
**Parent plan:** `plans/260407-0833-license-gen-page/` (complete)

## Phases

| # | Phase | Status | Effort | File |
|---|-------|--------|--------|------|
| 1 | [DB Migration + Repo Changes](./phase-01-db-migration-and-repo.md) | Complete | 0.5h | Backend |
| 2 | [Path Change + Blob Storage](./phase-02-path-change-and-blob-storage.md) | Complete | 1h | Backend |
| 3 | [Export & Delete Commands](./phase-03-export-and-delete-commands.md) | Complete | 0.5h | Backend |
| 4 | [Frontend UI Changes](./phase-04-frontend-ui-changes.md) | Complete | 1h | Frontend |

## Dependencies

- Phase 1 → Phase 2 → Phase 3 (sequential backend)
- Phase 3 → Phase 4 (frontend needs new commands)
- No new crate dependencies

## Key Decisions (from brainstorm)

- Store blob as `TEXT` column on existing `license_audit` table (nullable for backward compat)
- Hard delete (DB row + disk file)
- Keep `{userName}-license.dat` filename
- Left card: credential fields; Right card: payload fields (populated after generation)

## Validation Log

### Session 1 — 2026-04-07
**Trigger:** Initial plan validation before implementation
**Questions asked:** 4

#### Questions & Answers

1. **[Architecture]** The plan uses hard delete (removes DB row + disk file permanently). Should delete be reversible?
   - Options: Hard delete (Recommended) | Soft delete
   - **Answer:** Hard delete with prompt and confirmation before delete
   - **Custom input:** Hard delete but prompt and need confirmation before delete
   - **Rationale:** Keeps implementation simple. Confirmation already planned via `window.confirm()`.

2. **[Scope]** Export writes the license file to the standard output path (SF\LICENSE\{User_name}\). Should the user be able to choose a custom save location?
   - Options: Fixed path (Recommended) | Save dialog
   - **Answer:** Fixed path + add "Open Folder" button
   - **Custom input:** Fixed path and add button Open Folder to open the LICENSE folder
   - **Rationale:** Keeps export simple. Open Folder button improves discoverability without adding save-dialog complexity.

3. **[Architecture]** The safe_name sanitization logic is duplicated in generate_license and export_license. Should we extract it?
   - Options: Extract helper (Recommended) | Keep inline
   - **Answer:** Extract helper
   - **Rationale:** DRY — prevents divergence between generate and export path logic.

4. **[UI]** Delete confirmation uses browser-native window.confirm(). Should we use a styled dialog instead?
   - Options: window.confirm (Recommended) | Custom modal dialog
   - **Answer:** window.confirm
   - **Rationale:** MVP scope, simple and accessible.

#### Confirmed Decisions
- **Delete strategy**: Hard delete with `window.confirm()` confirmation — simple, irreversible
- **Export path**: Fixed standard path, add "Open Folder" button to open LICENSE directory
- **Sanitization**: Extract shared `sanitize_user_name()` helper function
- **Delete dialog**: Use `window.confirm()` — no custom modal needed

#### Action Items
- [x] Extract `sanitize_user_name()` helper in backend (Phase 2)
- [x] Add "Open Folder" button to export flow in frontend (Phase 4)
- [x] Ensure delete confirmation prompt exists (Phase 4 — already planned)

#### Impact on Phases
- Phase 2: Extract `sanitize_user_name()` into a shared helper function in `license_gen` module
- Phase 3: Use `sanitize_user_name()` helper in `export_license` command
- Phase 4: Add "Open Folder" button next to Export button in audit history table
