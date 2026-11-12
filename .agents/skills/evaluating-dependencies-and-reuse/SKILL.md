---
name: evaluating-dependencies-and-reuse
description: Use before adding any external crate/dependency ("dependicles") or creating a new UI component/helper during migration. Enforces Ponytail code reuse, OS-independent crate selection, Dependency Decision Records, and Reusable Code Proposals.
---

# Evaluating Dependencies & Code Reuse ("Dependicles" + Ponytail)

## 1. Trace & Reuse Before Writing (Ponytail Priority Order)

Before writing new code or adding a dependency, follow this exact priority order:

1. **Already in the codebase?** Search (`rg`) for existing components, helpers, domain types, config tokens (`radii`, `sizes`, `colors`, `strings`), and shared Slint primitives (`PopupDialog`, `PopupButton`, `PopupField`, `PopupSelect`). Reuse or extend them in place.
2. **Rust Standard Library (`std`) or Slint built-in?** Use `std` (`std::collections`, `std::sync`, `std::thread::scope`, `std::fs`, `std::path`) or built-in Slint elements (`Rectangle`, `VerticalLayout`, `HorizontalLayout`, `ScrollView`, `TouchArea`, `TextInput`, `Image`, `Path`) whenever sufficient.
3. **Already in `[workspace.dependencies]`?** Reuse the workspace-pinned crate rather than adding a second crate for the same job.
4. **External crate genuinely required?** Follow the Dependency Decision protocol below. Never add a crate for a few lines of logic.

## 2. Dependency Selection Rules for Desktop Targets

When an external Rust crate is required (e.g., SQLite persistence, HTTP/WebSocket transport, WebCrypto-compatible E2EE, audio/media, canvas rasterization):

- **Prefer Pure-Rust, OS-Independent Crates:** Choose crates that compile cleanly across Windows (`x86_64-pc-windows-msvc` / `gnu`) and Linux without external C/OpenSSL system dependencies (e.g., `reqwest` with `rustls-tls`, `rusqlite` with `bundled`, `aes-gcm`, `rsa`, `sha2`, `tiny-skia`, `qrcode`).
- **Isolate OS-Specific Crates:** Any crate that touches Win32/WinRT APIs (`windows-sys`, `winreg`, Windows Toast) MUST be confined to `<app>-adapter-platform` behind `[target.'cfg(target_os = "windows")'.dependencies]` with a portable fallback for non-Windows hosts.
- **Pin in Workspace Root:** Declare all shared dependencies once in root `Cargo.toml` under `[workspace.dependencies]` and reference them with `{ workspace = true }`.

## 3. Required Artifacts

### A. Dependency Decision Record (`docs/dependencies/NNN-<crate>.md`)
Record for every non-trivial dependency decision:
- Task & capability needed; existing caller/helper search performed.
- Candidate crates compared against a local implementation (with upstream docs/repo links).
- Exact version, license (`MIT` / `Apache-2.0`), maintenance status, known advisories, and transitive footprint.
- OS-independence and Windows/Linux cross-compilation impact.
- Chosen crate, rejected alternatives, and verification performed.

### B. Reusable Code Proposal (`docs/dependencies/REUSE_CANDIDATES.md`)
At the closeout of a substantial feature slice, assess whether touched code has a credible second consumer:
- Candidate or reasoned no-candidate outcome.
- Second consumer and stable API seam (`trait` or shared crate/component).
- Product-specific coupling and extraction cost.
