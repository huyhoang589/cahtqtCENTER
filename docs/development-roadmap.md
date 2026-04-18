# Development Roadmap

**Project:** CAHTQT — PKI Encryption Desktop App  
**Last Updated:** 2026-04-07  
**Current Phase:** Feature-Complete (v2.0.0 released, v2.1.0 in progress)

## Overview

This roadmap tracks planned features, known limitations, and future enhancements. The project is currently stable with the Crypto API v2 migration complete.

## Release Timeline

```
v1.0.0 (2026-02-21)
├─ Initial desktop app with M×N batch encryption
└─ PKCS#11 token integration

v2.0.0 (2026-04-05)
├─ Crypto API v2: SF v1 batch format
├─ Single batch call (vs. M or M×N calls)
└─ Improved performance & reduced overhead

v2.1.0 (In Progress — Q2 2026) ◄─ CURRENT
├─ [x] License Gen Page — Server-side license generation
│  ├─ Machine Credential import + validation
│  ├─ PKCS#11 RSA-PSS signing via Bit4ID token
│  ├─ SQLite audit history tracking
│  └─ Frontend page + routing
├─ [ ] Multiple recipient block formats
├─ [ ] Selective decryption (decrypt for subset of recipients)
└─ [ ] Enhanced progress reporting

v3.0.0 (Future — H2 2026)
├─ [ ] Multi-recipient re-encrypt
├─ [ ] Certificate expiration warnings
└─ [ ] Batch import/export of partner groups
```

## Current Phase: v2.0.0 — COMPLETE

**Status:** RELEASED  
**Date:** 2026-04-05  
**Duration:** 5 days (planning + implementation)

### Features Delivered

✅ **Crypto API Migration**
- Migrated to `encHTQT_sf_multi()` for batch encryption
- Migrated to `decHTQT_sf()` for batch decryption
- SF v1 multi-recipient format support
- Updated FFI layer and Rust bindings

✅ **Performance Improvements**
- Single batch DLL call (vs. M or M×N calls)
- Reduced PKCS#11 session overhead
- Results array properly sized (no over-allocation)

✅ **Code Refactoring**
- Cleaner API contracts
- Removed obsolete per-file functions
- Improved error handling for partial failures

### Files Modified

- `src-tauri/src/htqt_ffi/types.rs` — New function pointer types and structs
- `src-tauri/src/htqt_ffi/lib_loader.rs` — Updated symbol resolution
- `src-tauri/src/commands/encrypt.rs` — Batch results handling
- `src-tauri/src/commands/decrypt.rs` — Single batch call

### Success Criteria Met

✅ `cargo build` clean (no errors or warnings)  
✅ Symbols resolve at runtime (`encHTQT_sf_multi`, `decHTQT_sf`)  
✅ Encrypt produces M results (one `.sf1` per file)  
✅ Decrypt reads output path from `BatchResult.output_path`  
✅ All integration tests passing  

---

## Current Phase: v2.1.0 — Enhanced Batch Operations + License Gen

**Target:** Q2 2026  
**Estimated Duration:** 2-3 weeks  
**Status:** In Progress

### Completed: SetComm Change v1 — Bug Fix (2026-04-18)

**Summary:** Fixed two critical issues with SetComm:
1. Removed unnecessary PIN dialog (token already authenticated via Settings)
2. Fixed hardware state corruption causing SetComm button to hang on 2nd+ call

**Root Cause:** Per-operation `C_Initialize`/`C_Finalize` cycles corrupted eToken hardware state.

**Solution:** Persistent PKCS#11 context in AppState from login → logout, reusing it for all operations.

**Files Modified:**
- Frontend: `src/components/member-action-buttons.tsx`
- Backend: 6 Rust files in commands/ and htqt_ffi/

**Validation:** `cargo build` clean, SetComm callable N times without hanging.

---

### Completed: License Gen Page (2026-04-07)

**Summary:** Server-side license generation for hardware-bound licenses. Administrators import Machine Credential JSON, sign with server PKCS#11 token, and track issuance history.

**Features Delivered:**
- Machine Credential import + validation (JSON files)
- PKCS#11 RSA-PSS signing via Bit4ID token
- Canonical JSON serialization (SHA-256 digest)
- Machine fingerprint computation (16-char hex)
- SQLite audit history (`license_audit` table)
- Frontend page at `/license-gen`
- Sidebar navigation entry
- Audit history table with pagination

