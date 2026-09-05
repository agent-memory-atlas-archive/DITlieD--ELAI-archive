# E-47: LoRA fine-tuning pipeline — RESEARCH-GRADE / DEFERRED — spec'd but ship-gated behind hard "DO NOT DEFAULT-ON" flag
Objective: Spec the smallest-reversible LoRA fine-tuning pipeline for ELAI (Unsloth + QLoRA-NF4 + PiSSA init + DoRA + OPLoRA forgetting-mitigation + vLLM hot-swap) AND ship a hard ship-gate that keeps it off by default. The report's own conclusion is DEFER — E-47 captures the pipeline so a future decision is one-config-flip away, not a ground-up implementation, but explicitly does NOT ship a default-on LoRA path because ProjectProfile + CodeRAG already capture the style signal the LoRA would chase.
Estimated effort: 2 sessions, ~8 h (spec + ship-gate ONLY; if user ever flips to ship, an additional ~3-4 sessions for the actual training subpackage + per-tenant adapter management)
Pre-requisites: Sidecar canonical remote setup (CLAUDE.md REMOTE-SIDECAR block — provides the Qwen2.5-Coder-7B-Instruct-abliterated base), B-37 SHIPPED (RuntimeOrchestrator + PerRoleRuntime — adapter selection attach point), D-28..D-30 (PLANNED — sibling self-mod surfaces with their own DEFER discipline).
Parallel-safe with: E-46 (different surface), E-2 (local LLM speed tricks — coordinates on quantization choice), E-20 (local-cloud routing — coordinates on adapter availability gating).
Source report: workflow/improvement-sweep-2026-05-19/reports/47-lora-finetuning.md

## Context

The May 2026 LoRA + domain fine-tuning survey concludes DEFER for ELAI with five grounded reasons: (1) ELAI's existing ProjectProfile + prompt-rules + few-shot pipeline already captures the per-project style signal; (2) Anthropic's 2025 Contextual Retrieval and Anthropic Context Engineering writeups argue retrieval gains dominate fine-tune gains for code-style tasks once the base model is competent — and Anthropic ships Claude Code WITHOUT per-project fine-tunes; (3) CodeRAG-Bench (arXiv:2406.14497, NAACL 2025 Findings) ran 10 retrievers x 10 LMs and reports the bottleneck is retriever + generator-context use, NOT lack of fine-tuning; (4) Biderman et al. (arXiv:2405.09673) measured 5-19 pp catastrophic forgetting on out-of-domain code from standard LoRA, and OPLoRA (arXiv:2510.13003) confirmed Qwen2.5-7B specifically forgets without their orthogonal-projection fix; (5) base-model patch (e.g. Qwen2.5-Coder-7B → v0.2) breaks LoRA validity and PortLLM (arXiv:2410.10870) is nascent. The non-recommendation, if a user pushes ahead anyway, is: per-user voice LoRA (NOT per-project), Unsloth + QLoRA-NF4 + PiSSA init + DoRA, bound forgetting with OPLoRA — the smallest reversible bet. E-47 captures THAT pipeline as spec-only with a hard ship-gate, so the deferred-but-readable plan is available without committing the runtime risk.

Crucially: this E-N is RESEARCH-GRADE / DEFERRED. It MUST NOT ship a default-on LoRA path. The acceptance criteria below treat "the ship-gate is provably off-by-default" as a first-class requirement.

## Existing state (B/C/D coverage)

