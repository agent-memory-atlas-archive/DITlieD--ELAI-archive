B-11 Design System Restoration — DECISIONS
Date: 2026-06-05 (backfill)
Source: workflow/sessions/s-plan-audit-2026-06-04/plan-status/B/B-11_design_system_restoration.md

SHIPPED
- tokens.css (crates/dashboard-v2/src/styles/tokens.css) with the exact spec palette: #0a0e12
  bg-void, #0f1318 bg-base, #1B7A6E teal, #7C5CBF purple, #d0d7de text-primary. Off-spec colours
  (#0b0d12, #0c0e13) absent.
- JetBrains Mono set as --font-sans (token name retained for back-compat per C-6); Inter font absent
  from src/; --font-ui absent.
- Lucide icon set imported via @lucide/svelte in Rail/Sidebar/Drawer/Panel/CommandPalette/RecoveryModal.
- Reachability: dashboard root mounts base.css + tokens.css, imported by all components (frontend-only
  plan; no Rust caller required).

KEY DECISIONS
- C-6 back-compat deviation: kept the --font-sans token NAME even though the value is JetBrains Mono
  (a monospace), to avoid renaming every consumer. Documented as intentional.

DEFERRED / NOT DONE (genuine gaps)
- Type scale: 187 off-spec font-size instances remain across canvas/* and components/* (11/14/16/17/
  20px); spec scale is 10/12/13/15/18px. Many are canvas SVG labels.
- line-height: base.css body is 1.5; spec requires 1.6 (text) and 1.4 (code/pre). Not yet split.
- Visual regression test crates/dashboard-v2/tests/visual.spec.ts: ABSENT.

NOTES
- No ELAI_*_ENABLE flag (no allowlist row). wiring_contract_check FAILs are false negatives: CSS
  custom properties and JS/Svelte imports are not indexed as callable symbols in elai.map.
