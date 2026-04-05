# Project Changelog

**Project:** CAHTQT — PKI Encryption Desktop App  
**Format:** Semantic Versioning (MAJOR.MINOR.PATCH)  
**Current Version:** 2.0.0

All notable changes to this project are documented here. This file tracks features, bug fixes, and breaking changes.

## [Unreleased]

(No unreleased changes at this time)

---

## [2.0.0] — 2026-04-05

### Major Changes

#### Crypto API Migration: SF v1 Batch Format

**Status:** COMPLETE  
**Branch:** `feature/crypto.api.sf`  
**Impact:** Breaking change to encryption/decryption workflows

**Summary:**
Migrated from per-file crypto APIs to batch SF v1 format APIs. This change improves performance, reduces token session overhead, and aligns with the new DLL specification.

**Breaking Changes:**

1. **Encryption API Update**
   - Old: `encHTQT_multi()` produced M×N result entries, one per (file, recipient) pair
   - New: `encHTQT_sf_multi()` produces M result entries, one per file
   - Each output file embeds all N recipient blocks (SF v1 multi-recipient format)
   - **Impact:** Results array now sized to `file_count` (not `file_count × recipient_count`)

2. **Decryption API Update**
   - Old: `decHTQT_v2()` per-file function with individual output path
   - New: `decHTQT_sf()` batch function accepting `BatchSfDecryptParams` struct
   - Output filenames now come from SF header `orig_name` field (not input filename)
   - **Impact:** Single batch call replaces per-file loops

3. **FFI Layer Updates**
   - Renamed function pointer types: `FnEncHTQTMulti` → `FnEncHTQTSfMulti`, `FnDecHTQTV2` → `FnDecHTQTSf`
   - Added `BatchSfDecryptParams` struct for decrypt parameters
   - Updated `HtqtLib` fields: `enc_multi_fn` → `enc_sf_multi_fn`, `dec_v2_fn` → `dec_sf_fn`
   - Updated symbol resolution: `encHTQT_multi` → `encHTQT_sf_multi`, `decHTQT_v2` → `decHTQT_sf`

4. **Encrypt Command Changes**
   - Results vector now sized to `file_count` (was `file_count × recipient_count`)
   - Removed `total_pairs` variable and associated >10k batch size warning
   - Progress event iteration now per-file, not per-(file, recipient) pair in result loop
   - **Note:** DLL still calls progress callback per (file, recipient) pair during encryption

5. **Decrypt Command Changes**
   - Replaced per-file `dec_v2()` loop with single `dec_sf()` batch call
   - Removed `recipient_id` parameter from token state read
   - Output file path now read from `BatchResult.output_path` (DLL fills from SF header)
   - Uses `cert_cn_log` field for audit logging only

### Files Modified

**Rust Backend:**
- `src-tauri/src/htqt_ffi/types.rs`
  - Renamed `FnEncHTQTMulti` → `FnEncHTQTSfMulti`
  - Replaced `FnDecHTQTV2` with `FnDecHTQTSf`
  - Added `BatchSfDecryptParams` struct

- `src-tauri/src/htqt_ffi/lib_loader.rs`
  - Updated `HtqtLib` struct field names
  - Updated `load()` symbol resolution
  - Updated `enc_multi()` transmute type
  - Replaced `dec_v2()` with `dec_sf()` batch implementation

- `src-tauri/src/commands/encrypt.rs`
  - Results vec capacity: `file_count` (was `file_count × recip_count`)
  - Removed `total_pairs` variable
  - Updated result loop iteration

- `src-tauri/src/commands/decrypt.rs`
  - Single `dec_sf()` batch call (was: per-file `dec_v2()` loop)
  - Removed `recipient_id` from token state access
  - Output path from `BatchResult.output_path`
  - Updated imports: `BatchSfDecryptParams`, `FileEntry`

**Feature Reference:**
- `feature/1. api-sf-type/htqt-api.h` — New API specification (v2)

**Documentation:**
- Updated system architecture for v2 APIs
- Updated codebase summary for SF v1 format
- Created changelog entry

### Backward Compatibility

⚠️ **Breaking Change:** Applications using v1.x must update DLL integration.

**Migration Path:**
1. Update `htqt_crypto.dll` to v2 (SF v1 support)
2. Update Rust FFI types and lib_loader
3. Update encrypt/decrypt commands to use batch APIs
4. Rebuild and test with v2 DLL

**Advantages of v2:**
- Single batch call instead of M or M×N calls
- Reduced PKCS#11 session overhead
- SF v1 multi-recipient format (smaller encrypted files)
- Cleaner API contract

### Testing Notes

- Encrypt: Verify single `.sf1` output per input file (not M×N)
- Decrypt: Verify output filename matches SF header `orig_name` (not input `.sf1` filename)
- Progress: Monitor callback frequency (still per file-recipient pair, not per file)
- Error handling: Verify partial failure mode works correctly

### Migration Guidance for Users

**If updating from v1.x:**
1. Backup any in-progress encrypted files
2. Update crypto DLL to v2.0.0 or later
3. Restart application
4. Test with small batch first (1-2 files, 1-2 recipients)
5. Verify output file format (still `.sf1`, now with multiple recipient blocks)

---

## [1.0.0] — 2026-02-21

### Initial Release

**Features:**
- Batch M×N encryption with PKCS#11 token support
- Per-file decryption with signature verification
- Recipient group management
- Certificate import and validation
- Token library auto-detection
- SQLite settings and audit log persistence
- Real-time progress UI

**Tech Stack:**
- React 18 + TypeScript (frontend)
- Tauri v2 (desktop framework)
- Rust (backend)
- SQLite (persistence)
- PKCS#11 (smart card integration)

**Supported Platforms:**
- Windows 10/11

**Known Limitations:**
- Single-threaded crypto operations (DLL limitation)
- PKCS#11 library path must be configured or auto-detected
- Token session limited to one user login at a time

---

## Version History Summary

| Version | Date | Status | Focus |
|---------|------|--------|-------|
| 2.0.0 | 2026-04-05 | Current | Crypto API v2 migration (SF v1 batch) |
| 1.0.0 | 2026-02-21 | Stable | Initial release |

---

## How to Use This Changelog

**For Developers:**
- Check "Breaking Changes" before upgrading DLLs
- Reference "Files Modified" for code review
- Use "Migration Path" for integration steps

**For Users:**
- Check "Migration Guidance for Users" before updating
- Review "Advantages" to understand improvements
- Test with small batches after upgrade

**For Release Notes:**
- Copy relevant sections from "Major Changes"
- Include version number and date
- Highlight breaking changes prominently

---

## Release Process

**Before Release:**
1. Update changelog with all changes
2. Update version in `Cargo.toml` and `package.json`
3. Run full test suite
4. Create signed git tag (e.g., `v2.0.0`)

**After Release:**
1. Archive old builds
2. Update download links
3. Notify users of major changes
4. Monitor for compatibility issues

---

**See Also:**
- Development Roadmap: `docs/development-roadmap.md`
- Codebase Summary: `docs/codebase-summary.md`
- System Architecture: `docs/system-architecture.md`
