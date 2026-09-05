STACK ADAPTER — cross-stack rust + typescript (dashboard / tauri / wasm / gRPC patterns)

Loaded by orchestrate IN ADDITION TO rust.md AND typescript.md when detect-stack.ps1 reports composite stack="rust+typescript" (e.g. ELAI's .elai_cc/ workspace + .elai_cc/crates/dashboard-v2/ Svelte dashboard).

The premise: a Rust crate exposes APIs (Tauri commands, WASM exports, gRPC services, REST handlers) consumed by a TS frontend. Changes on the Rust side that ARE NOT mirrored in the TS bindings ship a silent regression — the dashboard misses the new capability.

This adapter adds bindings-drift detection on top of the per-stack adapters.


WHICH BINDING SCHEME

The adapter probes the Rust crate at compat.dashboard_wiring_paths or compat.rust_api_crate:
- Cargo.toml dependencies on `tauri-specta` or `specta` > scheme = "specta" (Tauri command bindings)
- Cargo.toml dependencies on `ts-rs` > scheme = "ts-rs" (manual #[derive(TS)] annotations)
- Cargo.toml dependencies on `wasm-bindgen` + `tsify` > scheme = "wasm-bindgen"
- Cargo.toml dependencies on `prost` or `tonic` + `*.proto` files > scheme = "gRPC"
- presence of `bindings.ts` or `<dashboard>/src/lib/api/generated.ts` + custom emit script > scheme = "custom"
- nothing detected > scheme = "manual" (binding drift detection falls back to grep-based)

The compat file SHOULD declare scheme explicitly (compat.bindings_scheme) — auto-detect is a fallback.


THE CROSS-STACK GATES

GATE XS1 — BINDINGS UP TO DATE (= no drift)
  Run the emit command per scheme:
    specta:     cargo run --bin emit-bindings  (or whatever the project wires; compat.bindings_emit_command)
    ts-rs:      cargo test --features ts-rs (ts-rs emits during its own test runs by convention)
    wasm-bindgen: wasm-pack build --target web (regenerates .d.ts)
    gRPC:       buf generate  (or prost-build runs via build.rs automatically — check `target/build/.../out.rs` timestamp)
    custom:     compat.bindings_emit_command verbatim
  
  After emit: git diff --exit-code -- <bindings_paths>
  Expected: zero diff (the committed bindings match what re-emission produces)
  
  bindings_paths typically:
    specta:     <dashboard>/src/lib/bindings/index.ts
    ts-rs:      <dashboard>/src/lib/bindings/*.ts
    wasm-bindgen: <dashboard>/src/lib/wasm/*.d.ts
    gRPC:       <dashboard>/src/lib/proto/*.ts
    custom:     compat.bindings_paths

GATE XS2 — EVERY NEW RUST API HAS A TS CONSUMER
  Diff the bindings file (committed HEAD vs HEAD~1) to find added type / function exports:
    git diff HEAD~1 HEAD -- <bindings_paths> | rg "^\+(export|declare)"
  For each added export, grep the dashboard src/ for at least one importer:
    rg "<exported_name>" <dashboard>/src --type ts --type svelte
  Expected: at least 1 match per added export. Zero match = "Rust API X added with no TS consumer" = gate fail.
  
  Exception: if the WIRING_CONTRACT.consumes_symbols block in the plan explicitly lists "<symbol_X> — frontend wiring deferred to next phase", skip the check for that symbol and emit a soft warning.

GATE XS3 — NO ORPHANED TS BINDINGS
  Inverse direction: for each export in <bindings_paths>, verify the Rust symbol still exists:
    grep the bindings file for exported names, grep the Rust source for the corresponding #[tauri::command] / #[specta::specta] / #[wasm_bindgen] / proto service entry
  Expected: every TS binding has a Rust origin. Stale = dead-code in dashboard.

GATE XS4 — SCHEMA / TYPE COMPATIBILITY
  For gRPC: buf breaking --against ".git#branch=HEAD~1"
    Expected: no breaking schema changes OR plan explicitly says "schema-breaking change accepted in this orchestrate run" via compat.schema_breaking_allowed=true
  For specta / ts-rs: type changes are inherent to bindings drift; covered by XS1
  For wasm-bindgen: wasm-bindgen's own type-checking catches incompatible signatures at build time

GATE XS5 — DASHBOARD-WIRING PRESENCE (per user's brief)
  If the plan adds ANY user-facing capability AND compat.dashboard_wiring_paths is non-empty:
    Check that at least one file under compat.dashboard_wiring_paths was touched in the orchestrate run:
      git diff HEAD~<num_sub_phases> HEAD --name-only | rg "^<dashboard_wiring_paths>"
  Expected: at least 1 match.
  Zero match + plan added user-facing capability = "feature added with no dashboard wiring" gate fail.
  
  "User-facing capability" detection (heuristic):
    plan WIRING_CONTRACT.observable_effect.trigger non-empty AND non-test
    OR plan touches files matching "*/commands/*.rs" or "*/handlers/*.rs" or "*/api/*.rs" or "*/lib/*.svelte" or "*.vue" or "*.tsx"
    OR plan.acceptance contains "dashboard" / "drawer" / "panel" / "view" mentions
  
  Override: plan can include "backend-only-justification: <reason>" in its content; that suppresses XS5 with the reason recorded in the report.


PER-SUB-PHASE SCOPING

Bindings drift is mostly a full-sweep concern. Per-sub-phase only run XS5 (dashboard wiring presence). The other XS gates run at full sweep + final commit boundary.

XS5 per sub-phase: if the sub-phase added user-facing capability AND zero touched files in dashboard_wiring_paths AND the sub-phase wasnt the last one in the plan: emit a WARNING (not BLOCK) — the next sub-phase may handle dashboard wiring. Track in orchestrate-run.md.

At final sweep: XS5 BLOCKS if no dashboard touches happened across the whole run.


COMMAND TEMPLATES (orchestrate-launcher.ps1)

per-sub-phase XS5:
  $changed = git diff --name-only HEAD~1 HEAD
  $touchedDashboard = $changed | Where-Object { $_ -match $env:DASHBOARD_WIRING_PATHS_REGEX }
  if ($touchedDashboard.Count -eq 0 -and $isUserFacingSubPhase) {
    Write-Warning "sub-phase added user-facing capability with no dashboard touch; may resolve in next sub-phase"
  }

full sweep XS1-XS4:
  Push-Location $RUST_WORKSPACE_ROOT
  Invoke-Expression $env:BINDINGS_EMIT_COMMAND
  git diff --exit-code -- $env:BINDINGS_PATHS; if ($LASTEXITCODE -ne 0) { exit 11 }
  Pop-Location
  
  Push-Location $DASHBOARD_ROOT
  # for each added export
  $added = git diff HEAD~$N_SUBPHASES HEAD -- $env:BINDINGS_PATHS | rg "^\+(export|declare)"
  foreach ($line in $added) {
    $exportName = $line -replace '^\+(export|declare)\s+(?:const |type |interface |function )?(\w+).*','$2'
    $consumers = rg $exportName src --type ts --type svelte
    if (-not $consumers) { Write-Host "no TS consumer for added export: $exportName"; exit 12 }
  }
  Pop-Location

full sweep XS5:
  $changedAll = git diff --name-only HEAD~$N_SUBPHASES HEAD
  $touchedDashboard = $changedAll | Where-Object { $_ -match $env:DASHBOARD_WIRING_PATHS_REGEX }
  if ($touchedDashboard.Count -eq 0 -and $planAddedUserFacing -and -not $env:BACKEND_ONLY_JUSTIFICATION) {
    Write-Host "plan added user-facing capability but no dashboard files touched"
    exit 15
  }


REPORTING

Cross-stack gate results in workflow/sessions/<sid>/phase-reports/<sub_phase_id>.md under "CROSS-STACK GATES" section.
Final sweep XS results in workflow/sessions/<sid>/orchestrate-final.md "BINDINGS + WIRING" section.


END stack-adapters/cross-stack-rust-ts.md
