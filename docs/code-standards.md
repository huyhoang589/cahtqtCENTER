# Code Standards & Conventions

**Project:** CAHTQT — PKI Encryption Desktop App  
**Last Updated:** 2026-04-05  
**Status:** Active (enforced for all commits)

## Overview

This document defines coding standards, naming conventions, and architectural patterns used across the CAHTQT project. All contributors must follow these standards to maintain consistency and readability.

## Rust Backend Standards

### File Organization

**Directory Structure:**
```
src-tauri/src/
├── htqt_ffi/            # Crypto FFI binding layer
├── commands/            # Tauri command handlers
├── db/                  # Database repositories
├── etoken/              # PKCS#11 token integration
├── lib.rs               # App initialization
├── main.rs              # Entry point
├── app_log.rs           # Log emission
├── cert_parser.rs       # Certificate parsing
├── ffi_helpers.rs       # C interop utilities
├── lock_helper.rs       # Concurrency utilities
├── output_dir.rs        # Output path resolution
├── models.rs            # Shared types
└── (feature modules)
```

**Principles:**
- One module per concern (FFI, commands, database, tokens)
- Keep files under 300 lines of code (split if larger)
- Place related functions in same file or closely connected modules
- Use `mod.rs` for module exports, not inline mod declarations

### Naming Conventions

**Files:**
- Use `snake_case` (Rust convention)
- Descriptive names: `token_manager.rs`, not `mgr.rs`
- Match primary type name: `struct TokenManager` in `token_manager.rs`

**Functions:**
```rust
// Public functions (exposed via FFI or commands)
pub async fn encrypt_batch(...) -> Result<EncryptResult, String>
pub fn load_crypto_dll(...) -> Result<HtqtLib, String>

// Internal functions
fn build_encrypt_params(...) -> BatchEncryptParams
fn validate_certificate(cert_der: &[u8]) -> Result<(), String>
```

**Types:**
- Use `PascalCase` for structs, enums, traits
- Abbreviations acceptable if clear: `FFI`, `DLL`, `PKCS11`
- Enum variants: `PascalCase` (e.g., `TokenStatus::LoggedIn`)

**Constants:**
```rust
// Public constants
pub const HTQT_OK: i32 = 0;
pub const HTQT_BATCH_CONTINUE_ON_ERROR: u32 = 0x01u;

// Module constants
const DLL_LOCK_TIMEOUT_MS: u64 = 5000;
const MAX_BATCH_SIZE: usize = 10000;
```

**Variables:**
```rust
// snake_case for all variables
let mut batch_results: Vec<BatchResult> = Vec::new();
let cert_der = read_certificate(&path)?;
```

### Error Handling

**Always use `Result<T, E>` for fallible operations:**

```rust
// ✅ Good
pub fn load_dll(path: &str) -> Result<HtqtLib, String> {
    let lib = unsafe { Library::new(path) }
        .map_err(|e| format!("Failed to load DLL: {}", e))?;
    Ok(HtqtLib { lib, ... })
}

// ❌ Bad
pub fn load_dll(path: &str) -> HtqtLib {
    unsafe { Library::new(path).unwrap() }  // Will panic!
}
```

**Error Messages:**
- Describe what failed and why
- Include context (file paths, values)
- Use lowercase unless referring to proper nouns

```rust
// ✅ Good
Err(format!("Symbol '{}' not found in {}", symbol_name, dll_path))

// ❌ Bad
Err("Symbol not found")
Err("Error")
```

**Panic Only For:**
- Invariant violations (logic errors, should never happen)
- Test code

```rust
// ✅ Acceptable
assert_eq!(expected, actual, "Results array size mismatch");

// ❌ Avoid in production
panic!("Unexpected state");
unwrap()  // on untrusted data
```

### Comments & Documentation

**Doc Comments (public items):**

