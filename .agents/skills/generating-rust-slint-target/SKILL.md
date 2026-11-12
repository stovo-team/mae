---
name: generating-rust-slint-target
description: Use when implementing or reviewing a native Rust + Slint desktop target produced from a React/TypeScript web application, enforcing DDD, Clean Architecture, Modular Monolith boundaries, config-driven curviness/tokens, OS-independence with Windows-first targeting, and Ponytail reuse.
---

# Generating Rust + Slint Target

## 1. Hard Native Compliance Rule

The accepted target uses pure Rust domain/application logic and declarative Slint UI (`ui/**/*.slint`).

- **Windows Subsystem Attribute:** Binary entrypoints (`src/main.rs`) must include `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]`.
- **Forbidden Wrappers:** Do NOT use WebView/browser embedding (`wry`, `tauri`, `webview2`, `webkit2gtk`), Electron, screenshot-as-UI, or whole-source proxying. `NativeComplianceVerifier` rejects these immediately.

## 2. DDD + Clean Architecture + Modular Monolith Layout

For multi-route applications, structure the target as a Cargo workspace with strict inward dependencies:

1. **`<app>-config`:** 1:1 pure-Rust mirror of the React `config/` barrel (`brand`, `radii`, `sizes`, `motion`, `colors`, `strings`, `AppearanceSettings`, `ServerManifest`) and `design-system/` palette/tokens. Zero I/O dependencies.
2. **`<app>-domain`:** Pure DDD core (`std` only) defining Entities, Value Objects, Domain Events, Policies, and OS-independent Port traits (`PreferencesStorePort`, `LocalReplicaRepositoryPort`, `HttpApiClientPort`, `RealtimeTransportPort`, `MessageCryptoPort`, `CallMediaEnginePort`, `DesktopShellPort`).
3. **`<app>-application`:** Modular monolith feature slices, `AppState`, `AppEvent`, `AppRoute`, and pure deterministic `reduce(&mut AppState, AppEvent)`. Zero Slint or OS dependencies; 100% testable in headless `cargo test`.
4. **`<app>-adapter-*`:** Concrete infrastructure adapters (`storage`, `network`, `crypto`, `media`, `platform`). Isolate all OS-specific code (`#[cfg(target_os = "windows")]`) inside `<app>-adapter-platform` with cross-platform fallbacks so >=95% of the codebase remains OS-independent.
5. **`<app>-desktop-app`:** Binary + thin property/callback bridge (`src/ui_bridge.rs`) + declarative `ui/**/*.slint`. Never place business rules, HTTP requests, or state mutations directly inside `.slint` files.

## 3. Config-Driven UI, Curviness & Popup Primitives

- **Preserve Every Curve (`radii`):** Map all source corner radii (`bubble` presets: `rounded=18px`, `soft=12px`, `square=6px`, slider `4px..24px`; `control=8px`, `card=8px`, `surface=16px`, `input=18px`, `panel=24px`, `mobileGlass=26px`, `pill=self.height / 2`) into `<app>-config` and `ui/theme/tokens.slint` (`export global ThemeTokens`).
- **Zero Raw Literals in Feature `.slint` Files:** Never hardcode hex/rgba colors (`#...`), `border-radius: Npx`, or `font-size: Npx` in page/component `.slint` files. Always bind to `ThemeTokens.*`.
- **Reuse Shared Popup & Control Primitives:** Always reuse shared Slint primitives (`PopupBackdrop`, `PopupDialog`, `PopupCloseButton`, `PopupButton`, `PopupField`, `PopupSelect`, thin `4px` scrollbars) rather than creating local dialog/button/input styles.

## 4. File Size & Verification Gates

- Keep every production `.rs` and `.slint` file **<= 400 lines** (split into submodules when approaching 350 lines).
- Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and boundary/token check scripts after every change. Never weaken verifier thresholds or delete tests to make code pass.
