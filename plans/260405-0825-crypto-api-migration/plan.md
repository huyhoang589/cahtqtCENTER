---
title: Crypto API Migration — htqt-api.h v2
status: complete
branch: feature/crypto.api.sf
created: 2026-04-05
brainstorm: plans/reports/brainstorm-260405-0825-crypto-api-migration.md
---

# Crypto API Migration

Adapt Rust FFI layer to new `htqt-api.h`:
- `encHTQT_multi` → `encHTQT_sf_multi` (SF v1 format, per-file results)
- `decHTQT_v2` (per-file) → `decHTQT_sf` (batch params struct)

## Phases

| # | Phase | Status | File(s) |
|---|-------|--------|---------|
| 1 | [FFI Types + Loader](phase-01-ffi-types-and-loader.md) | complete | `htqt_ffi/types.rs`, `htqt_ffi/lib_loader.rs` |
| 2 | [Encrypt Command](phase-02-encrypt-command.md) | complete | `commands/encrypt.rs` |
| 3 | [Decrypt Command](phase-03-decrypt-command.md) | complete | `commands/decrypt.rs` |

## Key Dependencies

- Phase 1 must complete before Phases 2 & 3 (types/symbols change)
- Phases 2 & 3 can run in parallel after Phase 1

## Success Criteria

- `cargo build` clean, no warnings
- `encHTQT_sf_multi` + `decHTQT_sf` symbols resolve at runtime
- Encrypt: M results (not M×N), `.sf1` output
- Decrypt: batch call, output path from `BatchResult.output_path`

## Validation Log

### Session 1 — 2026-04-05
**Trigger:** Initial plan validation before implementation
**Questions asked:** 4

#### Questions & Answers

1. **[Architecture]** Phase 1 renames the internal field to `enc_sf_multi_fn` but keeps the public method name as `enc_multi()`. Should the public method be renamed to `enc_sf_multi()` to match DLL symbol naming?
   - Options: Keep enc_multi() | Rename to enc_sf_multi()
   - **Answer:** Keep enc_multi()
   - **Rationale:** No call-site changes needed in encrypt.rs; plan already written assuming this. Internal rename only.

2. **[Scope]** Phase 2 is ambiguous about `total_pairs`: keep it for the >10k batch warning, or remove it entirely?
   - Options: Keep for warning only | Remove entirely
   - **Answer:** Remove entirely
   - **Rationale:** Simplifies code; the >10k warning log is dropped. Phase 2 todo updated accordingly.

3. **[Assumptions]** In Phase 3, if `cert_cn` is None (token not fully initialized), the plan uses `unwrap_or_default()` — decrypt proceeds with empty cert_cn in logs. Is this the correct behavior?
   - Options: Proceed with empty log | Fail early if None
   - **Answer:** Proceed with empty log
   - **Rationale:** cert_cn is logging-only; fingerprint matching handled via own_cert_der in CryptoCallbacksV2, so None is non-fatal.

4. **[Assumptions]** Phase 3 uses the file stem as `file_id` in FileEntry for result tracking. Is this correct, or should file_id be the full input path or an index?
   - Options: Stem | Full input path | Numeric index string
   - **Answer:** Stem
   - **Rationale:** Short, human-readable, consistent with brainstorm intent.

#### Confirmed Decisions
- `enc_multi()` method name: keep unchanged — internal field/type rename only
- `total_pairs`: remove entirely — no warning log retained
- `cert_cn` None handling: non-fatal, proceed with empty string in logs
- `file_id` in FileEntry: use file stem

#### Action Items
- [x] Update Phase 2 todo: remove `total_pairs` entirely (not keep for warning)

#### Impact on Phases
- Phase 2: Remove `total_pairs` variable entirely — drop the >10k warning guard too
