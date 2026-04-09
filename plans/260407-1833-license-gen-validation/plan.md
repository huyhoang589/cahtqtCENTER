---
title: "License Gen Input Validation Improvements"
description: "Strengthen credential import & license generation validation: date format, expires_at future check"
status: complete
priority: P2
effort: 0.5h
branch: feature/licenseGen
tags: [security, validation, backend]
blockedBy: []
blocks: []
created: 2026-04-07
completedDate: 2026-04-07
---

# License Gen Input Validation Improvements

## Overview

Harden license gen input validation per brainstorm report. Three targeted changes, ~20 lines total. No new files.

**Brainstorm:** `plans/reports/brainstorm-260407-1833-license-gen-validation.md`
**Parent feature:** `plans/260407-0833-license-gen-page/` (complete)

## Phases

| # | Phase | Status | Effort | File |
|---|-------|--------|--------|------|
| 1 | Implement validation improvements | Complete | 0.5h | [phase-01](./phase-01-implement-validation.md) |

## Key Files

- `src-tauri/src/license_gen/mod.rs` — credential validation
- `src-tauri/src/commands/license_gen.rs` — command handlers

## Success Criteria

- [x] `registered_at` rejects non-YYYY-MM-DD strings (chrono validation)
- [x] `expires_at` in the past rejected at generation time (strict, no buffer)
- [x] Project compiles with `cargo check`

## Validation Log

### Session 1 — 2026-04-07
**Trigger:** Initial plan validation before implementation
**Questions asked:** 4

#### Questions & Answers

1. **[Architecture]** chrono is already in Cargo.toml. Should we use chrono::NaiveDate::parse_from_str for registered_at validation instead of the manual parser?
   - Options: Use chrono (Recommended) | Manual parse
   - **Answer:** Use chrono
   - **Rationale:** chrono already a dependency; gives proper date validation including month/day range checks (rejects Feb 30 etc.)

2. **[Scope]** The plan sets a 10KB file size cap for credential imports. Is this the right threshold?
   - Options: 10KB is fine (Recommended) | Raise to 50KB | Lower to 5KB
   - **Answer:** No file size cap needed
   - **Custom input:** no need to set file size cap
   - **Rationale:** User decided file size validation is unnecessary for this use case. Removes scope.

3. **[Assumptions]** expires_at is Option<i64> (Unix seconds). Should None (no expiry) be allowed, or should every license require an expiry date?
   - Options: Allow None (Recommended) | Require expiry always
   - **Answer:** Allow None
   - **Rationale:** Keep current behavior — None means perpetual license. Only validate when Some.

4. **[Architecture]** Should the expires_at future check use a grace buffer (e.g. allow 5 min in the past for clock skew)?
   - Options: No buffer — strict (Recommended) | 5-minute grace period
   - **Answer:** No buffer — strict
   - **Rationale:** Operator sets expiry via UI date picker, clock skew unlikely. Strict check is appropriate.

#### Confirmed Decisions
- Date validation: use `chrono::NaiveDate` — already a dependency, proper validation
- File size cap: **removed from scope** — not needed
- Expiry: allow `None` (perpetual), validate only when `Some`
- Clock skew: no grace buffer, strict future check

#### Action Items
- [ ] Remove file size cap from phase-01 implementation steps and todo list

#### Impact on Phases
- Phase 1: Remove step 2 (file size cap) entirely. Update todo list to remove file size item. Use chrono instead of manual parse for step 1.
