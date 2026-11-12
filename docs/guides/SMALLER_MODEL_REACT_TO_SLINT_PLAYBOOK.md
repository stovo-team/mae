# Smaller-Model Playbook: React/TypeScript → Native Rust + Slint Desktop Migration

> **Audience:** Smaller and fast coding models (as well as autonomous worker agents) executing bounded migration tasks within the MAE framework.
> **Core Rule:** Treat every task as a **bounded contract-and-evidence problem**. Follow **Ponytail** minimal-complete-change discipline, **DDD + Clean Architecture Modular Monolith** boundaries, and **100% Config/Token-Driven UI & Curviness Parity**.

---

## 1. Golden Rules for Smaller Models (Read First)

1. **Never Exceed Context or File Budgets:**
   - Read only the assigned React source files (`.tsx`, `.ts`, `.css`), the shared `config/` + `tokens.slint` files, and the immediate Rust callers for your task slice.
   - **Hard File Limit:** Every production `.rs` and `.slint` file MUST stay **<= 400 lines**. If a file approaches 350 lines, split it into focused submodule files (`mod.rs` + child modules or `ui/pages/<feature>/<subview>.slint`).
   - **Hard PR Slice Limit:** Keep each PR / commit slice focused (300–400 lines target, **<= 445 added+deleted lines** hard cap).
2. **Strict Layer Separation (DDD + Clean Architecture Modular Monolith):**
   - `*-config` crate: Pure Rust structs/constants for `brand`, `radii` (curviness), `sizes`, `motion`, `colors`, `strings` (i18n), and `AppearanceSettings`. **Zero I/O.**
   - `*-domain` crate: Pure Rust Entities, Value Objects, Domain Events, and Port `trait`s. **Zero external I/O crates (`std` only).**
   - `*-application` crate: Pure Rust `AppState`, `AppEvent`, `AppRoute`, feature use-cases, and deterministic `reduce(&mut AppState, AppEvent)`. **Zero Slint or OS imports.**
   - `*-adapter-*` crates: Concrete implementations of Domain Ports (`storage`, `network`, `crypto`, `media`, `platform`). Put all `#[cfg(target_os = "windows")]` code strictly inside `*-adapter-platform` with a cross-platform default so Linux/Windows `cargo test` always compiles and runs headlessly.
   - `*-desktop-app` crate: `src/main.rs` (with `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]`), `src/ui_bridge.rs`, and declarative `ui/**/*.slint`. **Never put business logic, HTTP calls, or state reducers inside `.slint` files.**
3. **Native Compliance Gate (Hard Fail):**
   - **NEVER** add `wry`, `tauri`, `webview2`, `webkit2gtk`, `electron`, `headless_chrome`, or static screenshot images pretending to be controls. `NativeComplianceVerifier` scans `Cargo.toml` and `src/**/*.rs` and will immediately reject the task.
4. **Config-Driven UI & Curviness Parity (Hard Gate):**
   - **NEVER** hardcode hex/rgb colors (`#2aabee`, `rgba(...)`), corner radii (`border-radius: 18px`), or font sizes (`font-size: 14px`) inside feature `.slint` files.
   - Always import and bind `ThemeTokens` from `ui/theme/tokens.slint` (e.g., `border-radius: ThemeTokens.radius-bubble;`, `background: ThemeTokens.color-surface;`, `font-size: ThemeTokens.font-size-small;`).
   - For pill shapes (`--radius-pill: 9999px` or `50%` circle in CSS), use `border-radius: self.height / 2;` or `ThemeTokens.radius-pill` in Slint so the curvature is mathematically exact at any height.

---

## 2. Step-by-Step Task Execution Loop (6 Steps)

For every migration ticket/task, execute these exact 6 steps in order:

### Step 1 — Trace Callers & Reuse Existing Primitives (Ponytail Rule)
Before writing a single line of code:
1. Read the React source component(s) and trace their imports:
   - Which `config/` values (`radii`, `sizes`, `motion`, `colors`, `strings`) do they use?
   - Which shared UI primitives (`PopupDialog`, `PopupBackdrop`, `PopupCloseButton`, `PopupButton`, `PopupField`, `PopupSelect`, `Avatar`, `Badge`) do they render?
   - Which state hooks (`useState`, `useReducer`, `useContext`) and side effects (`useEffect`, `fetch`, socket events) do they trigger?
2. Search the target Rust + Slint repo (`rg`) for existing components, tokens, strings, and events. **Reuse or extend existing shared components; never copy-paste a parallel dialog, button, input, or dropdown.**