```rust
/// Batch encrypt M files for N recipients.
///
/// Produces one `.sf1` file per input file with all recipients embedded.
///
/// # Arguments
/// * `params` - File and recipient list with output directory
/// * `cbs` - Crypto callbacks (sign, encrypt, progress)
/// * `results` - Array sized to `file_count` for results
///
/// # Returns
/// - `HTQT_OK` on success
/// - `HTQT_ERR_PARTIAL` if some files succeeded
/// - Other error codes on total failure
///
/// # Errors
/// Returns error string if DLL is not loaded or operation fails.
pub fn enc_multi(...) -> Result<i32, String> { ... }
```

**Inline Comments (implementation details):**

```rust
// Convert C string to Rust String safely
let result_str = ffi_helpers::string_from_c_buf(&error_buf);

// Only cache cert_der if token fully initialized
let own_cert_der = scan.as_ref()
    .and_then(|s| s.certificates.first())
    .map(|e| e.certificate.raw_der.clone())
    .unwrap_or_default();
```

**Avoid:**
- Obvious comments: `let x = 5; // Set x to 5`
- Commented-out code (use git history instead)
- TODO comments without context or owner

### FFI & Unsafe Code

**Unsafe blocks must be documented:**

```rust
// SAFETY: Symbol is guaranteed to resolve to FnEncHTQTSfMulti.
// We transmute and call immediately within DLL_LOCK.
let rc = unsafe {
    let f: FnEncHTQTSfMulti = std::mem::transmute(self.enc_sf_multi_fn);
    f(params, cbs, results.as_mut_ptr(), err_buf.as_mut_ptr(), 512)
};
```

**Principles:**
- Minimize unsafe scope
- Document safety invariants
- Always hold DLL_LOCK during FFI calls
- Use `transmute` only for function pointers

### Testing & Test Code

**Test Organization:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_dll() {
        let result = HtqtLib::load("htqt_crypto.dll");
        assert!(result.is_ok());
    }

    #[test]
    fn test_encrypt_with_empty_files() {
        let params = BatchEncryptParams::default();
        let result = encrypt_batch(params);
        assert!(result.is_err());
    }
}
```

**Conventions:**
- Tests in same file as implementation (if small)
- Test functions describe behavior: `test_X_returns_Y_when_Z`
- Use assertions with descriptive messages

### Formatting & Linting

**Rust formatting (rustfmt):**
```bash
# Check formatting
cargo fmt -- --check

# Auto-fix formatting
cargo fmt
```

**Clippy linting:**
```bash
# Check for common mistakes
cargo clippy -- -D warnings
```

**Standards:**
- 4-space indentation
- Max 100 characters per line (flexible for URLs, long strings)
- One blank line between items, zero within functions (generally)
- Trailing commas in multi-line collections

---

## TypeScript/React Frontend Standards

### File Organization

**Directory Structure:**
```
src/
├── components/          # Reusable UI components
│   ├── encrypt-progress-panel.tsx
│   ├── decrypt-progress-panel.tsx
│   └── (other components)
├── pages/               # Page-level components
│   ├── EncryptPage.tsx
│   ├── DecryptPage.tsx
│   └── SettingsPage.tsx
├── hooks/               # Custom React hooks
│   ├── use-encrypt.ts
│   ├── use-decrypt.ts
│   └── use-token-status.ts
├── contexts/            # Context providers
├── types/               # TypeScript type definitions
├── lib/                 # Utility libraries (Tauri API, etc.)
├── App.tsx              # Root component
└── main.tsx             # Entry point
```

**Principles:**
- One component per file
- File name matches component name (kebab-case)
- Keep components under 200 lines (split if larger)

### Naming Conventions

**Files:**
- Components: `kebab-case.tsx` (e.g., `encrypt-progress-panel.tsx`)
- Hooks: `kebab-case.ts` (e.g., `use-encrypt.ts`)
- Utilities: `kebab-case.ts` (e.g., `tauri-api.ts`)

**Components:**
```tsx
// PascalCase for component names
export function EncryptProgressPanel(props: Props) { ... }
export const DecryptProgressPanel = (props: Props) => { ... }
```

**Types:**
```tsx
// PascalCase for interfaces and types
interface EncryptProgress {
  current: number;
  total: number;
  status: "processing" | "success" | "error";
}

