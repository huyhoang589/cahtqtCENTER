# CAHTQT — PKI Encryption Desktop App

A desktop application for M×N batch encryption using Public Key Infrastructure (PKI) cryptography. Encrypt files for multiple recipients in a single operation and decrypt received files using PKCS#11 tokens (smart cards, HSMs).

## Features

- **Batch Encrypt** — Select M files × N recipients → produce M×N encrypted files in one operation
- **Decrypt** — Decrypt received files via PKCS#11 token + PIN
- **Recipient Groups** — Organize recipients with X.509 certificate management
- **PKCS#11 Integration** — Smart card / HSM token enumeration and certificate listing
- **Settings** — Configure crypto DLL path, PKCS#11 library, output directory

## Tech Stack

| Layer | Technology |
|-------|-----------|
| UI | React 18 + TypeScript |
| Desktop | Tauri v2 (Rust) |
| Crypto | FFI bridge → `crypto_dll.dll` |
| Database | SQLite (via sqlx) |
| Build | Vite + Cargo |

## Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) (stable)
- [Tauri CLI v2](https://tauri.app/start/prerequisites/)
- Windows 10/11 (native target)
- `crypto_dll.dll` — place in the same directory as the built executable

## Getting Started

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build production executable
npm run tauri build
```

The built installer will be in `src-tauri/target/release/bundle/`.

## Project Structure

```
src/                  # React frontend
├── components/       # UI components
├── pages/            # Encrypt, Decrypt, Groups, Settings
├── hooks/            # useEncrypt, useDecrypt
└── lib/              # Tauri API bindings

src-tauri/src/        # Rust backend
├── lib.rs            # App state + Tauri setup
├── dll_wrapper.rs    # FFI bridge to crypto DLL
├── pkcs11_service.rs # PKCS#11 token integration
├── cert_parser.rs    # X.509 certificate parsing
└── models.rs         # Shared data models
```

## License

Private — all rights reserved.
