# Brainstorm Report: Crypto API Migration

**Date:** 2026-04-05  
**Branch:** feature/crypto.api.sf  
**Scope:** Adapt Rust FFI layer to new `htqt-api.h` (encHTQT_sf_multi + decHTQT_sf)

---

## Problem Statement

DLL API has been updated. Two breaking changes:
1. `encHTQT_multi` → `encHTQT_sf_multi` (SF v1 format, results size changes)
2. `decHTQT_v2(sf_path, output_path, recipient_id, ...)` → `decHTQT_sf(BatchSfDecryptParams*, ...)` (batch, DLL-driven output naming)

---

## Key API Differences

### Encrypt
| Aspect | Old | New |
|---|---|---|
| Symbol | `encHTQT_multi` | `encHTQT_sf_multi` |
| Output format | M×N `.sf` files | 1 `.sf1` per file (all recipients embedded) |
| Results capacity | `file_count * recip_count` | `file_count` |

### Decrypt
| Aspect | Old | New |
|---|---|---|
| Symbol | `decHTQT_v2` | `decHTQT_sf` |
| Call pattern | Per-file loop | Single batch call |
| Output path | Caller-specified | DLL from SF header `orig_name` |
| Recipient ID | Explicit string param | Fingerprint match via `own_cert_der` |

---

## Agreed Decisions

- **Enc progress**: Per-file events (M events, not M×N) — matches new results layout
- **Dec output logging**: Read `BatchResult.output_path` filled by DLL
- **Recipient ID**: Remove from `decrypt_batch` command signature

---

## Files to Change

| File | Change |
|---|---|
| `htqt_ffi/types.rs` | Add `BatchSfDecryptParams`; rename `FnDecHTQTV2` → `FnDecHTQTSf` with new sig |
| `htqt_ffi/lib_loader.rs` | New symbol names; `dec_v2()` → `dec_sf()` batch method; enc results capacity |
| `commands/encrypt.rs` | Results vec size = `file_count`; per-file progress loop |
| `commands/decrypt.rs` | Single batch call; remove `recipient_id`; log from `BatchResult.output_path` |

---

## Risks

| Risk | Mitigation |
|---|---|
| Old DLL missing new symbols | `HtqtLib::load()` errors early with clear message |
| `BatchResult.output_path` empty on failure | Fall back to `output_dir` in log |
| Output file name collisions | `HTQT_BATCH_OVERWRITE_OUTPUT` flag available |

---

## Unresolved Questions

- None
