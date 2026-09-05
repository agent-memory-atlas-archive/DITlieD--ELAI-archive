Phase A — dashboard-v2 website-creation wiring (session s-69820)

What I will implement
- New `scaffold::fim` module with FIM marker discovery, splice, Qwen prompt builder, and post-hoc artifact stripping. Pure string operations — no new deps.
- New `scaffold::build_vars` module with `default_build_vars()` so DashboardScaffoldApplier and contract tests share one source of truth for the seven template variables.
- `crates/dashboard-v2/src-tauri/src/scaffold_applier.rs` — port of DashboardScaffoldApplier from .bkp_old, using `scaffold::default_build_vars` and a `default_templates_dir()` helper that respects `ELAI_TEMPLATES_DIR` first and falls back to `CARGO_MANIFEST_DIR/../../../templates/web` (three uplift levels for src-tauri layout, not two).
- AppState gains `dev_servers: Arc<Mutex<BTreeMap<String, DevServerHandle>>>`, mirroring the existing active_streams BTreeMap pattern.
- Six new Tauri commands appended to commands.rs: `list_scaffold_templates`, `apply_scaffold_template`, `fim_fill_section`, `run_web_build_check`, `start_dev_server_for_project`, `stop_dev_server_for_project`. The existing `get_dev_servers` stub gets replaced with a registry read so the WebQualityDrawer sees real handles.
- Frontend: extend `ScaffoldTemplate` interface in types.ts, replace mock SCAFFOLD_TEMPLATES wiring in WebQualityDrawer.svelte with a real `invoke("list_scaffold_templates")` + `invoke("apply_scaffold_template", …)` round-trip via Tauri 2's `@tauri-apps/api/core` import path.

Key technical decision
The orchestrator crate cannot host a ScaffoldApplier impl (it deliberately does not depend on `scaffold`). The dashboard-v2 crate already pulls `scaffold` and `elai-web-validator`, so the applier and the dev-server registry both live in dashboard-v2. The FIM parser and the `default_build_vars` helper move into the `scaffold` crate so contract tests can hit them without linking against dashboard-v2 (which still has the disabled-contract problem from s-1430428).

Expected test outcome
`cargo test -p elai_cc_contracts --test dashboard_scaffold_phase_a_contract` compiles after scaffold::fim and scaffold::build_vars land; all 14 tests pass. `cargo check -p elai-dashboard-v2` exits 0 once the new commands compile against the existing reqwest::Client + sidecar_url state fields.

Unknowns to verify empirically
1. Whether the abliterated Qwen2.5-Coder model still emits coherent FIM output when given the special-token sandwich (model may have lost FIM tokens during abliteration). Mitigation: Approach B fallback already specced in code path; cannot test live without booting the sidecar (HARDWARE-HEAVY — out of scope for Phase A unit tests).
2. Whether Tauri 2's invoke signature on the Svelte side is `invoke<T>(cmd, args)` with `args` as object, or has shifted in the installed `@tauri-apps/api ^2`. Verified at code-edit time by reading existing `WebQualityDrawer.svelte` invoke calls.
3. Whether `scaffold::apply_template_from_root` rejects non-empty target dirs gracefully (yes — verified at apply.rs:43 with `TargetDirNotEmpty`).