### Step 2 — Check Dependencies ("Dependicles" Rule)
1. Can this task be solved with `std`, existing workspace crates, or already-installed dependencies in `[workspace.dependencies]`? **If yes, add zero dependencies.**
2. If a new crate is genuinely required for non-trivial standard functionality (e.g., SQLite, AES-GCM/RSA-OAEP, HTTP/WebSocket, audio/WebRTC):
   - Prefer **pure-Rust, OS-independent crates** (e.g., `reqwest` with `rustls-tls`, `rusqlite` with `bundled`, `aes-gcm`, `rsa`, `tiny-skia`) so Windows and Linux builds both succeed without native C/OpenSSL toolchain headaches.
   - Record a **Dependency Decision Record** in `docs/dependencies/` covering version, license, maintenance, advisories, OS-independence, and rejected alternatives.

### Step 3 — Update Config & Localization Catalogs First
1. If the UI slice introduces or consumes UI copy, ensure the exact key exists in **both** English (`strings/en.rs`) and Russian (`strings/ru.rs`) catalogs in the `*-config` crate.
2. If the UI slice relies on a radius, size, or semantic color token, verify it exists in `*-config` and `ui/theme/tokens.slint`.

### Step 4 — Implement Pure Rust Domain & Application State/Reducer (TDD)
1. Write a failing unit test in `*-domain` or `*-application` for the slice's state transitions, validation rules, and edge cases.
2. Add the minimal state fields to `AppState` (or the feature sub-state struct), variants to `AppEvent`, and transition logic in `reduce()`.
3. Run `cargo test -p <crate>` and confirm the unit test passes headlessly.

### Step 5 — Implement Declarative Slint UI Component (`ui/**/*.slint`)
Translate JSX + CSS into idiomatic Slint 1.x following the **Slint Anti-Hallucination Cheat-Sheet** below:

| React / CSS Construct | Slint 1.x Declarative Equivalent | Common Smaller-Model Hallucination (FORBIDDEN) |
| :--- | :--- | :--- |
| `<div className="flex flex-col gap-3 p-4">` | `VerticalLayout { spacing: ThemeTokens.spacing-md; padding: ThemeTokens.padding-md; ... }` | `div { display: flex; }` or `margin: 12px;` (Slint has no `margin` property; use layout `spacing` and `padding`!) |
| `<div className="flex flex-row items-center">` | `HorizontalLayout { alignment: LayoutAlignment.start; spacing: ...; ... }` | `flex-direction: row;` |
| `border-radius: var(--radius-bubble)` | `Rectangle { border-radius: ThemeTokens.radius-bubble; clip: true; ... }` | `borderRadius: "18px"` or raw `border-radius: 18px;` |
| Pill button (`height: 44px; border-radius: 9999px`) | `Rectangle { height: 44px; border-radius: self.height / 2; ... }` | `border-radius: 50%;` (Slint requires `length`, not `%`, for `border-radius`) |
| Circle avatar (`width: 34px; height: 34px; border-radius: 50%`) | `Rectangle { width: 34px; height: 34px; border-radius: self.width / 2; clip: true; ... }` | `border-radius: 50%;` |
| `onClick={() => onAction(id)}` | `TouchArea { mouse-cursor: MouseCursor.pointer; clicked => { root.action-triggered(root.item-id); } }` | `onClick => { ... }` on `Rectangle` |
| `onMouseEnter` / `:hover` CSS | `ta := TouchArea {}` + `background: ta.has-hover ? ThemeTokens.color-surface-hover : ThemeTokens.color-surface;` | `:hover { background: ... }` |
| `<input value={v} onChange={...} />` | `TextInput { text <=> root.value; edited => { root.value-changed(self.text); } }` | `<input>` tag |
| Overflow scrolling (`overflow-y: auto`) | `ScrollView { VerticalLayout { ... } }` (with thin `4px` custom scrollbar primitive) | `overflow: scroll;` |
| Conditional render `{open && <Modal />}` | `if root.open : PopupDialog { ... }` | `{root.open && <PopupDialog />}` |
| Array map `{items.map(i => <Row item={i} />)}` | `for item[idx] in root.items : Row { data: item; index: idx; }` | `.map(...)` |

### Step 6 — Run Deterministic Verification Gates
Never claim a task is complete until every required command exits with `0`:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bash scripts/check-solid-boundaries.sh
bash scripts/check-slint-tokens.sh
```
End your task report with a 1–2 line Ponytail summary stating what was verified, any intentional shortcut (`shortcut: <limit>, <when to upgrade>`), and any remaining risk.