- Sidecar canonical setup (CLAUDE.md REMOTE-SIDECAR block): runs huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated (NF4 4-bit, SDPA, ~1.92 GB VRAM) on remote 3090. The 7B abliterated variant lives locally at `F:\.ELAI_workflow\.elai_cc\models\Qwen2.5-Coder-7B-Instruct-abliterated\`. Both are HF-ecosystem-standard with no exotic layer layout per the report's base-model card check. EXTENSION: E-47 spec'd pipeline targets the 7B abliterated for LoRA but verifies-first that PEFT defaults still apply (open question Q2 in the report).
- B-37 SHIPPED (Multi-Runtime Backend Abstraction Phase 4M): provides `RuntimeAdapter` + `PerRoleRuntime` + `RuntimeOrchestrator` + 11 backends. EXTENSION: E-47 spec'd a `LoraAdapterSelector` that overlays on PerRoleRuntime at the `--lora-modules` flag level (vLLM-style); backward-compatible additive when off.
- D-28..D-30 PLANNED (sibling self-mod surfaces with DEFER discipline): structural precedent for "spec the surface, gate the ship" pattern.
- ProjectProfile (`crates/core/src/project_profile.rs::ProjectProfile` per elai.map line 287925, `crates/knowledge/src/incremental_profile.rs::load_or_detect_profile` per line 26144): provides the few-shot signal that the LoRA would ostensibly chase. CodeRAG-Bench evidence says ProjectProfile + retrieval is already the high-leverage move; E-47 documents the gap explicitly.
- Existing few-shot prompt-rules pipeline (per ELAI's Tier-2 ProjectProfile routing): style signal at >80% snake_case threshold etc. is already shipped via few-shot — this is exactly the "context engineering" cell in Anthropic's matrix, not the fine-tune cell.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\47-lora-finetuning.md` (full)
- `F:\.ELAI_workflow\.elai_cc\python\elai_inference\server.py` (existing sidecar — verify load_in_4bit + SDPA defaults; LoRA hot-swap attach point if shipped)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_orchestrator.rs` (B-37 PerRoleRuntime — verify backend selection interface)
- `F:\.ELAI_workflow\.elai_cc\crates\core\src\project_profile.rs` (ProjectProfile shape — input to dataset builder if ever shipped)
- `F:\.ELAI_workflow\CLAUDE.md` REMOTE-SIDECAR block (canonical sidecar config; LoRA path must not break this)
- workflow/papers/qlora-2305.14314-interface.md (after paper-ingestor mode=paper)
- workflow/papers/pissa-2404.02948-interface.md (after paper-ingestor mode=paper)
- workflow/papers/dora-2402.09353-interface.md (after paper-ingestor mode=paper)
- workflow/papers/oplora-2510.13003-interface.md (after paper-ingestor mode=paper — measured on Qwen2.5-7B)
- workflow/papers/biderman-2405.09673-interface.md (after paper-ingestor mode=paper — 5-19 pp forgetting)
- workflow/papers/coderag-bench-2406.14497-interface.md (after paper-ingestor mode=paper)
- https://unsloth.ai/blog/qwen-coder (WebFetch — 51 min full FT 7B; LoRA-16bit in 24GB)
- https://huggingface.co/docs/peft/developer_guides/lora (WebFetch — PEFT current API 2025/2026; `target_modules="all-linear"`, `use_dora=True`)
- https://docs.vllm.ai/en/latest/features/lora/ (WebFetch — multi-adapter hot-swap, `load_inplace`)

## Tasks

1. Ship the hard "DO NOT DEFAULT-ON" gate FIRST and ALONE. Before any other E-47 task, land a config schema in `.elai_cc/elai_cc.toml` under `[finetune.lora]` with: `enabled = false`, `ship_gate = "DEFERRED_PER_E47_REPORT"` (string literal — string-match enforced), `confirm_user_understanding_of_forgetting_risk = false`, `last_calibrated_on_qwen_v = ""`. Add a startup check in `crates/cli/src/main.rs` boot path: if `enabled = true` AND `confirm_user_understanding_of_forgetting_risk = false`, REFUSE to load any LoRA adapter and emit a clear stderr message citing arXiv:2405.09673 5-19 pp forgetting evidence + report 47 section "Recommendation for ELAI". Emit `lora_ship_gate_refused` SSB row. The string `DEFERRED_PER_E47_REPORT` is required because it forces the user to read the report when overriding. This Task 1 is the MINIMUM viable E-47 ship — every subsequent task is gated on it.

2. Spec a Python subpackage layout `python/elai_finetune/` (NOT YET CREATED — spec only at this stage) covering: `dataset.py` (ProjectProfile-driven Q&A pair builder; min 1K examples), `train.py` (Unsloth `FastLanguageModel.from_pretrained` + `get_peft_model` with `LoraConfig(init_lora_weights="pissa", use_dora=True, target_modules="all-linear", r=16, lora_alpha=32, ...)`), `eval.py` (small held-out probe set 50-200 examples, NOT full workspace per the report's anti-pattern), `serve_adapter.py` (vLLM hot-swap helper). The spec lives in a new markdown `python/elai_finetune/SPEC.md` (~250 LOC), NOT in `.py` source files. Spec-only because (a) shipping the source files invites "just turn it on" pressure; (b) the spec is what we want to converge on the day the deferral lifts.

3. Spec the verification gate that MUST pass before any user can flip `enabled = true`. The gate runs: (a) calibration probe — measure base-model quality on the project's probe set BEFORE training, store in `metrics/lora_baseline.toml`; (b) train run with logged loss curves; (c) post-training eval on (i) the project probe set (in-domain), (ii) a 200-example out-of-domain probe set (Rust + Python + TS mixed) to quantify catastrophic forgetting per Biderman 2024 + OPLoRA evidence; (d) hard refuse if forgetting > 5 pp on OOD set (configurable threshold, NOT softened by default). Spec lives in `python/elai_finetune/SPEC.md` section "Calibration + Refuse Conditions". Emits `lora_calibration_gate_passed` / `lora_calibration_gate_refused` SSB rows when implemented.

4. Spec the adapter-management surface that B-37 RuntimeOrchestrator would consume IF shipped. New (spec-only) symbol `runtime_orchestrator::LoraAdapterSelector { project_id: ProjectId, adapter_path: PathBuf, base_model_version: String }`. Selector consulted on every inference call IF the `[finetune.lora].enabled = true` AND adapter is registered for the active ProjectProfile. Selector REFUSES to route to the adapter when `base_model_version` differs from the currently loaded sidecar's reported model version (defense against the base-model-patch breakage flagged in the report). Spec only; do not emit symbols yet because Task 1 ship-gate prevents runtime activation. Wiring contract in this E-N's section below uses `produces_symbols (DEFERRED)` markers to make the deferred status explicit.

5. Spec the multi-adapter hot-swap surface (vLLM `--lora-modules` + aLoRA cross-model KV-cache reuse per arXiv:2512.17910). For the deferred-but-readable plan: the remote sidecar boot command in CLAUDE.md REMOTE-SIDECAR block would gain optional `--lora-modules=name=path,...` args; the local sidecar load_in_4bit code path would unchanged-but-augmented with a PEFT `PeftModel.from_pretrained` overlay call gated on the `enabled` flag. Per-project adapter storage layout `~/.elai/adapters/{project_id}/{adapter_sha}.safetensors` with a sidecar manifest `~/.elai/adapters/manifest.toml` (`{project_id, adapter_sha, base_model_version, trained_at_unix, forgetting_pp_ood, dataset_sha}`). Spec only.

6. Spec the per-user voice LoRA (NOT per-project — the report's non-recommendation if user pushes ahead). The voice signal is small (chat-only, NOT code) so the spec uses rank=8 (vs r=16 for code) and a 500-example dataset from the user's own historic chat transcripts (NOT auto-collected — explicit one-time opt-in via `elai finetune voice --consent-yes`). Voice adapter is overlaid ONLY on the `assistant_primary_chat` role per PerRoleRuntime — never on code-emitting roles. Spec only; same ship-gate as Task 1.

7. Document the anti-patterns from the report directly inside `python/elai_finetune/SPEC.md` "Anti-patterns" section, verbatim from the report so the spec carries the warning forward: (a) fine-tuning Qwen on single-language project then expecting cross-language quality (5-19 pp drop per arXiv:2405.09673); (b) stacking multiple LoRAs at inference without merging strategy (arXiv:2508.11624 LoRAtorio drift to base); (c) DoRA with same LR for magnitude and direction (FinLoRA arXiv:2505.19819 underperforms); (d) trusting LoRA validity across base-model patches (no paper supports; PortLLM nascent); (e) skipping Qwen2.5 pad_token bug (Unsloth docs flag `<|im_start|>` / `<|im_end|>` as untrained); (f) treating "abliterated" base as identical to stock Qwen-Coder (layer behavior may have drifted; baseline must be re-measured); (g) running full workspace tests during fine-tune iteration (Spectrum/GaLore/Q-GaLore papers flag this; use 50-200 example probe set).

8. Spec the operator runbook for a "decide to ship" flip. The decision gate has THREE blocking measurements that MUST be filled before flipping `enabled = true`: (1) the report's open question Q1 — measured gap between current ELAI prompt-rules+few-shot and Qwen-Coder-7B + QLoRA-PiSSA-DoRA on a project-style probe set; (2) Q2 — confirmed PEFT defaults still work on the abliterated 7B (or amended target_modules); (3) Q4 — OPLoRA forgetting-mitigation holds for ELAI's worst-case domain gap. Without all three numbers, the gate stays closed. Runbook lives at `workflow/decisions/E-47-flip-runbook.md` (NOT under `workflow/reasoning/` because flip is operator action, not session reasoning).

9. Dashboard panel `LoraShipGatePanel.svelte` (~80 LOC). Single-purpose: shows current `[finetune.lora]` config block, reads `lora_ship_gate_refused` SSB rows, and displays the calibration probe results IF the user has run them. The panel intentionally has NO "Enable" button — flipping `enabled = true` requires editing `elai_cc.toml` by hand, surfacing intent. Per the report's first-principles reason for DEFER, the dashboard reinforces that intent surfaces are the right control plane here.

10. Decisions log `workflow/reasoning/E-47-decisions.md` capturing the DEFER decision evidence (report citations + measured ELAI state) AND a "revisit triggers" section: revisit if (a) ELAI measures few-shot saturation on the project-style probe set (Q1 returns a 5+ pp gap that few-shot cannot close); (b) Anthropic/Qwen publishes new fine-tune-vs-RAG measurement that flips the conclusion; (c) a user explicitly requests voice LoRA AND consents to per-user data collection per Task 6.

## WIRING_CONTRACT

produces_symbols:
  - config::finetune::LoraConfig { enabled: bool, ship_gate: String, confirm_user_understanding_of_forgetting_risk: bool, last_calibrated_on_qwen_v: String }
  - cli::main::lora_ship_gate_refused() -> SsbRow (boot-path emit)
  - (DEFERRED, spec-only) python::elai_finetune::dataset::DatasetBuilder (per Task 2)
  - (DEFERRED, spec-only) python::elai_finetune::train::QwenLoraTrainer (per Task 2 — Unsloth + QLoRA + PiSSA + DoRA)
  - (DEFERRED, spec-only) python::elai_finetune::eval::ForgettingProbe (per Task 3)
  - (DEFERRED, spec-only) runtime_orchestrator::LoraAdapterSelector { project_id, adapter_path, base_model_version } (per Task 4)
  - (DEFERRED, spec-only) adapter manifest layout `~/.elai/adapters/manifest.toml` (per Task 5)
consumes_symbols:
  - crates/cli/src/main.rs::main (ship-gate check; refuses load)
  - crates/orchestrator/src/runtime_orchestrator.rs::PerRoleRuntime (DEFERRED — adapter overlay)
  - core::project_profile::ProjectProfile (DEFERRED — dataset builder input)
  - CLAUDE.md REMOTE-SIDECAR block boot command (DEFERRED — `--lora-modules` arg surface)
reachable_from:
  - crates/cli/src/main.rs (ship-gate check at startup; ALWAYS reachable)
  - python/elai_finetune/SPEC.md (DEFERRED — operator-facing spec only)
  - workflow/decisions/E-47-flip-runbook.md (DEFERRED — operator runbook)
stub_ban:
  - no `unimplemented!()` in the SHIPPED ship-gate code (Task 1)
  - no silent default-flip — if `confirm_user_understanding_of_forgetting_risk = true` is set without prior calibration probe results in `metrics/lora_baseline.toml`, REFUSE
  - no `.unwrap()` in the boot-path ship-gate check (refusal must propagate via `Result`)
  - the string `DEFERRED_PER_E47_REPORT` is a hard token; renaming requires an explicit E-47.1 patch and report-update; CI lints for the token presence
  - no source files under `python/elai_finetune/*.py` may be CREATED by this E-N (spec-only; SPEC.md only)
  - no Cargo dependency on `peft`-like or `unsloth`-like crates added in this E-N (Rust LoRA is not in scope)
  - no automated dataset collection from user transcripts without the `--consent-yes` flag (Task 6 voice LoRA)
frontend_pair: crates/dashboard-v2/src/lib/components/LoraShipGatePanel.svelte (new ~80 LOC — read-only, no Enable button by design)
observable_effect:
  trigger: set `[finetune.lora].enabled = true` AND `confirm_user_understanding_of_forgetting_risk = false` in elai_cc.toml; run `elai`
  expect_ssb: ≥1 `lora_ship_gate_refused` SSB row in current session with payload citing report 47 + arXiv:2405.09673
  expect_artifact: stderr contains the literal substring "DEFERRED_PER_E47_REPORT"; CLI exits with non-zero code or refuses to load LoRA adapter (configurable: refuse-and-continue is acceptable; default = refuse-and-continue with prominent warning)

## DETERMINISTIC_CHECK

```bash
# 1. Ship-gate token present in config schema
rg "DEFERRED_PER_E47_REPORT" .elai_cc/elai_cc.toml
# Expected: ≥1 hit

# 2. Boot-path refuses default-on without confirmation
rg "lora_ship_gate_refused" .elai_cc/crates/cli/src/main.rs
# Expected: ≥1 hit

# 3. SPEC.md authored, no .py files shipped under elai_finetune
rg --files-with-matches . .elai_cc/python/elai_finetune/ --type py
# Expected: empty result (zero .py files)
ls .elai_cc/python/elai_finetune/SPEC.md
# Expected: file exists

# 4. Report's anti-patterns verbatim inside SPEC.md
rg "5-19 pp|catastrophic forgetting|OPLoRA|PiSSA|abliterated base may have drifted" .elai_cc/python/elai_finetune/SPEC.md
# Expected: ≥5 hits

# 5. No peft/unsloth Cargo deps leaked into Cargo.toml
rg "peft\s*=|unsloth\s*=" .elai_cc/crates/*/Cargo.toml
# Expected: zero hits

