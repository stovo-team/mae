---
name: analyzing-react-source
description: Use when implementing or reviewing React/TypeScript source analysis for a migration, especially components, centralized config barrels (radii/sizes/motion/colors/strings), design-system tokens, state, effects, routing, and browser-specific capability gaps.
---

# Analyzing React Source

## 1. Core Principle

Prefer deterministic compiler/static/runtime evidence over LLM inference, and exclude all test artifacts (`*.test.*`, `*.spec.*`, `*.stories.*`, `__tests__`, `test`, `tests`, `testUtils`, `test-fixtures`, `e2e`) so production metrics and gap reports are never polluted by test files.

## 2. Required Extractions

1. **Centralized Config & Design Tokens ("The Configs Thing"):**
   - Inspect `src/config/` (`brand.ts`, `radii.ts`, `sizes.ts`, `motion.ts`, `colors/`, `strings/`, `profileManifest.ts`), `src/design-system/` (`tokens.css`, `palette.ts`), and `src/appearance/AppearanceContext.tsx`.
   - Extract every numeric radius (bubble presets, slider bounds, control, card, surface, panel, pill), icon/layout size, font scale/weight/line-height, semantic color alias, and bilingual string catalog key so the target `<app>-config` crate and `ui/theme/tokens.slint` mirror them 1-to-1.
2. **Component & Route Hierarchy:**
   - Extract all route paths (`<Route path="..." />`), tab order/motion rules, shared UI primitives (`components/ui/Popup*`), and feature modules (`features/*`, `pages/*`).
3. **Browser & Platform API Capability Gaps (`GapReasonCode`):**
   - Classify browser-only APIs (`window.localStorage`, `sessionStorage`, `IndexedDB`, `crypto.subtle`, `navigator.mediaDevices`, `WebRTC`/`LiveKit`, `<canvas>`, `Capacitor` plugins) with exact `GapReasonCode`s and map each to an **OS-independent Rust Port trait** in `<app>-domain` backed by a concrete adapter crate (`<app>-adapter-*`).

## 3. Output Discipline

Emit versioned artifacts (`SourceInventory`, `ModuleDependencyGraph`, `SemanticUiModel`, `CapabilityReport`) with explicit provenance:
- `DeterministicStatic`
- `RuntimeObservation`
- `InferredSemantic`
- `UnknownGap`

Never hide uncertainty or silently drop unsupported browser behavior.