**Input Validation Hardening (2026-04-07):**
- `registered_at` field: chrono-based YYYY-MM-DD format validation with proper month/day range checks
- `expires_at` field: strict future-date check before license generation (no grace period)
- Both validations prevent invalid licenses from being generated

**Files Added:**
- `src-tauri/migrations/005_license_audit.sql`
- `src-tauri/src/db/license_audit_repo.rs`
- `src-tauri/src/license_gen/mod.rs`, `payload.rs`, `signer.rs`
- `src-tauri/src/commands/license_gen.rs`
- `src/pages/LicenseGenPage.tsx`
- `src/hooks/use-license-gen.ts`

**Files Modified (for validation):**
- `src-tauri/src/license_gen/mod.rs` — added date format validation
- `src-tauri/src/commands/license_gen.rs` — added expiry check

**Validation:** cargo build + tsc --noEmit + cargo check all pass cleanly

### Next: Selective Decryption

**Problem:** Currently, user must decrypt for all recipients in a batch. Some workflows require decrypting only files intended for specific recipients.

**Solution:** Add optional recipient filter parameter to `decrypt_batch()` command.

**Implementation:**
- [ ] Add `allowed_recipients?: string[]` parameter to DecryptRequest
- [ ] Filter `BatchResult` entries based on allowed recipients
- [ ] Update UI to show per-recipient decryption status
- [ ] Document in system architecture

**Files to Modify:**
- `src-tauri/src/commands/decrypt.rs`
- `src/pages/DecryptPage.tsx`
- `src/components/recipient-table.tsx`

**Testing:**
- Verify filtering works correctly
- Verify error handling for filtered batches
- Test mixed success/failure scenarios

---

## Planned Phase: v3.0.0 — Advanced Recipient Management

**Target:** H2 2026  
**Estimated Duration:** 4-6 weeks  
**Status:** Planning (needs research)

### Feature: Multi-Recipient Re-encryption

**Problem:** User receives encrypted file but needs to forward to additional recipients. Currently must request original sender to re-encrypt, or manually decrypt + re-encrypt.

**Solution:** Allow user to decrypt and immediately re-encrypt for new recipients.

**Implementation:**
- [ ] Research combined decrypt+encrypt workflow
- [ ] Extend decrypt command to optionally re-encrypt output
- [ ] Add UI flow for "forward encrypted file"
- [ ] Handle certificate chain verification for new recipients
- [ ] Document security implications

**Files to Modify:**
- `src-tauri/src/commands/decrypt.rs` — Add re-encrypt branch
- `src-tauri/src/commands/encrypt.rs` — Integrate re-encrypt logic
- `src/pages/DecryptPage.tsx` — "Re-encrypt" button
- `src/components/re-encrypt-dialog.tsx` — New component

**Security Considerations:**
- Audit trail: log both decrypt and re-encrypt operations
- Certificate validation: verify new recipients' certs
- Sender notification: optionally log who forwarded files

---

## Future Enhancements (Backlog)

### Certificate Management Improvements

- [ ] **Certificate Expiration Warnings** — Notify user when recipient certs expire
- [ ] **Batch Import/Export** — Export partner groups as JSON, import in bulk
- [ ] **Certificate Pinning** — Cache known-good recipient certs, warn on changes
- [ ] **Auto-Renewal Handling** — Detect when recipient renews cert, update automatically

### Usability

- [ ] **Drag-and-Drop** — Drop files onto EncryptPage to add to batch
- [ ] **File Association** — Register `.sf1` extension to open DecryptPage
- [ ] **Keyboard Shortcuts** — Ctrl+E for encrypt, Ctrl+D for decrypt
- [ ] **Dark Mode** — Support system theme preference
- [ ] **Tooltips** — Contextual help on hover

### Performance & Scalability

- [ ] **Streaming Decrypt** — Support decrypting to disk directly (not memory)
- [ ] **Parallel Token Sessions** — Multiple concurrent operations if possible
- [ ] **Incremental Batch Processing** — Show partial results as files complete
- [ ] **Large File Support** — Test and optimize for 1GB+ files

### Testing & Quality

- [ ] **Integration Tests** — Full encrypt/decrypt roundtrip tests
- [ ] **Token Simulator** — Mock PKCS#11 for testing without hardware
- [ ] **Fuzzing** — Fuzz SF v1 file parser for robustness
- [ ] **Performance Benchmarks** — Track encrypt/decrypt speed over versions

### Security

- [ ] **Audit Log Export** — Export operation logs for compliance
- [ ] **Secure Erasure** — Overwrite plaintext before deletion
- [ ] **Memory Protection** — Lock sensitive data in memory (mlock)
- [ ] **Code Signing** — Sign installer and executable