# 6. Dashboard panel is read-only — no Enable button source
rg "Enable LoRA|enable.*lora.*button|on:click.*enable" .elai_cc/crates/dashboard-v2/src/lib/components/LoraShipGatePanel.svelte
# Expected: zero hits

# 7. Decisions log captures DEFER evidence + revisit triggers
rg "DEFER|revisit triggers|few-shot saturation" workflow/reasoning/E-47-decisions.md
# Expected: ≥3 hits

# 8. Flip-runbook lists THREE blocking measurements
rg "Q1 measured gap|Q2 PEFT defaults on abliterated|Q4 OPLoRA forgetting mitigation" workflow/decisions/E-47-flip-runbook.md
# Expected: ≥3 hits

# 9. Smoke: ship-gate refuses default-on
cd .elai_cc && cargo test -p cli lora_ship_gate_refuses_when_unconfirmed
# Expected: PASS
```

## ACCEPTANCE

- [ ] `[finetune.lora]` config block landed with `enabled = false`, `ship_gate = "DEFERRED_PER_E47_REPORT"`, `confirm_user_understanding_of_forgetting_risk = false` defaults
- [ ] Boot-path ship-gate check refuses to load any LoRA when `enabled = true` AND `confirm_user_understanding_of_forgetting_risk = false`; emits `lora_ship_gate_refused` SSB row
- [ ] `python/elai_finetune/SPEC.md` authored covering dataset, train, eval, serve_adapter surfaces — NO `.py` files shipped
- [ ] SPEC.md "Anti-patterns" section carries the 7 report-documented anti-patterns verbatim
- [ ] Calibration + refuse-conditions spec'd: in-domain probe + 200-example OOD probe + 5 pp forgetting cap (configurable, NOT softened by default)
- [ ] Adapter manifest layout `~/.elai/adapters/manifest.toml` spec'd in SPEC.md
- [ ] Operator flip-runbook at `workflow/decisions/E-47-flip-runbook.md` lists three blocking measurements (Q1, Q2, Q4 from the report)
- [ ] Dashboard `LoraShipGatePanel.svelte` is read-only; no Enable button source
- [ ] Decisions log at `workflow/reasoning/E-47-decisions.md` captures DEFER evidence + revisit triggers
- [ ] No new Cargo deps; no `.py` source files under elai_finetune/; no peft/unsloth crate additions
- [ ] Voice-LoRA spec explicitly requires `--consent-yes` flag and is per-USER, NOT per-project
- [ ] Smoke test `lora_ship_gate_refuses_when_unconfirmed` passes

## Tests to add

- `crates/cli/tests/lora_ship_gate_refuses_when_unconfirmed.rs`: sets `enabled = true` + `confirm = false`; asserts refusal + SSB row + stderr contains "DEFERRED_PER_E47_REPORT"
- `crates/cli/tests/lora_ship_gate_refuses_when_calibration_missing.rs`: sets both flags true but `metrics/lora_baseline.toml` absent; asserts refusal
- `crates/cli/tests/lora_ship_gate_passes_when_fully_confirmed.rs`: sets both flags + present baseline; asserts gate passes (does NOT actually load — the underlying loader is deferred)
- `crates/contracts/tests/lora_ship_gate_contract.rs`: contract test covering produces_symbols + observable_effect for the ship-gate

## Files touched

- MODIFY: `.elai_cc/elai_cc.toml` (+~15 lines: `[finetune.lora]` block)
- MODIFY: `crates/cli/src/main.rs` (+~50 LOC for boot-path ship-gate check + SSB emit)
- CREATE: `python/elai_finetune/SPEC.md` (~250 LOC — spec only, no source files)
- CREATE: `workflow/decisions/E-47-flip-runbook.md` (~80 LOC — operator runbook)
- CREATE: `workflow/reasoning/E-47-decisions.md` (DEFER evidence + revisit triggers)
- CREATE: `crates/dashboard-v2/src/lib/components/LoraShipGatePanel.svelte` (~80 LOC, read-only)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (+~25 LOC for `get_lora_ship_gate_status` Tauri command)

## Citations / Grounding

- arXiv:2305.14314 QLoRA (Dettmers) — baseline 4-bit NF4 + LoRA; grounds Task 2 spec
- arXiv:2404.02948 PiSSA (NeurIPS 2024) — +9 pp GSM8K over QLoRA; grounds Task 2 init choice
- https://developer.nvidia.com/blog/introducing-dora-a-high-performing-alternative-to-lora-for-fine-tuning/ DoRA NVIDIA writeup — +1-4 pp common-sense over LoRA; grounds Task 2 use_dora=True
- arXiv:2510.13003 OPLoRA (Oct 2025) — explicit measurement on Qwen2.5-7B; "significantly reduces forgetting"; grounds Task 3 + 5 pp forgetting cap
- arXiv:2405.09673 Biderman et al. "LoRA Learns Less and Forgets Less" — 5-19 pp out-of-domain code degradation; grounds Task 1 ship-gate AND Task 3 refuse-condition
- arXiv:2406.14497 CodeRAG-Bench (NAACL 2025) — retrieval > fine-tune for code-style; grounds DEFER decision
- https://www.anthropic.com/news/contextual-retrieval — 49% failed-retrieval reduction; grounds the "context-engineering cell" argument
- https://unsloth.ai/blog/qwen-coder — 51 min full FT 7B; LoRA-16bit in 24GB; grounds Task 2 trainer choice
- https://huggingface.co/docs/peft/developer_guides/lora — PEFT current API 2025/2026 `target_modules="all-linear"`, `use_dora=True`; grounds Task 2 PEFT config
- https://docs.vllm.ai/en/latest/features/lora/ — vLLM multi-adapter hot-swap; grounds Task 5
- arXiv:2512.17910 Activated LoRA (aLoRA, Dec 2025) — cross-model KV-cache reuse; grounds Task 5 hot-swap surface
- arXiv:2508.11624 LoRAtorio — multi-adapter drift to base; grounds Task 7 anti-pattern (b)
- arXiv:2410.10870 PortLLM — training-free patch portability nascent; grounds Task 4 base-model-version refusal
- arXiv:2409.04574 "Customizing LLM Generation Style using PEFT" (Sep 2024) — PEFT works for voice/style; grounds Task 6 voice LoRA spec
- huihui-ai/Qwen2.5-Coder-7B-Instruct-abliterated model card (HuggingFace) — base-model identity; LoRA-compatible per HF ecosystem
- CLAUDE.md REMOTE-SIDECAR block — canonical sidecar config; LoRA path must not break

## Open questions / risks

- The MOST IMPORTANT open question, copied verbatim from the report: "What is the measured gap between (current ELAI prompt-rules + few-shot) and (Qwen-Coder-7B with per-project QLoRA-PiSSA-DoRA) on a project-style probe set?" Without this number the DEFER decision is grounded in priors, not local measurement. Risk: ELAI may be leaving 3-5 pp on the table OR (more likely per the report) leaving nothing on the table. The flip-runbook (Task 8) Q1 line item exists to surface this.
- Q2: Does Qwen2.5-Coder-7B-Instruct-abliterated still respond to PEFT defaults the same as stock Qwen-Coder-7B? Abliteration modifies the residual stream; `target_modules="all-linear"` MAY need adjustment. Spec the calibration probe to test this directly before any flip.
- Q3: RTX 3090 wall-clock for QLoRA-PiSSA-DoRA on 1K/10K/100K examples is extrapolated from Unsloth profiling, not measured for this exact stack. Risk if flipped: training takes 3x predicted time.
- Q4: OPLoRA forgetting-mitigation may NOT hold for ELAI's worst-case domain gap (Rust-only project). Spec the 200-example mixed-language OOD probe (Task 3) to detect this.
- Adapter rot risk: when the sidecar base model is patched (Qwen2.5-Coder-7B → v0.2), all existing LoRA adapters become invalid. PortLLM is nascent. Mitigation: adapter manifest records `base_model_version`; LoraAdapterSelector refuses on mismatch. Discoverable corrupt state.
- Storage growth: ~50-200 MB per adapter; 5-20 project adapters fit on 3090 sidecar per the report; need eviction policy if user accumulates >20. Out of scope for E-47 (deferred).
- Pad-token bug: Qwen2.5 `<|im_start|>`/`<|im_end|>` are untrained special tokens; silent garbage output if not patched. Task 7 carries the warning into SPEC.md, but a real-implement E-47.1 must add a regression test on the patched tokens.
- "Abliterated" model behavior is a known unknown — the published huihui-ai variant may behave differently from stock Qwen-Coder under PEFT. Baseline re-measurement is mandatory pre-flip.
- The ship-gate token `DEFERRED_PER_E47_REPORT` is a deliberate friction point — if a future patch silently renames it, the gate weakens. CI lint in Task 1 enforces presence; a stub-tracking entry should reference E-47 if the gate is ever bypassed under a registered stub.

## Next

E-47 is deferred-by-design. It UNLOCKS three future actions IF and only if the operator flip-runbook's three measurements clear:

(1) If Q1 returns a 5+ pp gap that few-shot cannot close, schedule patch-E47.1 to author the actual `python/elai_finetune/*.py` source files and the `LoraAdapterSelector` overlay on `PerRoleRuntime`. This would be the smallest reversible bet — single per-USER voice LoRA, not per-project.

(2) If Q2 confirms PEFT defaults on the abliterated 7B require amended `target_modules`, the SPEC.md must be updated BEFORE any source files are written. The decisions log at `workflow/reasoning/E-47-decisions.md` captures this gating.

(3) If Q4 shows OPLoRA's mitigation does NOT hold for ELAI's worst-case domain gap, the spec must add the orthogonal-projection custom optimizer wrapping per arXiv:2510.13003 — also pre-source-files.

After E-47, ELAI's LoRA story is: "spec'd, gated, measured-cost-to-flip is one runbook + one PR. We don't ship it because the evidence says ProjectProfile + few-shot + CodeRAG already does the work that LoRA would chase." This is the discipline the report's Recommendation section explicitly endorses, and the discipline E-47 is structurally enforcing.