type FileStatus = "pending" | "processing" | "success" | "error";
```

**Variables & Functions:**
```tsx
// camelCase for variables and functions
const [isLoading, setIsLoading] = useState(false);
const handleEncryptClick = () => { ... };
const formatBytes = (bytes: number) => { ... };
```

### React Patterns

**Functional Components:**

```tsx
// ✅ Preferred: Function declarations with JSX
export function MyComponent(props: Props) {
  return <div>{props.title}</div>;
}

// Also acceptable: Arrow functions
export const MyComponent = (props: Props) => {
  return <div>{props.title}</div>;
};
```

**Hooks Usage:**

```tsx
export function EncryptPage() {
  const [files, setFiles] = useState<string[]>([]);
  const { encrypt, progress } = useEncrypt();
  const { tokenStatus } = useTokenStatus();

  const handleEncrypt = async () => {
    try {
      const result = await encrypt(files);
      // Handle success
    } catch (error) {
      // Handle error
    }
  };

  return (
    <div>
      {/* Render UI */}
    </div>
  );
}
```

**Props Typing:**

```tsx
interface EncryptProgressProps {
  current: number;
  total: number;
  fileName: string;
  status: "processing" | "success" | "error";
  error?: string;
}

export function EncryptProgress(props: EncryptProgressProps) {
  return <div>{props.fileName}: {props.current}/{props.total}</div>;
}
```

### Error Handling

**Async Operations:**

```tsx
// ✅ Good: Try-catch with user feedback
const handleEncrypt = async () => {
  try {
    const result = await encrypt(files);
    showSuccessNotification(`Encrypted ${result.success_count} files`);
  } catch (error) {
    showErrorNotification(`Encryption failed: ${error}`);
  }
};

// ❌ Bad: Unhandled promise rejection
const handleEncrypt = async () => {
  const result = await encrypt(files);  // Will crash if error
  // ...
};
```

**Error Messages:**
- Show to user in accessible language
- Include actionable steps if possible
- Log full error to console for debugging

```tsx
try {
  await encrypt(files);
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  console.error("Encrypt failed:", error);
  showUserError(`Failed to encrypt files: ${message}`);
}
```

### Comments & Documentation

**JSDoc for complex components:**

```tsx
/**
 * Displays real-time progress of encryption operation.
 *
 * @param props - EncryptProgressProps with current/total/status
 * @returns Rendered progress panel component
 *
 * @example
 * <EncryptProgress
 *   current={5}
 *   total={10}
 *   fileName="document.pdf"
 *   status="processing"
 * />
 */
export function EncryptProgress(props: EncryptProgressProps) { ... }
```

**Inline comments for logic:**

```tsx
// Debounce file selection changes to avoid rapid re-renders
const [files, setFiles] = useState<string[]>([]);
const debouncedSetFiles = useMemo(
  () => debounce(setFiles, 300),
  []
);
```

### Formatting & Linting

**ESLint configuration:**
```bash
# Check lint issues
npm run lint

# Auto-fix lint issues
npm run lint -- --fix
```

**Prettier formatting:**
```bash
# Check formatting
npx prettier --check src/

# Auto-format
npx prettier --write src/
```

**Standards:**
- 2-space indentation
- Single quotes for strings
- Trailing commas in multi-line objects/arrays
- Max 100 characters per line

---

## Tauri Bridge & IPC

### Command Naming

**Frontend → Backend (command invocation):**

```typescript
// camelCase for command names (matches Rust function)
const result = await invoke<EncryptResult>("encrypt_batch", {
  srcPaths: files,
  partnerName: group,
  certPaths: certs,
});
```

**Rust command definition:**

```rust
#[tauri::command]
pub async fn encrypt_batch(
    src_paths: Vec<String>,
    partner_name: String,
    cert_paths: Vec<String>,
) -> Result<EncryptResult, String> {
    // Implementation
}
```

**Convention:** Snake_case in Rust, camelCase in TypeScript arguments.

### Event Emitting

**Rust emitter:**

```rust
app.emit_all("encrypt_progress", EncryptProgress {
    current: 5,
    total: 10,
    file_name: "document.pdf".to_string(),
    status: "processing".to_string(),
    error: None,
})?;
```

**TypeScript listener:**

```typescript
const unlisten = await appWindow.listen<EncryptProgress>(
  "encrypt_progress",
  (event) => {
    updateProgress(event.payload);
  }
);

