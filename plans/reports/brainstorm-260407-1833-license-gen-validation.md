# Brainstorm: License Gen Input Validation Improvements

**Date:** 2026-04-07
**Focus:** Strengthen credential & license gen input validation

## Problem Statement

License gen accepts machine credential JSON from client machines. Current validation covers non-empty/placeholder checks but misses: `registered_at` date format, `expires_at` future check, file size limit, and multi-error reporting.

## Context

- CAHTQT CENTER (this app) = server-side license issuer
- CLIENT Registration Tool generates `machine_credential.json` on client machine
- Transfer method: USB or network (both possible)
- Credential fields: `board_serial`, `cpu_id`, `token_serial`, `user_name`, `registered_at`

## Evaluated Approaches

### A. Strict Format Validation (Rejected)
- Enforce regex patterns per field (hex-16 for cpu_id, etc.)
- **Rejected:** User wants KISS — just verify completeness, not format

### B. Simple Completeness Validation (Selected)
- Verify all fields non-empty, trimmed, not placeholder
- Add date format check for `registered_at`
- Add future-date check for `expires_at`
- Add file size cap (10KB)

### C. Credential Integrity (HMAC/Signature) (Deferred)
- Sign credential at Registration Tool, verify at CENTER
- **Deferred:** Not prioritized in this iteration

## Final Design

| Field | Rule |
|-------|------|
| cpu_id | Non-empty, trimmed, not placeholder |
| board_serial | Non-empty, trimmed, not placeholder |
| token_serial | Non-empty, trimmed, not placeholder |
| user_name | Non-empty, trimmed, not placeholder |
| registered_at | Non-empty, valid YYYY-MM-DD |
| expires_at | If provided, must be in the future |
| File | Max 10KB before parsing |

## Files to Modify

- `src-tauri/src/license_gen/mod.rs` — add registered_at date validation
- `src-tauri/src/commands/license_gen.rs` — file size check, expires_at future check

## Risks

- None significant. Changes are additive validation only.

## Next Steps

- Create implementation plan → implement → compile verify