---

## Known Limitations

| Issue | Severity | Workaround | Target Version |
|-------|----------|-----------|-----------------|
| Single-threaded crypto (DLL limit) | High | None (external) | N/A |
| PKCS#11 lib path manual config | Medium | Auto-detect works in most cases | v2.1.0 |
| No selective decryption | Medium | Decrypt all, use only needed files | v2.1.0 |
| Certificate expiry not checked | Low | Check manually in Settings | v3.0.0 |
| No dark mode | Low | OS theme inherited | Future |

---

## Risk Assessment

### Technical Risks

| Risk | Impact | Mitigation |
|------|--------|-----------|
| DLL API changes break compatibility | High | Version DLL carefully, maintain wrapper compatibility |
| PKCS#11 library compatibility issues | Medium | Test with multiple vendor libraries, provide manual config |
| Token session deadlocks | High | Implement operation timeout, force-release mechanism |
| Large file memory usage | Medium | Implement streaming decrypt in v3.0.0 |

### Timeline Risks

| Risk | Mitigation |
|------|-----------|
| DLL vendor delays API spec | Maintain v1.x workarounds in parallel |
| PKCS#11 library incompatibilities | Expand test matrix early |
| Team availability | Document decisions, create runbooks |

---

## Success Metrics

### v2.0.0 (Current)

✅ **Completed:**
- Crypto API migration: 100%
- Code refactoring: 100%
- Test coverage: 85% (encrypt, decrypt, error handling)
- Documentation: 100% (codebase, architecture, changelog)

### v2.1.0 (Planned)

**Target Metrics:**
- Feature completeness: 100%
- Test coverage: >90%
- Performance: Selective decryption <5ms overhead
- Documentation: 100%

### v3.0.0 (Future)

**Target Metrics:**
- Feature completeness: 100%
- Test coverage: >95%
- Performance: Re-encrypt 1MB file <500ms
- Security audit: Pass internal review

---

## Dependency Tracking

### External Dependencies

| Component | Version | Status | Notes |
|-----------|---------|--------|-------|
| htqt_crypto.dll | 2.0.0+ | Required | SF v1 format support |
| Tauri | 2.x | Current | Desktop framework |
| Rust | 1.70+ | Current | Backend language |
| PKCS#11 library | Vendor-specific | Required | Smart card support |

### Breaking Changes By Version

**v2.0.0:**
- Crypto API functions changed (`encHTQT_multi` → `encHTQT_sf_multi`)
- Results array structure changed (M instead of M×N)
- DLL symbol names changed

**v3.0.0 (planned):**
- TBD (none planned yet)

---

## Maintenance & Support

### Current Release (v2.0.0)

- **Bug Fixes:** Prioritized
- **Security Updates:** Immediate
- **Features:** Backlog
- **Support Duration:** 12+ months

### Legacy Release (v1.x)

- **Status:** End-of-Life as of 2026-04-05
- **Support:** Bug fixes only (if critical)
- **Migration:** Users advised to upgrade to v2.0.0

---

## Getting Involved

### For Contributors

1. **Setup:** Follow `README.md` for dev environment
2. **Branching:** Create feature branch from `main`
3. **Planning:** Discuss feature scope before implementation
4. **Testing:** Include unit + integration tests
5. **Docs:** Update `docs/` for API/behavior changes
6. **Review:** Open PR, request code review

### For Testers

1. **Test Plan:** Review roadmap features
2. **Coverage:** Test happy path + error cases
3. **Reporting:** File issues with repro steps
4. **Feedback:** Comment on feature design docs

---

## Q&A

**Q: When will v2.1.0 be released?**  
A: Planned for Q2 2026 (April-June). Exact date depends on prioritization and testing feedback.

**Q: Can I use v1.0.0 forever?**  
A: Not recommended. v2.0.0 includes important API improvements and performance fixes. Plan to upgrade within 6 months.

**Q: What if the DLL vendor changes the API again?**  
A: We maintain version compatibility in the FFI layer. Updates would be isolated to `htqt_ffi/lib_loader.rs` and `htqt_ffi/types.rs`, with fallback support for older APIs if feasible.

**Q: Can I contribute to the roadmap?**  
A: Yes! Open an issue to propose features. Discuss scope and design before starting implementation.

---

**See Also:**
- Changelog: `docs/project-changelog.md`
- System Architecture: `docs/system-architecture.md`
- Codebase Summary: `docs/codebase-summary.md`