// Clean up listener
unlisten();
```

---

## Database Standards

### Schema Design

**File: `src-tauri/migrations/NNN_description.sql`**

```sql
-- Each migration is numbered sequentially (001, 002, etc.)
-- Migration name describes the change

CREATE TABLE IF NOT EXISTS partners (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_partners_name ON partners(name);
```

**Principles:**
- One logical change per migration
- Always use `IF NOT EXISTS` for create statements
- Include indexes for frequently queried columns
- Document complex schemas with comments

### Repository Pattern

**File: `src-tauri/src/db/partners_repo.rs`**

```rust
/// Repository for partner (recipient group) persistence.
pub struct PartnersRepo;

impl PartnersRepo {
    /// List all partners.
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Partner>, String> { ... }

    /// Get partner by name.
    pub async fn get_by_name(pool: &SqlitePool, name: &str) -> Result<Partner, String> { ... }

    /// Create new partner.
    pub async fn create(pool: &SqlitePool, name: &str) -> Result<Partner, String> { ... }

    /// Delete partner by ID.
    pub async fn delete(pool: &SqlitePool, id: i32) -> Result<(), String> { ... }
}
```

**Conventions:**
- One repo per domain entity (Partner, Settings, etc.)
- Methods return `Result<T, String>` (user-friendly errors)
- Use `async` for all DB operations
- Pass `&SqlitePool` to each method

---

## Git & Commit Conventions

### Branch Naming

**Format:** `{type}/{short-description}`

```
feature/crypto-api-migration
fix/token-login-timeout
docs/system-architecture
refactor/consolidate-ffi-types
```

**Types:**
- `feature/` — New feature
- `fix/` — Bug fix
- `docs/` — Documentation only
- `refactor/` — Code cleanup (no behavior change)
- `test/` — Test additions/fixes
- `chore/` — Build, dependency updates

### Commit Messages

**Format:** Conventional Commits

```
feat: add batch decrypt API support

- Migrate from per-file decHTQT_v2 to batch decHTQT_sf
- Update BatchSfDecryptParams struct and types
- Handle output_path from DLL result
```

**Structure:**
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Example:**
```
fix(encrypt): fix results array size for v2 API

The results array was sized to file_count * recipient_count
in the old API. New API requires file_count only.

Fixes #123
```

**Types:**
- `feat:` — New feature
- `fix:` — Bug fix
- `docs:` — Documentation
- `refactor:` — Code restructuring
- `test:` — Test changes
- `chore:` — Build/dependencies
- `perf:` — Performance improvement

**Guidelines:**
- Subject line: imperative mood, present tense ("add" not "added" or "adds")
- Max 50 characters for subject
- Capitalize first letter
- No trailing period
- Body: explain WHAT and WHY, not HOW
- Reference issues: `Fixes #123`, `Refs #456`

---

## Security Standards

### Sensitive Data Handling

**Never commit:**
- `.env` files with API keys
- Certificate private keys
- Database credentials
- PKCS#11 PIN values

**Safe practices:**
```rust
// ❌ Bad: PIN in logs
println!("PIN: {}", pin);

// ✅ Good: Redact sensitive data
println!("PIN: [redacted]");

// ❌ Bad: Clear text in memory indefinitely
let pin_str = user_input.to_string();

// ✅ Good: Use secure types (if available)
// Use zeroize crate for sensitive data
```

### Certificate Validation

**Always validate:**
- Certificate chain (issuer verification)
- Signature validity
- Expiration dates (warn user)
- Key usage constraints

```rust
// ✅ Good: Validate before use
let cert = parse_cert(&cert_der)?;
cert.validate_chain(&issuer_certs)?;
cert.check_expiration()?;

// ❌ Bad: Use without validation
let cert = parse_cert(&cert_der)?;
use_cert_immediately(&cert)?;
```

---

## Testing Standards

### Unit Tests

**Location:** Same file as code being tested  
**Pattern:** `#[cfg(test)]` module at bottom of file

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_with_valid_params() {
        // Arrange
        let params = create_test_params();

        // Act
        let result = encrypt_batch(params);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_encrypt_fails_with_empty_recipients() {
        let mut params = create_test_params();
        params.recipients.clear();

        let result = encrypt_batch(params);

        assert!(result.is_err());
    }
}
```

### Integration Tests

**Location:** `src-tauri/tests/` directory  
**Pattern:** Full command invocation with test fixtures

```rust
// tests/encrypt_integration.rs
#[tokio::test]
async fn test_encrypt_batch_end_to_end() {
    // Setup test files, certs, etc.
    let app = create_test_app().await;

    // Invoke command
    let result = app.invoke_encrypt_batch(...).await;

    // Verify output files exist
    assert!(result.success_count > 0);
}
```

### Test Coverage Goals

- Encrypt: >85% coverage (happy path + error cases)
- Decrypt: >85% coverage (happy path + error cases)
- FFI layer: >80% coverage (symbol resolution, transmute safety)
- Token ops: >75% coverage (library detection, session mgmt)

---

## Performance Standards

### Benchmarks

**Target Performance:**
- Encrypt 1000 files × 10 recipients: <1 minute
- Decrypt 100 files: <30 seconds
- Token login: <5 seconds
- UI responsiveness: <100ms input latency

**Profiling:**
```bash
# Rust profiling (with cargo-flamegraph)
cargo flamegraph -- --bench encrypt

# Results analysis
firefox flamegraph.svg
```

### Memory Constraints

- Batch size limit: 10,000 file pairs (configurable)
- Warn user if batch exceeds recommended size
- Monitor memory usage during large operations

---

## Documentation Standards

### API Documentation

**For public functions/types:**

```rust
/// Brief one-line description.
///
/// Longer paragraph explaining context and usage.
///
/// # Arguments
/// * `param1` — Description
/// * `param2` — Description
///
/// # Returns
/// Description of return value.
///
/// # Errors
/// Description of error conditions.
///
/// # Example
/// ```
/// let result = function_name(arg1, arg2)?;
/// ```
pub fn function_name(param1: T, param2: U) -> Result<V, String> { ... }
```

### File Headers

**For complex modules:**

```rust
//! Module for PKCS#11 token integration.
//!
//! Provides token enumeration, certificate extraction, and session management.
//! See `TokenManager` for the main entry point.

use std::ffi::CString;
```

---

## Tooling & Automation

### Pre-commit Hooks

**Install:**
```bash
cd ./.claude/scripts
./setup-hooks.sh
```

**Checks:**
- Format check (rustfmt, prettier)
- Lint check (clippy, eslint)
- Type check (TypeScript)
- No secrets detection

### Build Commands

```bash
# Frontend
npm run lint              # Check TypeScript lint
npm run format            # Format React code

# Backend
cargo fmt                 # Format Rust
cargo clippy -- -D warnings  # Lint Rust
cargo test                # Run all tests
cargo build --release     # Production build
```

### CI/CD Pipeline

**GitHub Actions:** `.github/workflows/`

- Format check
- Lint check
- Unit tests
- Integration tests
- Security audit (clippy, npm audit)
- Build verification (cargo build, npm run build)

---

## Summary Table

| Aspect | Rust | TypeScript | Note |
|--------|------|-----------|------|
| File naming | snake_case | kebab-case | Match conventions |
| Functions | snake_case | camelCase | Tauri: snake_case in Rust |
| Types | PascalCase | PascalCase | Consistent |
| Constants | SCREAMING_SNAKE_CASE | SCREAMING_SNAKE_CASE | Rare in TS |
| Line length | 100 char | 100 char | Flexible for URLs |
| Indentation | 4 spaces | 2 spaces | Language defaults |
| Error handling | Result<T, E> | try-catch/Promise | Idiomatic |
| Testing | #[test] in file | .test.ts files | Flexible |
| Comments | Doc + inline | JSDoc + inline | Public items only |

---

**See Also:**
- System Architecture: `docs/system-architecture.md`
- Codebase Summary: `docs/codebase-summary.md`
- Development Roadmap: `docs/development-roadmap.md`
