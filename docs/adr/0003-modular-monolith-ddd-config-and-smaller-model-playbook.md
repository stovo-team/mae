# ADR 0003: Modular Monolith DDD, Config-Driven Curviness, Ponytail Reuse, and Smaller-Model Playbook

- **Status:** Accepted
- **Date:** 2026-11-12

## Context

While ADR 0002 established the v0.1 scope for migrating React/TypeScript frontends to native Windows Rust + Slint applications on a 3-route reference fixture, real-world production frontends (such as multi-route messengers with 20+ routes, 200+ components, centralized `config/` barrels, rich design-system tokens, E2EE cryptography, local SQLite replicas, and realtime sockets/media) cannot be emitted into a single flat `src/app/` module without violating SOLID boundaries or overwhelming smaller coding models.

Furthermore, production React applications rely on:
1. **Centralized Configuration & Design Tokens ("the configs thing"):** Typed TypeScript barrels (`config/radii.ts`, `config/sizes.ts`, `config/motion.ts`, `config/colors/`, `config/strings/`) paired with CSS variables (`design-system/tokens.css`) and runtime appearance contexts (`bubbleRadius`, `messageScale`, `themeMode`, `accent`) that control every curve, corner radius, spacing, and color.
2. **Platform Independence with Windows-First Native Execution:** Targeting native Windows (`#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]`) while keeping >=95% of application, domain, config, and Slint UI code OS-independent and headlessly testable on Linux/Windows CI.
3. **Disciplined Code Reuse & Dependency Governance:** Using **Ponytail** minimal-complete-change workflows (`ponytail`, `ponytail-review`, `ponytail-debt`) alongside explicit dependency evaluation ("dependicles": Dependency Decision Records and Reusable Code Proposals) so agents reuse shared primitives instead of duplicating widgets or bloating `Cargo.toml`.

## Decision

1. **DDD + Clean Architecture Modular Monolith Target Topology:**
   - Large React-to-Rust+Slint migrations MUST decompose the target workspace into bounded crates enforcing strict inward dependency rules:
     - `<app>-config`: 1:1 pure-Rust mirror of source `config/` (`radii`, `sizes`, `motion`, `brand`, `colors`, `strings`, `AppearanceSettings`, `ServerManifest`) and `design-system/` tokens. Zero I/O dependencies.
     - `<app>-domain`: Pure DDD core (Entities, Value Objects, Domain Events, Policies, and OS-independent Port traits). Zero outward/I/O dependencies.
     - `<app>-application`: Modular monolith feature slices, `AppState`, `AppEvent`, `AppRoute`, deterministic `reduce()`, keyboard shortcut registry, and deterministic mock-mode seeders. Depends only on `<app>-domain` and `<app>-config`.
     - `<app>-adapter-*` (`storage`, `network`, `crypto`, `media`, `platform`): Concrete implementations of domain ports. OS-specific code (`#[cfg(target_os = "windows")]`) is strictly isolated inside `<app>-adapter-platform` with portable cross-platform fallbacks.
     - `<app>-desktop-app`: Binary crate + Slint `.slint` declarative UI tree (`ui/theme/tokens.slint`, `ui/components/ui/`, `ui/pages/`) wired to `<app>-application` via thin property/callback bridges.
2. **Config-Driven UI & Curviness Preservation Invariant:**
   - Every border-radius ("curviness"), icon dimension, font scale, weight, line-height, motion duration, and color palette entry from the React source MUST be represented in `<app>-config` and exposed through `ui/theme/tokens.slint` (`export global ThemeTokens`).
   - Production `.slint` component and page files MUST NOT contain raw hex color literals (`#...`) or hardcoded `border-radius` / `font-size` pixel literals; they must bind to `ThemeTokens.*`.
   - Dynamic user appearance settings (e.g., chat bubble shape presets `rounded=18px`, `soft=12px`, `square=6px`, and custom slider `4px..24px`) must update `ThemeTokens` properties at runtime.
3. **Exclusion of Test Artifacts During Source Analysis:**
   - `mae-adapter-react` (`discovery`, `static_analyzer`, `gap_classifier`) excludes test files (`*.test.*`, `*.spec.*`, `*.stories.*`) and test directories (`__tests__`, `test`, `tests`, `testUtils`, `test-fixtures`, `e2e`) so capability gap reports and component counts reflect production UI only.
4. **Ponytail + Dependency Governance ("Dependicles") & Smaller-Model Playbook:**
   - All migration agents (especially smaller models) follow `docs/guides/SMALLER_MODEL_REACT_TO_SLINT_PLAYBOOK.md` and `.agents/skills/evaluating-dependencies-and-reuse/SKILL.md`:
     - Trace callers and reuse existing shared Slint primitives (`PopupDialog`, `PopupButton`, `PopupField`, `PopupSelect`, etc.) before writing new code.
     - Keep every production `.rs` and `.slint` file `<= 400` lines and every PR slice `<= 445` changed lines.
     - Document every new crate in a Dependency Decision Record and evaluate reusable abstractions via the Reuse Candidate template.
